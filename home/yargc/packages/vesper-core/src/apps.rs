use std::env;
use std::fs;
use std::path::PathBuf;

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, home};
use crate::process::{output, success};
use crate::wellbeing;

#[derive(Clone, Copy)]
enum PermissionKind {
    Share,
    Socket,
    Device,
    Feature,
    Filesystem,
}

#[derive(Clone, Copy)]
struct PermissionDef {
    id: &'static str,
    label: &'static str,
    key: &'static str,
    item: &'static str,
    kind: PermissionKind,
}

const PERMISSIONS: &[PermissionDef] = &[
    PermissionDef { id: "network", label: "Network", key: "shared", item: "network", kind: PermissionKind::Share },
    PermissionDef { id: "ipc", label: "IPC", key: "shared", item: "ipc", kind: PermissionKind::Share },
    PermissionDef { id: "wayland", label: "Wayland", key: "sockets", item: "wayland", kind: PermissionKind::Socket },
    PermissionDef { id: "x11", label: "X11", key: "sockets", item: "x11", kind: PermissionKind::Socket },
    PermissionDef { id: "fallback-x11", label: "Fallback X11", key: "sockets", item: "fallback-x11", kind: PermissionKind::Socket },
    PermissionDef { id: "audio", label: "PulseAudio", key: "sockets", item: "pulseaudio", kind: PermissionKind::Socket },
    PermissionDef { id: "printing", label: "CUPS printing", key: "sockets", item: "cups", kind: PermissionKind::Socket },
    PermissionDef { id: "ssh-auth", label: "SSH agent", key: "sockets", item: "ssh-auth", kind: PermissionKind::Socket },
    PermissionDef { id: "gpg-agent", label: "GPG agent", key: "sockets", item: "gpg-agent", kind: PermissionKind::Socket },
    PermissionDef { id: "pcsc", label: "Smart cards", key: "sockets", item: "pcsc", kind: PermissionKind::Socket },
    PermissionDef { id: "session-bus", label: "Full session bus", key: "sockets", item: "session-bus", kind: PermissionKind::Socket },
    PermissionDef { id: "system-bus", label: "Full system bus", key: "sockets", item: "system-bus", kind: PermissionKind::Socket },
    PermissionDef { id: "bluetooth", label: "Bluetooth", key: "features", item: "bluetooth", kind: PermissionKind::Feature },
    PermissionDef { id: "devel", label: "Development syscalls", key: "features", item: "devel", kind: PermissionKind::Feature },
    PermissionDef { id: "multiarch", label: "Multiarch", key: "features", item: "multiarch", kind: PermissionKind::Feature },
    PermissionDef { id: "devices-all", label: "All devices", key: "devices", item: "all", kind: PermissionKind::Device },
    PermissionDef { id: "dri", label: "GPU / DRI", key: "devices", item: "dri", kind: PermissionKind::Device },
    PermissionDef { id: "kvm", label: "KVM", key: "devices", item: "kvm", kind: PermissionKind::Device },
    PermissionDef { id: "shm", label: "Shared memory device", key: "devices", item: "shm", kind: PermissionKind::Device },
    PermissionDef { id: "input", label: "Input devices", key: "devices", item: "input", kind: PermissionKind::Device },
    PermissionDef { id: "usb", label: "USB devices", key: "devices", item: "usb", kind: PermissionKind::Device },
    PermissionDef { id: "home", label: "Home folder", key: "filesystems", item: "home", kind: PermissionKind::Filesystem },
    PermissionDef { id: "host", label: "Host filesystem", key: "filesystems", item: "host", kind: PermissionKind::Filesystem },
    PermissionDef { id: "host-os", label: "Host OS files", key: "filesystems", item: "host-os", kind: PermissionKind::Filesystem },
    PermissionDef { id: "host-etc", label: "Host /etc", key: "filesystems", item: "host-etc", kind: PermissionKind::Filesystem },
    PermissionDef { id: "desktop", label: "Desktop folder", key: "filesystems", item: "xdg-desktop", kind: PermissionKind::Filesystem },
    PermissionDef { id: "documents", label: "Documents folder", key: "filesystems", item: "xdg-documents", kind: PermissionKind::Filesystem },
    PermissionDef { id: "downloads", label: "Downloads folder", key: "filesystems", item: "xdg-download", kind: PermissionKind::Filesystem },
    PermissionDef { id: "music", label: "Music folder", key: "filesystems", item: "xdg-music", kind: PermissionKind::Filesystem },
    PermissionDef { id: "pictures", label: "Pictures folder", key: "filesystems", item: "xdg-pictures", kind: PermissionKind::Filesystem },
    PermissionDef { id: "videos", label: "Videos folder", key: "filesystems", item: "xdg-videos", kind: PermissionKind::Filesystem },
    PermissionDef { id: "public-share", label: "Public share folder", key: "filesystems", item: "xdg-public-share", kind: PermissionKind::Filesystem },
    PermissionDef { id: "templates", label: "Templates folder", key: "filesystems", item: "xdg-templates", kind: PermissionKind::Filesystem },
    PermissionDef { id: "config", label: "User config", key: "filesystems", item: "xdg-config", kind: PermissionKind::Filesystem },
    PermissionDef { id: "cache", label: "User cache", key: "filesystems", item: "xdg-cache", kind: PermissionKind::Filesystem },
    PermissionDef { id: "data", label: "User data", key: "filesystems", item: "xdg-data", kind: PermissionKind::Filesystem },
];

