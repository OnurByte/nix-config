use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const THEME_NAME: &str = "Vesper-Adaptive";
const SCHEMA_VERSION: u32 = 1;
const VALIDATOR_VERSION: u32 = 1;

#[derive(Clone)]
struct Config {
    enabled: bool,
    mode: String,
    provider: String,
    follow_palette: bool,
    scheme_mode: String,
    accent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "original".to_string(),
            provider: "openai".to_string(),
            follow_palette: true,
            scheme_mode: "dark".to_string(),
            accent: "#7aa2f7".to_string(),
        }
    }
}

#[derive(Clone)]
struct DesktopRecord {
    id: String,
    path: PathBuf,
    icon: String,
}

#[derive(Clone)]
struct IconCandidate {
    path: PathBuf,
    score: i64,
}

#[derive(Clone)]
struct InventoryItem {
    id: String,
    desktop_path: PathBuf,
    icon_key: String,
    source_path: Option<PathBuf>,
    fingerprint: String,
    source_kind: String,
    canonical_state: String,
    active: bool,
    excluded: bool,
    error: String,
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

fn data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

fn cache_root() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".cache"))
        .join("vesper/adaptive-icons")
}

fn canonical_root() -> PathBuf {
    data_home().join("vesper/adaptive-icons/canonical")
}

fn generations_root() -> PathBuf {
    data_home().join("vesper/adaptive-icons/themes")
}

fn theme_link() -> PathBuf {
    data_home().join("icons").join(THEME_NAME)
}

fn config_path() -> PathBuf {
    config_root().join("adaptive-icons.conf")
}

fn exclusions_path() -> PathBuf {
    config_root().join("adaptive-icons-excluded")
}

fn accent_path() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("caelestia/theme/vesper-icons")
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

fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | '+') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn write_atomic(path: &Path, data: impl AsRef<[u8]>) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid path: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("vesper"),
        std::process::id()
    ));
    fs::write(&tmp, data).map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())
}

fn command_output(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run {command}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{command} exited with {}", output.status.code().unwrap_or(-1))
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn command_success(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn valid_mode(value: &str) -> bool {
    matches!(
        value,
        "original" | "light" | "dark" | "tinted" | "clear" | "glass"
    )
}

fn valid_provider(value: &str) -> bool {
    matches!(value, "openai" | "anthropic" | "xai" | "openrouter" | "google")
}

fn valid_scheme_mode(value: &str) -> bool {
    matches!(value, "light" | "dark")
}

fn normalise_accent(value: &str) -> Option<String> {
    let value = value.trim().trim_start_matches('#');
    if value.len() == 6 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Some(format!("#{}", value.to_ascii_lowercase()))
    } else {
        None
    }
}

fn load_config() -> Config {
    let mut config = Config::default();
    let content = fs::read_to_string(config_path()).unwrap_or_default();
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "enabled" => config.enabled = value == "1" || value.eq_ignore_ascii_case("true"),
            "mode" if valid_mode(value) => config.mode = value.to_string(),
            "provider" if valid_provider(value) => config.provider = value.to_string(),
            "followPalette" => {
                config.follow_palette = value == "1" || value.eq_ignore_ascii_case("true")
            }
            "schemeMode" if valid_scheme_mode(value) => config.scheme_mode = value.to_string(),
            "accent" => {
                if let Some(accent) = normalise_accent(value) {
                    config.accent = accent;
                }
            }
            _ => {}
        }
    }

    if content.is_empty() {
        let legacy = state_root().join("enabled");
        if let Ok(value) = fs::read_to_string(legacy) {
            config.enabled = value.trim() == "1";
        }
    }

    config
}

fn save_config(config: &Config) -> Result<(), String> {
    let body = format!(
        "enabled={}\nmode={}\nprovider={}\nfollowPalette={}\nschemeMode={}\naccent={}\n",
        if config.enabled { 1 } else { 0 },
        config.mode,
        config.provider,
        if config.follow_palette { 1 } else { 0 },
        config.scheme_mode,
        config.accent
    );
    write_atomic(&config_path(), body)
}

fn load_exclusions() -> BTreeSet<String> {
    fs::read_to_string(exclusions_path())
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn save_exclusions(values: &BTreeSet<String>) -> Result<(), String> {
    let mut body = values.iter().cloned().collect::<Vec<_>>().join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    write_atomic(&exclusions_path(), body)
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

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn effective_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    push_unique(&mut dirs, data_home());

    let xdg_dirs = env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for path in xdg_dirs.split(':').filter(|path| !path.is_empty()) {
        push_unique(&mut dirs, PathBuf::from(path));
    }

    if let Ok(profiles) = env::var("NIX_PROFILES") {
        for profile in profiles.split_whitespace() {
            push_unique(&mut dirs, PathBuf::from(profile).join("share"));
        }
    }

    let user_profile = home().join(".nix-profile/share");
    if user_profile.exists() {
        push_unique(&mut dirs, user_profile);
    }

    if let Ok(user) = env::var("USER") {
        let profile = PathBuf::from("/etc/profiles/per-user")
            .join(user)
            .join("share");
        if profile.exists() {
            push_unique(&mut dirs, profile);
        }
    }

    for path in [
        home().join(".local/share/flatpak/exports/share"),
        PathBuf::from("/var/lib/flatpak/exports/share"),
        PathBuf::from("/run/current-system/sw/share"),
    ] {
        if path.exists() {
            push_unique(&mut dirs, path);
        }
    }

    dirs
}

fn collect_desktop_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(PathBuf, String)>,
) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_desktop_files(root, &path, files);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("desktop"))
            .unwrap_or(false)
        {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let id = relative
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "-");
            files.push((path, id));
        }
    }
}

