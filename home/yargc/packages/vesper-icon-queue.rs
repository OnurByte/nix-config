use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const QUEUE_SCHEMA_VERSION: u32 = 2;
const LEASE_MS: i64 = 5 * 60 * 1000;
const MAX_ATTEMPTS: u32 = 4;
const MAX_RETRY_DELAY_MS: i64 = 10 * 60 * 1000;

#[derive(Clone, Debug)]
struct InventoryItem {
    id: String,
    source_path: String,
    fingerprint: String,
    source_kind: String,
    canonical_state: String,
    excluded: bool,
}

#[derive(Clone, Debug)]
struct QueueJob {
    key: String,
    state: String,
    provider: String,
    source_kind: String,
    source_path: String,
    app_ids: BTreeSet<String>,
    attempts: u32,
    updated_ms: i64,
    next_run_ms: i64,
    lease_until_ms: i64,
    last_error: String,
}

fn home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/nonexistent"))
}

fn state_root() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("vesper/adaptive-icons")
}

fn config_root() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("vesper")
}

fn inventory_path() -> PathBuf {
    state_root().join("inventory.tsv")
}

fn db_path() -> PathBuf {
    state_root().join("state.sqlite3")
}

fn legacy_queue_path() -> PathBuf {
    state_root().join("conversion-queue.tsv")
}

fn legacy_manual_queue_path() -> PathBuf {
    state_root().join("queue")
}

