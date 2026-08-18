use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct DesktopRecord {
    pub id: String,
    pub path: PathBuf,
    pub icon: String,
    pub exec: String,
    pub startup_wm_class: String,
    pub flatpak_id: String,
    pub generated_shadow: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Identity {
    pub canonical_app_id: String,
    pub launch_desktop_id: String,
    pub runtime_ids: Vec<String>,
    pub icon_aliases: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Source {
    pub path: PathBuf,
    pub kind: String,
    pub resolver: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Default)]
pub struct InventoryItem {
    pub desktop: DesktopRecord,
    pub identity: Identity,
    pub source: Option<Source>,
    pub work_key: String,
    pub tier: String,
    pub queue_state: String,
    pub active: bool,
    pub excluded: bool,
    pub error: String,
}
