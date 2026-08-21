use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const THEME_NAME: &str = "Vesper-Adaptive";
const SCHEMA_VERSION: u32 = 1;
const VALIDATOR_VERSION: u32 = 1;
const GRID_REVISION: &str = "vesper-public-2026-r1";

#[derive(Clone, Copy)]
struct GridGeometry {
    content_x: i32,
    content_size: i32,
    enclosure_x: i32,
    enclosure_size: i32,
    enclosure_radius: i32,
    needs_enclosure: bool,
}

#[derive(Clone)]
struct Config {
    enabled: bool,
    mode: String,
    material: String,
    provider: String,
    remote_consent: bool,
    follow_palette: bool,
    scheme_mode: String,
    accent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "automatic".to_string(),
            material: "standard".to_string(),
            provider: "openai".to_string(),
            remote_consent: false,
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

fn db_path() -> PathBuf {
    state_root().join("state.sqlite3")
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

fn is_vesper_owned_source(path: &Path) -> bool {
    let owned_roots = [
        theme_link(),
        canonical_root(),
        generations_root(),
        cache_root(),
        data_home().join("vesper/adaptive-icons/exports"),
    ];
    if owned_roots.iter().any(|root| path.starts_with(root)) {
        return true;
    }

    // User-facing exports live in Downloads. Keep them out of provenance even
    // when a desktop entry later points at an exported file by absolute path.
    path.to_string_lossy()
        .split(std::path::MAIN_SEPARATOR)
        .any(|part| part.starts_with("Vesper-Adaptive-Icons-"))
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

fn valid_mode(value: &str) -> bool {
    matches!(
        value,
        "automatic" | "default" | "dark" | "tinted" | "clear" | "original" | "light"
    )
}

fn valid_material(value: &str) -> bool {
    matches!(value, "standard" | "glass")
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
            "mode" if value == "glass" => {
                config.mode = "clear".to_string();
                config.material = "glass".to_string();
            }
            "mode" if value == "light" => config.mode = "default".to_string(),
            "mode" if valid_mode(value) => config.mode = value.to_string(),
            "material" if valid_material(value) => config.material = value.to_string(),
            "provider" if valid_provider(value) => config.provider = value.to_string(),
            "remoteConsent" => {
                config.remote_consent = value == "1" || value.eq_ignore_ascii_case("true")
            }
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
        "enabled={}\nmode={}\nmaterial={}\nprovider={}\nremoteConsent={}\nfollowPalette={}\nschemeMode={}\naccent={}\n",
        if config.enabled { 1 } else { 0 },
        config.mode,
        config.material,
        config.provider,
        if config.remote_consent { 1 } else { 0 },
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
    let mut vesper_shadow = false;

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
            "X-Vesper-Adaptive-Shadow" => vesper_shadow = value.trim().eq_ignore_ascii_case("true"),
            _ => {}
        }
    }

    if kind != "Application" || hidden || no_display || icon.is_empty() || vesper_shadow {
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
        "jpg" | "jpeg" => 55_000,
        "ico" => 45_000,
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
    if is_vesper_owned_source(current) {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_vesper_owned_source(&path) {
            continue;
        }
        if path.is_dir() {
            index_icon_tree(root, &path, root_rank, depth + 1, index);
            continue;
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(
            extension.as_str(),
            "svg" | "svgz" | "png" | "webp" | "jpg" | "jpeg" | "ico" | "xpm"
        ) {
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
    if path.is_absolute() && path.is_file() && !is_vesper_owned_source(&path) {
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

fn xml_tag_values(content: &str, tag: &str) -> Vec<(String, String)> {
    let mut values = Vec::new();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut offset = 0usize;

    while let Some(relative) = content[offset..].find(&open) {
        let start = offset + relative;
        let Some(open_end_relative) = content[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_relative;
        let Some(close_relative) = content[open_end + 1..].find(&close) else {
            break;
        };
        let close_start = open_end + 1 + close_relative;
        let attrs = content[start + open.len()..open_end].trim().to_string();
        let value = content[open_end + 1..close_start].trim().to_string();
        if !value.is_empty() && !value.contains('<') && value.len() <= 4096 {
            values.push((attrs, value));
        }
        offset = close_start + close.len();
    }

    values
}

fn attr_is(attrs: &str, name: &str, value: &str) -> bool {
    let double = format!("{name}=\"{value}\"");
    let single = format!("{name}='{value}'");
    attrs.split_whitespace().any(|part| part == double || part == single)
}

fn build_appstream_icon_map(data_dirs: &[PathBuf]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    for data_dir in data_dirs {
        for metadata_dir in [data_dir.join("metainfo"), data_dir.join("appdata")] {
            let Ok(entries) = fs::read_dir(metadata_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() || is_vesper_owned_source(&path) {
                    continue;
                }
                let extension = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("");
                if !matches!(extension, "xml" | "metainfo" | "appdata") {
                    continue;
                }
                if fs::metadata(&path).map(|meta| meta.len() > 2_000_000).unwrap_or(true) {
                    continue;
                }
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };

                let desktop_ids = xml_tag_values(&content, "launchable")
                    .into_iter()
                    .filter(|(attrs, _)| attr_is(attrs, "type", "desktop-id"))
                    .map(|(_, value)| value)
                    .collect::<Vec<_>>();
                if desktop_ids.is_empty() {
                    continue;
                }

                let icon = xml_tag_values(&content, "icon")
                    .into_iter()
                    .find(|(attrs, value)| {
                        (attr_is(attrs, "type", "stock") || attr_is(attrs, "type", "local"))
                            && !value.starts_with("http://")
                            && !value.starts_with("https://")
                    })
                    .map(|(_, value)| value);
                let Some(icon) = icon else {
                    continue;
                };

                for desktop_id in desktop_ids {
                    map.entry(desktop_id).or_insert_with(|| icon.clone());
                }
            }
        }
    }

    map
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
        ("href=\"http://", "external-url"),
        ("href='http://", "external-url"),
        ("href=\"https://", "external-url"),
        ("href='https://", "external-url"),
        ("href=\"file://", "external-file"),
        ("href='file://", "external-file"),
        ("src=\"http://", "external-url"),
        ("src='http://", "external-url"),
        ("src=\"https://", "external-url"),
        ("src='https://", "external-url"),
        ("src=\"file://", "external-file"),
        ("src='file://", "external-file"),
        ("url(http://", "external-url"),
        ("url(https://", "external-url"),
        ("url(file://", "external-file"),
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

fn canonical_silhouette(canonical: &Path) -> String {
    let manifest = canonical
        .parent()
        .map(|dir| dir.join("icon.vicon/manifest.json"));
    if let Some(path) = manifest {
        if let Ok(content) = fs::read_to_string(path) {
            let needle = "\"silhouette\":\"";
            if let Some(start) = content.find(needle) {
                let rest = &content[start + needle.len()..];
                if let Some(end) = rest.find('"') {
                    let value = &rest[..end];
                    if matches!(value, "enclosed" | "circular" | "glyph" | "irregular" | "full-bleed") {
                        return value.to_string();
                    }
                }
            }
        }
    }

    let content = fs::read_to_string(canonical).unwrap_or_default().to_ascii_lowercase();
    let circles = content.matches("<circle").count() + content.matches("<ellipse").count();
    let rects = content.matches("<rect").count();
    if circles > 0 && rects == 0 {
        "circular".to_string()
    } else {
        "unknown".to_string()
    }
}

fn grid_geometry(silhouette: &str) -> GridGeometry {
    match silhouette {
        // Existing tiles and intentional full-bleed artwork keep their own
        // enclosure. The renderer must not place another tile behind them.
        "enclosed" | "full-bleed" => GridGeometry {
            content_x: 0,
            content_size: 1024,
            enclosure_x: 32,
            enclosure_size: 960,
            enclosure_radius: 224,
            needs_enclosure: false,
        },
        "circular" => GridGeometry {
            content_x: 176,
            content_size: 672,
            enclosure_x: 96,
            enclosure_size: 832,
            enclosure_radius: 190,
            needs_enclosure: true,
        },
        "glyph" | "irregular" => GridGeometry {
            content_x: 164,
            content_size: 696,
            enclosure_x: 96,
            enclosure_size: 832,
            enclosure_radius: 190,
            needs_enclosure: true,
        },
        _ => GridGeometry {
            content_x: 148,
            content_size: 728,
            enclosure_x: 96,
            enclosure_size: 832,
            enclosure_radius: 190,
            needs_enclosure: true,
        },
    }
}

fn compile_icon(canonical: &Path, config: &Config) -> Result<String, String> {
    let (inner, viewbox) = svg_inner_and_viewbox(canonical)?;
    let render_mode = match config.mode.as_str() {
        "automatic" => if config.scheme_mode == "light" { "default" } else { "dark" },
        "light" => "default",
        value => value,
    };

    if render_mode == "original" {
        return Ok(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\">{}</svg>\n",
            nested_svg(&inner, &viewbox, 0, 0, 1024)
        ));
    }

    let silhouette = canonical_silhouette(canonical);
    let grid = grid_geometry(&silhouette);
    let glyph = nested_svg(
        &inner,
        &viewbox,
        grid.content_x,
        grid.content_x,
        grid.content_size,
    );
    let enclosure = |fill: &str, fill_opacity: &str, stroke: &str, stroke_opacity: &str, stroke_width: i32| -> String {
        if !grid.needs_enclosure {
            return String::new();
        }
        format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"{}\" stroke-opacity=\"{}\" stroke-width=\"{}\"/>", grid.enclosure_x, grid.enclosure_x, grid.enclosure_size, grid.enclosure_size, grid.enclosure_radius, fill, fill_opacity, stroke, stroke_opacity, stroke_width)
    };
    let matrix = colour_matrix(&config.accent);
    let body = match render_mode {
        "default" => format!(
            "{}<g>{glyph}</g>", enclosure("#f7f7f8", "1", "#ffffff", "1", 10)
        ),
        "dark" => format!(
            "{}<g>{glyph}</g>", enclosure("#171719", "1", "#38383d", "1", 10)
        ),
        "tinted" => format!(
            "<defs><filter id=\"vesperTint\" color-interpolation-filters=\"sRGB\"><feColorMatrix type=\"matrix\" values=\"{matrix}\"/></filter></defs>{}<g filter=\"url(#vesperTint)\">{glyph}</g>",
            enclosure(&config.accent, "0.20", &config.accent, "0.55", 10)
        ),
        "clear" => {
            let foreground = if config.scheme_mode == "light" {
                "#202124"
            } else {
                "#ffffff"
            };
            let clear_matrix = colour_matrix(foreground);
            format!(
                "<defs><filter id=\"vesperClear\" color-interpolation-filters=\"sRGB\"><feColorMatrix type=\"matrix\" values=\"{clear_matrix}\"/></filter></defs>{}<g filter=\"url(#vesperClear)\">{glyph}</g>",
                enclosure(if config.scheme_mode == "light" { "#ffffff" } else { "#d8d9de" }, "0.10", foreground, "0.28", 8)
            )
        }
        _ => glyph,
    };

    let body = if config.material == "glass" {
        format!(
            "<defs><linearGradient id=\"vesperMaterialGlass\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.18\"/><stop offset=\"0.46\" stop-color=\"{}\" stop-opacity=\"0.08\"/><stop offset=\"1\" stop-color=\"#000000\" stop-opacity=\"0.06\"/></linearGradient><linearGradient id=\"vesperMaterialSpec\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.54\"/><stop offset=\"0.48\" stop-color=\"#ffffff\" stop-opacity=\"0.05\"/><stop offset=\"1\" stop-color=\"#ffffff\" stop-opacity=\"0\"/></linearGradient></defs><g>{body}</g><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"url(#vesperMaterialGlass)\" pointer-events=\"none\"/><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"url(#vesperMaterialSpec)\" pointer-events=\"none\"/>",
            config.accent,
            grid.enclosure_x, grid.enclosure_x, grid.enclosure_size, grid.enclosure_size, grid.enclosure_radius,
            grid.enclosure_x + 16, grid.enclosure_x + 16, grid.enclosure_size - 32, (grid.enclosure_size * 43) / 100, (grid.enclosure_radius - 16).max(0)
        )
    } else {
        body
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

fn json_i32_field(value: &str, key: &str) -> Option<i32> {
    let needle = format!("\"{key}\":");
    let rest = value.get(value.find(&needle)? + needle.len()..)?.trim_start();
    let end = rest
        .find(|character: char| !character.is_ascii_digit() && character != '-')
        .unwrap_or(rest.len());
    rest.get(..end)?.parse().ok()
}

fn json_string_field(value: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let rest = value.get(value.find(&needle)? + needle.len()..)?;
    Some(rest.get(..rest.find('"')?)?.to_string())
}

fn raster_data_uri(asset: &Path) -> Result<String, String> {
    let output = Command::new("base64")
        .args(["-w", "0"])
        .arg(asset)
        .output()
        .map_err(|error| format!("failed to encode vicon raster layer: {error}"))?;
    if !output.status.success() {
        return Err("failed to encode vicon raster layer".to_string());
    }
    Ok(format!(
        "data:image/png;base64,{}",
        String::from_utf8_lossy(&output.stdout)
    ))
}

fn vicon_static_svg(package: &Path) -> Result<String, String> {
    if !package.join("manifest.json").is_file() {
        return Err("vicon manifest unavailable".to_string());
    }

    let groups_root = package.join("groups");
    let mut groups = fs::read_dir(&groups_root)
        .map_err(|error| format!("vicon groups unavailable: {error}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            let path = entry.path();
            let metadata = fs::read_to_string(path.join("group.json"))
                .map_err(|error| format!("vicon group metadata unavailable: {error}"))?;
            let depth = json_i32_field(&metadata, "depth")
                .ok_or_else(|| "vicon group depth unavailable".to_string())?;
            let material = json_string_field(&metadata, "material")
                .unwrap_or_else(|| "standard".to_string());
            let mut layers = fs::read_dir(path.join("layers"))
                .map_err(|error| format!("vicon group layers unavailable: {error}"))?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .collect::<Vec<_>>();
            layers.sort();
            if layers.is_empty() {
                return Err("vicon group has no layers".to_string());
            }
            Ok((depth, path, material, layers))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if !(1..=4).contains(&groups.len()) {
        return Err(format!("vicon must contain 1 to 4 groups, found {}", groups.len()));
    }
    groups.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let mut defs = String::new();
    let mut body = String::new();
    for (index, (depth, _path, material, layers)) in groups.into_iter().enumerate() {
        let depth = depth.clamp(0, 8);
        let filter = format!("vesperViconDepth{index}");
        let shadow_opacity = 0.10 + f64::from(depth) * 0.035;
        defs.push_str(&format!(
            "<filter id=\"{filter}\" x=\"-30%\" y=\"-30%\" width=\"160%\" height=\"170%\"><feDropShadow dx=\"0\" dy=\"{}\" stdDeviation=\"{}\" flood-color=\"#000000\" flood-opacity=\"{shadow_opacity:.3}\"/></filter>",
            depth * 4,
            2 + depth * 2
        ));
        let opacity = if material == "glass" { "0.88" } else { "1" };
        body.push_str(&format!(
            "<g data-vesper-depth=\"{depth}\" data-vesper-material=\"{}\" opacity=\"{opacity}\" filter=\"url(#{filter})\">",
            json_escape(&material)
        ));
        for asset in layers {
            match asset
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str()
            {
                "svg" => {
                    let (inner, viewbox) = svg_inner_and_viewbox(&asset)?;
                    body.push_str(&nested_svg(&inner, &viewbox, 0, 0, 1024));
                }
                "png" => body.push_str(&format!(
                    "<image x=\"0\" y=\"0\" width=\"1024\" height=\"1024\" preserveAspectRatio=\"xMidYMid meet\" href=\"{}\"/>",
                    raster_data_uri(&asset)?
                )),
                _ => return Err(format!("unsupported vicon layer: {}", asset.display())),
            }
        }
        body.push_str("</g>");
    }

    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\"><defs>{defs}</defs>{body}</svg>\n"
    ))
}

fn static_render_source(id: &str, fingerprint: &str) -> Result<(PathBuf, bool), String> {
    let canonical_dir = canonical_dir(id, fingerprint);
    let package = canonical_dir.join("icon.vicon");
    if !package.join("manifest.json").is_file() {
        return existing_canonical(id, fingerprint)
            .map(|path| (path, false))
            .ok_or_else(|| "canonical source unavailable".to_string());
    }

    let scratch = canonical_dir.join(format!(
        ".vicon-static-source.{}.svg",
        std::process::id()
    ));
    fs::write(&scratch, vicon_static_svg(&package)?).map_err(|error| error.to_string())?;
    Ok((scratch, true))
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
            let (source, scratch) = match static_render_source(&item.id, &item.fingerprint) {
                Ok(value) => value,
                Err(error) => {
                    item.error = format!("canonical: {error}");
                    continue;
                }
            };
            let primary = icon_theme_name(&item.icon_key).unwrap_or_else(|| {
                safe_name(item.id.strip_suffix(".desktop").unwrap_or(&item.id))
            });
            let compiled = match compile_icon(&source, config) {
                Ok(compiled) => compiled,
                Err(error) => {
                    if scratch {
                        let _ = fs::remove_file(&source);
                    }
                    item.error = format!("compile: {error}");
                    continue;
                }
            };
            if scratch {
                let _ = fs::remove_file(&source);
            }

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

fn desktop_has_exec_icon_field_code(content: &str) -> bool {
    let mut in_desktop = false;
    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop = line == "[Desktop Entry]";
            continue;
        }
        if in_desktop {
            if let Some(value) = line.strip_prefix("Exec=") {
                return value.split_whitespace().any(|token| token == "%i" || token.contains("%i"));
            }
        }
    }
    false
}

fn shadow_content(upstream: &str, theme_icon: &str) -> Result<String, String> {
    let mut out = String::with_capacity(upstream.len() + 128);
    let mut in_desktop = false;
    let mut replaced_icon = false;
    let mut marker_written = false;

    for raw in upstream.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            if in_desktop && !marker_written {
                out.push_str("X-Vesper-Adaptive-Shadow=true\n");
                marker_written = true;
            }
            in_desktop = line == "[Desktop Entry]";
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        if in_desktop && line.starts_with("X-Vesper-Adaptive-Shadow=") {
            continue;
        }
        if in_desktop && line.starts_with("Icon=") {
            out.push_str("Icon=");
            out.push_str(theme_icon);
            out.push('\n');
            replaced_icon = true;
            continue;
        }
        out.push_str(raw);
        out.push('\n');
    }
    if in_desktop && !marker_written {
        out.push_str("X-Vesper-Adaptive-Shadow=true\n");
    }
    if !replaced_icon {
        return Err("upstream desktop entry lost its main Icon field".to_string());
    }
    Ok(out)
}

fn is_vesper_shadow(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|content| content.lines().any(|line| line.trim() == "X-Vesper-Adaptive-Shadow=true"))
        .unwrap_or(false)
}

fn persist_shadow_db(desired: &BTreeMap<PathBuf, String>) -> Result<(), String> {
    let updated_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64;
    let mut sql = String::from(
        "PRAGMA busy_timeout=5000; CREATE TABLE IF NOT EXISTS shadow_desktop_entries(path TEXT PRIMARY KEY, desktop_id TEXT NOT NULL, updated_ms INTEGER NOT NULL); BEGIN IMMEDIATE; DELETE FROM shadow_desktop_entries;\n",
    );
    for (path, desktop_id) in desired {
        sql.push_str(&format!(
            "INSERT INTO shadow_desktop_entries(path,desktop_id,updated_ms) VALUES({}, {}, {});\n",
            sql_quote(&path.to_string_lossy()),
            sql_quote(desktop_id),
            updated_ms,
        ));
    }
    sql.push_str("COMMIT;\n");
    sqlite(&sql)?;
    Ok(())
}

fn sync_shadow_entries(items: &[InventoryItem], config: &Config) -> Result<(), String> {
    let applications = data_home().join("applications");
    fs::create_dir_all(&applications).map_err(|error| error.to_string())?;
    let mut desired = BTreeMap::<PathBuf, String>::new();

    if config.enabled {
        for item in items {
            if !item.active || item.excluded || !Path::new(&item.icon_key).is_absolute() {
                continue;
            }
            if item.desktop_path.starts_with(&applications) {
                continue;
            }
            let upstream = match fs::read_to_string(&item.desktop_path) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if desktop_has_exec_icon_field_code(&upstream) {
                continue;
            }
            let theme_icon = safe_name(item.id.strip_suffix(".desktop").unwrap_or(&item.id));
            let target = applications.join(&item.id);
            if target.exists() && !is_vesper_shadow(&target) {
                continue;
            }
            let content = shadow_content(&upstream, &theme_icon)?;
            write_atomic(&target, content)?;
            desired.insert(target, item.id.clone());
        }
    }

    if let Ok(entries) = fs::read_dir(&applications) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_vesper_shadow(&path) && !desired.contains_key(&path) {
                let _ = fs::remove_file(path);
            }
        }
    }
    persist_shadow_db(&desired)?;
    Ok(())
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
    let appstream_icons = build_appstream_icon_map(&data_dirs);
    let exclusions = load_exclusions();
    let mut items = Vec::with_capacity(records.len());

    for record in records {
        let excluded = exclusions.contains(&record.id);
        let source_path = resolve_icon(&record.icon, &index).or_else(|| {
            let recovery = appstream_icons.get(&record.id)?;
            let path = PathBuf::from(recovery);
            if path.is_absolute() {
                if path.is_file() && !is_vesper_owned_source(&path) {
                    Some(path)
                } else {
                    None
                }
            } else {
                resolve_icon(recovery, &index)
            }
        });
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
            "png" | "webp" | "jpg" | "jpeg" | "ico" | "xpm" => {
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
        "{{\"enabled\":{},\"mode\":\"{}\",\"material\":\"{}\",\"gridRevision\":\"{}\",\"provider\":\"{}\",\"providerConfigured\":{},\"remoteConsent\":{},\"followPalette\":{},\"schemeMode\":\"{}\",\"accent\":\"{}\",\"theme\":\"{}\",\"discovered\":{},\"canonical\":{},\"pending\":{},\"failed\":{},\"excluded\":{},\"active\":{},\"current\":\"{}\",\"aiTransport\":\"worker\"}}\n",
        if config.enabled { "true" } else { "false" },
        json_escape(&config.mode),
        json_escape(&config.material),
        GRID_REVISION,
        json_escape(&config.provider),
        if provider_configured(&config.provider) { "true" } else { "false" },
        if config.remote_consent { "true" } else { "false" },
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

fn persist_inventory_db(items: &[InventoryItem]) -> Result<(), String> {
    let mut sql = String::from(
        "PRAGMA journal_mode=WAL;\n\
         PRAGMA synchronous=NORMAL;\n\
         PRAGMA busy_timeout=5000;\n\
         CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);\n\
         CREATE TABLE IF NOT EXISTS application_inventory (\n\
           desktop_id TEXT PRIMARY KEY,\n\
           desktop_path TEXT NOT NULL,\n\
           icon_key TEXT NOT NULL,\n\
           source_path TEXT NOT NULL,\n\
           source_fingerprint TEXT NOT NULL,\n\
           source_kind TEXT NOT NULL,\n\
           canonical_state TEXT NOT NULL,\n\
           active INTEGER NOT NULL,\n\
           excluded INTEGER NOT NULL,\n\
           error TEXT NOT NULL,\n\
           updated_ms INTEGER NOT NULL\n\
         );\n\
         CREATE INDEX IF NOT EXISTS application_inventory_fingerprint_idx ON application_inventory(source_fingerprint);\n\
         CREATE TABLE IF NOT EXISTS source_provenance (\n\
           fingerprint TEXT PRIMARY KEY,\n\
           source_path TEXT NOT NULL,\n\
           source_kind TEXT NOT NULL,\n\
           reference_count INTEGER NOT NULL,\n\
           updated_ms INTEGER NOT NULL\n\
         );\n\
         BEGIN IMMEDIATE;\n\
         DELETE FROM application_inventory;\n\
         DELETE FROM source_provenance;\n",
    );
    let updated_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64;
    let mut sources = BTreeMap::<String, (String, String, usize)>::new();
    for item in items {
        let source_path = item
            .source_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        sql.push_str(&format!(
            "INSERT INTO application_inventory(desktop_id, desktop_path, icon_key, source_path, source_fingerprint, source_kind, canonical_state, active, excluded, error, updated_ms) VALUES({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});\n",
            sql_quote(&item.id),
            sql_quote(&item.desktop_path.to_string_lossy()),
            sql_quote(&item.icon_key),
            sql_quote(&source_path),
            sql_quote(&item.fingerprint),
            sql_quote(&item.source_kind),
            sql_quote(&item.canonical_state),
            if item.active { 1 } else { 0 },
            if item.excluded { 1 } else { 0 },
            sql_quote(&item.error),
            updated_ms,
        ));
        if !item.fingerprint.is_empty() {
            let entry = sources
                .entry(item.fingerprint.clone())
                .or_insert_with(|| (source_path, item.source_kind.clone(), 0));
            entry.2 += 1;
        }
    }
    for (fingerprint, (source_path, source_kind, reference_count)) in sources {
        sql.push_str(&format!(
            "INSERT INTO source_provenance(fingerprint, source_path, source_kind, reference_count, updated_ms) VALUES({}, {}, {}, {}, {});\n",
            sql_quote(&fingerprint),
            sql_quote(&source_path),
            sql_quote(&source_kind),
            reference_count,
            updated_ms,
        ));
    }
    sql.push_str(
        "INSERT INTO meta(key, value) VALUES('inventorySchemaVersion', '1') ON CONFLICT(key) DO UPDATE SET value=excluded.value;\nCOMMIT;\n",
    );
    sqlite(&sql)?;
    Ok(())
}

fn write_state(items: &[InventoryItem], config: &Config, current: &str) -> Result<(), String> {
    fs::create_dir_all(state_root()).map_err(|error| error.to_string())?;
    persist_inventory_db(items)?;
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
    sync_shadow_entries(&items, &config)?;
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

fn set_material(material: &str) -> Result<(), String> {
    if !valid_material(material) {
        return Err(format!("unsupported icon material: {material}"));
    }
    let mut config = load_config();
    config.material = material.to_string();
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

fn set_remote_consent(enabled: bool) -> Result<(), String> {
    let mut config = load_config();
    config.remote_consent = enabled;
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
           grid-info\n\
           mode automatic|default|dark|tinted|clear\n\
           material standard|glass\n\
           provider openai|anthropic|xai|openrouter|google\n\
           remote-consent on|off\n\
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
        [command] if command == "grid-info" => Ok(println!(
            "{{\"revision\":\"{}\",\"canvas\":1024,\"enclosure\":832,\"circularContent\":672,\"primaryContent\":696}}", GRID_REVISION
        )),
        [command, mode] if command == "mode" => set_mode(mode),
        [command, material] if command == "material" => set_material(material),
        [command, provider] if command == "provider" => set_provider(provider),
        [command, value] if command == "remote-consent" => match value.as_str() {
            "on" => set_remote_consent(true),
            "off" => set_remote_consent(false),
            _ => usage(),
        },
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

#[cfg(test)]
mod tests {
    use super::{unsafe_svg_reason, vicon_static_svg};
    use std::fs;

    #[test]
    fn standard_svg_namespace_is_not_treated_as_external_content() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0h1v1z"/></svg>"#;
        assert_eq!(unsafe_svg_reason(svg), None);
    }

    #[test]
    fn external_svg_reference_is_rejected() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.invalid/icon.svg"/></svg>"#;
        assert_eq!(unsafe_svg_reason(svg), Some("embedded-image"));
    }

    #[test]
    fn vicon_static_render_uses_every_group_and_semantic_depth() {
        let root = std::env::temp_dir().join(format!(
            "vesper-vicon-render-test-{}",
            std::process::id()
        ));
        let first = root.join("groups/01-background/layers");
        let second = root.join("groups/02-foreground/layers");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        fs::write(root.join("manifest.json"), "{}\n").unwrap();
        fs::write(
            root.join("groups/01-background/group.json"),
            r#"{"id":"background","depth":0,"material":"standard"}"#,
        )
        .unwrap();
        fs::write(
            root.join("groups/02-foreground/group.json"),
            r#"{"id":"foreground","depth":2,"material":"glass"}"#,
        )
        .unwrap();
        fs::write(
            first.join("01.svg"),
            r##"<svg viewBox="0 0 1024 1024"><rect width="1024" height="1024" fill="#ff0000"/></svg>"##,
        )
        .unwrap();
        fs::write(
            second.join("01.svg"),
            r##"<svg viewBox="0 0 1024 1024"><circle cx="512" cy="512" r="256" fill="#0000ff"/></svg>"##,
        )
        .unwrap();

        let rendered = vicon_static_svg(&root).unwrap();
        assert!(rendered.contains("#ff0000"));
        assert!(rendered.contains("#0000ff"));
        assert!(rendered.contains("data-vesper-depth=\"2\""));
        assert!(rendered.contains("data-vesper-material=\"glass\""));

        fs::write(
            root.join("groups/02-foreground/group.json"),
            r#"{"id":"foreground","depth":5,"material":"standard"}"#,
        )
        .unwrap();
        assert_ne!(rendered, vicon_static_svg(&root).unwrap());
        let _ = fs::remove_dir_all(root);
    }
}
