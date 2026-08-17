use std::collections::BTreeMap;
use std::fs;

use crate::json::escape;
use crate::paths::config_root;

fn registry_path() -> std::path::PathBuf {
    config_root().join("mcp-registry.tsv")
}

fn names_path() -> std::path::PathBuf {
    config_root().join("mcp-servers")
}

#[derive(Clone, Debug)]
struct McpServer {
    name: String,
    command: String,
    args: String,
    source: String,
    version: String,
    transport: String,
    credential: String,
}

fn load() -> BTreeMap<String, McpServer> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(registry_path()).unwrap_or_default().lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        let name = parts.first().copied().unwrap_or("").trim();
        if name.is_empty() { continue; }
        values.insert(name.to_string(), McpServer {
            name: name.to_string(),
            command: parts.get(1).copied().unwrap_or("").to_string(),
            args: parts.get(2).copied().unwrap_or("").to_string(),
            source: parts.get(3).copied().unwrap_or("nix").to_string(),
            version: parts.get(4).copied().unwrap_or("").to_string(),
            transport: parts.get(5).copied().unwrap_or("stdio").to_string(),
            credential: parts.get(6).copied().unwrap_or("").to_string(),
        });
    }

    // Older Home Manager integration only exported the canonical server names.
    // Keep those visible instead of returning an empty MCP page, while being
    // explicit that command/tool metadata is unavailable from this fallback.
    for name in fs::read_to_string(names_path()).unwrap_or_default().lines().map(str::trim).filter(|line| !line.is_empty()) {
        values.entry(name.to_string()).or_insert_with(|| McpServer {
            name: name.to_string(),
            command: String::new(),
            args: String::new(),
            source: "nix".to_string(),
            version: String::new(),
            transport: "unknown".to_string(),
            credential: String::new(),
        });
    }
    values
}

pub fn status_json() -> String {
    let items = load().values().map(|server| {
        format!(
            "{{\"name\":\"{}\",\"command\":\"{}\",\"args\":\"{}\",\"source\":\"{}\",\"version\":\"{}\",\"transport\":\"{}\",\"credential\":\"{}\",\"ownership\":\"nix\",\"managedBy\":\"nix\",\"mutable\":false,\"state\":\"configured\",\"health\":null,\"tools\":[],\"permissions\":null}}",
            escape(&server.name),
            escape(&server.command),
            escape(&server.args),
            escape(&server.source),
            escape(&server.version),
            escape(&server.transport),
            escape(&server.credential)
        )
    }).collect::<Vec<_>>();
    format!("{{\"servers\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_server_metadata_is_honest() {
        let server = McpServer {
            name: "example".into(), command: String::new(), args: String::new(), source: "nix".into(),
            version: String::new(), transport: "unknown".into(), credential: String::new(),
        };
        assert!(server.command.is_empty());
        assert_eq!(server.transport, "unknown");
    }
}
