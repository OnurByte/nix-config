use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, config_root, state_root};
use crate::process::output;

const FORMAT_VERSION: u32 = 1;
const GENERATOR_VERSION: &str = "vesper-icon-curator-v1";
const MAX_JOBS_PER_RECONCILE: usize = 2;
const MAX_ATTEMPTS: u32 = 4;

#[derive(Clone, Debug)]
struct Config {
    enabled: bool,
    mode: String,
    tint: String,
    provider: String,
    credential: String,
    model: String,
}

impl Default for Config {
    fn default() -> Self {
        // Enabling App Icons may create paid AI requests for non-curated apps,
        // so production defaults to explicit opt-in.
        Self {
            enabled: false,
            mode: "original".to_string(),
            tint: "#8aadf4".to_string(),
            provider: "openai".to_string(),
            credential: "openai".to_string(),
            model: "gpt-5".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct App {
    id: String,
    name: String,
    icon: String,
}

#[derive(Clone, Debug)]
struct Job {
    id: String,
    name: String,
    source_icon: String,
    source_hash: String,
    state: String,
    attempts: u32,
    next_retry: u64,
    error: String,
    source_type: String,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn safe_id(value: &str) -> String {
    let value = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if value.is_empty() { "unknown".to_string() } else { value }
}

fn config_path() -> PathBuf { config_root().join("icons/config") }
fn queue_path() -> PathBuf { state_root().join("icons/queue.tsv") }
fn semantic_dir() -> PathBuf { state_root().join("icons/semantic") }
fn rendered_dir() -> PathBuf { state_root().join("icons/rendered") }
fn registry_path() -> PathBuf { state_root().join("icons/registry.json") }
fn semantic_path(id: &str) -> PathBuf { semantic_dir().join(format!("{}.svg", safe_id(id))) }
fn rendered_path(id: &str, config: &Config) -> PathBuf {
    rendered_dir().join(format!(
        "{}-{}-{}.svg",
        safe_id(id),
        config.mode,
        config.tint.trim_start_matches('#')
    ))
}

fn read_kv(path: &Path) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    values
}

fn valid_mode(value: &str) -> bool {
    matches!(value, "original" | "light" | "dark" | "tinted" | "clear")
}

fn valid_tint(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn valid_model(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.chars().any(|ch| ch.is_control() || ch.is_whitespace())
}

fn load_config() -> Config {
    let mut config = Config::default();
    let values = read_kv(&config_path());
    if let Some(value) = values.get("enabled") {
        config.enabled = matches!(value.as_str(), "true" | "1" | "on");
    }
    if let Some(value) = values.get("mode").filter(|value| valid_mode(value)) {
        config.mode = value.clone();
    }
    if let Some(value) = values.get("tint").filter(|value| valid_tint(value)) {
        config.tint = value.to_ascii_lowercase();
    }
    if let Some(value) = values.get("provider").filter(|value| valid_token(value)) {
        config.provider = value.clone();
    }
    if let Some(value) = values.get("credential").filter(|value| valid_token(value)) {
        config.credential = value.clone();
    }
    if let Some(value) = values.get("model").filter(|value| valid_model(value)) {
        config.model = value.clone();
    }
    config
}

fn save_config(config: &Config) -> Result<(), String> {
    atomic_write_private(
        &config_path(),
        format!(
            "enabled={}\nmode={}\ntint={}\nprovider={}\ncredential={}\nmodel={}\n",
            bool_lit(config.enabled),
            config.mode,
            config.tint,
            config.provider,
            config.credential,
            config.model
        )
        .as_bytes(),
    )
}

pub fn set_config(key: &str, value: &str) -> Result<(), String> {
    let mut config = load_config();
    match key {
        "enabled" => {
            config.enabled = match value {
                "on" | "true" | "1" => true,
                "off" | "false" | "0" => false,
                _ => return Err("enabled expects on or off".to_string()),
            };
        }
        "mode" if valid_mode(value) => config.mode = value.to_string(),
        "tint" if valid_tint(value) => config.tint = value.to_ascii_lowercase(),
        "provider" if valid_token(value) => config.provider = value.to_string(),
        "credential" if valid_token(value) => config.credential = value.to_string(),
        "model" if valid_model(value) => config.model = value.to_string(),
        "mode" => return Err("mode expects original, light, dark, tinted or clear".to_string()),
        "tint" => return Err("tint expects #RRGGBB".to_string()),
        "provider" | "credential" | "model" => return Err(format!("invalid {key}")),
        _ => return Err(format!("unknown icon setting: {key}")),
    }
    save_config(&config)?;
    render_registry(&config, &load_jobs())
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::paths::home().join(".local/share")),
    ];
    for value in env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string())
        .split(':')
    {
        if !value.is_empty() { dirs.push(PathBuf::from(value)); }
    }
    let home = crate::paths::home();
    dirs.push(home.join(".nix-profile/share"));
    if let Ok(user) = env::var("USER") {
        dirs.push(PathBuf::from(format!("/etc/profiles/per-user/{user}/share")));
    }
    dirs.push(PathBuf::from("/run/current-system/sw/share"));
    dirs.sort();
    dirs.dedup();
    dirs
}

fn parse_desktop(path: &Path) -> Option<App> {
    let text = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut name = String::new();
    let mut icon = String::new();
    let mut hidden = false;
    let mut no_display = false;
    let mut app_type = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry { continue; }
        if let Some((key, value)) = line.split_once('=') {
            match key {
                "Name" if name.is_empty() => name = value.trim().to_string(),
                "Icon" => icon = value.trim().to_string(),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Type" => app_type = value.trim().to_string(),
                _ => {}
            }
        }
    }
    if hidden || no_display || app_type != "Application" || name.is_empty() { return None; }
    Some(App {
        id: path.file_name()?.to_string_lossy().to_string(),
        name,
        icon,
    })
}

