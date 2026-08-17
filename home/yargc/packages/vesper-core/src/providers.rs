use std::collections::BTreeMap;
use std::fs;

use crate::json::{bool_lit, escape};
use crate::paths::{atomic_write_private, config_root};
use crate::process::output;

include!("../../vesper-provider-registry.rs");

#[derive(Clone, Debug)]
struct Provider {
    id: String,
    name: String,
    base_url: String,
    credential: String,
    model: String,
    budget_cents: u64,
    enabled: bool,
    custom: bool,
}

fn providers_path() -> std::path::PathBuf { config_root().join("ai/providers.tsv") }
fn routing_path() -> std::path::PathBuf { config_root().join("ai/routing") }

fn valid_token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn valid_url(value: &str) -> bool {
    value.len() <= 2048
        && !value.chars().any(|ch| ch.is_control() || ch.is_whitespace())
        && (value.starts_with("https://") || value.starts_with("http://127.0.0.1:") || value.starts_with("http://localhost:"))
}

fn clean(value: &str) -> String { value.replace(['\t', '\n', '\r'], " ") }

fn builtin_map() -> BTreeMap<String, Provider> {
    PROVIDER_ENDPOINTS.iter().map(|(id, name, url, credential)| {
        ((*id).to_string(), Provider {
            id: (*id).to_string(), name: (*name).to_string(), base_url: (*url).to_string(),
            credential: (*credential).to_string(), model: String::new(), budget_cents: 0, enabled: true, custom: false,
        })
    }).collect()
}

fn load() -> BTreeMap<String, Provider> {
    let mut values = builtin_map();
    for line in fs::read_to_string(providers_path()).unwrap_or_default().lines() {
        let p = line.split('\t').collect::<Vec<_>>();
        if p.len() != 8 || !valid_token(p[0]) { continue; }
        let custom = p[7] == "1";
        let Some(existing) = values.get(p[0]).cloned().or_else(|| {
            if custom && valid_url(p[2]) {
                Some(Provider { id: p[0].to_string(), name: p[1].to_string(), base_url: p[2].to_string(), credential: p[3].to_string(), model: p[4].to_string(), budget_cents: 0, enabled: true, custom: true })
            } else { None }
        }) else { continue; };
        values.insert(p[0].to_string(), Provider {
            id: p[0].to_string(),
            name: if p[1].is_empty() { existing.name } else { p[1].to_string() },
            base_url: if custom && valid_url(p[2]) { p[2].to_string() } else { existing.base_url },
            credential: if valid_token(p[3]) { p[3].to_string() } else { existing.credential },
            model: p[4].to_string(),
            budget_cents: p[5].parse().unwrap_or(0),
            enabled: p[6] != "0",
            custom,
        });
    }
    values
}

fn save(values: &BTreeMap<String, Provider>) -> Result<(), String> {
    let builtins = builtin_map();
    let mut data = String::new();
    for provider in values.values() {
        let default = builtins.get(&provider.id);
        let changed = provider.custom || default.map(|d| d.credential != provider.credential || provider.model != d.model || provider.budget_cents != 0 || !provider.enabled).unwrap_or(true);
        if !changed { continue; }
        data.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            clean(&provider.id), clean(&provider.name), clean(&provider.base_url), clean(&provider.credential), clean(&provider.model),
            provider.budget_cents, if provider.enabled { 1 } else { 0 }, if provider.custom { 1 } else { 0 }));
    }
    atomic_write_private(&providers_path(), data.as_bytes())
}

pub fn add(id: &str, name: &str, base_url: &str, credential: &str) -> Result<(), String> {
    if !valid_token(id) || !valid_token(credential) || !valid_url(base_url) { return Err("custom provider requires a safe id/credential alias and HTTPS (or localhost HTTP) base URL".to_string()); }
    if PROVIDER_ENDPOINTS.iter().any(|(builtin, _, _, _)| *builtin == id) { return Err("built-in provider ids cannot be replaced".to_string()); }
    let mut values = load();
    values.insert(id.to_string(), Provider { id: id.to_string(), name: clean(name), base_url: base_url.trim_end_matches('/').to_string(), credential: credential.to_string(), model: String::new(), budget_cents: 0, enabled: true, custom: true });
    save(&values)
}

pub fn remove(id: &str) -> Result<(), String> {
    let mut values = load();
    match values.get(id) {
        Some(p) if p.custom => { values.remove(id); save(&values) }
        Some(_) => Err("built-in providers cannot be removed".to_string()),
        None => Err("unknown provider".to_string()),
    }
}

