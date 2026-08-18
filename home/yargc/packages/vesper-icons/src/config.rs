use std::fs;

use crate::util::{config_root, write_atomic, xdg_state_home};

#[derive(Clone, Debug)]
pub struct Config {
    pub enabled: bool,
    pub automatic: bool,
    pub remote_consent: bool,
    pub appearance: String,
    pub material: String,
    pub provider: String,
    pub model: String,
    pub follow_palette: bool,
    pub scheme_mode: String,
    pub accent: String,
    pub queue_paused: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            automatic: true,
            remote_consent: false,
            appearance: "automatic".into(),
            material: "standard".into(),
            provider: "openai".into(),
            model: "gpt-5.6".into(),
            follow_palette: true,
            scheme_mode: "dark".into(),
            accent: "#7aa2f7".into(),
            queue_paused: false,
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    config_root().join("adaptive-icons.conf")
}

pub fn exclusions_path() -> std::path::PathBuf {
    config_root().join("adaptive-icons-excluded")
}

fn bool_value(value: &str) -> bool {
    value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("on")
}

pub fn valid_appearance(value: &str) -> bool {
    matches!(value, "automatic" | "default" | "dark" | "clear" | "tinted")
}

pub fn valid_material(value: &str) -> bool {
    matches!(value, "standard" | "glass")
}

pub fn valid_provider(value: &str) -> bool {
    matches!(value, "openai" | "anthropic" | "xai" | "openrouter" | "google")
}

pub fn valid_scheme(value: &str) -> bool {
    matches!(value, "light" | "dark")
}

pub fn normalize_accent(value: &str) -> Option<String> {
    let value = value.trim().trim_start_matches('#');
    (value.len() == 6 && value.chars().all(|character| character.is_ascii_hexdigit()))
        .then(|| format!("#{}", value.to_ascii_lowercase()))
}

pub fn load() -> Config {
    let mut cfg = Config::default();
    let text = fs::read_to_string(config_path()).unwrap_or_default();
    for line in text.lines() {
        let Some((key, raw)) = line.split_once('=') else {
            continue;
        };
        let value = raw.trim();
        match key.trim() {
            "enabled" => cfg.enabled = bool_value(value),
            "automatic" => cfg.automatic = bool_value(value),
            "remoteConsent" => cfg.remote_consent = bool_value(value),
            "appearance" if valid_appearance(value) => cfg.appearance = value.into(),
            "material" if valid_material(value) => cfg.material = value.into(),
            "provider" if valid_provider(value) => cfg.provider = value.into(),
            "model" if !value.is_empty() => cfg.model = value.into(),
            "followPalette" => cfg.follow_palette = bool_value(value),
            "schemeMode" if valid_scheme(value) => cfg.scheme_mode = value.into(),
            "accent" => {
                if let Some(value) = normalize_accent(value) {
                    cfg.accent = value;
                }
            }
            "queuePaused" => cfg.queue_paused = bool_value(value),
            // One-way compatibility with the prototype's combined rendering mode.
            "mode" => match value {
                "original" => {
                    cfg.enabled = false;
                    cfg.appearance = "automatic".into();
                    cfg.material = "standard".into();
                }
                "light" => {
                    cfg.appearance = "default".into();
                    cfg.material = "standard".into();
                }
                "dark" => {
                    cfg.appearance = "dark".into();
                    cfg.material = "standard".into();
                }
                "clear" => {
                    cfg.appearance = "clear".into();
                    cfg.material = "standard".into();
                }
                "tinted" => {
                    cfg.appearance = "tinted".into();
                    cfg.material = "standard".into();
                }
                "glass" => cfg.material = "glass".into(),
                _ => {}
            },
            _ => {}
        }
    }

    if text.is_empty() {
        if let Ok(value) = fs::read_to_string(xdg_state_home().join("vesper/adaptive-icons/enabled")) {
            cfg.enabled = bool_value(value.trim());
        }
    }
    cfg
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let body = format!(
        "enabled={}\nautomatic={}\nremoteConsent={}\nappearance={}\nmaterial={}\nprovider={}\nmodel={}\nfollowPalette={}\nschemeMode={}\naccent={}\nqueuePaused={}\n",
        cfg.enabled as u8,
        cfg.automatic as u8,
        cfg.remote_consent as u8,
        cfg.appearance,
        cfg.material,
        cfg.provider,
        cfg.model,
        cfg.follow_palette as u8,
        cfg.scheme_mode,
        cfg.accent,
        cfg.queue_paused as u8
    );
    write_atomic(&config_path(), body)
}

pub fn sync_palette(cfg: &mut Config) {
    if !cfg.follow_palette {
        return;
    }
    let path = xdg_state_home().join("caelestia/theme/vesper-icons");
    if let Ok(value) = fs::read_to_string(path) {
        if let Some(value) = normalize_accent(&value) {
            cfg.accent = value;
        }
    }
}