fn discover_apps() -> Vec<App> {
    let mut apps = BTreeMap::<String, App>::new();
    for data in data_dirs() {
        let Ok(entries) = fs::read_dir(data.join("applications")) else { continue; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("desktop") { continue; }
            if let Some(app) = parse_desktop(&path) {
                apps.entry(app.id.clone()).or_insert(app);
            }
        }
    }
    apps.into_values().collect()
}

fn is_icon_candidate(path: &Path, icon: &str) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else { return false; };
    ["svg", "png", "webp", "jpg", "jpeg"]
        .iter()
        .any(|ext| name == format!("{icon}.{ext}"))
}

fn find_icon_recursive(dir: &Path, icon: &str, depth: u8) -> Option<PathBuf> {
    if depth == 0 { return None; }
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_file() && is_icon_candidate(&path, icon) { return Some(path); }
        if path.is_dir() {
            if let Some(found) = find_icon_recursive(&path, icon, depth - 1) { return Some(found); }
        }
    }
    None
}

fn resolve_icon(icon: &str) -> Option<PathBuf> {
    if icon.is_empty() { return None; }
    let direct = PathBuf::from(icon);
    if direct.is_absolute() && direct.is_file() { return Some(direct); }
    let base = icon
        .strip_suffix(".svg")
        .or_else(|| icon.strip_suffix(".png"))
        .or_else(|| icon.strip_suffix(".webp"))
        .or_else(|| icon.strip_suffix(".jpg"))
        .unwrap_or(icon);
    for data in data_dirs() {
        for subdir in ["icons", "pixmaps"] {
            if let Some(found) = find_icon_recursive(&data.join(subdir), base, 8) { return Some(found); }
        }
    }
    None
}

fn hash_file(path: &Path) -> String {
    let path_text = path.to_string_lossy().to_string();
    output("sha256sum", &[&path_text])
        .ok()
        .and_then(|value| value.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| {
            let modified = fs::metadata(path)
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_secs())
                .unwrap_or(0);
            format!("mtime-{modified}")
        })
}