fn config_path() -> PathBuf {
    config_root().join("adaptive-icons.conf")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sqlite(sql: &str) -> Result<String, String> {
    fs::create_dir_all(state_root()).map_err(|error| error.to_string())?;
    let mut child = Command::new("sqlite3")
        .arg("-batch")
        .arg("-noheader")
        .arg("-separator")
        .arg("\t")
        .arg(db_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start sqlite3: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(sql.as_bytes())
            .map_err(|error| format!("failed to write sqlite query: {error}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for sqlite3: {error}"))?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if message.is_empty() {
            format!("sqlite3 exited with {}", output.status.code().unwrap_or(-1))
        } else {
            message
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn cleanup_legacy_manual_queue() -> Result<(), String> {
    let path = legacy_manual_queue_path();
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(format!(
            "legacy adaptive icon queue path is not a directory: {}",
            path.display()
        ));
    }
    fs::remove_dir_all(&path)
        .map_err(|error| format!("failed to remove legacy manual icon queue: {error}"))
}

fn init_db() -> Result<(), String> {
    sqlite(
        "PRAGMA journal_mode=WAL;\n\
         PRAGMA synchronous=NORMAL;\n\
         PRAGMA busy_timeout=5000;\n\
         CREATE TABLE IF NOT EXISTS meta (\n\
           key TEXT PRIMARY KEY,\n\
           value TEXT NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS jobs (\n\
           key TEXT PRIMARY KEY,\n\
           state TEXT NOT NULL,\n\
           provider TEXT NOT NULL,\n\
           source_kind TEXT NOT NULL,\n\
           source_path TEXT NOT NULL,\n\
           app_ids TEXT NOT NULL,\n\
           attempts INTEGER NOT NULL DEFAULT 0,\n\
           updated_ms INTEGER NOT NULL,\n\
           next_run_ms INTEGER NOT NULL DEFAULT 0,\n\
           lease_until_ms INTEGER NOT NULL DEFAULT 0,\n\
           last_error TEXT NOT NULL DEFAULT ''\n\
         );\n\
         CREATE INDEX IF NOT EXISTS jobs_state_next_idx ON jobs(state, next_run_ms, updated_ms);\n\
         INSERT INTO meta(key, value) VALUES('schemaVersion', '2')\n\
           ON CONFLICT(key) DO UPDATE SET value=excluded.value;\n\
         INSERT INTO meta(key, value) VALUES('paused', '0')\n\
           ON CONFLICT(key) DO NOTHING;\n",
    )?;
    migrate_legacy_queue()?;
    cleanup_legacy_manual_queue()
}

fn load_provider() -> String {
    let content = fs::read_to_string(config_path()).unwrap_or_default();
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "provider" {
            let value = value.trim();
            if matches!(value, "openai" | "anthropic" | "xai" | "openrouter" | "google") {
                return value.to_string();
            }
        }
    }
    "openai".to_string()
}

fn remote_consent() -> bool {
    let content = fs::read_to_string(config_path()).unwrap_or_default();
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "remoteConsent" {
            let value = value.trim();
            return value == "1" || value.eq_ignore_ascii_case("true");
        }
    }
    false
}

fn provider_configured(provider: &str) -> bool {
    Command::new("secret-tool")
        .args(["lookup", "service", "vesper-ai", "provider", provider])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn load_inventory() -> Vec<InventoryItem> {
    let mut items = Vec::new();
    if let Ok(content) = sqlite(
        "PRAGMA busy_timeout=5000; SELECT desktop_id,source_path,source_fingerprint,source_kind,canonical_state,excluded FROM application_inventory ORDER BY desktop_id;",
    ) {
        for line in content.lines() {
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() < 6 {
                continue;
            }
            items.push(InventoryItem {
                id: parts[0].to_string(),
                source_path: parts[1].to_string(),
                fingerprint: parts[2].to_string(),
                source_kind: parts[3].to_string(),
                canonical_state: parts[4].to_string(),
                excluded: parts[5] == "1",
            });
        }
        if !items.is_empty() {
            return items;
        }
    }

    // Migration fallback for a session where the engine has not populated the
    // transactional inventory yet. The next reconcile writes the DB and this
    // path becomes unused.
    let content = fs::read_to_string(inventory_path()).unwrap_or_default();
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() >= 9 {
            items.push(InventoryItem {
                id: parts[0].to_string(),
                source_path: parts[2].to_string(),
                fingerprint: parts[3].to_string(),
                source_kind: parts[4].to_string(),
                canonical_state: parts[5].to_string(),
                excluded: parts[7] == "1",
            });
        }
    }
    items
}

fn parse_apps(value: &str) -> BTreeSet<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn app_string(values: &BTreeSet<String>) -> String {
    values.iter().cloned().collect::<Vec<_>>().join(",")
}

fn load_jobs() -> Result<BTreeMap<String, QueueJob>, String> {
    init_db()?;
    let output = sqlite(
        "SELECT key,state,provider,source_kind,source_path,app_ids,attempts,updated_ms,next_run_ms,lease_until_ms,last_error FROM jobs ORDER BY key;",
    )?;
    let mut jobs = BTreeMap::new();
    for line in output.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 11 {
            continue;
        }
        let key = parts[0].to_string();
        jobs.insert(
            key.clone(),
            QueueJob {
                key,
                state: parts[1].to_string(),
                provider: parts[2].to_string(),
                source_kind: parts[3].to_string(),
                source_path: parts[4].to_string(),
                app_ids: parse_apps(parts[5]),
                attempts: parts[6].parse().unwrap_or(0),
                updated_ms: parts[7].parse().unwrap_or(0),
                next_run_ms: parts[8].parse().unwrap_or(0),
                lease_until_ms: parts[9].parse().unwrap_or(0),
                last_error: parts[10].to_string(),
            },
        );
    }
    Ok(jobs)
}

fn migrate_legacy_queue() -> Result<(), String> {
    if !legacy_queue_path().is_file() {
        return Ok(());
    }
    let count = sqlite("SELECT COUNT(*) FROM jobs;")?;
    if count.trim().parse::<usize>().unwrap_or(0) != 0 {
        return Ok(());
    }

    let content = fs::read_to_string(legacy_queue_path()).unwrap_or_default();
    let mut sql = String::from("BEGIN IMMEDIATE;\n");
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 {
            continue;
        }
        let attempts = parts[6].parse::<u32>().unwrap_or(0);
        let updated_ms = parts[7].parse::<i64>().unwrap_or_else(|_| now_ms());
        sql.push_str(&format!(
            "INSERT OR IGNORE INTO jobs(key,state,provider,source_kind,source_path,app_ids,attempts,updated_ms,next_run_ms,lease_until_ms,last_error) VALUES({},{},{},{},{},{},{},{},0,0,{});\n",
            sql_quote(parts[0]),
            sql_quote(parts[1]),
            sql_quote(parts[2]),
            sql_quote(parts[3]),
            sql_quote(parts[4]),
            sql_quote(parts[5]),
            attempts,
            updated_ms,
            sql_quote(parts[8]),
        ));
    }
    sql.push_str("COMMIT;\n");
    sqlite(&sql)?;
    let retired = state_root().join("conversion-queue.tsv.retired");
    let _ = fs::rename(legacy_queue_path(), retired);
    Ok(())
}

