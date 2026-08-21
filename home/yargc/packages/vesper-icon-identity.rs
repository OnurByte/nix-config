use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const IDENTITY_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug)]
struct App {
    id: String,
    icon_key: String,
    startup_wm_class: String,
    exec: String,
    flatpak_id: String,
}

#[derive(Clone, Debug)]
struct Mapping {
    desktop_id: String,
    theme_icon: String,
    evidence: String,
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

fn data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

fn inventory_path() -> PathBuf {
    state_root().join("inventory.tsv")
}

fn identity_path() -> PathBuf {
    state_root().join("identity.json")
}

fn db_path() -> PathBuf {
    state_root().join("state.sqlite3")
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

fn write_atomic(path: &Path, data: impl AsRef<[u8]>) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "invalid identity path".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(".identity.{}.tmp", std::process::id()));
    fs::write(&tmp, data).map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    push_unique(&mut dirs, data_home());
    for path in env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string())
        .split(':')
        .filter(|value| !value.is_empty())
    {
        push_unique(&mut dirs, PathBuf::from(path));
    }
    if let Ok(profiles) = env::var("NIX_PROFILES") {
        for profile in profiles.split_whitespace() {
            push_unique(&mut dirs, PathBuf::from(profile).join("share"));
        }
    }
    for path in [
        home().join(".nix-profile/share"),
        home().join(".local/share/flatpak/exports/share"),
        PathBuf::from("/var/lib/flatpak/exports/share"),
        PathBuf::from("/run/current-system/sw/share"),
    ] {
        if path.exists() {
            push_unique(&mut dirs, path);
        }
    }
    if let Ok(user) = env::var("USER") {
        let path = PathBuf::from("/etc/profiles/per-user").join(user).join("share");
        if path.exists() {
            push_unique(&mut dirs, path);
        }
    }
    dirs
}

fn collect_desktop_files(root: &Path, current: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_desktop_files(root, &path, out);
        } else if path.extension().and_then(|value| value.to_str()) == Some("desktop") {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let id = relative
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "-");
            out.push((id, path));
        }
    }
}

fn effective_desktops() -> BTreeMap<String, PathBuf> {
    let mut files = BTreeMap::new();
    for dir in data_dirs() {
        let root = dir.join("applications");
        if !root.is_dir() {
            continue;
        }
        let mut found = Vec::new();
        collect_desktop_files(&root, &root, &mut found);
        found.sort_by(|a, b| a.0.cmp(&b.0));
        for (id, path) in found {
            files.entry(id).or_insert(path);
        }
    }
    files
}

fn inventory_icons() -> BTreeMap<String, String> {
    let mut icons = BTreeMap::new();
    if let Ok(content) = sqlite(
        "PRAGMA busy_timeout=5000; SELECT desktop_id,icon_key FROM application_inventory WHERE excluded=0 ORDER BY desktop_id;",
    ) {
        for line in content.lines() {
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() >= 2 {
                icons.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        if !icons.is_empty() {
            return icons;
        }
    }

    let content = fs::read_to_string(inventory_path()).unwrap_or_default();
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 || parts[7] == "1" {
            continue;
        }
        icons.insert(parts[0].to_string(), parts[1].to_string());
    }
    icons
}

fn desktop_list(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn current_desktops() -> BTreeSet<String> {
    env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn desktop_visible(only_show_in: &[String], not_show_in: &[String], current: &BTreeSet<String>) -> bool {
    if not_show_in.iter().any(|desktop| current.contains(desktop)) {
        return false;
    }
    only_show_in.is_empty() || only_show_in.iter().any(|desktop| current.contains(desktop))
}

fn executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn executable_available(value: &str) -> bool {
    let path = Path::new(value);
    if path.is_absolute() {
        return executable_file(path);
    }
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|root| executable_file(&root.join(value))))
        .unwrap_or(false)
}

fn parse_desktop(id: String, path: &Path, icon_key: String) -> Option<App> {
    let content = fs::read_to_string(path).ok()?;
    let mut section = false;
    let mut kind = String::new();
    let mut hidden = false;
    let mut no_display = false;
    let mut startup = String::new();
    let mut exec = String::new();
    let mut flatpak = String::new();
    let mut only_show_in = Vec::new();
    let mut not_show_in = Vec::new();
    let mut try_exec = String::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line == "[Desktop Entry]";
            continue;
        }
        if !section || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "Type" => kind = value.to_string(),
            "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
            "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
            "StartupWMClass" => startup = value.to_string(),
            "Exec" => exec = value.to_string(),
            "X-Flatpak" => flatpak = value.to_string(),
            "OnlyShowIn" => only_show_in = desktop_list(value),
            "NotShowIn" => not_show_in = desktop_list(value),
            "TryExec" => try_exec = clean_token(value),
            _ => {}
        }
    }
    if kind != "Application"
        || hidden
        || no_display
        || !desktop_visible(&only_show_in, &not_show_in, &current_desktops())
        || (!try_exec.is_empty() && !executable_available(&try_exec))
    {
        return None;
    }
    Some(App {
        id,
        icon_key,
        startup_wm_class: startup,
        exec,
        flatpak_id: flatpak,
    })
}