fn parse_desktop(path: &Path, id: String) -> Option<DesktopRecord> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_desktop = false;
    let mut kind = String::new();
    let mut hidden = false;
    let mut no_display = false;
    let mut icon = String::new();

    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "Type" => kind = value.trim().to_string(),
            "Hidden" => hidden = value.trim().eq_ignore_ascii_case("true"),
            "NoDisplay" => no_display = value.trim().eq_ignore_ascii_case("true"),
            "Icon" => icon = value.trim().to_string(),
            _ => {}
        }
    }

    if kind != "Application" || hidden || no_display || icon.is_empty() {
        return None;
    }

    Some(DesktopRecord {
        id,
        path: path.to_path_buf(),
        icon,
    })
}

fn discover_desktops(data_dirs: &[PathBuf]) -> Vec<DesktopRecord> {
    let mut records = BTreeMap::<String, DesktopRecord>::new();

    for data_dir in data_dirs {
        let applications = data_dir.join("applications");
        if !applications.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        collect_desktop_files(&applications, &applications, &mut files);
        files.sort_by(|a, b| a.1.cmp(&b.1));
        for (path, id) in files {
            if records.contains_key(&id) {
                continue;
            }
            if let Some(record) = parse_desktop(&path, id.clone()) {
                records.insert(id, record);
            }
        }
    }

    records.into_values().collect()
}

fn raster_size_score(path: &Path) -> i64 {
    for component in path.components() {
        let text = component.as_os_str().to_string_lossy();
        let Some((left, right)) = text.split_once('x') else {
            continue;
        };
        if left == right {
            if let Ok(size) = left.parse::<i64>() {
                return size.min(4096);
            }
        }
    }
    0
}

fn icon_score(path: &Path, root_rank: usize) -> i64 {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let format = match extension.as_str() {
        "svg" | "svgz" => 100_000,
        "png" => 70_000,
        "webp" => 60_000,
        "xpm" => 20_000,
        _ => 0,
    };
    let hicolor = if path.to_string_lossy().contains("/hicolor/") {
        20_000
    } else {
        0
    };
    let rank = 10_000_i64.saturating_sub(root_rank as i64 * 100);
    format + hicolor + raster_size_score(path) + rank
}

fn index_icon_tree(
    root: &Path,
    current: &Path,
    root_rank: usize,
    depth: usize,
    index: &mut BTreeMap<String, Vec<IconCandidate>>,
) {
    if depth > 12 {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            index_icon_tree(root, &path, root_rank, depth + 1, index);
            continue;
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(extension.as_str(), "svg" | "svgz" | "png" | "webp" | "xpm") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let _ = root;
        index
            .entry(stem.to_string())
            .or_default()
            .push(IconCandidate {
                score: icon_score(&path, root_rank),
                path,
            });
    }
}

fn build_icon_index(data_dirs: &[PathBuf]) -> BTreeMap<String, Vec<IconCandidate>> {
    let mut index = BTreeMap::<String, Vec<IconCandidate>>::new();

    for (rank, data_dir) in data_dirs.iter().enumerate() {
        for root in [data_dir.join("icons"), data_dir.join("pixmaps")] {
            if root.is_dir() {
                index_icon_tree(&root, &root, rank, 0, &mut index);
            }
        }
    }

    for candidates in index.values_mut() {
        candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
    }

    index
}

fn resolve_icon(
    icon: &str,
    index: &BTreeMap<String, Vec<IconCandidate>>,
) -> Option<PathBuf> {
    let path = PathBuf::from(icon);
    if path.is_absolute() && path.is_file() {
        return Some(path);
    }

    let stem = Path::new(icon)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(icon);
    index
        .get(stem)
        .and_then(|candidates| candidates.first())
        .map(|candidate| candidate.path.clone())
}

fn source_kind(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("unknown")
        .to_ascii_lowercase()
}

fn fingerprint(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .filter(|value| value.len() == 64)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("invalid sha256sum output for {}", path.display()))
}

