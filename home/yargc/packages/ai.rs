use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};

const DEFAULT_MAX_AGE: u64 = 60;
const DEFAULT_CODEXBAR_TIMEOUT: u64 = 30;
const LOCK_STALE_AFTER: u64 = 120;

struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn env_u64(name: &str, default: u64, minimum: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
        .max(minimum)
}

fn cache_root() -> PathBuf {
    if let Ok(path) = env::var("XDG_CACHE_HOME") {
        if !path.trim().is_empty() {
            return PathBuf::from(path).join("vesper-ai");
        }
    }
    if let Ok(home) = env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home).join(".cache/vesper-ai");
        }
    }
    env::temp_dir().join("vesper-ai")
}

fn briefing_index() -> PathBuf {
    if let Ok(path) = env::var("VESPER_BRIEFING_DIR") {
        if !path.trim().is_empty() {
            return PathBuf::from(path).join("index.json");
        }
    }
    if let Ok(home) = env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home).join(".local/share/vesper/briefings/index.json");
        }
    }
    PathBuf::from("/nonexistent/vesper-briefings-index.json")
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

fn looks_like_json_object(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with('{') && trimmed.ends_with('}')
}

fn command_error(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr.chars().take(600).collect();
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout.chars().take(600).collect();
    }
    format!("exit {}", output.status.code().unwrap_or(-1))
}

fn run_json(command: &str, args: &[&str], timeout_secs: u64) -> Result<String, String> {
    let output = Command::new("timeout")
        .arg("--signal=KILL")
        .arg(format!("{}s", timeout_secs))
        .arg(command)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run {command}: {error}"))?;

    if !output.status.success() {
        return Err(command_error(&output));
    }

    let text = String::from_utf8(output.stdout)
        .map_err(|error| format!("invalid UTF-8 from {command}: {error}"))?;
    if !looks_like_json_object(&text) {
        return Err(format!("invalid JSON object from {command}"));
    }
    Ok(text.trim().to_string())
}

fn fallback_json(error: &str, kind: &str) -> String {
    match kind {
        "agents" => format!(
            "{{\"count\":0,\"class\":\"unknown\",\"tooltip\":\"{}\",\"agents\":[]}}",
            json_escape(error)
        ),
        "hermes" => format!(
            "{{\"count\":0,\"unread\":0,\"high\":0,\"class\":\"unknown\",\"latestTitle\":\"No briefings yet\",\"latestLane\":\"\",\"tooltip\":\"{}\"}}",
            json_escape(error)
        ),
        "privacy" => format!(
            "{{\"tor\":\"unknown\",\"mic\":\"unknown\",\"camera\":\"unknown\",\"clipboard\":\"unknown\",\"node\":\"unknown\",\"class\":\"unknown\",\"label\":\"--\",\"tooltip\":\"{}\"}}",
            json_escape(error)
        ),
        _ => "{}".to_string(),
    }
}