fn desired_jobs(items: &[InventoryItem], provider: &str, consent: bool) -> BTreeMap<String, QueueJob> {
    let provider_ready = provider_configured(provider);
    let desired_state = if !consent {
        "blocked-no-consent"
    } else if provider_ready {
        "ready"
    } else {
        "blocked-no-provider"
    };
    let mut jobs = BTreeMap::<String, QueueJob>::new();
    for item in items {
        let needs_remote_semantics = item.canonical_state == "pending-ai"
            || (item.canonical_state == "validated" && item.source_kind == "svg");
        if item.excluded || item.fingerprint.is_empty() || !needs_remote_semantics {
            continue;
        }
        let entry = jobs.entry(item.fingerprint.clone()).or_insert_with(|| QueueJob {
            key: item.fingerprint.clone(),
            state: desired_state.to_string(),
            provider: provider.to_string(),
            source_kind: item.source_kind.clone(),
            source_path: item.source_path.clone(),
            app_ids: BTreeSet::new(),
            attempts: 0,
            updated_ms: now_ms(),
            next_run_ms: 0,
            lease_until_ms: 0,
            last_error: String::new(),
        });
        entry.app_ids.insert(item.id.clone());
        if entry.source_path.is_empty() && !item.source_path.is_empty() {
            entry.source_path = item.source_path.clone();
        }
    }
    jobs
}