fn extract_attr_ci(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = name.to_ascii_lowercase();
    let mut offset = 0;

    while let Some(index) = lower[offset..].find(&needle) {
        let index = offset + index;
        let before_ok = index == 0
            || !lower.as_bytes()[index - 1].is_ascii_alphanumeric();
        let after_index = index + needle.len();
        let after_ok = after_index >= lower.len()
            || !lower.as_bytes()[after_index].is_ascii_alphanumeric();
        if before_ok && after_ok {
            let rest = &tag[after_index..];
            let eq = rest.find('=')?;
            if !rest[..eq].trim().is_empty() {
                offset = after_index;
                continue;
            }
            let mut chars = rest[eq + 1..].char_indices();
            let (quote_pos, quote) = chars.find(|(_, ch)| !ch.is_whitespace())?;
            if quote != '"' && quote != '\'' {
                return None;
            }
            let value_start = eq + 1 + quote_pos + quote.len_utf8();
            let value_rest = &rest[value_start..];
            let end = value_rest.find(quote)?;
            return Some(value_rest[..end].to_string());
        }
        offset = after_index;
    }
    None
}

fn svg_root_tag(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let start = lower.find("<svg")?;
    let end = content[start..].find('>')? + start;
    Some(content[start..=end].to_string())
}

fn parse_viewbox(content: &str) -> Option<String> {
    let tag = svg_root_tag(content)?;
    if let Some(viewbox) = extract_attr_ci(&tag, "viewBox") {
        let values = viewbox
            .replace(',', " ")
            .split_whitespace()
            .filter_map(|value| value.parse::<f64>().ok())
            .collect::<Vec<_>>();
        if values.len() == 4
            && values[2].is_finite()
            && values[3].is_finite()
            && values[2] > 0.0
            && values[3] > 0.0
            && values[2] <= 16384.0
            && values[3] <= 16384.0
        {
            return Some(format!("{} {} {} {}", values[0], values[1], values[2], values[3]));
        }
    }

    let width = extract_attr_ci(&tag, "width")?;
    let height = extract_attr_ci(&tag, "height")?;
    let parse_number = |value: &str| -> Option<f64> {
        let numeric = value
            .chars()
            .take_while(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+'))
            .collect::<String>();
        numeric.parse::<f64>().ok()
    };
    let width = parse_number(&width)?;
    let height = parse_number(&height)?;
    if width > 0.0 && height > 0.0 && width <= 16384.0 && height <= 16384.0 {
        Some(format!("0 0 {width} {height}"))
    } else {
        None
    }
}

fn unsafe_svg_reason(content: &str) -> Option<&'static str> {
    if content.len() > 2_000_000 {
        return Some("svg-too-large");
    }
    let lower = content.to_ascii_lowercase();
    let forbidden = [
        ("<script", "script"),
        ("<foreignobject", "foreign-object"),
        ("<image", "embedded-image"),
        ("javascript:", "javascript-url"),
        ("data:image", "embedded-raster"),
        ("http://", "external-url"),
        ("https://", "external-url"),
        ("file://", "external-file"),
        ("@import", "css-import"),
        ("@font-face", "external-font"),
        ("<iframe", "foreign-frame"),
        ("<audio", "foreign-media"),
        ("<video", "foreign-media"),
        (" onload=", "event-handler"),
        (" onclick=", "event-handler"),
        (" onerror=", "event-handler"),
        (" onmouseover=", "event-handler"),
        (" onbegin=", "event-handler"),
        (" onend=", "event-handler"),
    ];
    for (needle, reason) in forbidden {
        if lower.contains(needle) {
            return Some(reason);
        }
    }
    if lower.matches('<').count() > 12_000 {
        return Some("node-count");
    }
    None
}

fn validate_svg(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let lower = content.to_ascii_lowercase();
    if !lower.contains("<svg") || !lower.contains("</svg>") {
        return Err("missing-svg-root".to_string());
    }
    if let Some(reason) = unsafe_svg_reason(&content) {
        return Err(reason.to_string());
    }
    let viewbox = parse_viewbox(&content).ok_or_else(|| "invalid-viewbox".to_string())?;

    let result = Command::new("xmllint")
        .args(["--noout", "--nonet"])
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("failed to run xmllint: {error}"))?;
    if !result.status.success() {
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if message.is_empty() {
            "malformed-xml".to_string()
        } else {
            format!("malformed-xml: {message}")
        });
    }

    let render_root = cache_root().join("validation");
    fs::create_dir_all(&render_root).map_err(|error| error.to_string())?;
    for size in [16, 32, 64, 128] {
        let output_path = render_root.join(format!(
            "{}-{size}.png",
            std::process::id()
        ));
        let result = Command::new("rsvg-convert")
            .arg("-w")
            .arg(size.to_string())
            .arg("-h")
            .arg(size.to_string())
            .arg("-o")
            .arg(&output_path)
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| format!("failed to run rsvg-convert: {error}"))?;
        if !result.status.success() {
            let _ = fs::remove_file(&output_path);
            let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
            return Err(if message.is_empty() {
                format!("render-{size}-failed")
            } else {
                format!("render-{size}-failed: {message}")
            });
        }
        let size_ok = fs::metadata(&output_path)
            .map(|metadata| metadata.len() > 64)
            .unwrap_or(false);
        let _ = fs::remove_file(&output_path);
        if !size_ok {
            return Err(format!("render-{size}-empty"));
        }
    }

    Ok(viewbox)
}

