use std::collections::BTreeMap;
use std::fs;

use crate::json::escape;
use crate::paths::{atomic_write_private, config_root};

const CONSUMERS: &[(&str, &str)] = &[
    ("opencode", "native"),
    ("hermes", "xai"),
    ("icon-curator", "openai"),
    ("github-mcp", "github"),
];

fn path() -> std::path::PathBuf {
    config_root().join("ai/consumers.tsv")
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn default_for(consumer: &str) -> Option<&'static str> {
    CONSUMERS
        .iter()
        .find(|(name, _)| *name == consumer)
        .map(|(_, credential)| *credential)
}

fn load() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(path()).unwrap_or_default().lines() {
        let Some((consumer, credential)) = line.split_once('\t') else {
            continue;
        };
        if default_for(consumer).is_some() && (credential == "native" || valid_token(credential)) {
            values.insert(consumer.to_string(), credential.to_string());
        }
    }
    values
}

fn save(values: &BTreeMap<String, String>) -> Result<(), String> {
    let mut data = String::new();
    for (consumer, credential) in values {
        data.push_str(consumer);
        data.push('\t');
        data.push_str(credential);
        data.push('\n');
    }
    atomic_write_private(&path(), data.as_bytes())
}

pub fn credential_for(consumer: &str) -> Result<String, String> {
    let default = default_for(consumer).ok_or_else(|| format!("unknown AI consumer: {consumer}"))?;
    Ok(load()
        .get(consumer)
        .cloned()
        .unwrap_or_else(|| default.to_string()))
}

pub fn set_credential(consumer: &str, credential: &str) -> Result<(), String> {
    let default = default_for(consumer).ok_or_else(|| format!("unknown AI consumer: {consumer}"))?;
    if credential != "native" && !valid_token(credential) {
        return Err("credential selection must be 'native' or a safe credential alias".to_string());
    }

    let mut values = load();
    if credential == default {
        values.remove(consumer);
    } else {
        values.insert(consumer.to_string(), credential.to_string());
    }
    save(&values)
}

pub fn status_json() -> String {
    let values = load();
    let items = CONSUMERS
        .iter()
        .map(|(consumer, default)| {
            let selected = values
                .get(*consumer)
                .map(String::as_str)
                .unwrap_or(default);
            format!(
                "{{\"consumer\":\"{}\",\"credential\":\"{}\",\"defaultCredential\":\"{}\",\"nativeAuth\":{}}}",
                escape(consumer),
                escape(selected),
                escape(default),
                if selected == "native" { "true" } else { "false" }
            )
        })
        .collect::<Vec<_>>();
    format!("{{\"consumers\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_do_not_force_subscription_clients_into_api_key_mode() {
        assert_eq!(default_for("opencode"), Some("native"));
        assert_eq!(default_for("hermes"), Some("xai"));
    }

    #[test]
    fn credential_alias_validation_rejects_shell_syntax() {
        assert!(valid_token("openrouter-main"));
        assert!(!valid_token("openai;rm -rf /"));
    }
}
