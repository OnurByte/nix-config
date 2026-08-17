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

pub fn status_json() -> String {
    let tor_active = unit_active("tor.service");
    let zapret_active = unit_active("zapret.service");
    let firewall_active = unit_active("firewall.service");
    let tor_socks = tor_active && listening_on(":9050");
    let dns = dns_summary();
    let mac = mac_policy();

    format!(
        "{{\"tor\":{{\"active\":{},\"socksListening\":{},\"managedBy\":\"nix\",\"mutable\":false}},\"zapret\":{{\"active\":{},\"managedBy\":\"nix\",\"mutable\":false}},\"firewall\":{{\"active\":{},\"managedBy\":\"nixos\",\"mutable\":false}},\"network\":{{\"dns\":\"{}\",\"wifiMacPolicy\":\"{}\",\"dnsMutable\":false,\"macMutable\":false}}}}",
        bool_lit(tor_active),
        bool_lit(tor_socks),
        bool_lit(zapret_active),
        bool_lit(firewall_active),
        escape(&dns),
        escape(&mac),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_contract_never_claims_declarative_controls_are_mutable() {
        std::env::set_var("VESPER_CMD_SYSTEMCTL", "/bin/false");
        std::env::set_var("VESPER_CMD_RESOLVECTL", "/bin/false");
        std::env::set_var("VESPER_CMD_NMCLI", "/bin/false");
        let json = status_json();
        assert!(json.contains("\"managedBy\":\"nix\""));
        assert!(json.contains("\"mutable\":false"));
    }
}
