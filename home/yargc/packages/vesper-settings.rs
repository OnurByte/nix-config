use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

fn capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run {program}: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{program} exited with {}", output.status)
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|err| format!("failed to run {program}: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with {status}"))
    }
}

fn pipe_input(program: &str, args: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to run {program}: {err}"))?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| format!("{program} stdin unavailable"))?
        .write_all(input)
        .map_err(|err| format!("failed to write {program} input: {err}"))?;

    let output = child
        .wait_with_output()
        .map_err(|err| format!("failed to wait for {program}: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{program} exited with {}", output.status)
        } else {
            stderr
        });
    }

    Ok(output.stdout)
}

fn jq(input: &str, filter: &str) -> Result<String, String> {
    let output = pipe_input("jq", &["-r", filter], input.as_bytes())?;
    Ok(String::from_utf8_lossy(&output).trim().to_string())
}

fn focused_monitor_json() -> Result<String, String> {
    capture("hyprctl", &["-j", "monitors"])
}

fn display_info() -> Result<String, String> {
    let monitors = focused_monitor_json()?;
    let row = jq(
        &monitors,
        r#".[] | select(.focused == true) | [.name, (.width|tostring), (.height|tostring), ((.refreshRate|floor)|tostring), (.scale|tostring)] | @tsv"#,
    )?;
    let fields: Vec<&str> = row.split('\t').collect();
    if fields.len() < 5 {
        return Err("no focused monitor reported by Hyprland".to_string());
    }
    Ok(format!(
        "{} • {}x{}@{}Hz • {}x",
        fields[0], fields[1], fields[2], fields[3], fields[4]
    ))
}

fn set_display_scale(value: &str) -> Result<(), String> {
    const ALLOWED: &[&str] = &["0.75", "1", "1.0", "1.25", "1.5", "1.75", "2", "2.0"];
    if !ALLOWED.contains(&value) {
        return Err("unsupported display scale".to_string());
    }

    let monitors = focused_monitor_json()?;
    let name = jq(&monitors, r#".[] | select(.focused == true) | .name"#)?;
    if name.is_empty() {
        return Err("no focused monitor".to_string());
    }

    let rule = format!("{name},preferred,auto,{value}");
    run("hyprctl", &["keyword", "monitor", &rule])
}

fn brightness_get() -> Result<String, String> {
    let info = capture("brightnessctl", &["-m"])?;
    let first = info.lines().next().ok_or_else(|| "no backlight device".to_string())?;
    let percent = first
        .split(',')
        .next_back()
        .ok_or_else(|| "brightness percentage unavailable".to_string())?
        .trim()
        .trim_end_matches('%');
    percent
        .parse::<u8>()
        .map_err(|_| "invalid brightness percentage".to_string())?;
    Ok(percent.to_string())
}

fn brightness_set(value: &str) -> Result<(), String> {
    let value = value
        .parse::<u8>()
        .map_err(|_| "brightness must be 1-100".to_string())?;
    if !(1..=100).contains(&value) {
        return Err("brightness must be 1-100".to_string());
    }
    run("brightnessctl", &["set", &format!("{value}%")])
}

fn clipboard_list() -> Result<String, String> {
    capture("cliphist", &["list"])
}

fn clipboard_count() -> Result<String, String> {
    let history = clipboard_list()?;
    Ok(if history.is_empty() {
        "0".to_string()
    } else {
        history.lines().count().to_string()
    })
}

fn clipboard_entry(id: &str) -> Result<String, String> {
    if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("invalid clipboard id".to_string());
    }

    let prefix = format!("{id}\t");
    clipboard_list()?
        .lines()
        .find(|line| line.starts_with(&prefix))
        .map(str::to_string)
        .ok_or_else(|| "clipboard entry not found".to_string())
}

fn clipboard_copy(id: &str) -> Result<(), String> {
    let row = clipboard_entry(id)?;
    let decoded = pipe_input("cliphist", &["decode"], row.as_bytes())?;
    pipe_input("wl-copy", &[], &decoded)?;
    Ok(())
}

