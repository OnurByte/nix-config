use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PROVIDERS: &[(&str, &str, &str)] = &[
    ("openai", "OpenAI", "OPENAI_API_KEY"),
    ("anthropic", "Anthropic", "ANTHROPIC_API_KEY"),
    ("xai", "xAI", "XAI_API_KEY"),
    ("openrouter", "OpenRouter", "OPENROUTER_API_KEY"),
    ("google", "Google AI", "GEMINI_API_KEY"),
];

struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/nonexistent"))
}

fn state_root() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("vesper")
}

fn config_root() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("vesper")
}

fn runtime_root() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| state_root().join("runtime"))
        .join("vesper")
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

fn print_error(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

fn output(command: &str, args: &[&str]) -> Result<String, String> {
    let result = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run {command}: {error}"))?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{command} exited with {}", result.status.code().unwrap_or(-1))
        } else {
            stderr
        });
    }
    String::from_utf8(result.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|error| format!("invalid UTF-8 from {command}: {error}"))
}

fn success(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn provider(id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    PROVIDERS.iter().copied().find(|item| item.0 == id)
}

fn credential_configured(id: &str) -> bool {
    Command::new("secret-tool")
        .args(["lookup", "service", "vesper-ai", "provider", id])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn credential_lookup(id: &str) -> Result<String, String> {
    output("secret-tool", &["lookup", "service", "vesper-ai", "provider", id])
        .and_then(|value| if value.is_empty() { Err("credential is empty".to_string()) } else { Ok(value) })
}

fn stdin_line() -> Result<String, String> {
    let mut value = String::new();
    io::stdin()
        .lock()
        .read_line(&mut value)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(value
        .trim_end_matches(|ch| ch == '\n' || ch == '\r')
        .to_string())
}

fn credential_set(id: &str) -> Result<(), String> {
    let (_, label, _) = provider(id).ok_or_else(|| format!("unknown provider: {id}"))?;
    let key = stdin_line()?;
    let key = key.trim();
    if key.is_empty() {
        return Err("API key is empty".to_string());
    }

    let mut child = Command::new("secret-tool")
        .args([
            "store",
            &format!("--label=Vesper AI · {label}"),
            "service",
            "vesper-ai",
            "provider",
            id,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start secret-tool: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(key.as_bytes()).map_err(|error| error.to_string())?;
        stdin.write_all(b"\n").map_err(|error| error.to_string())?;
    }

    let result = child.wait_with_output().map_err(|error| error.to_string())?;
    if result.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        Err(if message.is_empty() { "Secret Service rejected the API key".to_string() } else { message })
    }
}

fn credential_clear(id: &str) -> Result<(), String> {
    provider(id).ok_or_else(|| format!("unknown provider: {id}"))?;
    let result = Command::new("secret-tool")
        .args(["clear", "service", "vesper-ai", "provider", id])
        .output()
        .map_err(|error| format!("failed to run secret-tool: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        Err(if message.is_empty() { "failed to clear credential".to_string() } else { message })
    }
}

fn credential_exec(id: &str, command: &[String]) -> ! {
    let (_, _, env_name) = provider(id).unwrap_or_else(|| print_error(&format!("unknown provider: {id}")));
    if command.is_empty() {
        print_error("credential exec needs a command");
    }
    let key = credential_lookup(id).unwrap_or_else(|error| print_error(&error));
    let error = Command::new(&command[0])
        .args(&command[1..])
        .env(env_name, key)
        .exec();
    print_error(&format!("failed to exec {}: {error}", command[0]));
}

fn list_dir_names(path: &Path) -> Vec<String> {
    let mut items = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    items.sort();
    items
}

fn mcp_names() -> Vec<String> {
    let path = config_root().join("mcp-servers");
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn ai_status() {
    let credentials = PROVIDERS
        .iter()
        .map(|(id, name, env_name)| {
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"env\":\"{}\",\"configured\":{}}}",
                json_escape(id),
                json_escape(name),
                json_escape(env_name),
                if credential_configured(id) { "true" } else { "false" }
            )
        })
        .collect::<Vec<_>>();

    let skills = list_dir_names(&home().join(".agents/skills"));
    let skills_json = skills
        .iter()
        .map(|name| format!("\"{}\"", json_escape(name)))
        .collect::<Vec<_>>();
    let mcp = mcp_names();
    let mcp_json = mcp
        .iter()
        .map(|name| format!("\"{}\"", json_escape(name)))
        .collect::<Vec<_>>();
    let hermes_registry = home().join(".config/vesper/hermes-jobs.json").exists();

    println!(
        "{{\"credentials\":[{}],\"skills\":{{\"count\":{},\"items\":[{}]}},\"mcp\":{{\"count\":{},\"items\":[{}]}},\"hermesRegistry\":{}}}",
        credentials.join(","),
        skills.len(),
        skills_json.join(","),
        mcp.len(),
        mcp_json.join(","),
        if hermes_registry { "true" } else { "false" }
    );
}

fn radio_status() -> (bool, bool) {
    let wifi = output("nmcli", &["radio", "wifi"])
        .map(|value| value == "enabled")
        .unwrap_or(false);
    let bluetooth = output("bluetoothctl", &["show"])
        .map(|value| value.lines().any(|line| line.trim() == "Powered: yes"))
        .unwrap_or(false);
    (wifi, bluetooth)
}

fn active_connection() -> Option<String> {
    let text = output("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show", "--active"]).ok()?;
    for line in text.lines() {
        if let Some((name, kind)) = line.rsplit_once(':') {
            if kind == "802-11-wireless" || kind == "wifi" {
                return Some(name.replace("\\:", ":"));
            }
        }
    }
    None
}

fn network_status() {
    let (wifi, bluetooth) = radio_status();
    let connection = active_connection().unwrap_or_default();
    let zapret = success("systemctl", &["is-active", "--quiet", "zapret2.service"])
        || success("systemctl", &["is-active", "--quiet", "zapret2"]);
    let proxy = config_root().join("proxy.env").exists();
    println!(
        "{{\"airplane\":{},\"wifi\":{},\"bluetooth\":{},\"connection\":\"{}\",\"zapret\":{},\"proxy\":{}}}",
        if !wifi && !bluetooth { "true" } else { "false" },
        if wifi { "true" } else { "false" },
        if bluetooth { "true" } else { "false" },
        json_escape(&connection),
        if zapret { "true" } else { "false" },
        if proxy { "true" } else { "false" }
    );
}

fn airplane(enabled: bool) -> Result<(), String> {
    let radio = if enabled { "off" } else { "on" };
    let bt = if enabled { "off" } else { "on" };
    if !success("nmcli", &["radio", "all", radio]) {
        return Err("NetworkManager rejected radio change".to_string());
    }
    let _ = Command::new("bluetoothctl")
        .args(["power", bt])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(())
}

fn wifi_qr() -> Result<PathBuf, String> {
    let connection = active_connection().ok_or_else(|| "no active Wi-Fi connection".to_string())?;
    let values = output(
        "nmcli",
        &[
            "-s",
            "-g",
            "802-11-wireless.ssid,802-11-wireless-security.key-mgmt,802-11-wireless-security.psk",
            "connection",
            "show",
            &connection,
        ],
    )?;
    let mut lines = values.lines();
    let ssid = lines.next().unwrap_or("").trim();
    let key_mgmt = lines.next().unwrap_or("").trim();
    let password = lines.next().unwrap_or("").trim();
    if ssid.is_empty() {
        return Err("active Wi-Fi has no SSID".to_string());
    }

    fn wifi_escape(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace(';', "\\;")
            .replace(',', "\\,")
            .replace(':', "\\:")
    }

    let auth = if key_mgmt.is_empty() || key_mgmt == "none" { "nopass" } else { "WPA" };
    let payload = format!(
        "WIFI:T:{};S:{};P:{};;",
        auth,
        wifi_escape(ssid),
        wifi_escape(password)
    );
    let root = runtime_root();
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let path = root.join("wifi-share.svg");
    let path_string = path.to_string_lossy().into_owned();
    let result = Command::new("qrencode")
        .args(["-t", "SVG", "-o", &path_string, "-m", "2", &payload])
        .output()
        .map_err(|error| format!("failed to run qrencode: {error}"))?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
    }
    Ok(path)
}

fn proxy_set() -> Result<(), String> {
    let value = stdin_line()?;
    let value = value.trim();
    if value.is_empty() || value.contains('\n') || value.contains('\r') || value.contains('"') {
        return Err("invalid proxy URL".to_string());
    }
    if !(value.starts_with("http://") || value.starts_with("https://") || value.starts_with("socks5://")) {
        return Err("proxy must start with http://, https:// or socks5://".to_string());
    }
    let root = config_root();
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    fs::write(root.join("proxy.env"), format!("{value}\n")).map_err(|error| error.to_string())?;

    let env_dir = home().join(".config/environment.d");
    fs::create_dir_all(&env_dir).map_err(|error| error.to_string())?;
    let content = if value.starts_with("socks5://") {
        format!("ALL_PROXY=\"{value}\"\nall_proxy=\"{value}\"\n")
    } else {
        format!(
            "HTTP_PROXY=\"{0}\"\nHTTPS_PROXY=\"{0}\"\nhttp_proxy=\"{0}\"\nhttps_proxy=\"{0}\"\n",
            value
        )
    };
    fs::write(env_dir.join("90-vesper-proxy.conf"), content).map_err(|error| error.to_string())?;
    Ok(())
}

fn proxy_clear() -> Result<(), String> {
    for path in [
        config_root().join("proxy.env"),
        home().join(".config/environment.d/90-vesper-proxy.conf"),
    ] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn today() -> String {
    output("date", &["+%F"]).unwrap_or_else(|_| "unknown-date".to_string())
}

fn wellbeing_dir() -> PathBuf {
    state_root().join("wellbeing")
}

fn sanitise_app(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch == '\t' || ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect::<String>()
        .trim()
        .to_string()
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let index = json.find(&needle)?;
    let rest = &json[index + needle.len()..];
    let colon = rest.find(':')?;
    let mut chars = rest[colon + 1..].chars().peekable();
    while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
        chars.next();
    }
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
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
    let value = sanitise_app(&value);
    if value.is_empty() { None } else { Some(value) }
}

fn load_wellbeing(path: &Path) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        if let Some((name, seconds)) = line.rsplit_once('\t') {
            if let Ok(seconds) = seconds.parse::<u64>() {
                map.insert(name.to_string(), seconds);
            }
        }
    }
    map
}

