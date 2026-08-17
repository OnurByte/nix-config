use std::collections::BTreeMap;
use std::fs;

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, config_root, home};
use crate::process::success;

fn config_path() -> std::path::PathBuf {
    config_root().join("proxy.tsv")
}

fn environment_path() -> std::path::PathBuf {
    home().join(".config/environment.d/90-vesper-proxy.conf")
}

fn load() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(config_path()).unwrap_or_default().lines() {
        let Some((key, value)) = line.split_once('\t') else { continue; };
        if matches!(key, "http" | "https" | "socks" | "noProxy") && !value.is_empty() {
            values.insert(key.to_string(), value.to_string());
        }
    }
    values
}

fn validate_port(port: Option<&str>) -> Result<(), String> {
    let Some(port) = port else { return Ok(()); };
    let port = port
        .parse::<u16>()
        .map_err(|_| "proxy port must be between 1 and 65535".to_string())?;
    if port == 0 {
        return Err("proxy port must be between 1 and 65535".to_string());
    }
    Ok(())
}

fn proxy_scheme(value: &str) -> Result<&'static str, String> {
    if value.starts_with("http://") {
        Ok("http")
    } else if value.starts_with("https://") {
        Ok("https")
    } else if value.starts_with("socks5://") {
        Ok("socks5")
    } else if value.starts_with("socks5h://") {
        Ok("socks5h")
    } else {
        Err("proxy must start with http://, https://, socks5:// or socks5h://".to_string())
    }
}

fn validate_proxy_url(value: &str) -> Result<&'static str, String> {
    if value.len() > 2048 {
        return Err("proxy URL is too long".to_string());
    }
    let scheme = proxy_scheme(value)?;
    let authority = value
        .split_once("//")
        .map(|(_, authority)| authority)
        .unwrap_or("");
    if authority.is_empty()
        || authority.chars().any(|ch| ch.is_whitespace() || ch.is_control())
        || authority.contains(['/', '?', '#', '@', '"', '\\', '$', '`'])
    {
        return Err("proxy must be a credential-free host[:port] URL".to_string());
    }

    if let Some(rest) = authority.strip_prefix('[') {
        let close = rest.find(']').ok_or_else(|| "invalid bracketed proxy host".to_string())?;
        let host = &rest[..close];
        let suffix = &rest[close + 1..];
        if host.is_empty() || !host.chars().all(|ch| ch.is_ascii_hexdigit() || matches!(ch, ':' | '.')) {
            return Err("invalid bracketed proxy host".to_string());
        }
        if suffix.is_empty() {
            validate_port(None)?;
        } else {
            validate_port(Some(
                suffix
                    .strip_prefix(':')
                    .ok_or_else(|| "invalid proxy port".to_string())?,
            ))?;
        }
    } else {
        if authority.matches(':').count() > 1 {
            return Err("IPv6 proxy hosts must use brackets".to_string());
        }
        let (host, port) = authority
            .rsplit_once(':')
            .map(|(host, port)| (host, Some(port)))
            .unwrap_or((authority, None));
        if host.is_empty()
            || !host.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'))
        {
            return Err("invalid proxy host".to_string());
        }
        validate_port(port)?;
    }
    Ok(scheme)
}

fn normalize_no_proxy(value: &str) -> Result<String, String> {
    if value.len() > 4096 || value.chars().any(|ch| ch.is_control() || matches!(ch, '"' | '\\' | '$' | '`' | ';')) {
        return Err("invalid NO_PROXY value".to_string());
    }
    let mut parts = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        if !part.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ':' | '[' | ']' | '/' | '*')) {
            return Err(format!("invalid NO_PROXY entry: {part}"));
        }
        parts.push(part);
    }
    Ok(parts.join(","))
}

