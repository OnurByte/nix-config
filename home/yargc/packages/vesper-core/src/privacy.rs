use std::env;

use crate::json::{bool_lit, escape};
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

fn active_wifi_profile() -> Option<String> {
    let rows = output("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show", "--active"]).ok()?;
    for line in rows.lines() {
        let Some((name, kind)) = line.rsplit_once(':') else { continue; };
        if matches!(kind, "wifi" | "802-11-wireless") && !name.is_empty() {
            return Some(name.replace("\\:", ":"));
        }
    }
    None
}

fn mac_policy() -> String {
    let Some(profile) = active_wifi_profile() else { return String::new(); };
    output(
        "nmcli",
        &["-g", "802-11-wireless.cloned-mac-address", "connection", "show", &profile],
    )
    .unwrap_or_default()
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
    let mac = mac_policy();

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
        "{{\"tor\":{{\"active\":{},\"socksListening\":{},\"managedBy\":\"nix\",\"mutable\":false}},\"zapret\":{{\"active\":{},\"managedBy\":\"nix\",\"mutable\":false}},\"firewall\":{{\"active\":{},\"managedBy\":\"nixos\",\"mutable\":false}},\"network\":{{\"dns\":\"{}\",\"wifiMacPolicy\":\"{}\",\"dnsMutable\":false,\"macMutable\":false}},\"metadataSanitizer\":{{\"available\":{},\"command\":\"mat2\"}},\"onionShare\":{{\"available\":{},\"safeWrapperAvailable\":{}}},\"monero\":{{\"monerodInstalled\":{},\"walletCliInstalled\":{},\"monerodRunning\":{}}},\"cuprate\":{{\"installed\":{},\"running\":{}}},\"node\":{{\"runtimeBackend\":\"{}\",\"selectionManagedByVesper\":false}}}}",
        bool_lit(tor_active),
        bool_lit(tor_socks),
        bool_lit(zapret_active),
        bool_lit(firewall_active),
        escape(&dns),
        escape(&mac),
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