fn write_wellbeing(path: &Path, values: &BTreeMap<String, u64>) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "invalid wellbeing path".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(".{}.tmp", std::process::id()));
    let mut data = String::new();
    for (name, seconds) in values {
        data.push_str(name);
        data.push('\t');
        data.push_str(&seconds.to_string());
        data.push('\n');
    }
    fs::write(&tmp, data).map_err(|error| error.to_string())?;
    fs::rename(tmp, path).map_err(|error| error.to_string())
}

fn acquire_wellbeing_lock() -> Result<LockGuard, String> {
    let dir = wellbeing_dir();
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let lock = dir.join("daemon.lock");

    for _ in 0..2 {
        match OpenOptions::new().write(true).create_new(true).open(&lock) {
            Ok(mut file) => {
                let _ = writeln!(file, "{}", std::process::id());
                return Ok(LockGuard(lock));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
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

fn wellbeing_daemon() -> Result<(), String> {
    let _lock = acquire_wellbeing_lock()?;
    let mut day = today();
    let mut path = wellbeing_dir().join(format!("{day}.tsv"));
    let mut values = load_wellbeing(&path);
    let tick = Duration::from_secs(5);
    let mut last_flush = Instant::now();

    loop {
        let started = Instant::now();
        let current_day = today();
        if current_day != day {
            write_wellbeing(&path, &values)?;
            day = current_day;
            path = wellbeing_dir().join(format!("{day}.tsv"));
            values = load_wellbeing(&path);
        }
        if let Some(app) = active_app() {
            *values.entry(app).or_insert(0) += tick.as_secs();
        }
        if last_flush.elapsed() >= Duration::from_secs(30) {
            write_wellbeing(&path, &values)?;
            last_flush = Instant::now();
        }
        let elapsed = started.elapsed();
        if elapsed < tick {
            thread::sleep(tick - elapsed);
        }
    }
}

fn wellbeing_summary() {
    let path = wellbeing_dir().join(format!("{}.tsv", today()));
    let values = load_wellbeing(&path);
    let total = values.values().sum::<u64>();
    let mut items = values.into_iter().collect::<Vec<_>>();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.truncate(12);
    let json = items
        .iter()
        .map(|(name, seconds)| format!("{{\"app\":\"{}\",\"seconds\":{}}}", json_escape(name), seconds))
        .collect::<Vec<_>>();
    println!("{{\"totalSeconds\":{},\"apps\":[{}]}}", total, json.join(","));
}

fn normalise_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn wellbeing_seconds_for(id: &str) -> u64 {
    let target = normalise_id(id.strip_suffix(".desktop").unwrap_or(id));
    if target.is_empty() {
        return 0;
    }
    let path = wellbeing_dir().join(format!("{}.tsv", today()));
    load_wellbeing(&path)
        .into_iter()
        .filter(|(name, _)| {
            let name = normalise_id(name);
            name.contains(&target) || target.contains(&name)
        })
        .map(|(_, seconds)| seconds)
        .sum()
}

fn flatpak_id(id: &str) -> &str {
    id.strip_suffix(".desktop").unwrap_or(id)
}

fn app_status(id: &str) {
    let flatpak_id = flatpak_id(id);
    let is_flatpak = success("flatpak", &["info", flatpak_id]);
    let permissions = if is_flatpak {
        output("flatpak", &["info", "--show-permissions", flatpak_id]).unwrap_or_default()
    } else {
        String::new()
    };
    println!(
        "{{\"sandbox\":\"{}\",\"flatpakId\":\"{}\",\"permissions\":\"{}\",\"todaySeconds\":{}}}",
        if is_flatpak { "flatpak" } else { "native" },
        json_escape(flatpak_id),
        json_escape(&permissions),
        wellbeing_seconds_for(id)
    );
}

fn app_permission(id: &str, permission: &str, enabled: bool) -> Result<(), String> {
    let id = flatpak_id(id);
    if !success("flatpak", &["info", id]) {
        return Err("app is not installed as Flatpak".to_string());
    }
    let flag = match (permission, enabled) {
        ("network", true) => "--share=network",
        ("network", false) => "--unshare=network",
        ("home", true) => "--filesystem=home",
        ("home", false) => "--nofilesystem=home",
        _ => return Err(format!("unsupported permission: {permission}")),
    };
    let result = Command::new("flatpak")
        .args(["override", "--user", flag, id])
        .output()
        .map_err(|error| format!("failed to run flatpak: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

fn app_reset_permissions(id: &str) -> Result<(), String> {
    let id = flatpak_id(id);
    if !success("flatpak", &["info", id]) {
        return Err("app is not installed as Flatpak".to_string());
    }
    let result = Command::new("flatpak")
        .args(["override", "--user", "--reset", id])
        .output()
        .map_err(|error| format!("failed to run flatpak: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

fn icon_state_path() -> PathBuf {
    state_root().join("adaptive-icons/enabled")
}

fn icon_enabled() -> bool {
    fs::read_to_string(icon_state_path())
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

fn icon_set(enabled: bool) -> Result<(), String> {
    let path = icon_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, if enabled { "1\n" } else { "0\n" }).map_err(|error| error.to_string())
}

fn icon_request(app_id: &str, icon: &str) -> Result<(), String> {
    if !icon_enabled() {
        return Err("adaptive icons are disabled".to_string());
    }
    let dir = state_root().join("adaptive-icons/queue");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let safe_name = app_id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' { ch } else { '_' })
        .collect::<String>();
    let body = format!(
        "{{\"schemaVersion\":1,\"appId\":\"{}\",\"sourceIcon\":\"{}\",\"state\":\"queued\"}}\n",
        json_escape(app_id),
        json_escape(icon)
    );
    fs::write(dir.join(format!("{safe_name}.json")), body).map_err(|error| error.to_string())
}

fn usage() -> ! {
    eprintln!(
        "vesper-control\n\
         commands:\n\
           ai-status\n\
           credential status|set|clear <provider>\n\
           credential exec <provider> <command> [args...]\n\
           network status\n\
           network airplane on|off\n\
           network wifi-qr\n\
           proxy status|set|clear\n\
           wellbeing-daemon\n\
           wellbeing-summary\n\
           app-status <desktop-id>\n\
           app-permission <desktop-id> network|home on|off\n\
           app-reset-permissions <desktop-id>\n\
           icon status|on|off\n\
           icon request <desktop-id> [source-icon]"
    );
    std::process::exit(2);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "ai-status" => ai_status(),
        [group, action, id] if group == "credential" && action == "status" => {
            if provider(id).is_none() {
                print_error(&format!("unknown provider: {id}"));
            }
            println!("{}", if credential_configured(id) { "configured" } else { "missing" });
        }
        [group, action, id] if group == "credential" && action == "set" => {
            credential_set(id).unwrap_or_else(|error| print_error(&error));
        }
        [group, action, id] if group == "credential" && action == "clear" => {
            credential_clear(id).unwrap_or_else(|error| print_error(&error));
        }
        [group, action, id, command @ ..] if group == "credential" && action == "exec" => {
            credential_exec(id, command);
        }
        [group, action] if group == "network" && action == "status" => network_status(),
        [group, action, value] if group == "network" && action == "airplane" => match value.as_str() {
            "on" => airplane(true).unwrap_or_else(|error| print_error(&error)),
            "off" => airplane(false).unwrap_or_else(|error| print_error(&error)),
            _ => usage(),
        },
        [group, action] if group == "network" && action == "wifi-qr" => {
            let path = wifi_qr().unwrap_or_else(|error| print_error(&error));
            println!("{}", path.display());
        }
        [group, action] if group == "proxy" && action == "status" => {
            println!("{}", if config_root().join("proxy.env").exists() { "configured" } else { "off" });
        }
        [group, action] if group == "proxy" && action == "set" => {
            proxy_set().unwrap_or_else(|error| print_error(&error));
        }
        [group, action] if group == "proxy" && action == "clear" => {
            proxy_clear().unwrap_or_else(|error| print_error(&error));
        }
        [command] if command == "wellbeing-daemon" => {
            wellbeing_daemon().unwrap_or_else(|error| print_error(&error));
        }
        [command] if command == "wellbeing-summary" => wellbeing_summary(),
        [command, id] if command == "app-status" => app_status(id),
        [command, id, permission, value] if command == "app-permission" => {
            let enabled = match value.as_str() {
                "on" => true,
                "off" => false,
                _ => usage(),
            };
            app_permission(id, permission, enabled).unwrap_or_else(|error| print_error(&error));
        }
        [command, id] if command == "app-reset-permissions" => {
            app_reset_permissions(id).unwrap_or_else(|error| print_error(&error));
        }
        [group, action] if group == "icon" && action == "status" => {
            println!("{}", if icon_enabled() { "on" } else { "off" });
        }
        [group, action] if group == "icon" && (action == "on" || action == "off") => {
            icon_set(action == "on").unwrap_or_else(|error| print_error(&error));
        }
        [group, action, app_id] if group == "icon" && action == "request" => {
            icon_request(app_id, "").unwrap_or_else(|error| print_error(&error));
        }
        [group, action, app_id, icon] if group == "icon" && action == "request" => {
            icon_request(app_id, icon).unwrap_or_else(|error| print_error(&error));
        }
        _ => usage(),
    }
}