fn curated_manifest() -> BTreeMap<String, PathBuf> {
    let mut values = BTreeMap::new();
    let Some(root) = env::var_os("VESPER_CURATED_ICON_DIR").map(PathBuf::from) else { return values; };
    for line in fs::read_to_string(root.join("manifest.txt")).unwrap_or_default().lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((id, filename)) = line.split_once('=') {
            let path = root.join(filename.trim());
            if path.is_file() { values.insert(id.trim().to_string(), path); }
        }
    }
    values
}

fn clean_field(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ").replace('\r', " ")
}

fn load_jobs() -> BTreeMap<String, Job> {
    let mut jobs = BTreeMap::new();
    for line in fs::read_to_string(queue_path()).unwrap_or_default().lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 { continue; }
        let job = Job {
            id: parts[0].to_string(),
            name: parts[1].to_string(),
            source_icon: parts[2].to_string(),
            source_hash: parts[3].to_string(),
            state: parts[4].to_string(),
            attempts: parts[5].parse().unwrap_or(0),
            next_retry: parts[6].parse().unwrap_or(0),
            error: parts[7].to_string(),
            source_type: parts[8].to_string(),
        };
        jobs.insert(job.id.clone(), job);
    }
    jobs
}

fn save_jobs(jobs: &BTreeMap<String, Job>) -> Result<(), String> {
    let mut text = String::new();
    for job in jobs.values() {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            clean_field(&job.id),
            clean_field(&job.name),
            clean_field(&job.source_icon),
            clean_field(&job.source_hash),
            clean_field(&job.state),
            job.attempts,
            job.next_retry,
            clean_field(&job.error),
            clean_field(&job.source_type)
        ));
    }
    atomic_write_private(&queue_path(), text.as_bytes())
}

fn extract_svg(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    let start = raw.find("<svg").ok_or_else(|| "generator did not return an SVG".to_string())?;
    let end = raw
        .rfind("</svg>")
        .ok_or_else(|| "generator returned an incomplete SVG".to_string())?
        + 6;
    if end <= start { return Err("generator returned malformed SVG".to_string()); }
    Ok(raw[start..end].to_string())
}

pub fn sanitize_svg(raw: &str) -> Result<String, String> {
    let svg = extract_svg(raw)?;
    if svg.len() > 262_144 { return Err("SVG exceeds 256 KiB safety limit".to_string()); }
    let lower = svg.to_ascii_lowercase();

    // xmlns="http://www.w3.org/2000/svg" is metadata, not a fetched resource.
    for forbidden in [
        "<script", "<foreignobject", "<!entity", "<!doctype", "javascript:", "file:",
        "xlink:href", "<image", "<iframe", "<audio", "<video", "<object", "<embed", "data:",
        "href=\"http://", "href=\"https://", "href='http://", "href='https://", "url(http://", "url(https://"
    ] {
        if lower.contains(forbidden) {
            return Err(format!("SVG contains forbidden active/external content: {forbidden}"));
        }
    }
    for event in ["onload=", "onclick=", "onerror=", "onmouseover=", "onfocus=", "onbegin=", "onend="] {
        if lower.contains(event) { return Err(format!("SVG contains forbidden event handler: {event}")); }
    }
    if lower.matches('<').count() > 1024
        || lower.matches("<path").count() > 256
        || lower.matches("<filter").count() > 16
    {
        return Err("SVG exceeds icon complexity limits".to_string());
    }
    if !lower.contains("viewbox=") { return Err("SVG must define a viewBox".to_string()); }
    Ok(svg)
}

