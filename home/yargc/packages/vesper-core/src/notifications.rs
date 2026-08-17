use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use crate::json::escape;
use crate::paths::{atomic_write_private, config_root};

#[derive(Clone, Debug)]
struct PolicyEntry {
    id: String,
    name: String,
    policy: String,
}

fn policy_path() -> std::path::PathBuf {
    config_root().join("notifications/policy.tsv")
}

fn registry_path() -> std::path::PathBuf {
    config_root().join("notifications/policy.json")
}

pub fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn desktop_variants(value: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let mut current = value.trim().to_string();
    if current.is_empty() {
        return variants;
    }
    variants.push(current.clone());
    while let Some(stripped) = current.strip_suffix(".desktop") {
        current = stripped.to_string();
        if current.is_empty() {
            break;
        }
        variants.push(current.clone());
    }
    variants
}

fn aliases(id: &str, name: &str) -> BTreeSet<String> {
    desktop_variants(id)
        .into_iter()
        .chain(desktop_variants(name))
        .map(|value| normalize(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn clean_field(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn load() -> BTreeMap<String, PolicyEntry> {
    let mut entries = BTreeMap::new();
    for line in fs::read_to_string(policy_path()).unwrap_or_default().lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() != 3 || !matches!(parts[2], "allow" | "block") {
            continue;
        }
        entries.insert(
            parts[0].to_string(),
            PolicyEntry {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                policy: parts[2].to_string(),
            },
        );
    }
    entries
}

fn save(entries: &BTreeMap<String, PolicyEntry>) -> Result<(), String> {
    let mut tsv = String::new();
    let mut registry = BTreeMap::<String, String>::new();
    for entry in entries.values() {
        tsv.push_str(&format!(
            "{}\t{}\t{}\n",
            clean_field(&entry.id),
            clean_field(&entry.name),
            entry.policy
        ));
        for alias in aliases(&entry.id, &entry.name) {
            registry.insert(alias, entry.policy.clone());
        }
    }

    let json = registry
        .iter()
        .map(|(alias, policy)| format!("\"{}\":\"{}\"", escape(alias), escape(policy)))
        .collect::<Vec<_>>()
        .join(",");

    atomic_write_private(&policy_path(), tsv.as_bytes())?;
    atomic_write_private(&registry_path(), format!("{{{json}}}\n").as_bytes())
}

pub fn policy_for(id: &str) -> &'static str {
    let wanted = desktop_variants(id)
        .into_iter()
        .map(|value| normalize(&value))
        .collect::<BTreeSet<_>>();
    for entry in load().values() {
        if aliases(&entry.id, &entry.name)
            .iter()
            .any(|alias| wanted.contains(alias))
        {
            return if entry.policy == "block" { "block" } else { "allow" };
        }
    }
    "inherit"
}

pub fn set_policy(id: &str, name: &str, policy: &str) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("notification policy requires an application id".to_string());
    }
    if !matches!(policy, "inherit" | "allow" | "block") {
        return Err("notification policy expects inherit, allow or block".to_string());
    }

    let mut entries = load();
    if policy == "inherit" {
        entries.remove(id);
    } else {
        entries.insert(
            id.to_string(),
            PolicyEntry {
                id: id.to_string(),
                name: name.to_string(),
                policy: policy.to_string(),
            },
        );
    }
    save(&entries)
}

pub fn status_json() -> String {
    let entries = load()
        .values()
        .map(|entry| {
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"policy\":\"{}\"}}",
                escape(&entry.id),
                escape(&entry.name),
                escape(&entry.policy)
            )
        })
        .collect::<Vec<_>>();
    format!("{{\"policies\":[{}]}}", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_desktop_ids_and_names() {
        assert_eq!(normalize("org.Example.App.desktop"), "orgexampleappdesktop");
        let values = aliases("org.Telegram.desktop.desktop", "Telegram Desktop");
        assert!(values.contains("orgtelegramdesktopdesktop"));
        assert!(values.contains("orgtelegramdesktop"));
        assert!(values.contains("orgtelegram"));
        assert!(values.contains("telegramdesktop"));
    }

    #[test]
    fn empty_aliases_are_not_exported() {
        assert!(aliases("", "").is_empty());
    }
}