fn clipboard_delete(id: &str) -> Result<(), String> {
    let row = clipboard_entry(id)?;
    pipe_input("cliphist", &["delete"], row.as_bytes())?;
    Ok(())
}

fn default_mimes(group: &str) -> Option<&'static [&'static str]> {
    match group {
        "web" => Some(&[
            "text/html",
            "application/xhtml+xml",
            "x-scheme-handler/http",
            "x-scheme-handler/https",
        ]),
        "file" => Some(&["inode/directory"]),
        "audio" => Some(&["audio/mpeg", "audio/flac", "audio/ogg", "audio/x-wav"]),
        "video" => Some(&["video/mp4", "video/webm", "video/x-matroska"]),
        "image" => Some(&["image/png", "image/jpeg", "image/webp"]),
        "pdf" => Some(&["application/pdf"]),
        "text" => Some(&["text/plain"]),
        _ => None,
    }
}

fn default_get(group: &str) -> Result<String, String> {
    let mime = default_mimes(group)
        .and_then(|mimes| mimes.first().copied())
        .ok_or_else(|| "unknown default-app group".to_string())?;
    capture("xdg-mime", &["query", "default", mime])
}

fn default_set(group: &str, desktop_id: &str) -> Result<(), String> {
    if desktop_id.is_empty() || desktop_id.contains('/') || desktop_id.contains('\n') {
        return Err("invalid desktop id".to_string());
    }
    let mimes = default_mimes(group).ok_or_else(|| "unknown default-app group".to_string())?;
    for mime in mimes {
        run("xdg-mime", &["default", desktop_id, mime])?;
    }
    Ok(())
}

fn battery_info() -> Result<String, String> {
    let devices = capture("upower", &["-e"])?;
    let battery = devices
        .lines()
        .find(|line| line.to_ascii_lowercase().contains("battery"))
        .ok_or_else(|| "No battery".to_string())?;
    let info = capture("upower", &["-i", battery])?;

    let mut state = "unknown";
    let mut percentage = "?";
    let mut capacity = "?";
    let mut time = "";
    for line in info.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("state:") {
            state = value.trim();
        } else if let Some(value) = line.strip_prefix("percentage:") {
            percentage = value.trim();
        } else if let Some(value) = line.strip_prefix("capacity:") {
            capacity = value.trim();
        } else if let Some(value) = line.strip_prefix("time to empty:") {
            time = value.trim();
        } else if let Some(value) = line.strip_prefix("time to full:") {
            time = value.trim();
        }
    }

    let health = if capacity == "?" { String::new() } else { format!(" • health {capacity}") };
    if time.is_empty() {
        Ok(format!("{percentage} • {state}{health}"))
    } else {
        Ok(format!("{percentage} • {state} • {time}{health}"))
    }
}

fn power_profile() -> Result<String, String> {
    capture("powerprofilesctl", &["get"])
}

fn power_set(profile: &str) -> Result<(), String> {
    if !["power-saver", "balanced", "performance"].contains(&profile) {
        return Err("invalid power profile".to_string());
    }
    run("powerprofilesctl", &["set", profile])
}

fn input_sensitivity_get() -> Result<String, String> {
    let option = capture("hyprctl", &["-j", "getoption", "input:sensitivity"])?;
    jq(&option, r#".float // .int // .str // 0"#)
}

fn input_sensitivity_set(value: &str) -> Result<(), String> {
    let value = value
        .parse::<f32>()
        .map_err(|_| "sensitivity must be between -1 and 1".to_string())?;
    if !(-1.0..=1.0).contains(&value) {
        return Err("sensitivity must be between -1 and 1".to_string());
    }
    run("hyprctl", &["keyword", "input:sensitivity", &value.to_string()])
}

fn natural_scroll_get() -> Result<String, String> {
    let option = capture("hyprctl", &["-j", "getoption", "input:touchpad:natural_scroll"])?;
    let value = jq(&option, r#".int // 0"#)?;
    Ok(if value == "1" { "true" } else { "false" }.to_string())
}

fn natural_scroll_set(value: &str) -> Result<(), String> {
    let value = match value {
        "true" | "1" => "true",
        "false" | "0" => "false",
        _ => return Err("natural-scroll expects true or false".to_string()),
    };
    run("hyprctl", &["keyword", "input:touchpad:natural_scroll", value])
}

fn wellbeing_status() -> String {
    match Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "vesper-wellbeing.service"])
        .status()
    {
        Ok(status) if status.success() => "active".to_string(),
        _ => "paused".to_string(),
    }
}

