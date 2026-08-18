use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{exclusions_path, Config};
use crate::model::InventoryItem;
use crate::util::{
    canonical_root, json_escape, now_epoch, safe_name, sql_escape, state_db, state_root,
    write_atomic,
};

fn db_path() -> String {
    state_db().to_string_lossy().into_owned()
}

pub fn sql(statement: &str) -> Result<String, String> {
    fs::create_dir_all(state_root()).map_err(|e| e.to_string())?;
    crate::util::command_output("sqlite3", &["-batch", "-noheader", &db_path(), statement])
}

pub fn init() -> Result<(), String> {
    let schema = r#"
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;
CREATE TABLE IF NOT EXISTS applications (
  id TEXT PRIMARY KEY,
  canonical_app_id TEXT NOT NULL,
  launch_desktop_id TEXT NOT NULL,
  runtime_ids TEXT NOT NULL,
  icon_aliases TEXT NOT NULL,
  desktop_path TEXT NOT NULL,
  icon_key TEXT NOT NULL,
  source_path TEXT NOT NULL DEFAULT '',
  source_fingerprint TEXT NOT NULL DEFAULT '',
  source_kind TEXT NOT NULL DEFAULT '',
  source_resolver TEXT NOT NULL DEFAULT '',
  work_key TEXT NOT NULL DEFAULT '',
  tier TEXT NOT NULL DEFAULT 'original-fallback',
  queue_state TEXT NOT NULL DEFAULT '',
  excluded INTEGER NOT NULL DEFAULT 0,
  active INTEGER NOT NULL DEFAULT 0,
  error TEXT NOT NULL DEFAULT '',
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS canonical_work (
  work_key TEXT PRIMARY KEY,
  source_fingerprint TEXT NOT NULL,
  schema_version INTEGER NOT NULL,
  prompt_revision TEXT NOT NULL,
  validator_revision TEXT NOT NULL,
  provider_family TEXT NOT NULL,
  model_family TEXT NOT NULL,
  package_path TEXT NOT NULL DEFAULT '',
  state TEXT NOT NULL DEFAULT 'missing',
  validation TEXT NOT NULL DEFAULT '',
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS jobs (
  work_key TEXT PRIMARY KEY,
  app_id TEXT NOT NULL,
  source_path TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  state TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  retry_at INTEGER NOT NULL DEFAULT 0,
  lease_until INTEGER NOT NULL DEFAULT 0,
  last_error TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS theme_generations (
  generation TEXT PRIMARY KEY,
  path TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS shadow_desktops (
  desktop_id TEXT PRIMARY KEY,
  upstream_path TEXT NOT NULL,
  upstream_fingerprint TEXT NOT NULL,
  shadow_path TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS jobs_state_retry ON jobs(state, retry_at);
CREATE INDEX IF NOT EXISTS apps_work_key ON applications(work_key);
"#;
    sql(schema)?;

    let now = now_epoch();
    sql(&format!(
        "UPDATE jobs SET state='retry-wait', lease_until=0, retry_at={now}, \
         last_error='recovered-expired-lease', updated_at={now} \
         WHERE state='running' AND lease_until < {now};"
    ))?;
    sync_application_queue_states()?;
    Ok(())
}

pub fn load_exclusions() -> BTreeSet<String> {
    fs::read_to_string(exclusions_path())
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn set_excluded(id: &str, excluded: bool) -> Result<(), String> {
    let mut values = load_exclusions();
    if excluded {
        values.insert(id.to_string());
    } else {
        values.remove(id);
    }

    let mut body = values.into_iter().collect::<Vec<_>>().join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    write_atomic(&exclusions_path(), body)
}

fn join(values: &[String]) -> String {
    values.join("\u{1f}")
}

pub fn upsert_app(item: &InventoryItem) -> Result<(), String> {
    let now = now_epoch();
    let source_path = item
        .source
        .as_ref()
        .map(|source| source.path.to_string_lossy().into_owned())
        .unwrap_or_default();
    let source_fingerprint = item
        .source
        .as_ref()
        .map(|source| source.fingerprint.clone())
        .unwrap_or_default();
    let source_kind = item
        .source
        .as_ref()
        .map(|source| source.kind.clone())
        .unwrap_or_default();
    let source_resolver = item
        .source
        .as_ref()
        .map(|source| source.resolver.clone())
        .unwrap_or_default();
    let q = |value: &str| format!("'{}'", sql_escape(value));

    let statement = format!(
        "INSERT INTO applications(\
            id,canonical_app_id,launch_desktop_id,runtime_ids,icon_aliases,desktop_path,icon_key,\
            source_path,source_fingerprint,source_kind,source_resolver,work_key,tier,queue_state,\
            excluded,active,error,updated_at\
         ) VALUES({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}) \
         ON CONFLICT(id) DO UPDATE SET \
            canonical_app_id=excluded.canonical_app_id,\
            launch_desktop_id=excluded.launch_desktop_id,\
            runtime_ids=excluded.runtime_ids,\
            icon_aliases=excluded.icon_aliases,\
            desktop_path=excluded.desktop_path,\
            icon_key=excluded.icon_key,\
            source_path=excluded.source_path,\
            source_fingerprint=excluded.source_fingerprint,\
            source_kind=excluded.source_kind,\
            source_resolver=excluded.source_resolver,\
            work_key=excluded.work_key,\
            tier=excluded.tier,\
            queue_state=excluded.queue_state,\
            excluded=excluded.excluded,\
            active=excluded.active,\
            error=excluded.error,\
            updated_at=excluded.updated_at;",
        q(&item.desktop.id),
        q(&item.identity.canonical_app_id),
        q(&item.identity.launch_desktop_id),
        q(&join(&item.identity.runtime_ids)),
        q(&join(&item.identity.icon_aliases)),
        q(&item.desktop.path.to_string_lossy()),
        q(&item.desktop.icon),
        q(&source_path),
        q(&source_fingerprint),
        q(&source_kind),
        q(&source_resolver),
        q(&item.work_key),
        q(&item.tier),
        q(&item.queue_state),
        item.excluded as u8,
        item.active as u8,
        q(&item.error),
        now
    );
    sql(&statement).map(|_| ())
}

pub fn remove_stale_apps(seen_ids: &[String]) -> Result<(), String> {
    if seen_ids.is_empty() {
        sql("DELETE FROM applications;")?;
        return Ok(());
    }

    let ids = seen_ids
        .iter()
        .map(|value| format!("'{}'", sql_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    sql(&format!("DELETE FROM applications WHERE id NOT IN ({ids});"))?;
    Ok(())
}

pub fn cancel_unreferenced_jobs() -> Result<(), String> {
    let now = now_epoch();
    sql(&format!(
        "UPDATE jobs SET state='cancelled', lease_until=0, retry_at=0, \
         last_error='superseded-or-removed-source', updated_at={now} \
         WHERE state NOT IN ('succeeded','cancelled') \
         AND work_key NOT IN (SELECT DISTINCT work_key FROM applications WHERE work_key <> '');"
    ))?;
    Ok(())
}

pub fn canonical_path(work_key: &str) -> PathBuf {
    canonical_root().join(format!("{}.vicon", safe_name(work_key)))
}

pub fn canonical_valid(work_key: &str) -> bool {
    if work_key.is_empty() {
        return false;
    }
    let package = canonical_path(work_key);
    package.join("manifest.json").is_file()
        && sql(&format!(
            "SELECT COUNT(*) FROM canonical_work WHERE work_key='{}' AND state='validated';",
            sql_escape(work_key)
        ))
        .ok()
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

pub fn register_work(work_key: &str, fingerprint: &str, cfg: &Config) -> Result<(), String> {
    let now = now_epoch();
    sql(&format!(
        "INSERT INTO canonical_work(\
            work_key,source_fingerprint,schema_version,prompt_revision,validator_revision,\
            provider_family,model_family,state,updated_at\
         ) VALUES('{}','{}',2,'vicon-semantic-v2','validator-v2','{}','{}','missing',{}) \
         ON CONFLICT(work_key) DO UPDATE SET \
            source_fingerprint=excluded.source_fingerprint,\
            provider_family=excluded.provider_family,\
            model_family=excluded.model_family,\
            updated_at=excluded.updated_at;",
        sql_escape(work_key),
        sql_escape(fingerprint),
        sql_escape(&cfg.provider),
        sql_escape(&cfg.model),
        now
    ))?;
    Ok(())
}

fn desired_state(cfg: &Config, provider_ready: bool) -> &'static str {
    if !cfg.remote_consent {
        "blocked-no-consent"
    } else if !provider_ready {
        "blocked-no-provider"
    } else if !cfg.automatic || cfg.queue_paused {
        "pending"
    } else {
        "ready"
    }
}

pub fn ensure_job(
    item: &InventoryItem,
    cfg: &Config,
    provider_ready: bool,
) -> Result<String, String> {
    let Some(source) = &item.source else {
        return Ok("unresolved-source".into());
    };
    if item.excluded || item.work_key.is_empty() || canonical_valid(&item.work_key) {
        return Ok(if item.excluded {
            "cancelled"
        } else {
            "succeeded"
        }
        .into());
    }

    register_work(&item.work_key, &source.fingerprint, cfg)?;
    let desired = desired_state(cfg, provider_ready);
    let now = now_epoch();
    let statement = format!(
        "INSERT INTO jobs(work_key,app_id,source_path,source_kind,state,created_at,updated_at) \
         VALUES('{}','{}','{}','{}','{}',{},{}) \
         ON CONFLICT(work_key) DO UPDATE SET \
            app_id=excluded.app_id,\
            source_path=excluded.source_path,\
            source_kind=excluded.source_kind,\
            state=CASE WHEN jobs.state IN ('succeeded','running','failed') THEN jobs.state ELSE excluded.state END,\
            updated_at=excluded.updated_at; \
         SELECT state FROM jobs WHERE work_key='{}';",
        sql_escape(&item.work_key),
        sql_escape(&item.desktop.id),
        sql_escape(&source.path.to_string_lossy()),
        sql_escape(&source.kind),
        desired,
        now,
        now,
        sql_escape(&item.work_key)
    );
    sql(&statement).map(|value| value.lines().last().unwrap_or(desired).trim().to_string())
}

pub fn refresh_blocked(cfg: &Config, provider_ready: bool) -> Result<(), String> {
    let state = desired_state(cfg, provider_ready);
    let now = now_epoch();
    sql(&format!(
        "UPDATE jobs SET state='{state}', updated_at={now} \
         WHERE state IN ('blocked-no-provider','blocked-no-consent','pending','ready') \
         AND work_key NOT IN (SELECT work_key FROM canonical_work WHERE state='validated');"
    ))?;
    sync_application_queue_states()?;
    Ok(())
}

fn sync_application_queue_states() -> Result<(), String> {
    sql(
        "UPDATE applications SET queue_state=COALESCE(\
            (SELECT jobs.state FROM jobs WHERE jobs.work_key=applications.work_key),\
            queue_state\
         ) WHERE work_key <> '';",
    )?;
    Ok(())
}

#[derive(Debug)]
pub struct Job {
    pub work_key: String,
    pub app_id: String,
    pub source_path: PathBuf,
    pub source_kind: String,
    pub attempts: i64,
}

pub fn lease_next() -> Result<Option<Job>, String> {
    let now = now_epoch();
    let lease_until = now + 180;
    let row = sql(&format!(
        "UPDATE jobs SET \
            state='running', attempts=attempts+1, lease_until={lease_until}, updated_at={now} \
         WHERE work_key=(\
            SELECT work_key FROM jobs \
            WHERE state IN ('ready','retry-wait') AND retry_at <= {now} \
            ORDER BY CASE WHEN state='ready' THEN 0 ELSE 1 END, updated_at DESC, created_at ASC \
            LIMIT 1\
         ) \
         AND state IN ('ready','retry-wait') \
         RETURNING work_key,app_id,source_path,source_kind,attempts;"
    ))?;

    let Some(line) = row.lines().next().filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 5 {
        return Err("invalid queue lease row".into());
    }

    let job = Job {
        work_key: parts[0].to_string(),
        app_id: parts[1].to_string(),
        source_path: PathBuf::from(parts[2]),
        source_kind: parts[3].to_string(),
        attempts: parts[4].parse().unwrap_or(1),
    };
    sql(&format!(
        "UPDATE applications SET queue_state='running' WHERE work_key='{}';",
        sql_escape(&job.work_key)
    ))?;
    Ok(Some(job))
}

pub fn job_success(job: &Job, package_path: &Path) -> Result<(), String> {
    let now = now_epoch();
    sql(&format!(
        "BEGIN IMMEDIATE; \
         UPDATE canonical_work SET package_path='{}',state='validated',validation='passed',updated_at={} \
         WHERE work_key='{}'; \
         UPDATE jobs SET state='succeeded',lease_until=0,retry_at=0,last_error='',updated_at={} \
         WHERE work_key='{}'; \
         UPDATE applications SET tier='canonical-ai',queue_state='succeeded',error='' \
         WHERE work_key='{}'; \
         COMMIT;",
        sql_escape(&package_path.to_string_lossy()),
        now,
        sql_escape(&job.work_key),
        now,
        sql_escape(&job.work_key),
        sql_escape(&job.work_key)
    ))?;
    Ok(())
}

pub fn job_failure(
    job: &Job,
    error: &str,
    retry_after: Option<i64>,
    permanent: bool,
) -> Result<(), String> {
    let now = now_epoch();
    let state = if permanent || job.attempts >= 4 {
        "failed"
    } else {
        "retry-wait"
    };
    let backoff = retry_after.unwrap_or_else(|| {
        (15_i64 * (1_i64 << job.attempts.min(6) as u32)).min(900)
    });
    let retry_at = if state == "retry-wait" {
        now + backoff.max(15)
    } else {
        0
    };

    sql(&format!(
        "UPDATE jobs SET state='{state}',lease_until=0,retry_at={retry_at},last_error='{}',updated_at={now} \
         WHERE work_key='{}'; \
         UPDATE applications SET queue_state='{state}',error='{}' WHERE work_key='{}';",
        sql_escape(error),
        sql_escape(&job.work_key),
        sql_escape(error),
        sql_escape(&job.work_key)
    ))?;
    Ok(())
}

pub fn pause(paused: bool) -> Result<(), String> {
    if paused {
        sql("UPDATE jobs SET state='pending' WHERE state='ready';")?;
        sync_application_queue_states()?;
    }
    Ok(())
}

pub fn retry_failed(app_id: Option<&str>) -> Result<(), String> {
    let now = now_epoch();
    let filter = app_id
        .map(|id| format!(" AND app_id='{}'", sql_escape(id)))
        .unwrap_or_default();
    sql(&format!(
        "UPDATE jobs SET state='pending',attempts=0,retry_at=0,lease_until=0,last_error='',updated_at={now} \
         WHERE state='failed'{filter};"
    ))?;
    sync_application_queue_states()?;
    Ok(())
}

pub fn app_json(id: &str) -> Result<String, String> {
    let row = sql(&format!(
        "SELECT id,canonical_app_id,launch_desktop_id,source_path,source_kind,source_fingerprint,\
         source_resolver,tier,queue_state,active,excluded,error,work_key \
         FROM applications WHERE id='{}' LIMIT 1;",
        sql_escape(id)
    ))?;
    let parts: Vec<&str> = row
        .lines()
        .next()
        .ok_or_else(|| format!("application not in adaptive icon inventory: {id}"))?
        .split('|')
        .collect();
    if parts.len() < 13 {
        return Err("invalid application state row".into());
    }

    Ok(format!(
        "{{\"id\":\"{}\",\"canonicalAppId\":\"{}\",\"launchDesktopId\":\"{}\",\
        \"sourcePath\":\"{}\",\"sourceKind\":\"{}\",\"fingerprint\":\"{}\",\
        \"sourceResolver\":\"{}\",\"canonicalState\":\"{}\",\"queueState\":\"{}\",\
        \"active\":{},\"excluded\":{},\"error\":\"{}\",\"workKey\":\"{}\"}}",
        json_escape(parts[0]),
        json_escape(parts[1]),
        json_escape(parts[2]),
        json_escape(parts[3]),
        json_escape(parts[4]),
        json_escape(parts[5]),
        json_escape(parts[6]),
        json_escape(parts[7]),
        json_escape(parts[8]),
        parts[9] == "1",
        parts[10] == "1",
        json_escape(parts[11]),
        json_escape(parts[12])
    ))
}

pub fn counts() -> Result<(i64, i64, i64, i64, i64, i64, i64, i64), String> {
    let row = sql(
        "SELECT \
            COUNT(*),\
            SUM(CASE WHEN tier='canonical-ai' THEN 1 ELSE 0 END),\
            SUM(CASE WHEN queue_state IN ('pending','ready') THEN 1 ELSE 0 END),\
            SUM(CASE WHEN queue_state='running' THEN 1 ELSE 0 END),\
            SUM(CASE WHEN queue_state='retry-wait' THEN 1 ELSE 0 END),\
            SUM(CASE WHEN queue_state='failed' THEN 1 ELSE 0 END),\
            SUM(CASE WHEN queue_state LIKE 'blocked-%' THEN 1 ELSE 0 END),\
            SUM(CASE WHEN active=1 THEN 1 ELSE 0 END) \
         FROM applications;",
    )?;
    let values: Vec<i64> = row
        .trim()
        .split('|')
        .map(|value| value.parse().unwrap_or(0))
        .collect();
    if values.len() != 8 {
        return Err("invalid status counts".into());
    }
    Ok((
        values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
    ))
}
