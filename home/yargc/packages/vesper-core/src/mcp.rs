use std::fs;

use crate::json::escape;
use crate::paths::config_root;

fn registry_path() -> std::path::PathBuf {
    config_root().join("mcp-registry.tsv")
}

pub fn status_json() -> String {
    let mut items = Vec::new();
    for line in fs::read_to_string(registry_path()).unwrap_or_default().lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.is_empty() || parts[0].trim().is_empty() {
            continue;
        }
        items.push(format!(
            "{{\"name\":\"{}\",\"command\":\"{}\",\"args\":\"{}\",\"ownership\":\"nix\",\"mutable\":false,\"state\":\"configured\"}}",
            escape(parts[0]),
            escape(parts.get(1).copied().unwrap_or("")),
            escape(parts.get(2).copied().unwrap_or(""))
        ));
    }
    items.sort();
    format!("{{\"servers\":[{}]}}", items.join(","))
}