fn canonical_dir(id: &str, fingerprint: &str) -> PathBuf {
    canonical_root()
        .join(safe_name(id))
        .join(fingerprint)
}

fn canonicalise_svg(
    record: &DesktopRecord,
    source: &Path,
    fingerprint: &str,
) -> Result<PathBuf, String> {
    let dir = canonical_dir(&record.id, fingerprint);
    let canonical = dir.join("canonical.svg");
    let metadata = dir.join("metadata.json");

    if canonical.is_file() && metadata.is_file() {
        return Ok(canonical);
    }

    let viewbox = validate_svg(source)?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let tmp = dir.join(format!(".canonical.{}.tmp", std::process::id()));
    fs::copy(source, &tmp).map_err(|error| error.to_string())?;
    validate_svg(&tmp)?;
    fs::rename(&tmp, &canonical).map_err(|error| error.to_string())?;

    let body = format!(
        "{{\"schemaVersion\":{},\"validatorVersion\":{},\"desktopId\":\"{}\",\"sourceFingerprint\":\"{}\",\"sourcePath\":\"{}\",\"sourceKind\":\"svg\",\"viewBox\":\"{}\",\"provenance\":\"local-vector\",\"compatibilityDerived\":false,\"validation\":\"passed\"}}\n",
        SCHEMA_VERSION,
        VALIDATOR_VERSION,
        json_escape(&record.id),
        json_escape(fingerprint),
        json_escape(&source.to_string_lossy()),
        json_escape(&viewbox)
    );
    write_atomic(&metadata, body)?;
    Ok(canonical)
}

fn existing_canonical(id: &str, fingerprint: &str) -> Option<PathBuf> {
    let dir = canonical_dir(id, fingerprint);
    let canonical = dir.join("canonical.svg");
    let metadata = dir.join("metadata.json");
    if canonical.is_file() && metadata.is_file() {
        Some(canonical)
    } else {
        None
    }
}

fn svg_inner_and_viewbox(path: &Path) -> Result<(String, String), String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let viewbox = parse_viewbox(&content).ok_or_else(|| "invalid-viewbox".to_string())?;
    let lower = content.to_ascii_lowercase();
    let start = lower
        .find("<svg")
        .ok_or_else(|| "missing-svg-root".to_string())?;
    let open_end = content[start..]
        .find('>')
        .ok_or_else(|| "malformed-svg-root".to_string())?
        + start;
    let close = lower
        .rfind("</svg>")
        .ok_or_else(|| "missing-svg-close".to_string())?;
    if close <= open_end {
        return Err("malformed-svg-body".to_string());
    }
    Ok((content[open_end + 1..close].to_string(), viewbox))
}

fn accent_rgb(accent: &str) -> (f64, f64, f64) {
    let hex = accent.trim_start_matches('#');
    if hex.len() != 6 {
        return (0.478, 0.635, 0.969);
    }
    let parse = |range: std::ops::Range<usize>| -> f64 {
        u8::from_str_radix(&hex[range], 16)
            .map(|value| value as f64 / 255.0)
            .unwrap_or(0.5)
    };
    (parse(0..2), parse(2..4), parse(4..6))
}

fn colour_matrix(accent: &str) -> String {
    let (r, g, b) = accent_rgb(accent);
    format!(
        "0 0 0 0 {r:.5} 0 0 0 0 {g:.5} 0 0 0 0 {b:.5} 0 0 0 1 0"
    )
}

fn nested_svg(inner: &str, viewbox: &str, x: i32, y: i32, size: i32) -> String {
    format!(
        "<svg x=\"{x}\" y=\"{y}\" width=\"{size}\" height=\"{size}\" viewBox=\"{}\" preserveAspectRatio=\"xMidYMid meet\">{}</svg>",
        json_escape(viewbox),
        inner
    )
}