fn sync() -> Result<(), String> {
    init_db()?;
    let provider = load_provider();
    let consent = remote_consent();
    let provider_ready = provider_configured(&provider);
    let desired = desired_jobs(&load_inventory(), &provider, consent);
    let existing = load_jobs()?;
    let timestamp = now_ms();
    let ready_state = if !consent {
        "blocked-no-consent"
    } else if provider_ready {
        "ready"
    } else {
        "blocked-no-provider"
    };
    let mut sql = String::from("BEGIN IMMEDIATE;\n");

    for (key, target) in &desired {
        let apps = app_string(&target.app_ids);
        if let Some(old) = existing.get(key) {
            let mut state = old.state.clone();
            let mut attempts = old.attempts;
            let mut last_error = old.last_error.clone();
            let mut next_run = old.next_run_ms;
            let mut lease_until = old.lease_until_ms;

            if old.provider != provider && matches!(state.as_str(), "failed" | "blocked-no-provider" | "ready" | "retry-wait") {
                state = ready_state.to_string();
                attempts = 0;
                last_error.clear();
                next_run = 0;
                lease_until = 0;
            } else if !consent && matches!(state.as_str(), "ready" | "retry-wait" | "blocked-no-provider") {
                state = "blocked-no-consent".to_string();
                next_run = 0;
                lease_until = 0;
            } else if state == "running" && lease_until <= timestamp {
                state = ready_state.to_string();
                last_error = "recovered expired conversion lease".to_string();
                lease_until = 0;
            } else if state == "retry-wait" && next_run <= timestamp {
                state = ready_state.to_string();
                next_run = 0;
            } else if state == "blocked-no-consent" && consent {
                state = if provider_ready {
                    "ready".to_string()
                } else {
                    "blocked-no-provider".to_string()
                };
            } else if state == "blocked-no-provider" && provider_ready && consent {
                state = "ready".to_string();
            } else if state == "ready" && (!provider_ready || !consent) {
                state = ready_state.to_string();
            } else if state == "superseded" {
                state = ready_state.to_string();
                attempts = 0;
                last_error.clear();
                next_run = 0;
                lease_until = 0;
            }

            sql.push_str(&format!(
                "UPDATE jobs SET state={},provider={},source_kind={},source_path={},app_ids={},attempts={},updated_ms={},next_run_ms={},lease_until_ms={},last_error={} WHERE key={};\n",
                sql_quote(&state),
                sql_quote(&provider),
                sql_quote(&target.source_kind),
                sql_quote(&target.source_path),
                sql_quote(&apps),
                attempts,
                timestamp,
                next_run,
                lease_until,
                sql_quote(&last_error),
                sql_quote(key),
            ));
        } else {
            sql.push_str(&format!(
                "INSERT INTO jobs(key,state,provider,source_kind,source_path,app_ids,attempts,updated_ms,next_run_ms,lease_until_ms,last_error) VALUES({},{},{},{},{},{},0,{},0,0,'');\n",
                sql_quote(key),
                sql_quote(ready_state),
                sql_quote(&provider),
                sql_quote(&target.source_kind),
                sql_quote(&target.source_path),
                sql_quote(&apps),
                timestamp,
            ));
        }
    }

    for (key, old) in &existing {
        if desired.contains_key(key) {
            continue;
        }
        if matches!(
            old.state.as_str(),
            "pending" | "ready" | "running" | "retry-wait" | "blocked-no-provider" | "blocked-no-consent"
        ) {
            sql.push_str(&format!(
                "UPDATE jobs SET state='superseded',updated_ms={},lease_until_ms=0 WHERE key={};\n",
                timestamp,
                sql_quote(key),
            ));
        }
    }

    sql.push_str(&format!(
        "INSERT INTO meta(key,value) VALUES('provider',{}) ON CONFLICT(key) DO UPDATE SET value=excluded.value;\n",
        sql_quote(&provider),
    ));
    sql.push_str("COMMIT;\n");
    sqlite(&sql)?;
    Ok(())
}

fn paused() -> Result<bool, String> {
    init_db()?;
    Ok(sqlite("SELECT value FROM meta WHERE key='paused';")?.trim() == "1")
}

fn set_paused(value: bool) -> Result<(), String> {
    init_db()?;
    sqlite(&format!(
        "INSERT INTO meta(key,value) VALUES('paused','{}') ON CONFLICT(key) DO UPDATE SET value=excluded.value;",
        if value { 1 } else { 0 }
    ))?;
    Ok(())
}

fn bounded_retry_delay(backoff_ms: i64, retry_after_ms: Option<i64>) -> i64 {
    retry_after_ms
        .unwrap_or(backoff_ms)
        .clamp(0, MAX_RETRY_DELAY_MS)
}

fn claim() -> Result<(), String> {
    sync()?;
    if paused()? {
        println!("{{\"job\":null,\"paused\":true}}");
        return Ok(());
    }
    let timestamp = now_ms();
    let lease = timestamp.saturating_add(LEASE_MS);
    let sql = format!(
        "BEGIN IMMEDIATE;\n\
         UPDATE jobs SET state='running',updated_ms={timestamp},lease_until_ms={lease}\n\
         WHERE key=(SELECT key FROM jobs WHERE state='ready' AND next_run_ms<={timestamp} ORDER BY attempts ASC,updated_ms ASC,key ASC LIMIT 1)\n\
         RETURNING key,provider,source_kind,source_path,app_ids,attempts;\n\
         COMMIT;\n"
    );
    let output = sqlite(&sql)?;
    let Some(line) = output.lines().find(|line| !line.trim().is_empty()) else {
        println!("{{\"job\":null,\"paused\":false}}");
        return Ok(());
    };
    let parts = line.split('\t').collect::<Vec<_>>();
    if parts.len() < 6 {
        return Err("invalid claimed queue row".to_string());
    }
    let apps = parse_apps(parts[4])
        .iter()
        .map(|id| format!("\"{}\"", json_escape(id)))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"job\":{{\"key\":\"{}\",\"provider\":\"{}\",\"sourceKind\":\"{}\",\"sourcePath\":\"{}\",\"appIds\":[{}],\"attempts\":{}}},\"paused\":false}}",
        json_escape(parts[0]),
        json_escape(parts[1]),
        json_escape(parts[2]),
        json_escape(parts[3]),
        apps,
        parts[5].parse::<u32>().unwrap_or(0),
    );
    Ok(())
}

