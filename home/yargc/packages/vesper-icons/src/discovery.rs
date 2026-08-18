use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{DesktopRecord, Identity, Source};
use crate::util::{home, safe_name, sha256, vesper_owned, xdg_data_home};

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|v| v == &path) { paths.push(path); }
}

pub fn effective_data_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![xdg_data_home()];
    let raw = env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for v in raw.split(':').filter(|v| !v.is_empty()) { push_unique(&mut dirs, PathBuf::from(v)); }
    if let Ok(raw) = env::var("NIX_PROFILES") {
        for v in raw.split_whitespace() { push_unique(&mut dirs, PathBuf::from(v).join("share")); }
    }
    for path in [
        home().join(".nix-profile/share"),
        home().join(".local/share/flatpak/exports/share"),
        PathBuf::from("/var/lib/flatpak/exports/share"),
        PathBuf::from("/run/current-system/sw/share"),
    ] { if path.exists() { push_unique(&mut dirs, path); } }
    if let Ok(user) = env::var("USER") {
        let p = PathBuf::from("/etc/profiles/per-user").join(user).join("share");
        if p.exists() { push_unique(&mut dirs, p); }
    }
    dirs
}

fn collect_desktops(root: &Path, here: &Path, out: &mut Vec<(String, PathBuf)>, depth: usize) {
    if depth > 8 { return; }
    let Ok(entries) = fs::read_dir(here) else { return; };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() { collect_desktops(root, &p, out, depth + 1); continue; }
        if p.extension().and_then(|v| v.to_str()).map(|v| v.eq_ignore_ascii_case("desktop")) != Some(true) { continue; }
        let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace(std::path::MAIN_SEPARATOR, "-");
        out.push((rel, p));
    }
}