fn compile_icon(canonical: &Path, config: &Config) -> Result<String, String> {
    let (inner, viewbox) = svg_inner_and_viewbox(canonical)?;
    if config.mode == "original" {
        return Ok(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\">{}</svg>\n",
            nested_svg(&inner, &viewbox, 0, 0, 1024)
        ));
    }

    let glyph = nested_svg(&inner, &viewbox, 136, 136, 752);
    let matrix = colour_matrix(&config.accent);
    let body = match config.mode.as_str() {
        "light" => format!(
            "<rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"#f7f7f8\" stroke=\"#ffffff\" stroke-width=\"10\"/><g>{glyph}</g>"
        ),
        "dark" => format!(
            "<rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"#171719\" stroke=\"#38383d\" stroke-width=\"10\"/><g>{glyph}</g>"
        ),
        "tinted" => format!(
            "<defs><filter id=\"vesperTint\" color-interpolation-filters=\"sRGB\"><feColorMatrix type=\"matrix\" values=\"{matrix}\"/></filter></defs><rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"{}\" fill-opacity=\"0.20\" stroke=\"{}\" stroke-opacity=\"0.55\" stroke-width=\"10\"/><g filter=\"url(#vesperTint)\">{glyph}</g>",
            config.accent,
            config.accent
        ),
        "clear" => {
            let foreground = if config.scheme_mode == "light" {
                "#202124"
            } else {
                "#ffffff"
            };
            let clear_matrix = colour_matrix(foreground);
            format!(
                "<defs><filter id=\"vesperClear\" color-interpolation-filters=\"sRGB\"><feColorMatrix type=\"matrix\" values=\"{clear_matrix}\"/></filter></defs><rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"{}\" fill-opacity=\"0.10\" stroke=\"{}\" stroke-opacity=\"0.28\" stroke-width=\"8\"/><g filter=\"url(#vesperClear)\">{glyph}</g>",
                if config.scheme_mode == "light" { "#ffffff" } else { "#d8d9de" },
                foreground
            )
        }
        "glass" => format!(
            "<defs><linearGradient id=\"vesperGlass\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.46\"/><stop offset=\"0.42\" stop-color=\"{}\" stop-opacity=\"0.18\"/><stop offset=\"1\" stop-color=\"#ffffff\" stop-opacity=\"0.08\"/></linearGradient><linearGradient id=\"vesperSpec\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.72\"/><stop offset=\"0.45\" stop-color=\"#ffffff\" stop-opacity=\"0.08\"/><stop offset=\"1\" stop-color=\"#ffffff\" stop-opacity=\"0\"/></linearGradient></defs><rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"url(#vesperGlass)\" stroke=\"#ffffff\" stroke-opacity=\"0.42\" stroke-width=\"8\"/><rect x=\"116\" y=\"116\" width=\"792\" height=\"396\" rx=\"172\" fill=\"url(#vesperSpec)\"/>{glyph}",
            config.accent
        ),
        _ => glyph,
    };

    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\">{body}</svg>\n"
    ))
}

fn icon_theme_name(icon_key: &str) -> Option<String> {
    let path = Path::new(icon_key);
    if path.is_absolute() || icon_key.contains('/') {
        return None;
    }
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or(icon_key);
    if stem.is_empty() {
        return None;
    }
    Some(safe_name(stem))
}

fn fallback_themes(config: &Config) -> &'static str {
    if config.scheme_mode == "light" {
        "Papirus-Light,Papirus,hicolor"
    } else {
        "Papirus-Dark,Papirus,hicolor"
    }
}

fn generation_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{millis}-{}", std::process::id())
}

fn switch_theme_generation(generation: &Path) -> Result<(), String> {
    let link = theme_link();
    let parent = link
        .parent()
        .ok_or_else(|| "invalid theme link".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let next = parent.join(format!(".{THEME_NAME}.next-{}", std::process::id()));
    let _ = fs::remove_file(&next);
    symlink(generation, &next).map_err(|error| error.to_string())?;
    fs::rename(&next, &link).map_err(|error| error.to_string())
}

fn gc_generations(current: &Path) {
    let Ok(entries) = fs::read_dir(generations_root()) else {
        return;
    };
    let mut dirs = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    dirs.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(UNIX_EPOCH)
    });
    dirs.reverse();

    let mut kept = 0usize;
    for path in dirs {
        if path == current || kept < 2 {
            kept += 1;
            continue;
        }
        let _ = fs::remove_dir_all(path);
    }
}

fn compile_theme(items: &mut [InventoryItem], config: &Config) -> Result<usize, String> {
    let generation = generations_root().join(generation_id());
    let apps_dir = generation.join("scalable/apps");
    fs::create_dir_all(&apps_dir).map_err(|error| error.to_string())?;

    let index = format!(
        "[Icon Theme]\nName=Vesper Adaptive\nComment=Generated Vesper application icons\nInherits={}\nDirectories=scalable/apps\n\n[scalable/apps]\nSize=128\nMinSize=16\nMaxSize=1024\nType=Scalable\nContext=Applications\n",
        fallback_themes(config)
    );
    fs::write(generation.join("index.theme"), index).map_err(|error| error.to_string())?;

    let mut written = BTreeSet::<String>::new();
    let mut active = 0usize;

    if config.enabled {
        for item in items.iter_mut() {
            if item.excluded || item.canonical_state != "validated" {
                continue;
            }
            let Some(source) = existing_canonical(&item.id, &item.fingerprint) else {
                continue;
            };
            let Some(primary) = icon_theme_name(&item.icon_key) else {
                continue;
            };
            let compiled = match compile_icon(&source, config) {
                Ok(compiled) => compiled,
                Err(error) => {
                    item.error = format!("compile: {error}");
                    continue;
                }
            };

            let mut aliases = BTreeSet::new();
            aliases.insert(primary);
            if let Some(id) = item.id.strip_suffix(".desktop") {
                aliases.insert(safe_name(id));
            }

            let mut any = false;
            for alias in aliases {
                if !written.insert(alias.clone()) {
                    continue;
                }
                let target = apps_dir.join(format!("{alias}.svg"));
                fs::write(target, &compiled).map_err(|error| error.to_string())?;
                any = true;
            }
            if any {
                item.active = true;
                active += 1;
            }
        }
    }

    switch_theme_generation(&generation)?;
    gc_generations(&generation);
    Ok(active)
}

