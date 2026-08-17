use std::env;
use std::fs;

use crate::json::{bool_lit, escape};
use crate::paths::config_root;
use crate::process::{output, success};

fn unit_active(unit: &str) -> bool {
    success("systemctl", &["is-active", "--quiet", unit])
}

fn listening_on(port: &str) -> bool {
    output("ss", &["-ltn"])
        .map(|text| text.lines().any(|line| line.split_whitespace().any(|field| field.ends_with(port))))
        .unwrap_or(false)
}

fn command_available(command: &str) -> bool {
    env::var_os("PATH")
        .map(|path| env::split_paths(&path).any(|dir| dir.join(command).is_file()))
        .unwrap_or(false)
}

fn process_running(name: &str) -> bool {
    success("pgrep", &["-x", name])
}

fn dns_summary() -> String {
    output("resolvectl", &["dns"])
        .map(|value| value.lines().take(8).collect::<Vec<_>>().join(" · "))
        .unwrap_or_default()
}

fn resolver_security_summary() -> String {
    output("resolvectl", &["status"])
        .map(|value| {
            value
                .lines()
                .map(str::trim)
                .filter(|line| {
                    line.starts_with("Protocols:")
                        || line.starts_with("DNSSEC")
                        || line.contains("DNSOverTLS")
                })
                .take(8)
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .unwrap_or_default()
}

fn active_connections() -> Vec<(String, String)> {
    let rows = output("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show", "--active"])
        .unwrap_or_default();
    rows.lines()
        .filter_map(|line| {
            let (name, kind) = line.rsplit_once(':')?;
            if name.is_empty() || kind.is_empty() {
                return None;
            }
            Some((name.replace("\\:", ":"), kind.to_string()))
        })
        .collect()
}

fn active_wifi_profile() -> Option<String> {
    active_connections()
        .into_iter()
        .find(|(_, kind)| matches!(kind.as_str(), "wifi" | "802-11-wireless"))
        .map(|(name, _)| name)
}

fn active_vpn_profiles() -> Vec<String> {
    active_connections()
        .into_iter()
        .filter(|(_, kind)| matches!(kind.as_str(), "vpn" | "wireguard" | "tun" | "ip-tunnel"))
        .map(|(name, _)| name)
        .collect()
}

fn mac_policy() -> String {
    let Some(profile) = active_wifi_profile() else { return String::new(); };
    output(
        "nmcli",
        &["-g", "802-11-wireless.cloned-mac-address", "connection", "show", &profile],
    )
    .unwrap_or_default()
}

fn proxy_state() -> (bool, bool) {
    let text = fs::read_to_string(config_root().join("proxy.tsv")).unwrap_or_default();
    let configured = text.lines().any(|line| !line.trim().is_empty());
    let tor_socks = text.lines().any(|line| {
        let Some((key, value)) = line.split_once('\t') else { return false; };
        key == "socks"
            && (value == "socks5h://127.0.0.1:9050"
                || value == "socks5://127.0.0.1:9050"
                || value == "socks5h://localhost:9050"
                || value == "socks5://localhost:9050")
    });
    (configured, tor_socks)
}

fn runtime_node_backend(monerod: bool, cuprated: bool) -> &'static str {
    match (monerod, cuprated) {
        (true, false) => "monerod",
        (false, true) => "cuprated",
        (true, true) => "multiple",
        (false, false) => "none",
    }
}

pub fn status_json() -> String {
    let tor_active = unit_active("tor.service");
    let zapret_active = unit_active("zapret.service");
    let firewall_active = unit_active("firewall.service");
    let tor_socks = tor_active && listening_on(":9050");
    let dns = dns_summary();
    let resolver_security = resolver_security_summary();
    let mac = mac_policy();
    let vpn_profiles = active_vpn_profiles();
    let vpn_json = vpn_profiles
        .iter()
        .map(|name| format!("\"{}\"", escape(name)))
        .collect::<Vec<_>>()
        .join(",");
    let (proxy_configured, proxy_uses_tor) = proxy_state();

    let mat2 = command_available("mat2");
    let onion_share = command_available("onionshare-cli");
    let onion_share_safe = command_available("onionshare-safe");
    let monerod_installed = command_available("monerod");
    let wallet_cli = command_available("monero-wallet-cli");
    let cuprated_installed = command_available("cuprated");
    let monerod_running = monerod_installed && process_running("monerod");
    let cuprated_running = cuprated_installed && process_running("cuprated");
    let runtime_backend = runtime_node_backend(monerod_running, cuprated_running);

    format!(
        "{{\"tor\":{{\"active\":{},\"socksListening\":{},\"managedBy\":\"nix\",\"mutable\":false}},\"zapret\":{{\"active\":{},\"managedBy\":\"nix\",\"mutable\":false}},\"firewall\":{{\"active\":{},\"managedBy\":\"nixos\",\"mutable\":false}},\"network\":{{\"dns\":\"{}\",\"resolverSecurity\":\"{}\",\"resolverSecurityClaim\":\"observed-resolvectl-status-only\",\"wifiMacPolicy\":\"{}\",\"dnsMutable\":false,\"macMutable\":false,\"vpnProfiles\":[{}],\"vpnActive\":{},\"killSwitchSupported\":false,\"proxyConfigured\":{},\"proxyUsesTorSocks\":{},\"torRoutingClaim\":\"process-proxy-only-when-configured\"}},\"metadataSanitizer\":{{\"available\":{},\"command\":\"mat2\"}},\"onionShare\":{{\"available\":{},\"safeWrapperAvailable\":{}}},\"monero\":{{\"monerodInstalled\":{},\"walletCliInstalled\":{},\"monerodRunning\":{}}},\"cuprate\":{{\"installed\":{},\"running\":{}}},\"node\":{{\"runtimeBackend\":\"{}\",\"selectionManagedByVesper\":false}}}}",
        bool_lit(tor_active),
        bool_lit(tor_socks),
        bool_lit(zapret_active),
        bool_lit(firewall_active),
        escape(&dns),
        escape(&resolver_security),
        escape(&mac),
        vpn_json,
        bool_lit(!vpn_profiles.is_empty()),
        bool_lit(proxy_configured),
        bool_lit(proxy_uses_tor),
        bool_lit(mat2),
        bool_lit(onion_share),
        bool_lit(onion_share_safe),
        bool_lit(monerod_installed),
        bool_lit(wallet_cli),
        bool_lit(monerod_running),
        bool_lit(cuprated_installed),
        bool_lit(cuprated_running),
        escape(runtime_backend),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_node_backend_is_explicit() {
        assert_eq!(runtime_node_backend(false, false), "none");
        assert_eq!(runtime_node_backend(true, false), "monerod");
        assert_eq!(runtime_node_backend(false, true), "cuprated");
        assert_eq!(runtime_node_backend(true, true), "multiple");
    }
}