fn render_semantic(svg: &str, config: &Config) -> Result<String, String> {
    let svg = sanitize_svg(svg)?;
    if config.mode == "original" { return Ok(svg); }
    let open_end = svg.find('>').ok_or_else(|| "SVG root is malformed".to_string())?;
    let close = svg.rfind("</svg>").ok_or_else(|| "SVG root is incomplete".to_string())?;
    let head = &svg[..=open_end];
    let body = &svg[open_end + 1..close];
    let transformed = match config.mode.as_str() {
        "tinted" => format!(
            "<defs><filter id=\"vesperTint\" color-interpolation-filters=\"sRGB\"><feFlood flood-color=\"{}\" result=\"tint\"/><feComposite in=\"tint\" in2=\"SourceAlpha\" operator=\"in\"/></filter></defs><g filter=\"url(#vesperTint)\">{body}</g>",
            config.tint
        ),
        "light" => format!(
            "<defs><filter id=\"vesperLight\"><feComponentTransfer><feFuncR type=\"linear\" slope=\"0.82\" intercept=\"0.04\"/><feFuncG type=\"linear\" slope=\"0.82\" intercept=\"0.04\"/><feFuncB type=\"linear\" slope=\"0.82\" intercept=\"0.04\"/></feComponentTransfer></filter></defs><g filter=\"url(#vesperLight)\">{body}</g>"
        ),
        "dark" => format!(
            "<defs><filter id=\"vesperDark\"><feComponentTransfer><feFuncR type=\"linear\" slope=\"0.78\" intercept=\"0.18\"/><feFuncG type=\"linear\" slope=\"0.78\" intercept=\"0.18\"/><feFuncB type=\"linear\" slope=\"0.78\" intercept=\"0.18\"/></feComponentTransfer></filter></defs><g filter=\"url(#vesperDark)\">{body}</g>"
        ),
        "clear" => format!("<g opacity=\"0.68\">{body}</g>"),
        _ => return Err("unknown render mode".to_string()),
    };
    Ok(format!("{head}{transformed}</svg>"))
}

fn write_semantic(id: &str, raw: &str) -> Result<(), String> {
    atomic_write_private(&semantic_path(id), sanitize_svg(raw)?.as_bytes())
}

fn generator_output(config: &Config, app: &App, source: &Path) -> Result<String, String> {
    if config.provider != "openai" {
        return Err(format!("provider '{}' has no App Icons adapter yet", config.provider));
    }
    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let legacy = exe.with_file_name("vesper-control-legacy");
    let generator = exe.with_file_name("vesper-icon-generator");
    let result = Command::new(legacy)
        .args(["credential", "exec", &config.credential, "--"])
        .arg(generator)
        .arg("openai")
        .arg(&config.model)
        .arg(&app.id)
        .arg(&app.name)
        .arg(source)
        .output()
        .map_err(|error| format!("failed to start icon generator: {error}"))?;
    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if error.is_empty() { "icon generator failed".to_string() } else { error });
    }
    String::from_utf8(result.stdout)
        .map_err(|error| format!("generator returned invalid UTF-8: {error}"))
}

fn prepare_curated(job: &mut Job, path: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|error| format!("cannot read curated SVG: {error}"))?;
    write_semantic(&job.id, &raw)?;
    job.state = "prepared".to_string();
    job.error.clear();
    job.source_type = "curated".to_string();
    Ok(())
}

fn process_ai_job(config: &Config, job: &mut Job, app: &App, source: &Path) {
    if job.attempts >= MAX_ATTEMPTS || now() < job.next_retry { return; }
    job.state = "processing".to_string();
    job.attempts += 1;
    match generator_output(config, app, source).and_then(|raw| write_semantic(&job.id, &raw)) {
        Ok(()) => {
            job.state = "prepared".to_string();
            job.error.clear();
            job.next_retry = 0;
            job.source_type = "ai-generated".to_string();
        }
        Err(error) => {
            let lower = error.to_ascii_lowercase();
            job.state = if lower.contains("not configured") || lower.contains("credential") || lower.contains("secret") {
                "waiting-for-provider".to_string()
            } else if lower.contains("429") || lower.contains("quota") || lower.contains("rate limit") {
                "waiting-for-quota".to_string()
            } else if job.attempts >= MAX_ATTEMPTS {
                "fallback".to_string()
            } else {
                "failed".to_string()
            };
            let backoff = (300u64.saturating_mul(1u64 << job.attempts.min(6))).min(21_600);
            job.next_retry = now().saturating_add(backoff);
            job.error = error.chars().take(512).collect();
        }
    }
}

