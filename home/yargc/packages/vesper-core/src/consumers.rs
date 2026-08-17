use std::collections::BTreeMap;
use std::fs;

use crate::json::escape;
use crate::paths::{atomic_write_private, config_root};

const CONSUMERS: &[(&str, &str, bool)] = &[
    ("opencode", "native", true),
    ("hermes", "xai", false),
    ("icon-curator", "openai", false),
    ("github-mcp", "github", false),
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

fn consumer_def(consumer: &str) -> Option<(&'static str, bool)> {
    CONSUMERS
        .iter()
        .find(|(name, _, _)| *name == consumer)
        .map(|(_, default, allows_native)| (*default, *allows_native))
}

fn default_for(consumer: &str) -> Option<&'static str> {
    consumer_def(consumer).map(|(default, _)| default)
}

fn load() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(path()).unwrap_or_default().lines() {
        let Some((consumer, credential)) = line.split_once('\t') else {
            continue;
        };
        let Some((_, allows_native)) = consumer_def(consumer) else { continue; };
        if valid_token(credential) && (credential != "native" || allows_native) {
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
    let (default, allows_native) = consumer_def(consumer).ok_or_else(|| format!("unknown AI consumer: {consumer}"))?;
    if !valid_token(credential) {
        return Err("credential selection must be a safe credential alias".to_string());
    }
    if credential == "native" && !allows_native {
        return Err(format!("{consumer} requires a Vesper API-key credential alias"));
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
        .map(|(consumer, default, allows_native)| {
            let selected = values
                .get(*consumer)
                .map(String::as_str)
                .unwrap_or(default);
            format!(
                "{{\"consumer\":\"{}\",\"credential\":\"{}\",\"defaultCredential\":\"{}\",\"allowsNative\":{},\"nativeAuth\":{}}}",
                escape(consumer),
                escape(selected),
                escape(default),
                if *allows_native { "true" } else { "false" },
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
    fn api_key_consumers_cannot_fall_back_to_native_auth() {
        assert_eq!(consumer_def("opencode"), Some(("native", true)));
        assert_eq!(consumer_def("hermes"), Some(("xai", false)));
        assert_eq!(consumer_def("icon-curator"), Some(("openai", false)));
        assert_eq!(consumer_def("github-mcp"), Some(("github", false)));
    }

    #[test]
    fn credential_alias_validation_rejects_shell_syntax() {
        assert!(valid_token("openrouter-main"));
        assert!(!valid_token("openai;rm -rf /"));
    }
}
