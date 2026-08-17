use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, config_root, state_root};
use crate::process::output;

#[derive(Clone, Debug, Default)]
struct AppPolicy {
    id: String,
    excluded: bool,
    daily_limit: u64,
    category: String,
}

struct LockGuard(PathBuf);
impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn dir() -> PathBuf { state_root().join("wellbeing") }
fn enabled_path() -> PathBuf { config_root().join("wellbeing.enabled") }
fn focus_path() -> PathBuf { config_root().join("wellbeing/focus.enabled") }
fn goal_path() -> PathBuf { config_root().join("wellbeing/daily-goal-seconds") }
fn policy_path() -> PathBuf { config_root().join("wellbeing/apps.tsv") }

fn day_path(day: &str) -> PathBuf { dir().join(format!("{day}.tsv")) }

fn today() -> String {
    output("date", &["+%F"]).unwrap_or_else(|_| "unknown-date".to_string())
}

fn day_ago(offset: u32) -> String {
    if offset == 0 { return today(); }
    output("date", &["-d", &format!("{offset} days ago"), "+%F"])
        .unwrap_or_else(|_| format!("unknown-{offset}"))
}

pub fn enabled() -> bool {
    match fs::read_to_string(enabled_path()) {
        Ok(value) => matches!(value.trim(), "1" | "on" | "true"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    }
}

pub fn set_enabled(value: bool) -> Result<(), String> {
    atomic_write_private(&enabled_path(), if value { b"1\n" } else { b"0\n" })
}

pub fn focus_enabled() -> bool {
    fs::read_to_string(focus_path())
        .map(|value| matches!(value.trim(), "1" | "on" | "true"))
        .unwrap_or(false)
}

pub fn set_focus(value: bool) -> Result<(), String> {
    let action = if value { "enableDnd" } else { "disableDnd" };
    output("caelestia-shell", &["ipc", "call", "notifs", action])
        .map_err(|error| format!("could not change DND for Focus mode: {error}"))?;
    atomic_write_private(&focus_path(), if value { b"1\n" } else { b"0\n" })
}

pub fn daily_goal() -> u64 {
    fs::read_to_string(goal_path())
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

pub fn set_daily_goal(seconds: u64) -> Result<(), String> {
    if seconds > 86_400 {
        return Err("daily wellbeing goal cannot exceed 24 hours".to_string());
    }
    atomic_write_private(&goal_path(), format!("{seconds}\n").as_bytes())
}

pub fn normalize_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn clean(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ").trim().to_string()
}

fn load_day(path: &Path) -> BTreeMap<String, u64> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        if let Some((name, seconds)) = line.rsplit_once('\t') {
            if let Ok(seconds) = seconds.parse::<u64>() {
                values.insert(name.to_string(), seconds);
            }
        }
    }
    values
}

fn save_day(path: &Path, values: &BTreeMap<String, u64>) -> Result<(), String> {
    let mut data = String::new();
    for (name, seconds) in values {
        data.push_str(&clean(name));
        data.push('\t');
        data.push_str(&seconds.to_string());
        data.push('\n');
    }
    atomic_write_private(path, data.as_bytes())
}

fn load_policies() -> BTreeMap<String, AppPolicy> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(policy_path()).unwrap_or_default().lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() != 4 { continue; }
        let id = parts[0].to_string();
        values.insert(id.clone(), AppPolicy {
            id,
            excluded: parts[1] == "1",
            daily_limit: parts[2].parse().unwrap_or(0),
            category: parts[3].to_string(),
        });
    }
    values
}

fn save_policies(values: &BTreeMap<String, AppPolicy>) -> Result<(), String> {
    let mut data = String::new();
    for policy in values.values() {
        data.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            clean(&policy.id),
            if policy.excluded { 1 } else { 0 },
            policy.daily_limit,
            clean(&policy.category)
        ));
    }
    atomic_write_private(&policy_path(), data.as_bytes())
}

fn matching_policy<'a>(app: &str, values: &'a BTreeMap<String, AppPolicy>) -> Option<&'a AppPolicy> {
    let target = normalize_id(app.strip_suffix(".desktop").unwrap_or(app));
    if target.is_empty() { return None; }
    values.values().find(|policy| {
        let id = normalize_id(policy.id.strip_suffix(".desktop").unwrap_or(&policy.id));
        !id.is_empty() && (id.contains(&target) || target.contains(&id))
    })
}