fn theme_icon(icon_key: &str, id: &str) -> String {
    let path = Path::new(icon_key);
    if !path.is_absolute() && !icon_key.contains('/') {
        if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
            if !stem.is_empty() {
                return stem.to_string();
            }
        }
    }
    id.strip_suffix(".desktop").unwrap_or(id).to_string()
}

fn clean_token(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches(';')
        .to_string()
}

fn parse_exec_tokens(value: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut started = false;

    for ch in value.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            started = true;
            continue;
        }
        match ch {
            '\\' => {
                escaped = true;
                started = true;
            }
            '"' => {
                quoted = !quoted;
                started = true;
            }
            ch if !quoted && ch.is_whitespace() => {
                if started {
                    tokens.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            ch => {
                current.push(ch);
                started = true;
            }
        }
    }

    if quoted || escaped {
        return None;
    }
    if started {
        tokens.push(current);
    }
    Some(tokens)
}

fn is_field_code(value: &str) -> bool {
    let value = value.trim();
    value.len() == 2 && value.starts_with('%')
}

fn generic_runtime(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "env"
            | "electron"
            | "electron-wayland"
            | "wine"
            | "wine64"
            | "steam"
            | "flatpak"
            | "sh"
            | "bash"
            | "zsh"
    )
}

fn basename(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .to_string()
}

