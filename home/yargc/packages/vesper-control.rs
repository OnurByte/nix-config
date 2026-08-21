use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::os::unix::fs::OpenOptionsExt;
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

#[derive(Clone, Copy)]
struct VicinaeSettings {
    follow_theme: bool,
    follow_icons: bool,
    use_glass: bool,
    close_on_focus_loss: bool,
    pop_to_root_on_close: bool,
    layer_shell: bool,
}

impl Default for VicinaeSettings {
    fn default() -> Self {
        Self {
            follow_theme: true,
            follow_icons: true,
            use_glass: true,
            close_on_focus_loss: true,
            pop_to_root_on_close: true,
            layer_shell: true,
        }
    }
}

fn xdg_config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
}

fn xdg_state_home() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
}

fn xdg_data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

fn vicinae_config_dir() -> PathBuf {
    xdg_config_home().join("vicinae")
}

fn vicinae_settings_state_path() -> PathBuf {
    config_root().join("vicinae.conf")
}

fn vicinae_import_path() -> PathBuf {
    vicinae_config_dir().join("vesper.json")
}

fn vicinae_theme_path(mode: &str) -> PathBuf {
    xdg_data_home()
        .join("vicinae/themes")
        .join(format!("vesper-{mode}.toml"))
}

fn write_atomic(path: &Path, body: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid path: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("vesper");
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    fs::write(&temporary, body).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

fn parse_bool(value: &str) -> bool {
    matches!(value.trim(), "1" | "true" | "on" | "yes")
}

fn load_vicinae_settings() -> VicinaeSettings {
    let mut settings = VicinaeSettings::default();
    let content = fs::read_to_string(vicinae_settings_state_path()).unwrap_or_default();
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "followTheme" => settings.follow_theme = parse_bool(value),
            "followIcons" => settings.follow_icons = parse_bool(value),
            "useGlass" => settings.use_glass = parse_bool(value),
            "closeOnFocusLoss" => settings.close_on_focus_loss = parse_bool(value),
            "popToRootOnClose" => settings.pop_to_root_on_close = parse_bool(value),
            "layerShell" => settings.layer_shell = parse_bool(value),
            _ => {}
        }
    }
    settings
}

fn save_vicinae_settings(settings: VicinaeSettings) -> Result<(), String> {
    let body = format!(
        "followTheme={}\nfollowIcons={}\nuseGlass={}\ncloseOnFocusLoss={}\npopToRootOnClose={}\nlayerShell={}\n",
        if settings.follow_theme { 1 } else { 0 },
        if settings.follow_icons { 1 } else { 0 },
        if settings.use_glass { 1 } else { 0 },
        if settings.close_on_focus_loss { 1 } else { 0 },
        if settings.pop_to_root_on_close { 1 } else { 0 },
        if settings.layer_shell { 1 } else { 0 },
    );
    write_atomic(&vicinae_settings_state_path(), &body)
}

fn normalise_hex(value: &str) -> Option<String> {
    let value = value
        .trim()
        .trim_matches(|ch| matches!(ch, '"' | '\'' | ';' | ','))
        .trim_start_matches('#');
    if value.len() == 6 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Some(format!("#{}", value.to_ascii_lowercase()))
    } else {
        None
    }
}

fn current_accent() -> String {
    let lua = xdg_config_home().join("hypr/scheme/current.lua");
    if let Ok(content) = fs::read_to_string(lua) {
        if let Some(value) = content.lines().find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "primary").then(|| normalise_hex(value)).flatten()
        }) {
            return value;
        }
    }

    let generated = xdg_state_home().join("caelestia/theme/vesper-icons");
    fs::read_to_string(generated)
        .ok()
        .and_then(|value| normalise_hex(&value))
        .unwrap_or_else(|| "#89b4fa".to_string())
}

