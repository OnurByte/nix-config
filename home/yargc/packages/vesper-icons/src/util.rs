use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/nonexistent"))
}

pub fn xdg_data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".local/share"))
}

pub fn xdg_state_home() -> PathBuf {
    env::var_os("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".local/state"))
}

pub fn xdg_cache_home() -> PathBuf {
    env::var_os("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".cache"))
}

pub fn xdg_config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config"))
}

pub fn state_root() -> PathBuf { xdg_state_home().join("vesper/adaptive-icons") }
pub fn data_root() -> PathBuf { xdg_data_home().join("vesper/adaptive-icons") }
pub fn cache_root() -> PathBuf { xdg_cache_home().join("vesper/adaptive-icons") }
pub fn config_root() -> PathBuf { xdg_config_home().join("vesper") }
pub fn canonical_root() -> PathBuf { data_root().join("canonical") }
pub fn fallback_root() -> PathBuf { cache_root().join("fallback") }
pub fn generation_root() -> PathBuf { data_root().join("themes") }
pub fn export_root() -> PathBuf { data_root().join("exports") }
pub fn shadow_root() -> PathBuf { data_root().join("desktop-shadows") }
pub fn state_db() -> PathBuf { state_root().join("state.sqlite3") }

pub fn now_epoch() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

pub fn json_escape(value: &str) -> String {
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

pub fn sql_escape(value: &str) -> String { value.replace('\'', "''") }

pub fn safe_name(value: &str) -> String {
    let mut out: String = value.chars().map(|ch| if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | '+') { ch } else { '_' }).collect();
    if out.is_empty() { out.push_str("unknown"); }
    out
}

pub fn write_atomic(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("invalid path {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = parent.join(format!(".{}.{}.tmp", path.file_name().and_then(|v| v.to_str()).unwrap_or("vesper"), std::process::id()));
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn command_output(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command).args(args).output().map_err(|e| format!("failed to run {command}: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() { format!("{command} exited with {}", output.status.code().unwrap_or(-1)) } else { stderr });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn command_stdin(command: &str, args: &[&str], input: &str) -> Result<String, String> {
    let mut child = Command::new(command).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("failed to run {command}: {e}"))?;
    child.stdin.as_mut().ok_or_else(|| format!("failed to open {command} stdin"))?.write_all(input.as_bytes()).map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() { format!("{command} exited with {}", output.status.code().unwrap_or(-1)) } else { stderr });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn sha256(path: &Path) -> Result<String, String> {
    let text = command_output("sha256sum", &[&path.to_string_lossy()])?;
    text.split_whitespace().next().filter(|v| v.len() == 64).map(str::to_owned).ok_or_else(|| "invalid sha256sum output".to_string())
}

pub fn hash_text(value: &str) -> Result<String, String> {
    let mut child = Command::new("sha256sum").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    child.stdin.as_mut().ok_or_else(|| "sha256sum stdin unavailable".to_string())?.write_all(value.as_bytes()).map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() { return Err("sha256sum failed".into()); }
    String::from_utf8_lossy(&output.stdout).split_whitespace().next().map(str::to_owned).ok_or_else(|| "sha256sum returned no digest".into())
}

pub fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i < bytes.len() {
        let a = bytes[i];
        let b = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        let c = if i + 2 < bytes.len() { bytes[i + 2] } else { 0 };
        out.push(TABLE[(a >> 2) as usize] as char);
        out.push(TABLE[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if i + 1 < bytes.len() { out.push(TABLE[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char); } else { out.push('='); }
        if i + 2 < bytes.len() { out.push(TABLE[(c & 0x3f) as usize] as char); } else { out.push('='); }
        i += 3;
    }
    out
}

pub fn is_under(path: &Path, root: &Path) -> bool {
    let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let r = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    p.starts_with(r)
}

pub fn vesper_owned(path: &Path) -> bool {
    [data_root(), cache_root(), export_root(), shadow_root(), xdg_data_home().join("icons/Vesper-Adaptive")]
        .iter().any(|root| is_under(path, root))
}