fn retire_legacy_queue() {
    let queue = state_root().join("queue");
    if !queue.exists() {
        return;
    }
    let retired = state_root().join("queue.retired");
    if retired.exists() {
        let _ = fs::remove_dir_all(&queue);
    } else {
        let _ = fs::rename(queue, retired);
    }
}

fn build_inventory(config: &Config) -> Vec<InventoryItem> {
    let data_dirs = effective_data_dirs();
    let records = discover_desktops(&data_dirs);
    let index = build_icon_index(&data_dirs);
    let exclusions = load_exclusions();
    let mut items = Vec::with_capacity(records.len());

    for record in records {
        let excluded = exclusions.contains(&record.id);
        let source_path = resolve_icon(&record.icon, &index);
        let mut item = InventoryItem {
            id: record.id.clone(),
            desktop_path: record.path.clone(),
            icon_key: record.icon.clone(),
            source_path: source_path.clone(),
            fingerprint: String::new(),
            source_kind: "missing".to_string(),
            canonical_state: "missing".to_string(),
            active: false,
            excluded,
            error: String::new(),
        };

        let Some(source) = source_path else {
            item.error = "source-icon-not-found".to_string();
            items.push(item);
            continue;
        };

        item.source_kind = source_kind(&source);
        let fingerprint = match fingerprint(&source) {
            Ok(value) => value,
            Err(error) => {
                item.canonical_state = "failed".to_string();
                item.error = error;
                items.push(item);
                continue;
            }
        };
        item.fingerprint = fingerprint.clone();

        if excluded {
            item.canonical_state = "excluded".to_string();
            items.push(item);
            continue;
        }

        if existing_canonical(&record.id, &fingerprint).is_some() {
            item.canonical_state = "validated".to_string();
            items.push(item);
            continue;
        }

        match item.source_kind.as_str() {
            "svg" => match canonicalise_svg(&record, &source, &fingerprint) {
                Ok(_) => item.canonical_state = "validated".to_string(),
                Err(error) => {
                    item.canonical_state = "pending-ai".to_string();
                    item.error = format!("local-svg: {error}");
                }
            },
            "svgz" => {
                item.canonical_state = "pending-ai".to_string();
                item.error = "compressed-svg-requires-normalization".to_string();
            }
            "png" | "webp" | "xpm" => {
                item.canonical_state = "pending-ai".to_string();
            }
            _ => {
                item.canonical_state = "failed".to_string();
                item.error = "unsupported-source-format".to_string();
            }
        }

        items.push(item);
    }

    let _ = config;
    items
}