fn refresh_inventory(
    jobs: &mut BTreeMap<String, Job>,
    apps: &[App],
    curated: &BTreeMap<String, PathBuf>,
) {
    for app in apps {
        if let Some(curated_path) = curated.get(&app.id) {
            let hash = hash_file(curated_path);
            let changed = jobs
                .get(&app.id)
                .map(|job| job.source_hash != hash || job.source_type != "curated")
                .unwrap_or(true);
            if changed {
                let mut job = Job {
                    id: app.id.clone(),
                    name: app.name.clone(),
                    source_icon: curated_path.display().to_string(),
                    source_hash: hash,
                    state: "pending".to_string(),
                    attempts: 0,
                    next_retry: 0,
                    error: String::new(),
                    source_type: "curated".to_string(),
                };
                if let Err(error) = prepare_curated(&mut job, curated_path) {
                    job.state = "fallback".to_string();
                    job.error = error;
                }
                jobs.insert(app.id.clone(), job);
            }
            continue;
        }

        let Some(source) = resolve_icon(&app.icon) else {
            jobs.insert(app.id.clone(), Job {
                id: app.id.clone(),
                name: app.name.clone(),
                source_icon: app.icon.clone(),
                source_hash: "unresolved".to_string(),
                state: "fallback".to_string(),
                attempts: 0,
                next_retry: 0,
                error: "original icon could not be resolved to a local image".to_string(),
                source_type: "ai-generated".to_string(),
            });
            continue;
        };

        let hash = hash_file(&source);
        let changed = jobs
            .get(&app.id)
            .map(|job| job.source_hash != hash || job.source_type == "curated")
            .unwrap_or(true);
        if changed {
            jobs.insert(app.id.clone(), Job {
                id: app.id.clone(),
                name: app.name.clone(),
                source_icon: source.display().to_string(),
                source_hash: hash,
                state: "pending".to_string(),
                attempts: 0,
                next_retry: 0,
                error: String::new(),
                source_type: "ai-generated".to_string(),
            });
        }
    }

    let installed = apps.iter().map(|app| app.id.clone()).collect::<BTreeSet<_>>();
    jobs.retain(|id, _| installed.contains(id));
}

fn render_registry(config: &Config, jobs: &BTreeMap<String, Job>) -> Result<(), String> {
    let mut entries = Vec::new();
    if config.enabled && config.mode != "original" {
        fs::create_dir_all(rendered_dir()).map_err(|error| error.to_string())?;
        for job in jobs.values() {
            if job.state != "prepared" { continue; }
            let Ok(raw) = fs::read_to_string(semantic_path(&job.id)) else { continue; };
            let Ok(rendered) = render_semantic(&raw, config) else { continue; };
            let path = rendered_path(&job.id, config);
            if atomic_write_private(&path, rendered.as_bytes()).is_err() { continue; }
            entries.push(format!(
                "\"{}\":\"file://{}\"",
                escape(&job.id),
                escape(&path.display().to_string())
            ));
        }
    }
    atomic_write_private(
        &registry_path(),
        format!("{{{}}}\n", entries.join(",")).as_bytes(),
    )
}

pub fn reconcile() -> Result<(), String> {
    let config = load_config();
    let apps = discover_apps();
    let curated = curated_manifest();
    let mut jobs = load_jobs();
    refresh_inventory(&mut jobs, &apps, &curated);

    if config.enabled {
        let app_map = apps
            .iter()
            .map(|app| (app.id.clone(), app))
            .collect::<BTreeMap<_, _>>();
        let mut processed = 0usize;
        for job in jobs.values_mut() {
            if processed >= MAX_JOBS_PER_RECONCILE { break; }
            if job.source_type != "ai-generated"
                || !matches!(
                    job.state.as_str(),
                    "pending" | "failed" | "waiting-for-provider" | "waiting-for-quota"
                )
                || now() < job.next_retry
            {
                continue;
            }
            let Some(app) = app_map.get(&job.id).copied() else { continue; };
            let source = PathBuf::from(&job.source_icon);
            if !source.is_file() {
                job.state = "fallback".to_string();
                job.error = "source icon disappeared; original icon fallback remains active".to_string();
                continue;
            }
            process_ai_job(&config, job, app, &source);
            processed += 1;
        }
    }

    save_jobs(&jobs)?;
    render_registry(&config, &jobs)
}

