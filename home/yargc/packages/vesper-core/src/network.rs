use std::fs;

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, config_root};
use crate::process::{output, success};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RadioState {
    wifi: bool,
    bluetooth: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AirplaneRecord {
    wifi: bool,
    bluetooth: bool,
}

fn airplane_state_path() -> std::path::PathBuf {
    config_root().join("network/airplane.state")
}

fn parse_record(value: &str) -> Option<AirplaneRecord> {
    let mut wifi = None;
    let mut bluetooth = None;
    for line in value.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let parsed = match value.trim() {
            "1" | "true" | "on" => Some(true),
            "0" | "false" | "off" => Some(false),
            _ => None,
        };
        match key.trim() {
            "wifi" => wifi = parsed,
            "bluetooth" => bluetooth = parsed,
            _ => {}
        }
    }
    Some(AirplaneRecord {
        wifi: wifi?,
        bluetooth: bluetooth?,
    })
}

fn load_airplane_record() -> Option<AirplaneRecord> {
    fs::read_to_string(airplane_state_path())
        .ok()
        .and_then(|value| parse_record(&value))
}

fn save_airplane_record(record: AirplaneRecord) -> Result<(), String> {
    atomic_write_private(
        &airplane_state_path(),
        format!(
            "wifi={}\nbluetooth={}\n",
            if record.wifi { 1 } else { 0 },
            if record.bluetooth { 1 } else { 0 }
        )
        .as_bytes(),
    )
}

fn clear_airplane_record() -> Result<(), String> {
    match fs::remove_file(airplane_state_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn radio_status() -> RadioState {
    let wifi = output("nmcli", &["radio", "wifi"])
        .map(|value| value == "enabled")
        .unwrap_or(false);
    let bluetooth = output("bluetoothctl", &["show"])
        .map(|value| value.lines().any(|line| line.trim() == "Powered: yes"))
        .unwrap_or(false);
    RadioState { wifi, bluetooth }
}

fn set_wifi(enabled: bool) -> bool {
    success("nmcli", &["radio", "wifi", if enabled { "on" } else { "off" }])
}

fn set_bluetooth(enabled: bool) -> bool {
    success("bluetoothctl", &["power", if enabled { "on" } else { "off" }])
}

fn active_connection() -> Option<String> {
    let text = output(
        "nmcli",
        &["-t", "-f", "NAME,TYPE", "connection", "show", "--active"],
    )
    .ok()?;
    for line in text.lines() {
        if let Some((name, kind)) = line.rsplit_once(':') {
            if kind == "802-11-wireless" || kind == "wifi" {
                return Some(name.replace("\\:", ":"));
            }
        }
    }
    None
}

pub fn status_json() -> String {
    let radios = radio_status();
    let connection = active_connection().unwrap_or_default();
    let airplane = load_airplane_record().is_some();
    let zapret = success("systemctl", &["is-active", "--quiet", "nfqws2@default.service"]);
    let proxy = config_root().join("proxy.tsv").exists();
    format!(
        "{{\"airplane\":{},\"wifi\":{},\"bluetooth\":{},\"connection\":\"{}\",\"zapret\":{},\"proxy\":{},\"airplaneState\":\"{}\"}}",
        bool_lit(airplane),
        bool_lit(radios.wifi),
        bool_lit(radios.bluetooth),
        escape(&connection),
        bool_lit(zapret),
        bool_lit(proxy),
        if airplane { "explicit" } else { "off" }
    )
}

pub fn set_airplane(enabled: bool) -> Result<(), String> {
    if enabled {
        if load_airplane_record().is_some() {
            let wifi_ok = set_wifi(false);
            let bluetooth_ok = set_bluetooth(false);
            return if wifi_ok && bluetooth_ok {
                Ok(())
            } else {
                Err("airplane mode is active but one or more radios could not be disabled".to_string())
            };
        }

        let previous = radio_status();
        save_airplane_record(AirplaneRecord {
            wifi: previous.wifi,
            bluetooth: previous.bluetooth,
        })?;

        if !set_wifi(false) {
            let _ = clear_airplane_record();
            return Err("NetworkManager rejected Wi-Fi radio change".to_string());
        }
        if !set_bluetooth(false) {
            let _ = set_wifi(previous.wifi);
            let _ = clear_airplane_record();
            return Err("BlueZ rejected Bluetooth radio change".to_string());
        }
        Ok(())
    } else {
        let Some(previous) = load_airplane_record() else {
            return Ok(());
        };

        let wifi_ok = set_wifi(previous.wifi);
        let bluetooth_ok = set_bluetooth(previous.bluetooth);
        if wifi_ok && bluetooth_ok {
            clear_airplane_record()
        } else {
            Err("could not fully restore the radio state saved before airplane mode".to_string())
        }
    }
}

fn zapret_profile_raw() -> String {
    let value = fs::read_to_string("/etc/vesper/zapret-profile.json").unwrap_or_default();
    let trimmed = value.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed.to_string()
    } else {
        "{}".to_string()
    }
}

pub fn dpi_status_json() -> String {
    let active = success("systemctl", &["is-active", "--quiet", "nfqws2@default.service"]);
    format!(
        "{{\"active\":{},\"service\":\"nfqws2@default.service\",\"managedBy\":\"nix\",\"mutableProfile\":false,\"profile\":{}}}",
        bool_lit(active),
        zapret_profile_raw()
    )
}

fn valid_test_domain(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && !value.starts_with('.')
        && !value.ends_with('.')
        && value.contains('.')
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'))
}

pub fn dpi_test_json(domain: &str) -> String {
    if !valid_test_domain(domain) {
        return format!(
            "{{\"domain\":\"{}\",\"reachable\":false,\"httpCode\":\"\",\"error\":\"invalid test domain\",\"claim\":\"reachability-only\"}}",
            escape(domain)
        );
    }
    let url = format!("https://{domain}/");
    match output(
        "curl",
        &[
            "--silent",
            "--show-error",
            "--output",
            "/dev/null",
            "--connect-timeout",
            "5",
            "--max-time",
            "10",
            "--write-out",
            "%{http_code}",
            &url,
        ],
    ) {
        Ok(code) => format!(
            "{{\"domain\":\"{}\",\"reachable\":true,\"httpCode\":\"{}\",\"error\":\"\",\"claim\":\"reachability-only\"}}",
            escape(domain),
            escape(&code)
        ),
        Err(error) => format!(
            "{{\"domain\":\"{}\",\"reachable\":false,\"httpCode\":\"\",\"error\":\"{}\",\"claim\":\"reachability-only\"}}",
            escape(domain),
            escape(&error)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_saved_radio_state() {
        assert_eq!(
            parse_record("wifi=1\nbluetooth=0\n"),
            Some(AirplaneRecord {
                wifi: true,
                bluetooth: false
            })
        );
    }

    #[test]
    fn rejects_incomplete_saved_state() {
        assert_eq!(parse_record("wifi=1\n"), None);
        assert_eq!(parse_record("bluetooth=0\n"), None);
    }

    #[test]
    fn rejects_invalid_boolean_values() {
        assert_eq!(parse_record("wifi=maybe\nbluetooth=0\n"), None);
    }

    #[test]
    fn dpi_test_domains_are_strict() {
        assert!(valid_test_domain("example.com"));
        assert!(valid_test_domain("sub.example.org"));
        assert!(!valid_test_domain("localhost"));
        assert!(!valid_test_domain("example.com/path"));
        assert!(!valid_test_domain("example.com;id"));
    }
}