fn save(values: &BTreeMap<String, String>) -> Result<(), String> {
    let mut state = String::new();
    for key in ["http", "https", "socks", "noProxy"] {
        if let Some(value) = values.get(key) {
            state.push_str(key);
            state.push('\t');
            state.push_str(value);
            state.push('\n');
        }
    }
    if state.is_empty() {
        match fs::remove_file(config_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    } else {
        atomic_write_private(&config_path(), state.as_bytes())?;
    }

    let mut env = String::new();
    if let Some(value) = values.get("http") {
        env.push_str(&format!("HTTP_PROXY=\"{value}\"\nhttp_proxy=\"{value}\"\n"));
    }
    if let Some(value) = values.get("https") {
        env.push_str(&format!("HTTPS_PROXY=\"{value}\"\nhttps_proxy=\"{value}\"\n"));
    }
    if let Some(value) = values.get("socks") {
        env.push_str(&format!("ALL_PROXY=\"{value}\"\nall_proxy=\"{value}\"\n"));
    }
    if let Some(value) = values.get("noProxy") {
        env.push_str(&format!("NO_PROXY=\"{value}\"\nno_proxy=\"{value}\"\n"));
    }

    if env.is_empty() {
        match fs::remove_file(environment_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    } else {
        atomic_write_private(&environment_path(), env.as_bytes())?;
    }
    let _ = success("systemctl", &["--user", "daemon-reload"]);
    Ok(())
}

pub fn set(kind: &str, value: &str) -> Result<(), String> {
    let mut values = load();
    match kind {
        "http" | "https" => {
            let scheme = validate_proxy_url(value)?;
            if !matches!(scheme, "http" | "https") {
                return Err(format!("{kind} proxy expects an http:// or https:// URL"));
            }
            values.insert(kind.to_string(), value.to_string());
        }
        "socks" => {
            let scheme = validate_proxy_url(value)?;
            if !matches!(scheme, "socks5" | "socks5h") {
                return Err("SOCKS proxy expects socks5:// or socks5h://".to_string());
            }
            values.insert(kind.to_string(), value.to_string());
        }
        "no-proxy" => {
            let value = normalize_no_proxy(value)?;
            if value.is_empty() {
                values.remove("noProxy");
            } else {
                values.insert("noProxy".to_string(), value);
            }
        }
        _ => return Err("proxy field expects http, https, socks or no-proxy".to_string()),
    }
    save(&values)
}

pub fn set_legacy(value: &str) -> Result<(), String> {
    let scheme = validate_proxy_url(value)?;
    let mut values = load();
    if matches!(scheme, "socks5" | "socks5h") {
        values.insert("socks".to_string(), value.to_string());
    } else {
        values.insert("http".to_string(), value.to_string());
        values.insert("https".to_string(), value.to_string());
    }
    save(&values)
}

pub fn clear(kind: Option<&str>) -> Result<(), String> {
    let mut values = load();
    match kind {
        None | Some("all") => values.clear(),
        Some("http") | Some("https") | Some("socks") => {
            values.remove(kind.unwrap());
        }
        Some("no-proxy") => {
            values.remove("noProxy");
        }
        Some(_) => return Err("proxy clear expects http, https, socks, no-proxy or all".to_string()),
    }
    save(&values)
}

pub fn status_json() -> String {
    let values = load();
    format!(
        "{{\"configured\":{},\"http\":\"{}\",\"https\":\"{}\",\"socks\":\"{}\",\"noProxy\":\"{}\",\"authSupported\":false,\"pacSupported\":false,\"appliesTo\":\"new-processes\"}}",
        bool_lit(!values.is_empty()),
        escape(values.get("http").map(String::as_str).unwrap_or("")),
        escape(values.get("https").map(String::as_str).unwrap_or("")),
        escape(values.get("socks").map(String::as_str).unwrap_or("")),
        escape(values.get("noProxy").map(String::as_str).unwrap_or("")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_credentials_paths_and_bad_ports() {
        assert!(validate_proxy_url("http://localhost:8080").is_ok());
        assert!(validate_proxy_url("socks5h://127.0.0.1:9050").is_ok());
        assert!(validate_proxy_url("http://user:pass@localhost:8080").is_err());
        assert!(validate_proxy_url("http://localhost/path").is_err());
        assert!(validate_proxy_url("http://localhost:0").is_err());
    }

    #[test]
    fn normalizes_no_proxy_without_shell_syntax() {
        assert_eq!(normalize_no_proxy(" localhost, .example.com,10.0.0.0/8 ").unwrap(), "localhost,.example.com,10.0.0.0/8");
        assert!(normalize_no_proxy("localhost;touch /tmp/pwn").is_err());
    }
}