fn hermes_status() -> Result<String, String> {
    let index = briefing_index();
    if !index.exists() {
        return Ok("{\"count\":0,\"unread\":0,\"high\":0,\"class\":\"idle\",\"latestTitle\":\"No briefings yet\",\"latestLane\":\"\",\"tooltip\":\"Hermes · no briefings yet\"}".to_string());
    }

    let filter = r#"
      (if type == "array" then . else [] end) as $items
      | [$items[] | select(.unread == true)] as $unread
      | [$unread[] | select((.priority // "normal") == "high" or (.priority // "normal") == "critical")] as $high
      | ($items[0] // {}) as $latest
      | ($latest.title // "No briefings yet" | tostring) as $title
      | {
          count: ($items | length),
          unread: ($unread | length),
          high: ($high | length),
          class: (if ($high | length) > 0 then "attention" elif ($unread | length) > 0 then "unread" else "idle" end),
          latestTitle: $title,
          latestLane: ($latest.lane // "" | tostring),
          tooltip: (if ($items | length) == 0 then "Hermes · no briefings yet" else "Hermes · \($unread | length) unread · \($title)" end)
        }
    "#;
    let path = index.to_string_lossy().into_owned();
    run_json("jq", &["-c", filter, &path], 5)
}

fn transform_snapshot(codexbar: &str, agents: &str, hermes: &str, privacy: &str) -> Result<String, String> {
    let filter = env::var("VESPER_AI_JQ_FILTER")
        .map_err(|_| "VESPER_AI_JQ_FILTER is not set".to_string())?;

    let mut child = Command::new("jq")
        .args(["-s", "-c", "-f", filter.as_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start jq: {error}"))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "failed to open jq stdin".to_string())?;
        writeln!(stdin, "{codexbar}").map_err(|error| error.to_string())?;
        writeln!(stdin, "{agents}").map_err(|error| error.to_string())?;
        writeln!(stdin, "{hermes}").map_err(|error| error.to_string())?;
        writeln!(stdin, "{privacy}").map_err(|error| error.to_string())?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed waiting for jq: {error}"))?;
    if !output.status.success() {
        return Err(command_error(&output));
    }

    let text = String::from_utf8(output.stdout)
        .map_err(|error| format!("invalid UTF-8 from jq: {error}"))?;
    if !looks_like_json_object(&text) {
        return Err("jq returned an invalid snapshot".to_string());
    }
    Ok(text.trim().to_string())
}

fn build_fresh() -> Result<String, String> {
    let codexbar_timeout = env_u64(
        "VESPER_AI_CODEXBAR_TIMEOUT",
        DEFAULT_CODEXBAR_TIMEOUT,
        5,
    );
    let timeout_arg = codexbar_timeout.to_string();
    let codexbar = run_json(
        "codexbar",
        &["dashboard", "--identity", "redacted", "--timeout", &timeout_arg],
        codexbar_timeout + 5,
    )?;

    let agents = match run_json("vesper-agent-cockpit", &["status"], 5) {
        Ok(value) => value,
        Err(error) => fallback_json(&error, "agents"),
    };
    let hermes = match hermes_status() {
        Ok(value) => value,
        Err(error) => fallback_json(&error, "hermes"),
    };
    let privacy = match run_json("vesper-privacy-hud", &["status"], 5) {
        Ok(value) => value,
        Err(error) => fallback_json(&error, "privacy"),
    };

    transform_snapshot(&codexbar, &agents, &hermes, &privacy)
}

fn read_cache(path: &Path) -> Option<String> {
    let mut value = String::new();
    File::open(path).ok()?.read_to_string(&mut value).ok()?;
    if looks_like_json_object(&value) {
        Some(value.trim().to_string())
    } else {
        None
    }
}

fn cache_age(path: &Path) -> Duration {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .unwrap_or(Duration::MAX)
}

fn write_cache(path: &Path, value: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "cache path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(".snapshot.{}.tmp", std::process::id()));
    fs::write(&tmp, format!("{value}\n")).map_err(|error| error.to_string())?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())?;
    Ok(())
}

fn acquire_lock(path: &Path) -> Result<LockGuard, String> {
    for _ in 0..60 {
        match OpenOptions::new().write(true).create_new(true).mode(0o600).open(path) {
            Ok(mut file) => {
                let _ = writeln!(file, "{}", std::process::id());
                return Ok(LockGuard {
                    path: path.to_path_buf(),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if cache_age(path) > Duration::from_secs(LOCK_STALE_AFTER) {
                    let _ = fs::remove_file(path);
                    continue;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => return Err(format!("failed to acquire refresh lock: {error}")),
        }
    }
    Err("timed out waiting for AI refresh lock".to_string())
}

fn stale_snapshot(cached: Option<String>, error: &str) -> String {
    if let Some(value) = cached {
        let mut stale = value.replacen("\"stale\":false", "\"stale\":true", 1);
        if let Some(index) = stale.rfind('}') {
            stale.insert_str(
                index,
                &format!(",\"backendError\":\"{}\"", json_escape(error)),
            );
            return stale;
        }
    }

    format!(
        "{{\"schemaVersion\":2,\"generatedAt\":\"\",\"stale\":true,\"backendError\":\"{}\",\"summary\":{{\"providerCount\":0,\"criticalCount\":0,\"warningCount\":0,\"maxUsedPercent\":-1,\"maxProvider\":\"\",\"class\":\"stale\"}},\"providers\":[],\"agents\":{{\"count\":0,\"class\":\"unknown\",\"agents\":[]}},\"hermes\":{{\"count\":0,\"unread\":0,\"high\":0,\"class\":\"unknown\",\"latestTitle\":\"No briefings yet\",\"latestLane\":\"\"}},\"privacy\":{{\"tor\":\"unknown\",\"mic\":\"unknown\",\"camera\":\"unknown\",\"clipboard\":\"unknown\",\"node\":\"unknown\",\"class\":\"unknown\",\"label\":\"--\"}},\"codexbar\":{{\"version\":\"\",\"generatedAt\":\"\"}}}}",
        json_escape(error)
    )
}

fn snapshot(force: bool, max_age: u64) -> String {
    let root = cache_root();
    let cache = root.join("snapshot.json");
    let lock = root.join("refresh.lock");
    let _ = fs::create_dir_all(&root);
    let _ = fs::set_permissions(&root, fs::Permissions::from_mode(0o700));

    let cached = read_cache(&cache);
    if !force && cached.is_some() && cache_age(&cache) <= Duration::from_secs(max_age) {
        return cached.unwrap();
    }

    let _guard = match acquire_lock(&lock) {
        Ok(guard) => guard,
        Err(error) => return stale_snapshot(cached, &error),
    };

    let cached = read_cache(&cache);
    if !force && cached.is_some() && cache_age(&cache) <= Duration::from_secs(max_age) {
        return cached.unwrap();
    }

    match build_fresh() {
        Ok(fresh) => {
            if let Err(error) = write_cache(&cache, &fresh) {
                return stale_snapshot(Some(fresh), &format!("cache write failed: {error}"));
            }
            fresh
        }
        Err(error) => stale_snapshot(cached, &error),
    }
}

fn pretty_json(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + value.len() / 8);
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for ch in value.chars() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                out.push(ch);
            }
            '{' | '[' => {
                out.push(ch);
                depth += 1;
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
                out.push(ch);
            }
            ',' => {
                out.push(ch);
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
            }
            ':' => out.push_str(": "),
            c if c.is_whitespace() => {}
            c => out.push(c),
        }
    }
    out
}

fn usage() -> ! {
    eprintln!("usage: vesper-ai [status|refresh] [--max-age SECONDS] [--pretty]");
    std::process::exit(2);
}

fn main() {
    let mut command = "status".to_string();
    let mut max_age = env_u64("VESPER_AI_MAX_AGE", DEFAULT_MAX_AGE, 15);
    let mut pretty = false;
    let args: Vec<String> = env::args().skip(1).collect();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "status" | "refresh" => command = args[index].clone(),
            "--pretty" => pretty = true,
            "--max-age" => {
                index += 1;
                if index >= args.len() {
                    usage();
                }
                max_age = args[index].parse::<u64>().unwrap_or_else(|_| usage());
            }
            "-h" | "--help" => usage(),
            _ => usage(),
        }
        index += 1;
    }

    let value = snapshot(command == "refresh", max_age);
    if pretty {
        println!("{}", pretty_json(&value));
    } else {
        println!("{value}");
    }
}
