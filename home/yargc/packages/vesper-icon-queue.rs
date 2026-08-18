use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const QUEUE_SCHEMA_VERSION: u32 = 1;

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
    updated_ms: u128,
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

fn queue_tsv_path() -> PathBuf {
    state_root().join("conversion-queue.tsv")
}

fn queue_json_path() -> PathBuf {
    state_root().join("conversion-queue.json")
}

fn queue_status_path() -> PathBuf {
    state_root().join("conversion-queue-status.json")
}

fn config_path() -> PathBuf {
    config_root().join("adaptive-icons.conf")
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
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

fn tsv_escape(value: &str) -> String {
    value
        .chars()
        .map(|ch| if matches!(ch, '\t' | '\n' | '\r') { ' ' } else { ch })
        .collect()
}

fn write_atomic(path: &Path, data: impl AsRef<[u8]>) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid path: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("queue"),
        std::process::id()
    ));
    fs::write(&tmp, data).map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())
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
    let content = fs::read_to_string(inventory_path()).unwrap_or_default();
    let mut items = Vec::new();
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 {
            continue;
        }
        items.push(InventoryItem {
            id: parts[0].to_string(),
            source_path: parts[2].to_string(),
            fingerprint: parts[3].to_string(),
            source_kind: parts[4].to_string(),
            canonical_state: parts[5].to_string(),
            excluded: parts[7] == "1",
        });
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

fn load_queue() -> BTreeMap<String, QueueJob> {
    let content = fs::read_to_string(queue_tsv_path()).unwrap_or_default();
    let mut jobs = BTreeMap::new();
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 {
            continue;
        }
        let attempts = parts[6].parse::<u32>().unwrap_or(0);
        let updated_ms = parts[7].parse::<u128>().unwrap_or(0);
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
                attempts,
                updated_ms,
                last_error: parts[8].to_string(),
            },
        );
    }
    jobs
}

fn desired_jobs(items: &[InventoryItem], provider: &str) -> BTreeMap<String, QueueJob> {
    let ready = provider_configured(provider);
    let mut jobs = BTreeMap::<String, QueueJob>::new();
    for item in items {
        if item.excluded
            || item.fingerprint.is_empty()
            || item.canonical_state != "pending-ai"
        {
            continue;
        }
        let entry = jobs.entry(item.fingerprint.clone()).or_insert_with(|| QueueJob {
            key: item.fingerprint.clone(),
            state: if ready { "ready" } else { "blocked-no-provider" }.to_string(),
            provider: provider.to_string(),
            source_kind: item.source_kind.clone(),
            source_path: item.source_path.clone(),
            app_ids: BTreeSet::new(),
            attempts: 0,
            updated_ms: now_ms(),
            last_error: String::new(),
        });
        entry.app_ids.insert(item.id.clone());
        if entry.source_path.is_empty() && !item.source_path.is_empty() {
            entry.source_path = item.source_path.clone();
        }
    }
    jobs
}

fn reconcile_jobs(
    mut existing: BTreeMap<String, QueueJob>,
    desired: BTreeMap<String, QueueJob>,
    provider: &str,
) -> BTreeMap<String, QueueJob> {
    let ready = provider_configured(provider);
    let timestamp = now_ms();
    let mut next = BTreeMap::new();

    for (key, mut target) in desired {
        if let Some(old) = existing.remove(&key) {
            target.attempts = old.attempts;
            target.last_error = old.last_error;
            target.updated_ms = old.updated_ms;

            target.state = match old.state.as_str() {
                "running" => {
                    target.last_error = "recovered interrupted conversion".to_string();
                    target.updated_ms = timestamp;
                    if ready { "ready" } else { "blocked-no-provider" }.to_string()
                }
                "retry-wait" | "failed" | "cancelled" => old.state,
                "succeeded" => "superseded".to_string(),
                _ => {
                    let wanted = if ready { "ready" } else { "blocked-no-provider" };
                    if old.state != wanted || old.provider != provider {
                        target.updated_ms = timestamp;
                    }
                    wanted.to_string()
                }
            };
        }
        target.provider = provider.to_string();
        next.insert(key, target);
    }

    for (key, mut old) in existing {
        if !matches!(old.state.as_str(), "superseded" | "cancelled") {
            old.state = "superseded".to_string();
            old.updated_ms = timestamp;
        }
        next.insert(key, old);
    }

    next
}