fn inventory_json(items: &[InventoryItem]) -> String {
    let records = items
        .iter()
        .map(|item| {
            format!(
                "{{\"id\":\"{}\",\"desktopPath\":\"{}\",\"iconKey\":\"{}\",\"sourcePath\":\"{}\",\"fingerprint\":\"{}\",\"sourceKind\":\"{}\",\"canonicalState\":\"{}\",\"active\":{},\"excluded\":{},\"error\":\"{}\"}}",
                json_escape(&item.id),
                json_escape(&item.desktop_path.to_string_lossy()),
                json_escape(&item.icon_key),
                json_escape(
                    &item
                        .source_path
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_default()
                ),
                json_escape(&item.fingerprint),
                json_escape(&item.source_kind),
                json_escape(&item.canonical_state),
                if item.active { "true" } else { "false" },
                if item.excluded { "true" } else { "false" },
                json_escape(&item.error)
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]\n", records.join(","))
}

fn inventory_tsv(items: &[InventoryItem]) -> String {
    let clean = |value: &str| {
        value
            .chars()
            .map(|ch| if matches!(ch, '\t' | '\n' | '\r') { ' ' } else { ch })
            .collect::<String>()
    };
    let mut body = String::new();
    for item in items {
        body.push_str(&clean(&item.id));
        body.push('\t');
        body.push_str(&clean(&item.icon_key));
        body.push('\t');
        body.push_str(&clean(
            &item
                .source_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
        ));
        body.push('\t');
        body.push_str(&item.fingerprint);
        body.push('\t');
        body.push_str(&item.source_kind);
        body.push('\t');
        body.push_str(&item.canonical_state);
        body.push('\t');
        body.push_str(if item.active { "1" } else { "0" });
        body.push('\t');
        body.push_str(if item.excluded { "1" } else { "0" });
        body.push('\t');
        body.push_str(&clean(&item.error));
        body.push('\n');
    }
    body
}

fn status_json(items: &[InventoryItem], config: &Config, current: &str) -> String {
    let discovered = items.len();
    let canonical = items
        .iter()
        .filter(|item| item.canonical_state == "validated")
        .count();
    let pending = items
        .iter()
        .filter(|item| item.canonical_state == "pending-ai")
        .count();
    let failed = items
        .iter()
        .filter(|item| item.canonical_state == "failed" || item.canonical_state == "missing")
        .count();
    let excluded = items.iter().filter(|item| item.excluded).count();
    let active = items.iter().filter(|item| item.active).count();

    format!(
        "{{\"enabled\":{},\"mode\":\"{}\",\"provider\":\"{}\",\"providerConfigured\":{},\"followPalette\":{},\"schemeMode\":\"{}\",\"accent\":\"{}\",\"theme\":\"{}\",\"discovered\":{},\"canonical\":{},\"pending\":{},\"failed\":{},\"excluded\":{},\"active\":{},\"current\":\"{}\",\"aiTransport\":\"pending\"}}\n",
        if config.enabled { "true" } else { "false" },
        json_escape(&config.mode),
        json_escape(&config.provider),
        if provider_configured(&config.provider) { "true" } else { "false" },
        if config.follow_palette { "true" } else { "false" },
        json_escape(&config.scheme_mode),
        json_escape(&config.accent),
        THEME_NAME,
        discovered,
        canonical,
        pending,
        failed,
        excluded,
        active,
        json_escape(current)
    )
}

fn write_state(items: &[InventoryItem], config: &Config, current: &str) -> Result<(), String> {
    fs::create_dir_all(state_root()).map_err(|error| error.to_string())?;
    write_atomic(&state_root().join("inventory.json"), inventory_json(items))?;
    write_atomic(&state_root().join("inventory.tsv"), inventory_tsv(items))?;
    write_atomic(
        &state_root().join("status.json"),
        status_json(items, config, current),
    )?;
    Ok(())
}

fn reconcile() -> Result<String, String> {
    retire_legacy_queue();
    let config = load_config();
    let mut items = build_inventory(&config);
    let active = compile_theme(&mut items, &config)?;
    write_state(&items, &config, "idle")?;
    Ok(format!(
        "discovered={} canonical={} active={}",
        items.len(),
        items
            .iter()
            .filter(|item| item.canonical_state == "validated")
            .count(),
        active
    ))
}

fn ensure_theme() -> Result<(), String> {
    if theme_link().exists() {
        return Ok(());
    }
    let config = load_config();
    let mut items = Vec::new();
    compile_theme(&mut items, &config)?;
    write_state(&items, &config, "idle")
}

fn print_status() -> Result<(), String> {
    let config = load_config();
    let path = state_root().join("status.json");
    if path.is_file() {
        let mut text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let current = format!("\"enabled\":{}", if config.enabled { "true" } else { "false" });
        if !text.contains(&current) {
            reconcile()?;
            text = fs::read_to_string(path).map_err(|error| error.to_string())?;
        }
        print!("{text}");
        Ok(())
    } else {
        reconcile()?;
        let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
        print!("{text}");
        Ok(())
    }
}

fn set_enabled(enabled: bool) -> Result<(), String> {
    let mut config = load_config();
    config.enabled = enabled;
    save_config(&config)?;
    reconcile()?;
    Ok(())
}

fn set_mode(mode: &str) -> Result<(), String> {
    if !valid_mode(mode) {
        return Err(format!("unsupported icon mode: {mode}"));
    }
    let mut config = load_config();
    config.mode = mode.to_string();
    save_config(&config)?;
    reconcile()?;
    Ok(())
}

fn set_provider(provider: &str) -> Result<(), String> {
    if !valid_provider(provider) {
        return Err(format!("unsupported provider: {provider}"));
    }
    let mut config = load_config();
    config.provider = provider.to_string();
    save_config(&config)?;
    reconcile()?;
    Ok(())
}

fn set_follow_palette(enabled: bool) -> Result<(), String> {
    let mut config = load_config();
    config.follow_palette = enabled;
    if enabled {
        if let Ok(value) = fs::read_to_string(accent_path()) {
            if let Some(accent) = normalise_accent(&value) {
                config.accent = accent;
            }
        }
    }
    save_config(&config)?;
    reconcile()?;
    Ok(())
}

fn sync_theme(mode: &str) -> Result<(), String> {
    if !valid_scheme_mode(mode) {
        return Err(format!("unsupported scheme mode: {mode}"));
    }
    let mut config = load_config();
    config.scheme_mode = mode.to_string();
    if config.follow_palette {
        if let Ok(value) = fs::read_to_string(accent_path()) {
            if let Some(accent) = normalise_accent(&value) {
                config.accent = accent;
            }
        }
    }
    save_config(&config)?;
    reconcile()?;
    Ok(())
}

fn load_inventory_from_tsv() -> Vec<InventoryItem> {
    let content = fs::read_to_string(state_root().join("inventory.tsv")).unwrap_or_default();
    let mut items = Vec::new();
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 {
            continue;
        }
        items.push(InventoryItem {
            id: parts[0].to_string(),
            desktop_path: PathBuf::new(),
            icon_key: parts[1].to_string(),
            source_path: if parts[2].is_empty() {
                None
            } else {
                Some(PathBuf::from(parts[2]))
            },
            fingerprint: parts[3].to_string(),
            source_kind: parts[4].to_string(),
            canonical_state: parts[5].to_string(),
            active: parts[6] == "1",
            excluded: parts[7] == "1",
            error: parts[8].to_string(),
        });
    }
    items
}

fn print_app_status(id: &str) -> Result<(), String> {
    let items = load_inventory_from_tsv();
    let item = items
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(|| format!("application not in adaptive icon inventory: {id}"))?;
    println!(
        "{{\"id\":\"{}\",\"iconKey\":\"{}\",\"sourcePath\":\"{}\",\"sourceKind\":\"{}\",\"fingerprint\":\"{}\",\"canonicalState\":\"{}\",\"active\":{},\"excluded\":{},\"error\":\"{}\"}}",
        json_escape(&item.id),
        json_escape(&item.icon_key),
        json_escape(
            &item
                .source_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default()
        ),
        json_escape(&item.source_kind),
        json_escape(&item.fingerprint),
        json_escape(&item.canonical_state),
        if item.active { "true" } else { "false" },
        if item.excluded { "true" } else { "false" },
        json_escape(&item.error)
    );
    Ok(())
}

fn set_excluded(id: &str, excluded: bool) -> Result<(), String> {
    let mut values = load_exclusions();
    if excluded {
        values.insert(id.to_string());
    } else {
        values.remove(id);
    }
    save_exclusions(&values)?;
    reconcile()?;
    Ok(())
}

fn retry_app(id: &str) -> Result<(), String> {
    let path = canonical_root().join(safe_name(id));
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    reconcile()?;
    Ok(())
}

fn rebuild_canonical() -> Result<(), String> {
    let root = canonical_root();
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    }
    reconcile()?;
    Ok(())
}