fn extract_flag(tokens: &[String], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    for (index, token) in tokens.iter().enumerate() {
        if let Some(value) = token.strip_prefix(&prefix) {
            let value = clean_token(value);
            if !value.is_empty() && !value.starts_with('%') {
                return Some(value);
            }
        }
        if token == name {
            if let Some(value) = tokens.get(index + 1) {
                let value = clean_token(value);
                if !value.is_empty() && !value.starts_with('-') && !value.starts_with('%') {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn steam_id(exec: &str, tokens: &[String]) -> Option<String> {
    if let Some(index) = exec.find("steam://rungameid/") {
        let rest = &exec[index + "steam://rungameid/".len()..];
        let digits = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect::<String>();
        if !digits.is_empty() {
            return Some(digits);
        }
    }
    for (index, token) in tokens.iter().enumerate() {
        if token == "-applaunch" {
            if let Some(value) = tokens.get(index + 1) {
                let digits = value.chars().take_while(|ch| ch.is_ascii_digit()).collect::<String>();
                if !digits.is_empty() {
                    return Some(digits);
                }
            }
        }
    }
    None
}

fn flatpak_from_exec(tokens: &[String]) -> Option<String> {
    let run = tokens.iter().position(|value| basename(value) == "flatpak")?;
    let run_kw = tokens.iter().skip(run + 1).position(|value| value == "run")? + run + 1;
    for token in tokens.iter().skip(run_kw + 1) {
        let token = clean_token(token);
        if token.starts_with('-') || token.is_empty() || is_field_code(&token) {
            continue;
        }
        return Some(token);
    }
    None
}

fn add_alias(
    aliases: &mut BTreeMap<String, Mapping>,
    conflicts: &mut BTreeSet<String>,
    alias: &str,
    mapping: &Mapping,
) {
    let alias = alias.trim();
    if alias.is_empty() || conflicts.contains(alias) {
        return;
    }
    if let Some(existing) = aliases.get(alias) {
        if existing.desktop_id != mapping.desktop_id {
            aliases.remove(alias);
            conflicts.insert(alias.to_string());
        }
        return;
    }
    aliases.insert(alias.to_string(), mapping.clone());
}

fn add_alias_with_normalized(
    aliases: &mut BTreeMap<String, Mapping>,
    conflicts: &mut BTreeSet<String>,
    alias: &str,
    mapping: &Mapping,
) {
    add_alias(aliases, conflicts, alias, mapping);
    let lower = alias.to_ascii_lowercase();
    if lower != alias {
        add_alias(aliases, conflicts, &lower, mapping);
    }
}

fn build_identity() -> BTreeMap<String, Mapping> {
    let icons = inventory_icons();
    let desktops = effective_desktops();
    let mut aliases = BTreeMap::new();
    let mut conflicts = BTreeSet::new();

    for (id, icon_key) in icons {
        let Some(path) = desktops.get(&id) else {
            continue;
        };
        let Some(app) = parse_desktop(id.clone(), path, icon_key) else {
            continue;
        };
        let icon = theme_icon(&app.icon_key, &app.id);
        let base = Mapping {
            desktop_id: app.id.clone(),
            theme_icon: icon,
            evidence: "desktop-id".to_string(),
        };
        add_alias_with_normalized(&mut aliases, &mut conflicts, &app.id, &base);
        if let Some(stripped) = app.id.strip_suffix(".desktop") {
            add_alias_with_normalized(&mut aliases, &mut conflicts, stripped, &base);
        }

        if !app.startup_wm_class.is_empty() {
            let mut mapping = base.clone();
            mapping.evidence = "StartupWMClass".to_string();
            add_alias_with_normalized(
                &mut aliases,
                &mut conflicts,
                &app.startup_wm_class,
                &mapping,
            );
        }

        let tokens = parse_exec_tokens(&app.exec)
            .unwrap_or_default()
            .into_iter()
            .filter(|value| !value.is_empty() && !value.starts_with('%'))
            .collect::<Vec<_>>();

        if let Some(value) = extract_flag(&tokens, "--class").or_else(|| extract_flag(&tokens, "--name")) {
            let mut mapping = base.clone();
            mapping.evidence = "exec-explicit-class".to_string();
            add_alias_with_normalized(&mut aliases, &mut conflicts, &value, &mapping);
        }
        if let Some(value) = extract_flag(&tokens, "--app-id") {
            let mut mapping = base.clone();
            mapping.evidence = "pwa-app-id".to_string();
            add_alias_with_normalized(&mut aliases, &mut conflicts, &value, &mapping);
        }

        if let Some(app_id) = steam_id(&app.exec, &tokens) {
            let mut mapping = base.clone();
            mapping.evidence = "steam-app-id".to_string();
            add_alias(&mut aliases, &mut conflicts, &format!("steam:{app_id}"), &mapping);
            add_alias(&mut aliases, &mut conflicts, &format!("steam_app_{app_id}"), &mapping);
        }

        let flatpak = if !app.flatpak_id.is_empty() {
            Some(app.flatpak_id.clone())
        } else {
            flatpak_from_exec(&tokens)
        };
        if let Some(flatpak) = flatpak {
            let mut mapping = base.clone();
            mapping.evidence = "flatpak-app-id".to_string();
            add_alias_with_normalized(&mut aliases, &mut conflicts, &flatpak, &mapping);
        }

        let mut command = None;
        for token in &tokens {
            if token.contains('=') && !token.starts_with('/') {
                continue;
            }
            let name = basename(token);
            if generic_runtime(&name) || name.starts_with('-') {
                continue;
            }
            command = Some(name);
            break;
        }
        if let Some(command) = command {
            let mut mapping = base.clone();
            mapping.evidence = "exact-executable".to_string();
            add_alias_with_normalized(&mut aliases, &mut conflicts, &command, &mapping);
        }
    }

    aliases
}

fn persist_identity(aliases: &BTreeMap<String, Mapping>) -> Result<(), String> {
    let mut applications = BTreeMap::<String, String>::new();
    for mapping in aliases.values() {
        applications
            .entry(mapping.desktop_id.clone())
            .or_insert_with(|| mapping.theme_icon.clone());
    }

    let mut sql = String::from(
        "PRAGMA journal_mode=WAL;\n\
         PRAGMA synchronous=NORMAL;\n\
         PRAGMA busy_timeout=5000;\n\
         CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);\n\
         CREATE TABLE IF NOT EXISTS applications (\n\
           canonical_app_id TEXT PRIMARY KEY,\n\
           launch_desktop_id TEXT NOT NULL,\n\
           theme_icon TEXT NOT NULL,\n\
           updated_ms INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS identity_aliases (\n\
           alias TEXT PRIMARY KEY,\n\
           canonical_app_id TEXT NOT NULL,\n\
           launch_desktop_id TEXT NOT NULL,\n\
           theme_icon TEXT NOT NULL,\n\
           evidence TEXT NOT NULL\n\
         );\n\
         CREATE INDEX IF NOT EXISTS identity_aliases_app_idx ON identity_aliases(canonical_app_id);\n\
         BEGIN IMMEDIATE;\n\
         DELETE FROM identity_aliases;\n\
         DELETE FROM applications;\n",
    );
    let timestamp = now_ms();
    for (desktop_id, theme_icon) in applications {
        sql.push_str(&format!(
            "INSERT INTO applications(canonical_app_id, launch_desktop_id, theme_icon, updated_ms) VALUES({}, {}, {}, {});\n",
            sql_quote(&desktop_id),
            sql_quote(&desktop_id),
            sql_quote(&theme_icon),
            timestamp,
        ));
    }
    for (alias, mapping) in aliases {
        sql.push_str(&format!(
            "INSERT INTO identity_aliases(alias, canonical_app_id, launch_desktop_id, theme_icon, evidence) VALUES({}, {}, {}, {}, {});\n",
            sql_quote(alias),
            sql_quote(&mapping.desktop_id),
            sql_quote(&mapping.desktop_id),
            sql_quote(&mapping.theme_icon),
            sql_quote(&mapping.evidence),
        ));
    }
    sql.push_str(&format!(
        "INSERT INTO meta(key, value) VALUES('identitySchemaVersion', '{}') ON CONFLICT(key) DO UPDATE SET value=excluded.value;\nCOMMIT;\n",
        IDENTITY_SCHEMA_VERSION
    ));
    sqlite(&sql)?;
    Ok(())
}

fn sync() -> Result<(), String> {
    let aliases = build_identity();
    persist_identity(&aliases)?;
    let rows = aliases
        .iter()
        .map(|(alias, mapping)| {
            format!(
                "\"{}\":{{\"desktopId\":\"{}\",\"themeIcon\":\"{}\",\"evidence\":\"{}\"}}",
                json_escape(alias),
                json_escape(&mapping.desktop_id),
                json_escape(&mapping.theme_icon),
                json_escape(&mapping.evidence),
            )
        })
        .collect::<Vec<_>>();
    let body = format!(
        "{{\"schemaVersion\":{},\"aliases\":{{{}}}}}\n",
        IDENTITY_SCHEMA_VERSION,
        rows.join(",")
    );
    write_atomic(&identity_path(), body)
}

fn resolve(value: &str) -> Result<(), String> {
    let aliases = build_identity();
    let direct = aliases.get(value).or_else(|| aliases.get(&value.to_ascii_lowercase()));
    if let Some(mapping) = direct {
        println!(
            "{{\"resolved\":true,\"desktopId\":\"{}\",\"themeIcon\":\"{}\",\"evidence\":\"{}\"}}",
            json_escape(&mapping.desktop_id),
            json_escape(&mapping.theme_icon),
            json_escape(&mapping.evidence),
        );
    } else {
        println!("{{\"resolved\":false}}");
    }
    Ok(())
}

fn daemon() -> Result<(), String> {
    loop {
        if let Err(error) = sync() {
            eprintln!("adaptive icon identity sync failed: {error}");
        }
        thread::sleep(Duration::from_secs(10));
    }
}

fn usage() -> ! {
    eprintln!(
        "vesper-icon-identity\n\
         commands:\n\
           sync\n\
           resolve <runtime-id>\n\
           daemon"
    );
    std::process::exit(2);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [command] if command == "sync" => sync(),
        [command, value] if command == "resolve" => resolve(value),
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
    use super::*;

    fn mapping(id: &str) -> Mapping {
        Mapping {
            desktop_id: id.to_string(),
            theme_icon: id.trim_end_matches(".desktop").to_string(),
            evidence: "test".to_string(),
        }
    }

    #[test]
    fn conflicting_aliases_are_removed_not_guessed() {
        let mut aliases = BTreeMap::new();
        let mut conflicts = BTreeSet::new();
        add_alias(&mut aliases, &mut conflicts, "shared", &mapping("one.desktop"));
        add_alias(&mut aliases, &mut conflicts, "shared", &mapping("two.desktop"));
        assert!(!aliases.contains_key("shared"));
        assert!(conflicts.contains("shared"));
    }

    #[test]
    fn steam_ids_stay_exact() {
        let tokens = vec!["steam".to_string(), "-applaunch".to_string(), "730".to_string()];
        assert_eq!(steam_id("steam -applaunch 730", &tokens).as_deref(), Some("730"));
        assert_eq!(steam_id("steam://rungameid/570", &[]).as_deref(), Some("570"));
    }

    #[test]
    fn generic_runtimes_are_never_identity() {
        for value in ["electron", "wine", "wine64", "steam", "flatpak"] {
            assert!(generic_runtime(value));
        }
    }

    #[test]
    fn normalized_alias_is_exact_casefold_only() {
        let mut aliases = BTreeMap::new();
        let mut conflicts = BTreeSet::new();
        let item = mapping("Example.desktop");
        add_alias_with_normalized(&mut aliases, &mut conflicts, "Example.App", &item);
        assert!(aliases.contains_key("Example.App"));
        assert!(aliases.contains_key("example.app"));
        assert!(!aliases.contains_key("example"));
    }

    #[test]
    fn exec_parser_preserves_quoted_paths_and_escaped_arguments() {
        assert_eq!(
            parse_exec_tokens("\"/opt/My App/bin\" --class=\"My\\\"App\" %U"),
            Some(vec![
                "/opt/My App/bin".to_string(),
                "--class=My\"App".to_string(),
                "%U".to_string(),
            ])
        );
        assert_eq!(parse_exec_tokens("/broken \"path"), None);
    }

    #[test]
    fn exec_field_codes_never_become_identity_aliases() {
        let tokens = parse_exec_tokens("app --class=%U").expect("valid Exec");
        assert_eq!(extract_flag(&tokens, "--class"), None);
    }

    #[test]
    fn desktop_visibility_honours_only_and_not_show_in() {
        let current = ["GNOME", "X-Custom"]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        assert!(desktop_visible(&[], &[], &current));
        assert!(desktop_visible(&["GNOME".to_string()], &[], &current));
        assert!(!desktop_visible(&["KDE".to_string()], &[], &current));
        assert!(!desktop_visible(&[], &["GNOME".to_string()], &current));
    }

    #[test]
    fn try_exec_requires_an_executable_file() {
        assert!(executable_available("/bin/sh"));
        assert!(!executable_available("/vesper/does-not-exist"));
    }
}