fn queue_json(jobs: &BTreeMap<String, QueueJob>) -> String {
    let rows = jobs
        .values()
        .map(|job| {
            let apps = job
                .app_ids
                .iter()
                .map(|id| format!("\"{}\"", json_escape(id)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"key\":\"{}\",\"state\":\"{}\",\"provider\":\"{}\",\"sourceKind\":\"{}\",\"sourcePath\":\"{}\",\"appIds\":[{}],\"attempts\":{},\"updatedMs\":{},\"lastError\":\"{}\"}}",
                json_escape(&job.key),
                json_escape(&job.state),
                json_escape(&job.provider),
                json_escape(&job.source_kind),
                json_escape(&job.source_path),
                apps,
                job.attempts,
                job.updated_ms,
                json_escape(&job.last_error)
            )
        })
        .collect::<Vec<_>>();
    format!(
        "{{\"schemaVersion\":{},\"jobs\":[{}]}}\n",
        QUEUE_SCHEMA_VERSION,
        rows.join(",")
    )
}

fn queue_tsv(jobs: &BTreeMap<String, QueueJob>) -> String {
    let mut body = String::new();
    for job in jobs.values() {
        let apps = job.app_ids.iter().cloned().collect::<Vec<_>>().join(",");
        body.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            tsv_escape(&job.key),
            tsv_escape(&job.state),
            tsv_escape(&job.provider),
            tsv_escape(&job.source_kind),
            tsv_escape(&job.source_path),
            tsv_escape(&apps),
            job.attempts,
            job.updated_ms,
            tsv_escape(&job.last_error)
        ));
    }
    body
}

fn status_json(jobs: &BTreeMap<String, QueueJob>, provider: &str) -> String {
    let count = |state: &str| jobs.values().filter(|job| job.state == state).count();
    let pending = jobs
        .values()
        .filter(|job| matches!(job.state.as_str(), "pending" | "ready" | "running" | "retry-wait" | "blocked-no-provider" | "blocked-no-consent"))
        .count();
    format!(
        "{{\"schemaVersion\":{},\"provider\":\"{}\",\"providerConfigured\":{},\"total\":{},\"pending\":{},\"ready\":{},\"running\":{},\"retryWait\":{},\"blockedNoProvider\":{},\"failed\":{},\"succeeded\":{},\"superseded\":{},\"transport\":\"not-implemented\"}}\n",
        QUEUE_SCHEMA_VERSION,
        json_escape(provider),
        if provider_configured(provider) { "true" } else { "false" },
        jobs.len(),
        pending,
        count("ready"),
        count("running"),
        count("retry-wait"),
        count("blocked-no-provider"),
        count("failed"),
        count("succeeded"),
        count("superseded")
    )
}

fn persist(jobs: &BTreeMap<String, QueueJob>, provider: &str) -> Result<(), String> {
    fs::create_dir_all(state_root()).map_err(|error| error.to_string())?;
    write_atomic(&queue_tsv_path(), queue_tsv(jobs))?;
    write_atomic(&queue_json_path(), queue_json(jobs))?;
    write_atomic(&queue_status_path(), status_json(jobs, provider))?;
    Ok(())
}

fn sync() -> Result<(), String> {
    let provider = load_provider();
    let items = load_inventory();
    let existing = load_queue();
    let desired = desired_jobs(&items, &provider);
    let jobs = reconcile_jobs(existing, desired, &provider);
    persist(&jobs, &provider)
}

fn retry_app(id: &str) -> Result<(), String> {
    sync()?;
    let provider = load_provider();
    let ready = provider_configured(&provider);
    let mut jobs = load_queue();
    let mut matched = false;
    for job in jobs.values_mut() {
        if job.app_ids.contains(id) && !matches!(job.state.as_str(), "superseded" | "cancelled") {
            job.state = if ready { "ready" } else { "blocked-no-provider" }.to_string();
            job.last_error.clear();
            job.updated_ms = now_ms();
            matched = true;
        }
    }
    if !matched {
        return Err(format!("application has no queued conversion: {id}"));
    }
    persist(&jobs, &provider)
}

fn print_status() -> Result<(), String> {
    if !queue_status_path().is_file() {
        sync()?;
    }
    let text = fs::read_to_string(queue_status_path()).map_err(|error| error.to_string())?;
    print!("{text}");
    Ok(())
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|metadata| metadata.modified()).ok()
}

fn daemon() -> Result<(), String> {
    let mut last_inventory = None;
    let mut last_config = None;
    let mut last_provider_ready = None;
    loop {
        let inventory_mtime = modified(&inventory_path());
        let config_mtime = modified(&config_path());
        let provider = load_provider();
        let provider_ready = provider_configured(&provider);
        if inventory_mtime != last_inventory
            || config_mtime != last_config
            || Some(provider_ready) != last_provider_ready
        {
            if let Err(error) = sync() {
                eprintln!("adaptive icon queue sync failed: {error}");
            }
            last_inventory = inventory_mtime;
            last_config = config_mtime;
            last_provider_ready = Some(provider_ready);
        }
        thread::sleep(Duration::from_secs(5));
    }
}

fn usage() -> ! {
    eprintln!(
        "vesper-icon-queue\n\
         commands:\n\
           sync\n\
           status\n\
           retry-app <desktop-id>\n\
           daemon"
    );
    std::process::exit(2);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [command] if command == "sync" => sync(),
        [command] if command == "status" => print_status(),
        [command, id] if command == "retry-app" => retry_app(id),
        [command] if command == "daemon" => daemon(),
        _ => usage(),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