fn flatpak_id(id: &str) -> &str {
    id.strip_suffix(".desktop").unwrap_or(id)
}

fn valid_flatpak_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 255
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn value_item_matches(value: &str, item: &str) -> Option<bool> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let (allowed, value) = if let Some(value) = value.strip_prefix('!') {
        (false, value)
    } else {
        (true, value)
    };
    let base = value.split(':').next().unwrap_or(value);
    if base == item { Some(allowed) } else { None }
}

fn item_state(text: &str, key: &str, item: &str) -> Option<bool> {
    for line in text.lines() {
        let Some(values) = line.trim().strip_prefix(&format!("{key}=")) else {
            continue;
        };
        for value in values.split(';') {
            if let Some(state) = value_item_matches(value, item) {
                return Some(state);
            }
        }
    }
    None
}

fn json_nullable_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    }
}

fn permission_override_name(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "allow",
        Some(false) => "deny",
        None => "inherit",
    }
}

fn permission_json(def: PermissionDef, packaged: &str, overrides: &str, effective: &str) -> String {
    let packaged_state = item_state(packaged, def.key, def.item);
    let override_state = item_state(overrides, def.key, def.item);
    let effective_state = item_state(effective, def.key, def.item).unwrap_or(false);
    format!(
        "{{\"id\":\"{}\",\"label\":\"{}\",\"category\":\"{}\",\"packaged\":{},\"userOverride\":\"{}\",\"effective\":{},\"backend\":\"Flatpak-enforced\"}}",
        escape(def.id),
        escape(def.label),
        escape(def.key),
        json_nullable_bool(packaged_state),
        permission_override_name(override_state),
        bool_lit(effective_state)
    )
}

pub fn status_json(id: &str) -> String {
    let id = flatpak_id(id);
    let is_flatpak = success("flatpak", &["info", id]);
    if !is_flatpak {
        return format!(
            "{{\"sandbox\":\"native\",\"flatpakId\":\"{}\",\"permissionsManageable\":false,\"enforcementBackend\":\"native/unrestricted\",\"networkAllowed\":false,\"homeAllowed\":false,\"permissionItems\":[],\"portalPermissions\":\"\",\"todaySeconds\":{}}}",
            escape(id),
            wellbeing::seconds_for(id)
        );
    }

    let packaged = output("flatpak", &["info", "--show-metadata", id]).unwrap_or_default();
    let effective = output("flatpak", &["info", "--show-permissions", id]).unwrap_or_default();
    let overrides = output("flatpak", &["override", "--user", "--show", id]).unwrap_or_default();
    let portal_permissions = output("flatpak", &["permission-show", id]).unwrap_or_default();
    let permissions = PERMISSIONS
        .iter()
        .copied()
        .map(|def| permission_json(def, &packaged, &overrides, &effective))
        .collect::<Vec<_>>();
    let network_allowed = item_state(&effective, "shared", "network").unwrap_or(false);
    let home_allowed = item_state(&effective, "filesystems", "home").unwrap_or(false);

    format!(
        "{{\"sandbox\":\"flatpak\",\"flatpakId\":\"{}\",\"permissionsManageable\":true,\"enforcementBackend\":\"Flatpak-enforced\",\"networkAllowed\":{},\"homeAllowed\":{},\"permissionItems\":[{}],\"portalPermissions\":\"{}\",\"packagedPermissions\":\"{}\",\"userOverrides\":\"{}\",\"effectivePermissions\":\"{}\",\"todaySeconds\":{}}}",
        escape(id),
        bool_lit(network_allowed),
        bool_lit(home_allowed),
        permissions.join(","),
        escape(&portal_permissions),
        escape(&packaged),
        escape(&overrides),
        escape(&effective),
        wellbeing::seconds_for(id)
    )
}

