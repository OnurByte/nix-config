use std::collections::BTreeMap;
use std::fs;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, config_root, home};
use crate::process::{binary, output, success};

include!("../../vesper-provider-registry.rs");

fn provider(id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    PROVIDERS.iter().copied().find(|item| item.0 == id)
}

fn registry_path() -> std::path::PathBuf {
    config_root().join("credentials.tsv")
}

fn valid_alias(alias: &str) -> bool {
    !alias.is_empty()
        && alias.len() <= 80
        && alias
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn load_registry() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(registry_path()).unwrap_or_default().lines() {
        let Some((alias, provider_id)) = line.split_once('\t') else { continue; };
        if valid_alias(alias) && provider(provider_id).is_some() {
            values.insert(alias.to_string(), provider_id.to_string());
        }
    }
    values
}

fn save_registry(values: &BTreeMap<String, String>) -> Result<(), String> {
    let mut data = String::new();
    for (alias, provider_id) in values {
        data.push_str(alias);
        data.push('\t');
        data.push_str(provider_id);
        data.push('\n');
    }
    atomic_write_private(&registry_path(), data.as_bytes())
}

fn default_configured(id: &str) -> bool {
    success("secret-tool", &["lookup", "service", "vesper-ai", "provider", id])
}

fn alias_configured(alias: &str, provider_id: &str) -> bool {
    success(
        "secret-tool",
        &[
            "lookup",
            "service",
            "vesper-ai",
            "credential",
            alias,
            "provider",
            provider_id,
        ],
    )
}

fn default_lookup(id: &str) -> Result<String, String> {
    output("secret-tool", &["lookup", "service", "vesper-ai", "provider", id])
        .and_then(|value| if value.is_empty() { Err("credential is empty".to_string()) } else { Ok(value) })
}

fn alias_lookup(alias: &str, provider_id: &str) -> Result<String, String> {
    output(
        "secret-tool",
        &[
            "lookup",
            "service",
            "vesper-ai",
            "credential",
            alias,
            "provider",
            provider_id,
        ],
    )
    .and_then(|value| if value.is_empty() { Err("credential is empty".to_string()) } else { Ok(value) })
}

fn store_secret(args: &[String], secret: &str) -> Result<(), String> {
    if secret.trim().is_empty() {
        return Err("API key is empty".to_string());
    }
    let mut child = Command::new(binary("secret-tool"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start secret-tool: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(secret.trim().as_bytes())
            .map_err(|error| error.to_string())?;
        stdin.write_all(b"\n").map_err(|error| error.to_string())?;
    }
    let result = child.wait_with_output().map_err(|error| error.to_string())?;
    if result.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        Err(if message.is_empty() {
            "Secret Service rejected the API key".to_string()
        } else {
            message
        })
    }
}

pub fn set(provider_id: &str, alias: Option<&str>, secret: &str) -> Result<(), String> {
    let (_, label, _) = provider(provider_id).ok_or_else(|| format!("unknown provider: {provider_id}"))?;
    match alias {
        None => store_secret(
            &[
                "store".to_string(),
                format!("--label=Vesper AI · {label}"),
                "service".to_string(),
                "vesper-ai".to_string(),
                "provider".to_string(),
                provider_id.to_string(),
            ],
            secret,
        ),
        Some(alias) => {
            if !valid_alias(alias) {
                return Err("credential alias may contain only letters, digits, '.', '_' and '-'".to_string());
            }
            if provider(alias).is_some() {
                return Err("provider IDs are reserved as default credential names; choose another alias".to_string());
            }
            let mut registry = load_registry();
            if let Some(existing) = registry.get(alias) {
                if existing != provider_id {
                    return Err(format!("credential alias {alias} already belongs to provider {existing}"));
                }
            }
            store_secret(
                &[
                    "store".to_string(),
                    format!("--label=Vesper AI · {label} · {alias}"),
                    "service".to_string(),
                    "vesper-ai".to_string(),
                    "credential".to_string(),
                    alias.to_string(),
                    "provider".to_string(),
                    provider_id.to_string(),
                ],
                secret,
            )?;
            registry.insert(alias.to_string(), provider_id.to_string());
            save_registry(&registry)
        }
    }
}

pub fn status(name: &str) -> Result<bool, String> {
    if provider(name).is_some() {
        return Ok(default_configured(name));
    }
    let registry = load_registry();
    let provider_id = registry
        .get(name)
        .ok_or_else(|| format!("unknown credential alias: {name}"))?;
    Ok(alias_configured(name, provider_id))
}