pub fn regenerate(id: &str) -> Result<(), String> {
    let mut jobs = load_jobs();
    let job = jobs
        .get_mut(id)
        .ok_or_else(|| format!("unknown app icon job: {id}"))?;
    if job.source_type == "curated" {
        return Err("curated icons are repository-owned; replace the curated SVG instead".to_string());
    }
    job.state = "pending".to_string();
    job.attempts = 0;
    job.next_retry = 0;
    job.error.clear();
    let _ = fs::remove_file(semantic_path(id));
    save_jobs(&jobs)?;
    reconcile()
}

pub fn status_json() -> String {
    let config = load_config();
    let jobs = load_jobs();
    let mut counts = BTreeMap::<String, u64>::new();
    for job in jobs.values() { *counts.entry(job.state.clone()).or_default() += 1; }
    let count = |key: &str| counts.get(key).copied().unwrap_or(0);
    let items = jobs
        .values()
        .map(|job| {
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"state\":\"{}\",\"sourceType\":\"{}\",\"attempts\":{},\"nextRetry\":{},\"error\":\"{}\"}}",
                escape(&job.id),
                escape(&job.name),
                escape(&job.state),
                escape(&job.source_type),
                job.attempts,
                job.next_retry,
                escape(&job.error)
            )
        })
        .collect::<Vec<_>>();
    format!(
        "{{\"enabled\":{},\"mode\":\"{}\",\"tint\":\"{}\",\"provider\":\"{}\",\"credential\":\"{}\",\"model\":\"{}\",\"semanticFormatVersion\":{},\"generatorVersion\":\"{}\",\"counts\":{{\"pending\":{},\"processing\":{},\"prepared\":{},\"failed\":{},\"waitingForProvider\":{},\"waitingForQuota\":{},\"fallback\":{}}},\"jobs\":[{}]}}",
        bool_lit(config.enabled),
        escape(&config.mode),
        escape(&config.tint),
        escape(&config.provider),
        escape(&config.credential),
        escape(&config.model),
        FORMAT_VERSION,
        GENERATOR_VERSION,
        count("pending"),
        count("processing"),
        count("prepared"),
        count("failed"),
        count("waiting-for-provider"),
        count("waiting-for-quota"),
        count("fallback"),
        items.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_restricted_svg_and_standard_namespace() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M2 2h20v20H2z" fill="#fff"/></svg>"##;
        assert!(sanitize_svg(svg).is_ok());
    }

    #[test]
    fn strips_markdown_wrapper_by_extracting_svg() {
        let svg = "```svg\n<svg viewBox=\"0 0 24 24\"><circle cx=\"12\" cy=\"12\" r=\"10\"/></svg>\n```";
        assert!(sanitize_svg(svg).unwrap().starts_with("<svg"));
    }

    #[test]
    fn rejects_active_and_external_svg_content() {
        assert!(sanitize_svg(r#"<svg viewBox="0 0 1 1"><script>alert(1)</script></svg>"#).is_err());
        assert!(sanitize_svg(r#"<svg viewBox="0 0 1 1"><image href="https://example.com/x.png"/></svg>"#).is_err());
        assert!(sanitize_svg(r#"<svg viewBox="0 0 1 1"><path onclick="boom()"/></svg>"#).is_err());
        assert!(sanitize_svg(r#"<svg viewBox="0 0 1 1"><a href="https://example.com"><path/></a></svg>"#).is_err());
    }

    #[test]
    fn validates_modes_and_tints() {
        assert!(valid_mode("original"));
        assert!(valid_mode("tinted"));
        assert!(!valid_mode("sepia"));
        assert!(valid_tint("#aabbcc"));
        assert!(!valid_tint("blue"));
    }

    #[test]
    fn production_default_is_opt_in() {
        assert!(!Config::default().enabled);
        assert_eq!(Config::default().mode, "original");
    }
}