pub fn set_app_policy(id: &str, field: &str, value: &str) -> Result<(), String> {
    if id.trim().is_empty() { return Err("app id is required".to_string()); }
    let mut policies = load_policies();
    let policy = policies.entry(id.to_string()).or_insert_with(|| AppPolicy {
        id: id.to_string(),
        ..AppPolicy::default()
    });
    match field {
        "excluded" => policy.excluded = matches!(value, "1" | "true" | "on"),
        "limit" => {
            let seconds = value.parse::<u64>().map_err(|_| "limit expects seconds".to_string())?;
            if seconds > 86_400 { return Err("daily app limit cannot exceed 24 hours".to_string()); }
            policy.daily_limit = seconds;
        }
        "category" => {
            if value.len() > 80 || value.chars().any(|ch| ch.is_control()) {
                return Err("invalid wellbeing category".to_string());
            }
            policy.category = value.to_string();
        }
        _ => return Err("wellbeing app field expects excluded, limit or category".to_string()),
    }
    if !policy.excluded && policy.daily_limit == 0 && policy.category.is_empty() {
        policies.remove(id);
    }
    save_policies(&policies)
}

pub fn seconds_for(id: &str) -> u64 {
    let target = normalize_id(id.strip_suffix(".desktop").unwrap_or(id));
    if target.is_empty() { return 0; }
    load_day(&day_path(&today()))
        .into_iter()
        .filter(|(name, _)| {
            let name = normalize_id(name);
            !name.is_empty() && (name.contains(&target) || target.contains(&name))
        })
        .map(|(_, seconds)| seconds)
        .sum()
}

fn shell_bool(target: &str, method: &str) -> Option<bool> {
    let value = output("caelestia-shell", &["ipc", "call", target, method]).ok()?;
    match value.trim().trim_matches('"') {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn sampling_allowed(lock: Option<bool>, idle: Option<bool>) -> bool {
    matches!((lock, idle), (Some(false), Some(false)))
}

fn live_sampling_allowed() -> bool {
    sampling_allowed(shell_bool("lock", "isLocked"), shell_bool("idle", "isIdle"))
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let index = json.find(&needle)?;
    let rest = &json[index + needle.len()..];
    let colon = rest.find(':')?;
    let mut chars = rest[colon + 1..].chars().peekable();
    while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) { chars.next(); }
    if chars.next()? != '"' { return None; }
    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(match ch { 'n' => '\n', 'r' => '\r', 't' => '\t', other => other });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(out);
        } else {
            out.push(ch);
        }
    }
    None
}

fn active_app() -> Option<String> {
    let json = output("hyprctl", &["activewindow", "-j"]).ok()?;
    let value = extract_json_string(&json, "class")
        .or_else(|| extract_json_string(&json, "initialClass"))?;
    let value = clean(&value);
    if value.is_empty() { None } else { Some(value) }
}