pub fn clear(name: &str) -> Result<(), String> {
    if provider(name).is_some() {
        let result = Command::new(binary("secret-tool"))
            .args(["clear", "service", "vesper-ai", "provider", name])
            .output()
            .map_err(|error| format!("failed to run secret-tool: {error}"))?;
        if result.status.success() {
            return Ok(());
        }
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if message.is_empty() { "failed to clear credential".to_string() } else { message });
    }

    let mut registry = load_registry();
    let provider_id = registry
        .get(name)
        .cloned()
        .ok_or_else(|| format!("unknown credential alias: {name}"))?;
    let result = Command::new(binary("secret-tool"))
        .args([
            "clear",
            "service",
            "vesper-ai",
            "credential",
            name,
            "provider",
            &provider_id,
        ])
        .output()
        .map_err(|error| format!("failed to run secret-tool: {error}"))?;
    if !result.status.success() {
        let message = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if message.is_empty() { "failed to clear credential".to_string() } else { message });
    }
    registry.remove(name);
    save_registry(&registry)
}

fn resolve(name: &str) -> Result<(&'static str, String), String> {
    if let Some((_, _, env_name)) = provider(name) {
        return Ok((env_name, default_lookup(name)?));
    }
    let registry = load_registry();
    let provider_id = registry
        .get(name)
        .ok_or_else(|| format!("unknown credential alias: {name}"))?;
    let (_, _, env_name) = provider(provider_id).ok_or_else(|| format!("unknown provider: {provider_id}"))?;
    Ok((env_name, alias_lookup(name, provider_id)?))
}

pub fn exec(name: &str, command: &[String]) -> Result<(), String> {
    let command = if command.first().map(String::as_str) == Some("--") {
        &command[1..]
    } else {
        command
    };
    if command.is_empty() {
        return Err("credential exec needs a command".to_string());
    }
    let (env_name, secret) = resolve(name)?;
    let error = Command::new(&command[0])
        .args(&command[1..])
        .env(env_name, secret)
        .exec();
    Err(format!("failed to exec {}: {error}", command[0]))
}

pub fn list_json() -> String {
    let registry = load_registry();
    let items = registry
        .iter()
        .filter_map(|(alias, provider_id)| {
            let (_, name, env_name) = provider(provider_id)?;
            Some(format!(
                "{{\"id\":\"{}\",\"provider\":\"{}\",\"providerName\":\"{}\",\"env\":\"{}\",\"configured\":{},\"managedBy\":\"vesper\"}}",
                escape(alias),
                escape(provider_id),
                escape(name),
                escape(env_name),
                bool_lit(alias_configured(alias, provider_id))
            ))
        })
        .collect::<Vec<_>>();
    format!("{{\"credentials\":[{}]}}", items.join(","))
}

fn list_dir_names(path: &std::path::Path) -> Vec<String> {
    let mut items = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    items.sort();
    items
}

fn mcp_names() -> Vec<String> {
    fs::read_to_string(config_root().join("mcp-servers"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn ai_status_json() -> String {
    let credentials = PROVIDERS
        .iter()
        .map(|(id, name, env_name)| {
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"env\":\"{}\",\"configured\":{}}}",
                escape(id), escape(name), escape(env_name), bool_lit(default_configured(id))
            )
        })
        .collect::<Vec<_>>();
    let skills = list_dir_names(&home().join(".agents/skills"));
    let skills_json = skills.iter().map(|name| format!("\"{}\"", escape(name))).collect::<Vec<_>>();
    let mcp = mcp_names();
    let mcp_json = mcp.iter().map(|name| format!("\"{}\"", escape(name))).collect::<Vec<_>>();
    let hermes_registry = home().join(".config/vesper/hermes-jobs.json").exists();
    format!(
        "{{\"credentials\":[{}],\"skills\":{{\"count\":{},\"items\":[{}]}},\"mcp\":{{\"count\":{},\"items\":[{}]}},\"hermesRegistry\":{}}}",
        credentials.join(","), skills.len(), skills_json.join(","), mcp.len(), mcp_json.join(","), bool_lit(hermes_registry)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_aliases_are_shell_safe_tokens() {
        assert!(valid_alias("openai-main"));
        assert!(valid_alias("work.key_2"));
        assert!(!valid_alias("bad alias"));
        assert!(!valid_alias("x;rm"));
    }

    #[test]
    fn static_provider_ids_resolve_to_environment_names() {
        assert_eq!(provider("openai").map(|item| item.2), Some("OPENAI_API_KEY"));
        assert_eq!(provider("github").map(|item| item.2), Some("GITHUB_PERSONAL_ACCESS_TOKEN"));
    }
}