fn parse_desktop(id: String, path: &Path) -> Option<DesktopRecord> {
    let text = fs::read_to_string(path).ok()?;
    let mut record = DesktopRecord { id, path: path.to_path_buf(), ..Default::default() };
    let mut section = false;
    let mut kind = String::new();
    let mut hidden = false;
    let mut no_display = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') { section = line == "[Desktop Entry]"; continue; }
        if !section || line.starts_with('#') { continue; }
        let Some((key, value)) = line.split_once('=') else { continue; };
        let value = value.trim();
        match key.trim() {
            "Type" => kind = value.into(),
            "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
            "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
            "Icon" => record.icon = value.into(),
            "Exec" => record.exec = value.into(),
            "StartupWMClass" => record.startup_wm_class = value.into(),
            "X-Flatpak" => record.flatpak_id = value.into(),
            "X-Vesper-Generated" => record.generated_shadow = value.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    if kind != "Application" || hidden || no_display || record.icon.is_empty() { return None; }
    Some(record)
}

pub fn desktops() -> Vec<DesktopRecord> {
    let mut map = BTreeMap::new();
    for data in effective_data_dirs() {
        let root = data.join("applications");
        if !root.is_dir() { continue; }
        let mut files = Vec::new();
        collect_desktops(&root, &root, &mut files, 0);
        files.sort_by(|a, b| a.0.cmp(&b.0));
        for (id, path) in files {
            if map.contains_key(&id) { continue; }
            let Some(record) = parse_desktop(id.clone(), &path) else { continue; };
            // A synchronized Vesper user shadow must never hide its own upstream source from discovery.
            if record.generated_shadow { continue; }
            map.insert(id, record);
        }
    }
    map.into_values().collect()
}

fn exact_arg(exec: &str, needle: &str) -> Option<String> {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    for (index, part) in parts.iter().enumerate() {
        if *part == needle { return parts.get(index + 1).map(|v| v.trim_matches(['\'', '"']).to_string()); }
        if let Some(v) = part.strip_prefix(&format!("{needle}=")) { return Some(v.trim_matches(['\'', '"']).to_string()); }
    }
    None
}

fn steam_app_id(exec: &str) -> Option<String> {
    if let Some(pos) = exec.find("steam://rungameid/") {
        let tail = &exec[pos + "steam://rungameid/".len()..];
        let id: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !id.is_empty() { return Some(id); }
    }
    exact_arg(exec, "-applaunch").filter(|v| v.chars().all(|c| c.is_ascii_digit()))
}

fn add_exact(set: &mut BTreeSet<String>, value: &str) {
    let value = value.trim();
    if value.is_empty() { return; }
    let lower = value.to_ascii_lowercase();
    if matches!(lower.as_str(), "electron" | "wine" | "wine64") { return; }
    set.insert(value.to_string());
    if lower != value { set.insert(lower); }
}

pub fn identity(record: &DesktopRecord) -> Identity {
    let launch = record.id.trim_end_matches(".desktop").to_string();
    let mut runtime = BTreeSet::new();
    let mut aliases = BTreeSet::new();
    add_exact(&mut runtime, &launch);
    add_exact(&mut aliases, &launch);
    add_exact(&mut runtime, &record.startup_wm_class);
    if !record.flatpak_id.is_empty() { add_exact(&mut runtime, &record.flatpak_id); add_exact(&mut aliases, &record.flatpak_id); }
    if let Some(id) = exact_arg(&record.exec, "--class") { add_exact(&mut runtime, &id); }
    if let Some(id) = exact_arg(&record.exec, "--name") { add_exact(&mut runtime, &id); }
    if let Some(id) = exact_arg(&record.exec, "--app-id") { add_exact(&mut runtime, &id); add_exact(&mut aliases, &id); }
    if let Some(id) = steam_app_id(&record.exec) {
        add_exact(&mut runtime, &format!("steam_app_{id}"));
        add_exact(&mut aliases, &format!("steam_app_{id}"));
    }
    let icon_stem = Path::new(&record.icon).file_stem().and_then(|v| v.to_str()).unwrap_or("");
    if !icon_stem.is_empty() && !Path::new(&record.icon).is_absolute() { add_exact(&mut aliases, icon_stem); }
    Identity {
        canonical_app_id: launch.clone(),
        launch_desktop_id: launch,
        runtime_ids: runtime.into_iter().collect(),
        icon_aliases: aliases.into_iter().collect(),
    }
}

#[derive(Clone)]
struct Candidate { path: PathBuf, score: i64, resolver: &'static str }

fn raster_size(path: &Path) -> i64 {
    for c in path.components() {
        let s = c.as_os_str().to_string_lossy();
        if let Some((a, b)) = s.split_once('x') { if a == b { if let Ok(v) = a.parse::<i64>() { return v.min(8192); } } }
    }
    0
}

fn score(path: &Path, rank: usize) -> i64 {
    let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("").to_ascii_lowercase();
    let format = match ext.as_str() { "svg" => 100_000, "svgz" => 90_000, "png" => 80_000, "webp" => 75_000, "ico" => 65_000, "jpg" | "jpeg" => 55_000, "xpm" => 30_000, _ => 0 };
    let hicolor = if path.to_string_lossy().contains("/hicolor/") { 10_000 } else { 0 };
    format + hicolor + raster_size(path) + (10_000i64.saturating_sub(rank as i64 * 100))
}

fn index_tree(root: &Path, here: &Path, rank: usize, depth: usize, map: &mut BTreeMap<String, Vec<Candidate>>) {
    if depth > 12 || vesper_owned(here) { return; }
    let Ok(entries) = fs::read_dir(here) else { return; };
    for entry in entries.flatten() {
        let path = entry.path();
        if vesper_owned(&path) { continue; }
        if path.is_dir() { index_tree(root, &path, rank, depth + 1, map); continue; }
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("").to_ascii_lowercase();
        if !matches!(ext.as_str(), "svg" | "svgz" | "png" | "webp" | "ico" | "jpg" | "jpeg" | "xpm") { continue; }
        let Some(stem) = path.file_stem().and_then(|v| v.to_str()) else { continue; };
        let resolver = if path.to_string_lossy().contains("/pixmaps/") { "pixmaps" } else if path.to_string_lossy().contains("/hicolor/") { "hicolor" } else { "theme-inheritance" };
        let _ = root;
        map.entry(stem.to_string()).or_default().push(Candidate { score: score(&path, rank), path, resolver });
    }
}

fn icon_index() -> BTreeMap<String, Vec<Candidate>> {
    let mut map = BTreeMap::new();
    for (rank, data) in effective_data_dirs().into_iter().enumerate() {
        for root in [data.join("icons"), data.join("pixmaps")] { if root.is_dir() { index_tree(&root, &root, rank, 0, &mut map); } }
    }
    for values in map.values_mut() { values.sort_by(|a,b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path))); }
    map
}