fn acquire_lock() -> Result<LockGuard, String> {
    let root = dir();
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let lock = root.join("daemon.lock");
    for _ in 0..2 {
        match OpenOptions::new().write(true).create_new(true).open(&lock) {
            Ok(mut file) => {
                let _ = writeln!(file, "{}", std::process::id());
                return Ok(LockGuard(lock));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = fs::read_to_string(&lock)
                    .ok()
                    .and_then(|value| value.trim().parse::<u32>().ok())
                    .map(|pid| !Path::new(&format!("/proc/{pid}")).exists())
                    .unwrap_or(true);
                if stale {
                    let _ = fs::remove_file(&lock);
                    continue;
                }
                return Err("wellbeing daemon already running".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("could not acquire wellbeing daemon lock".to_string())
}

pub fn daemon() -> Result<(), String> {
    let _lock = acquire_lock()?;
    let mut day = today();
    let mut path = day_path(&day);
    let mut values = load_day(&path);
    let tick = Duration::from_secs(5);
    let mut last_flush = Instant::now();

    loop {
        let started = Instant::now();
        let current_day = today();
        if current_day != day {
            save_day(&path, &values)?;
            day = current_day;
            path = day_path(&day);
            values = load_day(&path);
        }
        if enabled() && live_sampling_allowed() {
            if let Some(app) = active_app() {
                let policies = load_policies();
                let excluded = matching_policy(&app, &policies).map(|policy| policy.excluded).unwrap_or(false);
                if !excluded {
                    *values.entry(app).or_insert(0) += tick.as_secs();
                }
            }
        }
        if last_flush.elapsed() >= Duration::from_secs(30) {
            save_day(&path, &values)?;
            last_flush = Instant::now();
        }
        let elapsed = started.elapsed();
        if elapsed < tick { thread::sleep(tick - elapsed); }
    }
}

fn day_total(day: &str) -> u64 {
    load_day(&day_path(day)).values().sum()
}

fn days_json(count: u32) -> (Vec<String>, u64) {
    let mut total = 0u64;
    let mut days = Vec::new();
    for offset in 0..count {
        let date = day_ago(offset);
        let seconds = day_total(&date);
        total += seconds;
        days.push(format!("{{\"date\":\"{}\",\"seconds\":{seconds}}}", escape(&date)));
    }
    (days, total)
}

pub fn summary_json() -> String {
    let values = load_day(&day_path(&today()));
    let total = values.values().sum::<u64>();
    let mut items = values.into_iter().collect::<Vec<_>>();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.truncate(12);
    let apps = items
        .iter()
        .map(|(name, seconds)| format!("{{\"app\":\"{}\",\"seconds\":{seconds}}}", escape(name)))
        .collect::<Vec<_>>();
    format!("{{\"totalSeconds\":{total},\"apps\":[{}]}}", apps.join(","))
}

pub fn report_json() -> String {
    let (days, month) = days_json(30);
    let week = (0..7).map(|offset| day_total(&day_ago(offset))).sum::<u64>();
    let today_values = load_day(&day_path(&today()));
    let today_total = today_values.values().sum::<u64>();
    let policies = load_policies();
    let mut apps = today_values.into_iter().collect::<Vec<_>>();
    apps.sort_by(|a, b| b.1.cmp(&a.1));
    let app_json = apps.into_iter().take(20).map(|(app, seconds)| {
        let policy = matching_policy(&app, &policies);
        let excluded = policy.map(|value| value.excluded).unwrap_or(false);
        let limit = policy.map(|value| value.daily_limit).unwrap_or(0);
        let category = policy.map(|value| value.category.as_str()).unwrap_or("");
        format!(
            "{{\"app\":\"{}\",\"seconds\":{seconds},\"excluded\":{},\"dailyLimitSeconds\":{limit},\"overLimit\":{},\"category\":\"{}\"}}",
            escape(&app), bool_lit(excluded), bool_lit(limit > 0 && seconds >= limit), escape(category)
        )
    }).collect::<Vec<_>>();
    let goal = daily_goal();
    format!(
        "{{\"enabled\":{},\"focus\":{},\"limitBehavior\":\"advisory\",\"todaySeconds\":{today_total},\"weekSeconds\":{week},\"monthSeconds\":{month},\"dailyGoalSeconds\":{goal},\"goalReached\":{},\"days\":[{}],\"apps\":[{}]}}",
        bool_lit(enabled()), bool_lit(focus_enabled()), bool_lit(goal > 0 && today_total >= goal), days.join(","), app_json.join(",")
    )
}

pub fn app_json(id: &str) -> String {
    let policies = load_policies();
    let policy = matching_policy(id, &policies);
    let limit = policy.map(|value| value.daily_limit).unwrap_or(0);
    let seconds = seconds_for(id);
    let category = policy.map(|value| value.category.as_str()).unwrap_or("");
    let excluded = policy.map(|value| value.excluded).unwrap_or(false);
    let history = (0..30).map(|offset| {
        let date = day_ago(offset);
        let target = normalize_id(id.strip_suffix(".desktop").unwrap_or(id));
        let seconds = load_day(&day_path(&date)).into_iter()
            .filter(|(name, _)| {
                let name = normalize_id(name);
                !name.is_empty() && (name.contains(&target) || target.contains(&name))
            })
            .map(|(_, value)| value)
            .sum::<u64>();
        format!("{{\"date\":\"{}\",\"seconds\":{seconds}}}", escape(&date))
    }).collect::<Vec<_>>();
    format!(
        "{{\"id\":\"{}\",\"todaySeconds\":{seconds},\"excluded\":{},\"dailyLimitSeconds\":{limit},\"overLimit\":{},\"category\":\"{}\",\"limitBehavior\":\"advisory\",\"history\":[{}]}}",
        escape(id), bool_lit(excluded), bool_lit(limit > 0 && seconds >= limit), escape(category), history.join(",")
    )
}

pub fn reset(scope: &str) -> Result<(), String> {
    match scope {
        "today" => match fs::remove_file(day_path(&today())) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        },
        "all" => {
            for entry in fs::read_dir(dir()).map_err(|error| error.to_string())?.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) == Some("tsv") {
                    fs::remove_file(path).map_err(|error| error.to_string())?;
                }
            }
            Ok(())
        }
        _ => Err("wellbeing reset expects today or all".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_or_locked_time_is_never_sampled() {
        assert!(sampling_allowed(Some(false), Some(false)));
        assert!(!sampling_allowed(Some(true), Some(false)));
        assert!(!sampling_allowed(Some(false), Some(true)));
        assert!(!sampling_allowed(None, Some(false)));
        assert!(!sampling_allowed(Some(false), None));
    }

    #[test]
    fn app_identity_normalization_is_stable() {
        assert_eq!(normalize_id("org.Example-App.desktop"), "orgexampleappdesktop");
    }
}