fn watch_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for data_dir in effective_data_dirs() {
        for path in [
            data_dir.join("applications"),
            data_dir.join("icons/hicolor"),
            data_dir.join("pixmaps"),
        ] {
            if path.is_dir() {
                push_unique(&mut paths, path);
            }
        }
    }
    paths
}

fn daemon() -> Result<(), String> {
    let _ = reconcile();
    loop {
        let paths = watch_paths();
        if paths.is_empty() {
            thread::sleep(Duration::from_secs(60));
            let _ = reconcile();
            continue;
        }

        let mut command = Command::new("inotifywait");
        command.args([
            "-q",
            "-r",
            "-e",
            "close_write,create,delete,move,attrib",
            "-t",
            "900",
            "--",
        ]);
        for path in &paths {
            command.arg(path);
        }
        command.stdout(Stdio::null()).stderr(Stdio::null());

        match command.status() {
            Ok(status) if status.success() => {
                thread::sleep(Duration::from_secs(2));
                let _ = reconcile();
            }
            Ok(status) if status.code() == Some(2) => {
                let _ = reconcile();
            }
            Ok(_) | Err(_) => {
                thread::sleep(Duration::from_secs(15));
                let _ = reconcile();
            }
        }
    }
}

fn usage() -> ! {
    eprintln!(
        "vesper-icon-engine\n\
         commands:\n\
           status\n\
           enable|disable\n\
           reconcile\n\
           ensure-theme\n\
           mode original|light|dark|tinted|clear|glass\n\
           provider openai|anthropic|xai|openrouter|google\n\
           follow-palette on|off\n\
           sync-theme light|dark\n\
           app-status <desktop-id>\n\
           app-exclude <desktop-id> on|off\n\
           app-retry <desktop-id>\n\
           rebuild-canonical\n\
           daemon"
    );
    std::process::exit(2);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [command] if command == "status" => print_status(),
        [command] if command == "enable" => set_enabled(true),
        [command] if command == "disable" => set_enabled(false),
        [command] if command == "reconcile" => reconcile().map(|summary| println!("{summary}")),
        [command] if command == "ensure-theme" => ensure_theme(),
        [command, mode] if command == "mode" => set_mode(mode),
        [command, provider] if command == "provider" => set_provider(provider),
        [command, value] if command == "follow-palette" => match value.as_str() {
            "on" => set_follow_palette(true),
            "off" => set_follow_palette(false),
            _ => usage(),
        },
        [command, mode] if command == "sync-theme" => sync_theme(mode),
        [command, id] if command == "app-status" => print_app_status(id),
        [command, id, value] if command == "app-exclude" => match value.as_str() {
            "on" => set_excluded(id, true),
            "off" => set_excluded(id, false),
            _ => usage(),
        },
        [command, id] if command == "app-retry" => retry_app(id),
        [command] if command == "rebuild-canonical" => rebuild_canonical(),
        [command] if command == "daemon" => daemon(),
        _ => usage(),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