pub fn set(id: &str, field: &str, value: &str) -> Result<(), String> {
    let mut values = load();
    let provider = values.get_mut(id).ok_or_else(|| "unknown provider".to_string())?;
    match field {
        "credential" if valid_token(value) => provider.credential = value.to_string(),
        "model" if value.len() <= 128 && !value.chars().any(|c| c.is_control() || c.is_whitespace()) => provider.model = value.to_string(),
        "budget" => provider.budget_cents = value.parse::<u64>().map_err(|_| "budget expects integer cents; 0 disables".to_string())?,
        "enabled" => provider.enabled = matches!(value, "1" | "true" | "on"),
        "base-url" if provider.custom && valid_url(value) => provider.base_url = value.trim_end_matches('/').to_string(),
        _ => return Err("unsupported or invalid provider field".to_string()),
    }
    save(&values)
}

fn routing() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(routing_path()).unwrap_or_default().lines() {
        if let Some((k, v)) = line.split_once('=') { values.insert(k.to_string(), v.to_string()); }
    }
    values
}

pub fn set_routing(default_provider: &str, default_model: &str, fallbacks: &str) -> Result<(), String> {
    let providers = load();
    if !providers.get(default_provider).map(|p| p.enabled).unwrap_or(false) { return Err("default provider must exist and be enabled".to_string()); }
    for id in fallbacks.split(',').filter(|v| !v.is_empty()) {
        if !providers.get(id).map(|p| p.enabled).unwrap_or(false) { return Err(format!("fallback provider is unknown or disabled: {id}")); }
    }
    if default_model.len() > 128 || default_model.chars().any(|c| c.is_control() || c.is_whitespace()) { return Err("invalid default model".to_string()); }
    atomic_write_private(&routing_path(), format!("defaultProvider={default_provider}\ndefaultModel={default_model}\nfallbacks={fallbacks}\n").as_bytes())
}

fn endpoint_probe(provider: &Provider) -> (Option<bool>, Option<u64>) {
    let url = format!("{}/models", provider.base_url.trim_end_matches('/'));
    match output("curl", &[
        "--silent", "--show-error", "--output", "/dev/null",
        "--connect-timeout", "3", "--max-time", "5",
        "--write-out", "%{http_code}\t%{time_total}", &url,
    ]) {
        Ok(value) => {
            let mut parts = value.split('\t');
            let code = parts.next().unwrap_or("");
            let latency = parts
                .next()
                .and_then(|seconds| seconds.parse::<f64>().ok())
                .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
                .map(|seconds| (seconds * 1000.0).round() as u64);
            (Some(code.len() == 3 && code != "000"), latency)
        }
        Err(_) => (Some(false), None),
    }
}

pub fn status_json(test_endpoints: bool) -> String {
    let values = load();
    let route = routing();
    let providers = values.values().map(|p| {
        let (reachable, latency) = if test_endpoints { endpoint_probe(p) } else { (None, None) };
        let reachable_json = reachable.map(|v| bool_lit(v).to_string()).unwrap_or_else(|| "null".to_string());
        let latency_json = latency.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());
        format!("{{\"id\":\"{}\",\"name\":\"{}\",\"baseUrl\":\"{}\",\"credential\":\"{}\",\"model\":\"{}\",\"budgetCents\":{},\"enabled\":{},\"custom\":{},\"endpointReachable\":{},\"authValid\":null,\"latencyMs\":{},\"quota\":null}}",
            escape(&p.id), escape(&p.name), escape(&p.base_url), escape(&p.credential), escape(&p.model), p.budget_cents,
            bool_lit(p.enabled), bool_lit(p.custom), reachable_json, latency_json)
    }).collect::<Vec<_>>();
    format!("{{\"defaultProvider\":\"{}\",\"defaultModel\":\"{}\",\"fallbacks\":\"{}\",\"providers\":[{}]}}",
        escape(route.get("defaultProvider").map(String::as_str).unwrap_or("openai")),
        escape(route.get("defaultModel").map(String::as_str).unwrap_or("")),
        escape(route.get("fallbacks").map(String::as_str).unwrap_or("")), providers.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn custom_url_policy_requires_tls_except_loopback() { assert!(valid_url("https://example.com/v1")); assert!(valid_url("http://127.0.0.1:8080/v1")); assert!(!valid_url("http://example.com/v1")); }
    #[test] fn builtins_remain_distinct_from_custom() { assert!(builtin_map().values().all(|p| !p.custom)); }
    #[test] fn endpoint_registry_matches_credential_registry() {
        assert!(PROVIDER_ENDPOINTS.iter().all(|(id, _, _, _)| PROVIDERS.iter().any(|(credential_id, _, _)| credential_id == id)));
    }
}