fn appstream_candidate(record: &DesktopRecord) -> Option<PathBuf> {
    let app_id = record.id.trim_end_matches(".desktop");
    for data in effective_data_dirs() {
        for dir in [data.join("metainfo"), data.join("appdata")] {
            if !dir.is_dir() { continue; }
            let Ok(entries) = fs::read_dir(dir) else { continue; };
            for entry in entries.flatten() {
                let p = entry.path();
                let name = p.file_name().and_then(|v| v.to_str()).unwrap_or("");
                if !name.starts_with(app_id) || !name.ends_with(".xml") { continue; }
                let text = fs::read_to_string(&p).ok()?;
                for raw in text.lines() {
                    let line = raw.trim();
                    if line.contains("<icon") && line.contains("type=\"local\"") {
                        if let (Some(a), Some(b)) = (line.find('>'), line.rfind("</icon>")) {
                            let candidate = PathBuf::from(line[a+1..b].trim());
                            if candidate.is_absolute() && candidate.is_file() && !vesper_owned(&candidate) { return Some(candidate); }
                        }
                    }
                }
            }
        }
    }
    None
}

fn steam_cache_candidate(record: &DesktopRecord) -> Option<PathBuf> {
    let id = steam_app_id(&record.exec)?;
    let roots = [home().join(".local/share/Steam"), home().join(".steam/steam"), home().join(".var/app/com.valvesoftware.Steam/data/Steam")];
    for root in roots {
        let cache = root.join("appcache/librarycache");
        if !cache.is_dir() { continue; }
        for suffix in ["_icon.jpg", "_icon.png", ".jpg", ".png"] {
            let p = cache.join(format!("{id}{suffix}"));
            if p.is_file() { return Some(p); }
        }
        let app_dir = cache.join(&id);
        if app_dir.is_dir() {
            let Ok(entries) = fs::read_dir(app_dir) else { continue; };
            if let Some(p) = entries.flatten().map(|e| e.path()).find(|p| p.is_file() && matches!(p.extension().and_then(|v| v.to_str()).unwrap_or("").to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg")) { return Some(p); }
        }
    }
    None
}

pub fn resolve_sources(records: &[DesktopRecord]) -> BTreeMap<String, Option<Source>> {
    let index = icon_index();
    let mut out = BTreeMap::new();
    for record in records {
        let raw = PathBuf::from(&record.icon);
        let selected: Option<(PathBuf, String)> = if raw.is_absolute() && raw.is_file() && !vesper_owned(&raw) {
            Some((raw, "desktop-absolute".into()))
        } else {
            let stem = Path::new(&record.icon).file_stem().and_then(|v| v.to_str()).unwrap_or(&record.icon);
            index.get(stem).and_then(|v| v.first()).map(|v| (v.path.clone(), v.resolver.into()))
                .or_else(|| appstream_candidate(record).map(|p| (p, "appstream".into())))
                .or_else(|| steam_cache_candidate(record).map(|p| (p, "steam-librarycache".into())))
        };
        let source = selected.and_then(|(path, resolver)| {
            let fingerprint = sha256(&path).ok()?;
            let kind = path.extension().and_then(|v| v.to_str()).unwrap_or("unknown").to_ascii_lowercase();
            Some(Source { path, kind, resolver, fingerprint })
        });
        out.insert(record.id.clone(), source);
    }
    out
}

pub fn alias_name(value: &str) -> String { safe_name(value.trim_end_matches(".desktop")) }

#[cfg(test)]
mod tests {
    use super::*;
    fn record(id: &str, exec: &str, wm: &str) -> DesktopRecord { DesktopRecord { id: id.into(), exec: exec.into(), startup_wm_class: wm.into(), icon: "test".into(), ..Default::default() } }

    #[test]
    fn steam_games_do_not_collapse_into_client() {
        let i = identity(&record("steam-570.desktop", "steam steam://rungameid/570", ""));
        assert!(i.runtime_ids.iter().any(|v| v == "steam_app_570"));
        assert_ne!(i.canonical_app_id, "steam");
    }

    #[test]
    fn generic_wine_and_electron_are_not_identity_aliases() {
        let a = identity(&record("foo.desktop", "wine foo.exe", "wine"));
        assert!(!a.runtime_ids.iter().any(|v| v.eq_ignore_ascii_case("wine")));
        let b = identity(&record("bar.desktop", "electron app", "electron"));
        assert!(!b.runtime_ids.iter().any(|v| v.eq_ignore_ascii_case("electron")));
    }

    #[test]
    fn pwa_ids_remain_distinct() {
        let a = identity(&record("chrome-a.desktop", "chromium --app-id=AAAA", ""));
        let b = identity(&record("chrome-b.desktop", "chromium --app-id=BBBB", ""));
        assert!(a.runtime_ids.iter().any(|v| v == "AAAA"));
        assert!(b.runtime_ids.iter().any(|v| v == "BBBB"));
        assert_ne!(a.canonical_app_id, b.canonical_app_id);
    }
}
