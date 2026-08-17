use crate::json::{bool_lit, escape};
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
    PermissionDef { id: "pcsc", label: "Smart cards", key: "sockets", item: "pcsc", kind: PermissionKind::Socket },
    PermissionDef { id: "bluetooth", label: "Bluetooth", key: "features", item: "bluetooth", kind: PermissionKind::Feature },
    PermissionDef { id: "devices-all", label: "All devices", key: "devices", item: "all", kind: PermissionKind::Device },
    PermissionDef { id: "dri", label: "GPU / DRI", key: "devices", item: "dri", kind: PermissionKind::Device },
    PermissionDef { id: "kvm", label: "KVM", key: "devices", item: "kvm", kind: PermissionKind::Device },
    PermissionDef { id: "shm", label: "Shared memory device", key: "devices", item: "shm", kind: PermissionKind::Device },
    PermissionDef { id: "home", label: "Home folder", key: "filesystems", item: "home", kind: PermissionKind::Filesystem },
    PermissionDef { id: "host", label: "Host filesystem", key: "filesystems", item: "host", kind: PermissionKind::Filesystem },
    PermissionDef { id: "desktop", label: "Desktop folder", key: "filesystems", item: "xdg-desktop", kind: PermissionKind::Filesystem },
    PermissionDef { id: "documents", label: "Documents folder", key: "filesystems", item: "xdg-documents", kind: PermissionKind::Filesystem },
    PermissionDef { id: "downloads", label: "Downloads folder", key: "filesystems", item: "xdg-download", kind: PermissionKind::Filesystem },
    PermissionDef { id: "music", label: "Music folder", key: "filesystems", item: "xdg-music", kind: PermissionKind::Filesystem },
    PermissionDef { id: "pictures", label: "Pictures folder", key: "filesystems", item: "xdg-pictures", kind: PermissionKind::Filesystem },
    PermissionDef { id: "videos", label: "Videos folder", key: "filesystems", item: "xdg-videos", kind: PermissionKind::Filesystem },
];

fn flatpak_id(id: &str) -> &str {
    id.strip_suffix(".desktop").unwrap_or(id)
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
        "{{\"id\":\"{}\",\"label\":\"{}\",\"packaged\":{},\"userOverride\":\"{}\",\"effective\":{},\"backend\":\"Flatpak-enforced\"}}",
        escape(def.id),
        escape(def.label),
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
}
