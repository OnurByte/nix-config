use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/nonexistent"))
}

pub fn config_root() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("vesper")
}

pub fn state_root() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("vesper")
}

pub fn ensure_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

pub fn atomic_write_private(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid state path: {}", path.display()))?;
    ensure_private_dir(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(".{}.{}.tmp", path.file_name().and_then(|name| name.to_str()).unwrap_or("state"), std::process::id()));
    fs::write(&tmp, data).map_err(|error| error.to_string())?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600)).map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())
}