fn heartbeat(key: &str) -> Result<(), String> {
    let timestamp = now_ms();
    let lease = timestamp.saturating_add(LEASE_MS);
    sqlite(&format!(
        "UPDATE jobs SET updated_ms={timestamp},lease_until_ms={lease} WHERE key={} AND state='running';",
        sql_quote(key),
    ))?;
    Ok(())
}

fn complete(key: &str) -> Result<(), String> {
    let timestamp = now_ms();
    sqlite(&format!(
        "UPDATE jobs SET state='succeeded',updated_ms={timestamp},next_run_ms=0,lease_until_ms=0,last_error='' WHERE key={};",
        sql_quote(key),
    ))?;
    Ok(())
}

fn fail(
    key: &str,
    message: &str,
    permanent: bool,
    retry_after_ms: Option<i64>,
) -> Result<(), String> {
    init_db()?;
    let query = format!("SELECT attempts FROM jobs WHERE key={};", sql_quote(key));
    let attempts = sqlite(&query)?.trim().parse::<u32>().unwrap_or(0).saturating_add(1);
    let timestamp = now_ms();
    let (state, next_run) = if permanent || attempts >= MAX_ATTEMPTS {
        ("failed", 0)
    } else {
        let exponent = attempts.saturating_sub(1).min(5);
        let backoff = 15_000_i64.saturating_mul(1_i64 << exponent);
        let delay = bounded_retry_delay(backoff, retry_after_ms);
        ("retry-wait", timestamp.saturating_add(delay))
    };
    sqlite(&format!(
        "UPDATE jobs SET state={},attempts={},updated_ms={},next_run_ms={},lease_until_ms=0,last_error={} WHERE key={};",
        sql_quote(state),
        attempts,
        timestamp,
        next_run,
        sql_quote(message),
        sql_quote(key),
    ))?;
    Ok(())
}

fn retry_app(id: &str) -> Result<(), String> {
    sync()?;
    let provider = load_provider();
    let state = if !remote_consent() {
        "blocked-no-consent"
    } else if provider_configured(&provider) {
        "ready"
    } else {
        "blocked-no-provider"
    };
    let jobs = load_jobs()?;
    let mut keys = Vec::new();
    for job in jobs.values() {
        if job.app_ids.contains(id) && !matches!(job.state.as_str(), "superseded" | "cancelled") {
            keys.push(job.key.clone());
        }
    }
    if keys.is_empty() {
        return Err(format!("application has no queued conversion: {id}"));
    }
    let timestamp = now_ms();
    let mut sql = String::from("BEGIN IMMEDIATE;\n");
    for key in keys {
        sql.push_str(&format!(
            "UPDATE jobs SET state={},attempts=0,updated_ms={},next_run_ms=0,lease_until_ms=0,last_error='' WHERE key={};\n",
            sql_quote(state),
            timestamp,
            sql_quote(&key),
        ));
    }
    sql.push_str("COMMIT;\n");
    sqlite(&sql)?;
    Ok(())
}