fn permission_def(id: &str) -> Option<PermissionDef> {
    PERMISSIONS.iter().copied().find(|def| def.id == id)
}

fn flag_for(def: PermissionDef, enabled: bool) -> String {
    match def.kind {
        PermissionKind::Share => format!("--{}share={}", if enabled { "" } else { "un" }, def.item),
        PermissionKind::Socket => format!("--{}socket={}", if enabled { "" } else { "no" }, def.item),
        PermissionKind::Device => format!("--{}device={}", if enabled { "" } else { "no" }, def.item),
        PermissionKind::Feature => format!("--{}allow={}", if enabled { "" } else { "dis" }, def.item),
        PermissionKind::Filesystem => format!("--{}filesystem={}", if enabled { "" } else { "no" }, def.item),
    }
}

fn ensure_flatpak(id: &str) -> Result<&str, String> {
    let id = flatpak_id(id);
    if !valid_flatpak_id(id) {
        return Err("invalid Flatpak application id".to_string());
    }
    if success("flatpak", &["info", id]) {
        Ok(id)
    } else {
        Err("app is not installed as Flatpak".to_string())
    }
}

pub fn set_permission(id: &str, permission: &str, enabled: bool) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    let def = permission_def(permission).ok_or_else(|| format!("unsupported Flatpak permission: {permission}"))?;
    let flag = flag_for(def, enabled);
    output("flatpak", &["override", "--user", &flag, id]).map(|_| ())
}

fn user_data_root() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

fn app_override_path(id: &str) -> PathBuf {
    user_data_root().join("flatpak/overrides").join(id)
}

fn remove_context_item(text: &str, key: &str, item: &str) -> String {
    let mut section = String::new();
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].to_string();
            out.push(line.to_string());
            continue;
        }
        if section == "Context" {
            if let Some((line_key, values)) = line.split_once('=') {
                if line_key.trim() == key {
                    let kept = values
                        .split(';')
                        .filter(|value| {
                            let value = value.trim();
                            if value.is_empty() {
                                return false;
                            }
                            let value = value.strip_prefix('!').unwrap_or(value);
                            value.split(':').next().unwrap_or(value) != item
                        })
                        .collect::<Vec<_>>();
                    if !kept.is_empty() {
                        out.push(format!("{}={};", line_key.trim(), kept.join(";")));
                    }
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    if out.is_empty() { String::new() } else { format!("{}\n", out.join("\n")) }
}

fn remove_key_from_section(text: &str, section_name: &str, key: Option<&str>) -> String {
    let mut section = String::new();
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].to_string();
            out.push(line.to_string());
            continue;
        }
        if section == section_name {
            match key {
                None => continue,
                Some(expected) => {
                    if line
                        .split_once('=')
                        .map(|(line_key, _)| line_key.trim() == expected)
                        .unwrap_or(false)
                    {
                        continue;
                    }
                }
            }
        }
        out.push(line.to_string());
    }
    if out.is_empty() { String::new() } else { format!("{}\n", out.join("\n")) }
}

fn has_override_assignments(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        !line.is_empty() && !line.starts_with('#') && !line.starts_with(';') && !line.starts_with('[') && line.contains('=')
    })
}

fn write_app_override(id: &str, text: &str) -> Result<(), String> {
    let path = app_override_path(id);
    if !has_override_assignments(text) {
        return match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
    atomic_write_private(&path, text.as_bytes())
}

pub fn reset_permission(id: &str, permission: &str) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    let def = permission_def(permission).ok_or_else(|| format!("unsupported Flatpak permission: {permission}"))?;
    let path = app_override_path(id);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    write_app_override(id, &remove_context_item(&text, def.key, def.item))
}

pub fn reset_category(id: &str, category: &str) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    let path = app_override_path(id);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    let updated = match category {
        "shared" | "sockets" | "devices" | "features" | "filesystems" => {
            remove_key_from_section(&text, "Context", Some(category))
        }
        "environment" => remove_key_from_section(&text, "Environment", None),
        "session-bus" => remove_key_from_section(&text, "Session Bus Policy", None),
        "system-bus" => remove_key_from_section(&text, "System Bus Policy", None),
        _ => return Err("unknown Flatpak override category".to_string()),
    };
    write_app_override(id, &updated)
}

fn valid_filesystem(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.chars().any(|ch| ch.is_control())
        && (value.starts_with('/') || value.starts_with("~/") || value.starts_with("xdg-"))
}