fn vicinae_theme(mode: &str, accent: &str) -> String {
    let light = mode == "light";
    let (background, foreground, secondary, border, selection) = if light {
        ("#f7f8fc", "#1a1d24", "#eef0f6", "#8a919f", "#dbe5ff")
    } else {
        ("#111318", "#f1f3f9", "#1c1f26", "#3f4654", "#29344a")
    };
    let name = if light { "Vesper Light" } else { "Vesper Dark" };
    format!(
        "[meta]\nversion = 1\nname = \"{name}\"\ndescription = \"Vesper system palette with the current Caelestia accent\"\nvariant = \"{mode}\"\ninherits = \"vicinae-{mode}\"\n\n[colors.core]\nbackground = \"{background}\"\nforeground = \"{foreground}\"\nsecondary_background = \"{secondary}\"\nborder = \"{border}\"\naccent = \"{accent}\"\n\n[colors.accents]\nblue = \"{accent}\"\ngreen = \"#81c995\"\nmagenta = \"#d7a8e8\"\norange = \"#f2b880\"\npurple = \"#c6b1f5\"\nred = \"#f28b82\"\nyellow = \"#fdd663\"\ncyan = \"#78d5e8\"\n\n[colors.list.item.selection]\nbackground = \"{selection}\"\nsecondary_background = \"{secondary}\"\n\n[colors.grid.item]\nbackground = \"{secondary}\"\n"
    )
}

fn vicinae_import(settings: VicinaeSettings) -> String {
    let light_theme = if settings.follow_theme { "vesper-light" } else { "vicinae-light" };
    let dark_theme = if settings.follow_theme { "vesper-dark" } else { "vicinae-dark" };
    let icon_theme = if settings.follow_icons { "Vesper-Adaptive" } else { "auto" };
    let opacity = if settings.use_glass { "0.92" } else { "1.0" };
    format!(
        "{{\n  \"$schema\": \"https://vicinae.com/schemas/config.json\",\n  \"close_on_focus_loss\": {},\n  \"pop_to_root_on_close\": {},\n  \"theme\": {{\n    \"light\": {{\"name\": \"{light_theme}\", \"icon_theme\": \"{icon_theme}\"}},\n    \"dark\": {{\"name\": \"{dark_theme}\", \"icon_theme\": \"{icon_theme}\"}}\n  }},\n  \"launcher_window\": {{\n    \"layer_shell\": {{\"enabled\": {}}},\n    \"opacity\": {opacity}\n  }}\n}}\n",
        if settings.close_on_focus_loss { "true" } else { "false" },
        if settings.pop_to_root_on_close { "true" } else { "false" },
        if settings.layer_shell { "true" } else { "false" },
    )
}

fn vicinae_sync_theme() -> Result<(), String> {
    let settings = load_vicinae_settings();
    let accent = current_accent();
    save_vicinae_settings(settings)?;
    write_atomic(&vicinae_import_path(), &vicinae_import(settings))?;
    write_atomic(&vicinae_theme_path("light"), &vicinae_theme("light", &accent))?;
    write_atomic(&vicinae_theme_path("dark"), &vicinae_theme("dark", &accent))
}

fn vicinae_status() {
    let settings = load_vicinae_settings();
    println!(
        "{{\"followTheme\":{},\"followIcons\":{},\"useGlass\":{},\"closeOnFocusLoss\":{},\"popToRootOnClose\":{},\"layerShell\":{},\"accent\":\"{}\"}}",
        settings.follow_theme,
        settings.follow_icons,
        settings.use_glass,
        settings.close_on_focus_loss,
        settings.pop_to_root_on_close,
        settings.layer_shell,
        json_escape(&current_accent()),
    );
}