fn wellbeing_toggle(enabled: bool) -> Result<(), String> {
    run(
        "systemctl",
        &[
            "--user",
            if enabled { "enable" } else { "disable" },
            "--now",
            "vesper-wellbeing.service",
        ],
    )
}

fn wellbeing_report() -> Result<String, String> {
    capture("niri-screen-time", &["-json"])
}

fn usage() -> &'static str {
    "vesper-settings display info|scale <value>\n\
vesper-settings brightness get|set <1-100>\n\
vesper-settings clipboard list|count|copy <id>|delete <id>|compact|wipe\n\
vesper-settings defaults get <group>|set <group> <desktop-id>\n\
vesper-settings battery info\n\
vesper-settings power get|set <power-saver|balanced|performance>\n\
vesper-settings input sensitivity|get-natural-scroll|set-sensitivity <n>|set-natural-scroll <bool>\n\
vesper-settings wellbeing status|enable|disable|report"
}

fn dispatch(args: &[String]) -> Result<Option<String>, String> {
    let command = args.get(1).map(String::as_str).unwrap_or("");
    let action = args.get(2).map(String::as_str).unwrap_or("");

    match (command, action) {
        ("display", "info") => display_info().map(Some),
        ("display", "scale") => {
            set_display_scale(args.get(3).ok_or("missing scale")?)?;
            Ok(None)
        }
        ("brightness", "get") => brightness_get().map(Some),
        ("brightness", "set") => {
            brightness_set(args.get(3).ok_or("missing brightness")?)?;
            Ok(None)
        }
        ("clipboard", "list") => clipboard_list().map(Some),
        ("clipboard", "count") => clipboard_count().map(Some),
        ("clipboard", "copy") => {
            clipboard_copy(args.get(3).ok_or("missing clipboard id")?)?;
            Ok(None)
        }
        ("clipboard", "delete") => {
            clipboard_delete(args.get(3).ok_or("missing clipboard id")?)?;
            Ok(None)
        }
        ("clipboard", "compact") => {
            run("cliphist", &["compact"])?;
            Ok(None)
        }
        ("clipboard", "wipe") => {
            run("cliphist", &["wipe"])?;
            Ok(None)
        }
        ("defaults", "get") => default_get(args.get(3).ok_or("missing group")?).map(Some),
        ("defaults", "set") => {
            default_set(
                args.get(3).ok_or("missing group")?,
                args.get(4).ok_or("missing desktop id")?,
            )?;
            Ok(None)
        }
        ("battery", "info") => battery_info().map(Some),
        ("power", "get") => power_profile().map(Some),
        ("power", "set") => {
            power_set(args.get(3).ok_or("missing power profile")?)?;
            Ok(None)
        }
        ("input", "sensitivity") => input_sensitivity_get().map(Some),
        ("input", "get-natural-scroll") => natural_scroll_get().map(Some),
        ("input", "set-sensitivity") => {
            input_sensitivity_set(args.get(3).ok_or("missing sensitivity")?)?;
            Ok(None)
        }
        ("input", "set-natural-scroll") => {
            natural_scroll_set(args.get(3).ok_or("missing natural-scroll value")?)?;
            Ok(None)
        }
        ("wellbeing", "status") => Ok(Some(wellbeing_status())),
        ("wellbeing", "enable") => {
            wellbeing_toggle(true)?;
            Ok(None)
        }
        ("wellbeing", "disable") => {
            wellbeing_toggle(false)?;
            Ok(None)
        }
        ("wellbeing", "report") => wellbeing_report().map(Some),
        _ => Err(usage().to_string()),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match dispatch(&args) {
        Ok(Some(value)) => println!("{value}"),
        Ok(None) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