pub fn set_filesystem(id: &str, filesystem: &str, enabled: bool) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    if !valid_filesystem(filesystem) {
        return Err("filesystem must be an absolute, ~/ relative, or xdg-* Flatpak path".to_string());
    }
    let flag = format!("--{}filesystem={filesystem}", if enabled { "" } else { "no" });
    output("flatpak", &["override", "--user", &flag, id]).map(|_| ())
}

fn valid_bus_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && !value.chars().any(|ch| ch.is_whitespace() || ch.is_control())
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '*'))
}

pub fn set_dbus(id: &str, bus: &str, name: &str, access: &str) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    if !valid_bus_name(name) {
        return Err("invalid D-Bus well-known name".to_string());
    }
    let prefix = match (bus, access) {
        ("session", "talk") => "--talk-name=",
        ("session", "own") => "--own-name=",
        ("session", "deny") => "--no-talk-name=",
        ("system", "talk") => "--system-talk-name=",
        ("system", "own") => "--system-own-name=",
        ("system", "deny") => "--system-no-talk-name=",
        _ => return Err("D-Bus access expects session|system and talk|own|deny".to_string()),
    };
    let flag = format!("{prefix}{name}");
    output("flatpak", &["override", "--user", &flag, id]).map(|_| ())
}

fn valid_env_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .enumerate()
            .all(|(index, ch)| (ch == '_' || ch.is_ascii_alphanumeric()) && (index > 0 || ch == '_' || ch.is_ascii_alphabetic()))
}

fn valid_env_value(value: &str) -> bool {
    value.len() <= 8192 && !value.chars().any(|ch| matches!(ch, '\0' | '\n' | '\r'))
}

pub fn set_env(id: &str, name: &str, value: &str) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    if !valid_env_name(name) || !valid_env_value(value) {
        return Err("invalid Flatpak environment override".to_string());
    }
    let flag = format!("--env={name}={value}");
    output("flatpak", &["override", "--user", &flag, id]).map(|_| ())
}

pub fn unset_env(id: &str, name: &str) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    if !valid_env_name(name) {
        return Err("invalid Flatpak environment variable name".to_string());
    }
    let flag = format!("--unset-env={name}");
    output("flatpak", &["override", "--user", &flag, id]).map(|_| ())
}

pub fn reset_all(id: &str) -> Result<(), String> {
    let id = ensure_flatpak(id)?;
    output("flatpak", &["override", "--user", "--reset", id]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_allow_and_deny_items() {
        let value = "shared=network;!ipc;\nsockets=wayland;!x11;\nfilesystems=home:ro;!host;\n";
        assert_eq!(item_state(value, "shared", "network"), Some(true));
        assert_eq!(item_state(value, "shared", "ipc"), Some(false));
        assert_eq!(item_state(value, "sockets", "x11"), Some(false));
        assert_eq!(item_state(value, "filesystems", "home"), Some(true));
        assert_eq!(item_state(value, "filesystems", "host"), Some(false));
    }

    #[test]
    fn validates_custom_filesystems() {
        assert!(valid_filesystem("/srv/data"));
        assert!(valid_filesystem("~/Projects"));
        assert!(valid_filesystem("xdg-download/project"));
        assert!(!valid_filesystem("relative/path"));
    }

    #[test]
    fn validates_environment_overrides_without_shell_parsing() {
        assert!(valid_env_name("MY_APP_MODE"));
        assert!(!valid_env_name("1BAD"));
        assert!(!valid_env_name("BAD-NAME"));
        assert!(valid_env_value("value with spaces; stays one argv"));
        assert!(!valid_env_value("line1\nline2"));
    }

    #[test]
    fn removes_one_context_item_without_destroying_siblings() {
        let source = "[Context]\nshared=network;ipc;\nsockets=wayland;!x11;\n";
        let updated = remove_context_item(source, "shared", "network");
        assert!(updated.contains("shared=ipc;"));
        assert!(updated.contains("sockets=wayland;!x11;"));
        assert!(!updated.contains("shared=network"));
    }

    #[test]
    fn category_reset_preserves_other_sections() {
        let source = "[Context]\nfilesystems=home;xdg-download;\nsockets=wayland;\n[Environment]\nMODE=test\n";
        let updated = remove_key_from_section(source, "Context", Some("filesystems"));
        assert!(!updated.contains("filesystems="));
        assert!(updated.contains("sockets=wayland;"));
        assert!(updated.contains("MODE=test"));
    }
}