fn app_status(id: &str) -> Result<(), String> {
    sync()?;
    let jobs = load_jobs()?;
    let mut matches = jobs
        .values()
        .filter(|job| job.app_ids.contains(id))
        .collect::<Vec<_>>();
    matches.sort_by_key(|job| {
        let priority = match job.state.as_str() {
            "running" => 0,
            "ready" => 1,
            "retry-wait" => 2,
            "blocked-no-provider" | "blocked-no-consent" => 3,
            "failed" => 4,
            "succeeded" => 5,
            _ => 6,
        };
        (priority, std::cmp::Reverse(job.updated_ms))
    });
    let Some(job) = matches.first() else {
        println!("{{\"state\":\"none\"}}");
        return Ok(());
    };
    println!(
        "{{\"state\":\"{}\",\"provider\":\"{}\",\"attempts\":{},\"updatedMs\":{},\"nextRunMs\":{},\"lastError\":\"{}\"}}",
        json_escape(&job.state),
        json_escape(&job.provider),
        job.attempts,
        job.updated_ms,
        job.next_run_ms,
        json_escape(&job.last_error),
    );
    Ok(())
}

fn print_status() -> Result<(), String> {
    sync()?;
    let jobs = load_jobs()?;
    let provider = load_provider();
    let paused = paused()?;
    let consent = remote_consent();
    let count = |state: &str| jobs.values().filter(|job| job.state == state).count();
    let pending = jobs
        .values()
        .filter(|job| {
            matches!(
                job.state.as_str(),
                "pending" | "ready" | "running" | "retry-wait" | "blocked-no-provider" | "blocked-no-consent"
            )
        })
        .count();
    println!(
        "{{\"schemaVersion\":{},\"provider\":\"{}\",\"providerConfigured\":{},\"consentGranted\":{},\"paused\":{},\"total\":{},\"pending\":{},\"ready\":{},\"running\":{},\"retryWait\":{},\"blockedNoProvider\":{},\"blockedNoConsent\":{},\"failed\":{},\"succeeded\":{},\"superseded\":{},\"transport\":\"active\"}}",
        QUEUE_SCHEMA_VERSION,
        json_escape(&provider),
        if provider_configured(&provider) { "true" } else { "false" },
        if consent { "true" } else { "false" },
        if paused { "true" } else { "false" },
        jobs.len(),
        pending,
        count("ready"),
        count("running"),
        count("retry-wait"),
        count("blocked-no-provider"),
        count("blocked-no-consent"),
        count("failed"),
        count("succeeded"),
        count("superseded"),
    );
    Ok(())
}

fn daemon() -> Result<(), String> {
    loop {
        if let Err(error) = sync() {
            eprintln!("adaptive icon queue sync failed: {error}");
        }
        thread::sleep(Duration::from_secs(5));
    }
}

#[cfg(test)]
mod tests {
    use super::bounded_retry_delay;

    #[test]
    fn retry_after_overrides_backoff_with_a_cap() {
        assert_eq!(bounded_retry_delay(15_000, Some(45_000)), 45_000);
        assert_eq!(bounded_retry_delay(15_000, Some(999_999_999)), 600_000);
        assert_eq!(bounded_retry_delay(15_000, None), 15_000);
    }
}

fn usage() -> ! {
    eprintln!(
        "vesper-icon-queue\n\
         commands:\n\
           sync\n\
           status\n\
           app-status <desktop-id>\n\
           claim\n\
           heartbeat <job-key>\n\
           complete <job-key>\n\
           fail <job-key> transient|permanent [retry-after-ms] <message>\n\
           retry-app <desktop-id>\n\
           pause|resume\n\
           daemon"
    );
    std::process::exit(2);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [command] if command == "sync" => sync(),
        [command] if command == "status" => print_status(),
        [command, id] if command == "app-status" => app_status(id),
        [command] if command == "claim" => claim(),
        [command, key] if command == "heartbeat" => heartbeat(key),
        [command, key] if command == "complete" => complete(key),
        [command, key, kind, message] if command == "fail" => {
            fail(key, message, kind == "permanent", None)
        }
        [command, key, kind, retry_ms, message] if command == "fail" => {
            let retry_after_ms = retry_ms.parse::<i64>().ok().filter(|value| *value >= 0);
            fail(key, message, kind == "permanent", retry_after_ms)
        }
        [command, id] if command == "retry-app" => retry_app(id),
        [command] if command == "pause" => set_paused(true),
        [command] if command == "resume" => set_paused(false),
        [command] if command == "daemon" => daemon(),
        _ => usage(),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