fn vicinae_set(key: &str, value: &str) -> Result<(), String> {
    let mut settings = load_vicinae_settings();
    let value = match value {
        "on" | "off" | "true" | "false" | "1" | "0" => parse_bool(value),
        _ => return Err("Vicinae setting value must be on or off".to_string()),
    };
    match key {
        "follow-theme" => settings.follow_theme = value,
        "follow-icons" => settings.follow_icons = value,
        "use-glass" => settings.use_glass = value,
        "close-on-focus-loss" => settings.close_on_focus_loss = value,
        "pop-to-root-on-close" => settings.pop_to_root_on_close = value,
        "layer-shell" => settings.layer_shell = value,
        _ => return Err(format!("unknown Vicinae setting: {key}")),
    }
    save_vicinae_settings(settings)?;
    vicinae_sync_theme()
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

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct RadioState {
    wifi: bool,
    wwan: bool,
    bluetooth: Option<bool>,
}

fn nm_radio_state(radio: &str) -> Result<bool, String> {
    match output("nmcli", &["radio", radio])?.as_str() {
        "enabled" => Ok(true),
        "disabled" => Ok(false),
        value => Err(format!("NetworkManager returned an unknown {radio} state: {value}")),
    }
}

fn bluetooth_power_state() -> Result<Option<bool>, String> {
    let output = output("bluetoothctl", &["show"])?;
    for line in output.lines() {
        if let Some(value) = line.trim().strip_prefix("Powered: ") {
            return match value {
                "yes" => Ok(Some(true)),
                "no" => Ok(Some(false)),
                value => Err(format!("bluetoothctl returned an unknown power state: {value}")),
            };
        }
    }
    Ok(None)
}

fn radio_state() -> Result<RadioState, String> {
    Ok(RadioState {
        wifi: nm_radio_state("wifi")?,
        wwan: nm_radio_state("wwan")?,
        bluetooth: bluetooth_power_state()?,
    })
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
    let radios = radio_state().unwrap_or_default();
    let wifi = radios.wifi;
    let bluetooth = radios.bluetooth.unwrap_or(false);
    let connection = active_connection().unwrap_or_default();
    let zapret = success("systemctl", &["is-active", "--quiet", "zapret2.service"])
        || success("systemctl", &["is-active", "--quiet", "zapret2"]);
    let proxy = proxy_environment_path().is_file();
    println!(
        "{{\"airplane\":{},\"wifi\":{},\"wwan\":{},\"bluetooth\":{},\"bluetoothAvailable\":{},\"connection\":\"{}\",\"zapret\":{},\"proxy\":{}}}",
        if airplane_state_path().is_file() { "true" } else { "false" },
        if wifi { "true" } else { "false" },
        if radios.wwan { "true" } else { "false" },
        if bluetooth { "true" } else { "false" },
        if radios.bluetooth.is_some() { "true" } else { "false" },
        json_escape(&connection),
        if zapret { "true" } else { "false" },
        if proxy { "true" } else { "false" }
    );
}

fn airplane(enabled: bool) -> Result<(), String> {
    let state_path = airplane_state_path();
    if enabled {
        if state_path.is_file() {
            return Err("airplane mode is already enabled".to_string());
        }
        let state = radio_state()?;
        write_airplane_state(state)?;
        set_nm_radio("wifi", false)?;
        set_nm_radio("wwan", false)?;
        if state.bluetooth.is_some() {
            set_bluetooth_power(false)?;
        }
        Ok(())
    } else {
        let state = read_airplane_state()?;
        set_nm_radio("wifi", state.wifi)?;
        set_nm_radio("wwan", state.wwan)?;
        if let Some(powered) = state.bluetooth {
            set_bluetooth_power(powered)?;
        }
        match fs::remove_file(state_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

fn set_nm_radio(radio: &str, enabled: bool) -> Result<(), String> {
    let value = if enabled { "on" } else { "off" };
    if success("nmcli", &["radio", radio, value]) {
        Ok(())
    } else {
        Err(format!("NetworkManager rejected {radio} radio change"))
    }
}

fn set_bluetooth_power(enabled: bool) -> Result<(), String> {
    let value = if enabled { "on" } else { "off" };
    if success("bluetoothctl", &["power", value]) {
        Ok(())
    } else {
        Err("bluetoothctl rejected the power change".to_string())
    }
}

fn airplane_state_path() -> PathBuf {
    runtime_root().join("airplane-state")
}

fn write_airplane_state(state: RadioState) -> Result<(), String> {
    let bluetooth = match state.bluetooth {
        Some(true) => "on",
        Some(false) => "off",
        None => "unavailable",
    };
    write_atomic(
        &airplane_state_path(),
        &format!("wifi={}\nwwan={}\nbluetooth={bluetooth}\n", if state.wifi { "on" } else { "off" }, if state.wwan { "on" } else { "off" }),
    )
}

fn read_airplane_state() -> Result<RadioState, String> {
    let content = fs::read_to_string(airplane_state_path())
        .map_err(|error| format!("airplane state is unavailable: {error}"))?;
    let mut wifi = None;
    let mut wwan = None;
    let mut bluetooth = None;
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        match key {
            "wifi" => wifi = Some(parse_radio_state(value)?),
            "wwan" => wwan = Some(parse_radio_state(value)?),
            "bluetooth" => bluetooth = match value {
                "on" => Some(Some(true)),
                "off" => Some(Some(false)),
                "unavailable" => Some(None),
                value => return Err(format!("invalid saved Bluetooth state: {value}")),
            },
            _ => {}
        }
    }
    Ok(RadioState {
        wifi: wifi.ok_or("saved Wi-Fi state is missing")?,
        wwan: wwan.ok_or("saved WWAN state is missing")?,
        bluetooth: bluetooth.ok_or("saved Bluetooth state is missing")?,
    })
}

fn parse_radio_state(value: &str) -> Result<bool, String> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        value => Err(format!("invalid saved radio state: {value}")),
    }
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
    let mut child = Command::new("qrencode")
        .args(["-t", "SVG", "-o", &path_string, "-m", "2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start qrencode: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(payload.as_bytes())
            .map_err(|error| error.to_string())?;
        stdin
            .write_all(b"\n")
            .map_err(|error| error.to_string())?;
    }

    let result = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for qrencode: {error}"))?;
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
    let content = if value.starts_with("socks5://") {
        format!("ALL_PROXY=\"{value}\"\nall_proxy=\"{value}\"\n")
    } else {
        format!(
            "HTTP_PROXY=\"{0}\"\nHTTPS_PROXY=\"{0}\"\nhttp_proxy=\"{0}\"\nhttps_proxy=\"{0}\"\n",
            value
        )
    };
    // The effective environment file is the single status authority. Do not
    // commit a separate marker before this write succeeds.
    write_private_atomic(&proxy_environment_path(), &content)
}

fn proxy_clear() -> Result<(), String> {
    for path in [
        proxy_environment_path(),
        config_root().join("proxy.env"), // legacy marker from older builds
    ] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn proxy_environment_path() -> PathBuf {
    home().join(".config/environment.d/90-vesper-proxy.conf")
}

fn write_private_atomic(path: &Path, body: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid path: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("vesper");
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(body.as_bytes()).map_err(|error| error.to_string())?;
    drop(file);
    fs::rename(&temporary, path).map_err(|error| error.to_string())
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

fn identity_path() -> PathBuf {
    xdg_state_home().join("vesper/adaptive-icons/identity.json")
}

fn canonical_app_id(runtime_id: &str) -> Option<String> {
    let output = Command::new("jq")
        .args([
            "-r",
            "--arg",
            "alias",
            runtime_id,
            "(.aliases[$alias] // .aliases[($alias | ascii_downcase)]).desktopId // empty",
        ])
        .arg(identity_path())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn active_app() -> Option<String> {
    let json = output("hyprctl", &["activewindow", "-j"]).ok()?;
    let value = extract_json_string(&json, "class")
        .or_else(|| extract_json_string(&json, "initialClass"))?;
    let value = sanitise_app(&value);
    if value.is_empty() {
        None
    } else {
        Some(canonical_app_id(&value).unwrap_or(value))
    }
}

fn session_allows_wellbeing(properties: &str) -> Option<bool> {
    let mut idle_hint = None;
    let mut locked_hint = None;
    for line in properties.lines() {
        if let Some(value) = line.strip_prefix("IdleHint=") {
            idle_hint = Some(value.trim());
        } else if let Some(value) = line.strip_prefix("LockedHint=") {
            locked_hint = Some(value.trim());
        }
    }
    Some(idle_hint? == "no" && locked_hint? == "no")
}

fn session_allows_wellbeing_live() -> Option<bool> {
    let session = env::var("XDG_SESSION_ID").ok()?;
    if session.trim().is_empty() {
        return None;
    }
    let output = Command::new("loginctl")
        .args(["show-session", session.trim(), "-p", "IdleHint", "-p", "LockedHint"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    session_allows_wellbeing(&String::from_utf8_lossy(&output.stdout))
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
        // ponytail: one bounded logind query per sample; move to an event
        // subscription if session-state probing becomes measurable overhead.
        if session_allows_wellbeing_live() == Some(true) {
            if let Some(app) = active_app() {
                *values.entry(app).or_insert(0) += tick.as_secs();
            }
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

fn wellbeing_identity_keys(id: &str, canonical: Option<&str>) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(canonical) = canonical.filter(|value| !value.is_empty()) {
        keys.push(canonical.to_string());
    }
    if !id.is_empty() && !keys.iter().any(|key| key == id) {
        keys.push(id.to_string());
    }
    if let Some(stripped) = id.strip_suffix(".desktop") {
        if !stripped.is_empty() && !keys.iter().any(|key| key == stripped) {
            keys.push(stripped.to_string());
        }
    }
    keys
}

fn wellbeing_seconds_for(id: &str) -> u64 {
    let canonical = canonical_app_id(id);
    let keys = wellbeing_identity_keys(id, canonical.as_deref());
    if keys.is_empty() {
        return 0;
    }
    let path = wellbeing_dir().join(format!("{}.tsv", today()));
    load_wellbeing(&path)
        .into_iter()
        .filter(|(name, _)| keys.iter().any(|key| name == key))
        .map(|(_, seconds)| seconds)
        .sum()
}

#[derive(Clone, Debug)]
struct DesktopOwner {
    path: PathBuf,
    flatpak_id: Option<String>,
    hidden: bool,
}

#[derive(Clone, Debug)]
struct FlatpakOwner {
    desktop_path: PathBuf,
    id: String,
    scope: &'static str,
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn application_data_dirs() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_unique_path(&mut paths, xdg_data_home());
    for path in env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string())
        .split(':')
        .filter(|value| !value.is_empty())
    {
        push_unique_path(&mut paths, PathBuf::from(path));
    }
    for path in [
        home().join(".nix-profile/share"),
        xdg_data_home().join("flatpak/exports/share"),
        PathBuf::from("/var/lib/flatpak/exports/share"),
        PathBuf::from("/run/current-system/sw/share"),
    ] {
        if path.exists() {
            push_unique_path(&mut paths, path);
        }
    }
    if let Ok(user) = env::var("USER") {
        let path = PathBuf::from("/etc/profiles/per-user").join(user).join("share");
        if path.exists() {
            push_unique_path(&mut paths, path);
        }
    }
    paths
}

fn desktop_file_id(path: &Path, root: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let mut parts = relative.components();
    let mut id = String::new();
    while let Some(component) = parts.next() {
        let value = component.as_os_str().to_str()?;
        if !id.is_empty() {
            id.push('-');
        }
        id.push_str(value);
    }
    Some(id)
}

fn find_desktop_file(root: &Path, current: &Path, wanted: &str, depth: usize) -> Option<PathBuf> {
    if depth > 8 {
        return None;
    }
    let entries = fs::read_dir(current).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let file_type = entry.file_type().ok()?;
        if file_type.is_dir() {
            if let Some(found) = find_desktop_file(root, &path, wanted, depth + 1) {
                return Some(found);
            }
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("desktop")
            && desktop_file_id(&path, root).as_deref() == Some(wanted)
        {
            return Some(path);
        }
    }
    None
}

fn desktop_entry_value(path: &Path, wanted: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut section = false;
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
        if key.trim() == wanted {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn desktop_owner(id: &str) -> Option<DesktopOwner> {
    let wanted = if id.ends_with(".desktop") {
        id.to_string()
    } else {
        format!("{id}.desktop")
    };
    for data_dir in application_data_dirs() {
        let root = data_dir.join("applications");
        if !root.is_dir() {
            continue;
        }
        let Some(path) = find_desktop_file(&root, &root, &wanted, 0) else {
            continue;
        };
        let hidden = desktop_entry_value(&path, "Hidden")
            .map(|value| parse_bool(&value))
            .unwrap_or(false);
        let flatpak_id = desktop_entry_value(&path, "X-Flatpak")
            .filter(|value| !value.is_empty())
            .or_else(|| {
                let exported = data_dir.ends_with("flatpak/exports/share");
                exported.then(|| wanted.trim_end_matches(".desktop").to_string())
            });
        return Some(DesktopOwner { path, flatpak_id, hidden });
    }
    None
}

fn flatpak_owner(id: &str) -> Option<FlatpakOwner> {
    let desktop = desktop_owner(id)?;
    if desktop.hidden {
        return None;
    }
    let flatpak_id = desktop.flatpak_id?;
    if success("flatpak", &["info", "--user", &flatpak_id]) {
        return Some(FlatpakOwner { desktop_path: desktop.path, id: flatpak_id, scope: "user" });
    }
    if success("flatpak", &["info", "--system", &flatpak_id]) {
        return Some(FlatpakOwner { desktop_path: desktop.path, id: flatpak_id, scope: "system" });
    }
    None
}

fn app_status(id: &str) {
    let desktop = desktop_owner(id);
    let flatpak = flatpak_owner(id);
    let unresolved_flatpak = desktop.as_ref().and_then(|owner| owner.flatpak_id.as_ref()).is_some() && flatpak.is_none();
    let sandbox = if flatpak.is_some() {
        "flatpak"
    } else if unresolved_flatpak || desktop.is_none() {
        "unknown"
    } else {
        "native"
    };
    let permissions = flatpak
        .as_ref()
        .and_then(|owner| output("flatpak", &["info", "--show-permissions", &owner.id]).ok())
        .unwrap_or_default();
    let flatpak_id = flatpak.as_ref().map(|owner| owner.id.as_str()).unwrap_or("");
    let owner = flatpak
        .as_ref()
        .map(|value| if value.scope == "user" { "flatpak-user" } else { "flatpak-system" })
        .or_else(|| desktop.as_ref().map(|_| "desktop-entry"))
        .unwrap_or("unresolved");
    let desktop_path = flatpak
        .as_ref()
        .map(|value| value.desktop_path.as_path())
        .or_else(|| desktop.as_ref().map(|value| value.path.as_path()))
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
    println!(
        "{{\"sandbox\":\"{}\",\"owner\":\"{}\",\"desktopPath\":\"{}\",\"flatpakScope\":\"{}\",\"flatpakId\":\"{}\",\"permissions\":\"{}\",\"removable\":{},\"todaySeconds\":{}}}",
        sandbox,
        owner,
        json_escape(&desktop_path),
        flatpak.as_ref().map(|value| value.scope).unwrap_or(""),
        json_escape(flatpak_id),
        json_escape(&permissions),
        if flatpak.as_ref().map(|value| value.scope == "user").unwrap_or(false) { "true" } else { "false" },
        wellbeing_seconds_for(id)
    );
}

fn app_permission(id: &str, permission: &str, enabled: bool) -> Result<(), String> {
    let owner = flatpak_owner(id).ok_or_else(|| "effective desktop entry is not an installed Flatpak owner".to_string())?;
    let flag = match (permission, enabled) {
        ("network", true) => "--share=network",
        ("network", false) => "--unshare=network",
        ("home", true) => "--filesystem=home",
        ("home", false) => "--nofilesystem=home",
        _ => return Err(format!("unsupported permission: {permission}")),
    };
    let result = Command::new("flatpak")
        .args(["override", "--user", flag, &owner.id])
        .output()
        .map_err(|error| format!("failed to run flatpak: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

fn app_reset_permissions(id: &str) -> Result<(), String> {
    let owner = flatpak_owner(id).ok_or_else(|| "effective desktop entry is not an installed Flatpak owner".to_string())?;
    let result = Command::new("flatpak")
        .args(["override", "--user", "--reset", &owner.id])
        .output()
        .map_err(|error| format!("failed to run flatpak: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

fn app_remove(id: &str) -> Result<(), String> {
    let owner = flatpak_owner(id).ok_or_else(|| "effective desktop entry is not an installed Flatpak owner".to_string())?;
    if owner.scope != "user" {
        return Err("removal is only available for user-installed Flatpak apps here".to_string());
    }

    let result = Command::new("flatpak")
        .args(["uninstall", "--user", "-y", &owner.id])
        .output()
        .map_err(|error| format!("failed to run flatpak: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        Err(if message.is_empty() {
            "Flatpak removal failed".to_string()
        } else {
            message
        })
    }
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
           vicinae-status\n\
           vicinae-setting follow-theme|follow-icons|use-glass|close-on-focus-loss|pop-to-root-on-close|layer-shell on|off\n\
           vicinae-sync-theme\n\
           wellbeing-daemon\n\
           wellbeing-summary\n\
           app-status <desktop-id>\n\
           app-permission <desktop-id> network|home on|off\n\
           app-reset-permissions <desktop-id>\n\
           app-remove <desktop-id>"
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
            println!("{}", if proxy_environment_path().is_file() { "configured" } else { "off" });
        }
        [group, action] if group == "proxy" && action == "set" => {
            proxy_set().unwrap_or_else(|error| print_error(&error));
        }
        [group, action] if group == "proxy" && action == "clear" => {
            proxy_clear().unwrap_or_else(|error| print_error(&error));
        }
        [command] if command == "vicinae-status" => vicinae_status(),
        [command, key, value] if command == "vicinae-setting" => {
            vicinae_set(key, value).unwrap_or_else(|error| print_error(&error));
        }
        [command] if command == "vicinae-sync-theme" => {
            vicinae_sync_theme().unwrap_or_else(|error| print_error(&error));
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
        [command, id] if command == "app-remove" => {
            app_remove(id).unwrap_or_else(|error| print_error(&error));
        }
        _ => usage(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        airplane_state_path, read_airplane_state, session_allows_wellbeing,
        desktop_entry_value, desktop_file_id, wellbeing_identity_keys, write_airplane_state,
        RadioState,
    };
    use std::env;
    use std::fs;

    #[test]
    fn wellbeing_requires_an_explicit_unlocked_active_session() {
        assert_eq!(
            session_allows_wellbeing("IdleHint=no\nLockedHint=no\n"),
            Some(true)
        );
        assert_eq!(
            session_allows_wellbeing("IdleHint=yes\nLockedHint=no\n"),
            Some(false)
        );
        assert_eq!(
            session_allows_wellbeing("IdleHint=no\nLockedHint=yes\n"),
            Some(false)
        );
        assert_eq!(session_allows_wellbeing("IdleHint=no\n"), None);
    }

    #[test]
    fn wellbeing_identity_keys_are_exact_and_canonical_first() {
        assert_eq!(
            wellbeing_identity_keys("org.example.desktop", Some("org.example.desktop")),
            vec!["org.example.desktop", "org.example"]
        );
        assert_eq!(
            wellbeing_identity_keys("Firefox", Some("firefox.desktop")),
            vec!["firefox.desktop", "Firefox"]
        );
    }

    #[test]
    fn airplane_state_round_trip_preserves_wwan_and_optional_bluetooth() {
        let runtime = env::temp_dir().join(format!("vesper-control-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&runtime);
        env::set_var("XDG_RUNTIME_DIR", &runtime);
        let state = RadioState { wifi: true, wwan: false, bluetooth: Some(true) };
        write_airplane_state(state).expect("save airplane state");
        assert_eq!(read_airplane_state().expect("load airplane state"), state);
        fs::remove_file(airplane_state_path()).expect("remove airplane state");
        let _ = fs::remove_dir_all(runtime.join("vesper"));
        let _ = fs::remove_dir_all(runtime);
    }

    #[test]
    fn desktop_owner_evidence_stays_with_the_effective_entry() {
        let root = env::temp_dir().join(format!("vesper-owner-test-{}", std::process::id()));
        let path = root.join("applications/org.example.App.desktop");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "[Desktop Entry]\nType=Application\nX-Flatpak=org.example.App\n",
        )
        .unwrap();
        assert_eq!(desktop_file_id(&path, &root.join("applications")).as_deref(), Some("org.example.App.desktop"));
        assert_eq!(desktop_entry_value(&path, "X-Flatpak").as_deref(), Some("org.example.App"));
        assert_eq!(desktop_entry_value(&path, "Missing"), None);
        let _ = fs::remove_dir_all(root);
    }
}
