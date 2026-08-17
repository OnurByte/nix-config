use std::path::Path;

use crate::json::{bool_lit, escape};
use crate::process::{output, success};

const JOB: &str = "borgbackup-job-nix-config.service";
const TIMER: &str = "borgbackup-job-nix-config.timer";
const REPOSITORY: &str = "/var/lib/borg-nix-config";

fn property(unit: &str, name: &str) -> String {
    output("systemctl", &["show", unit, "--property", name, "--value"]).unwrap_or_default()
}

pub fn status_json() -> String {
    let timer_active = success("systemctl", &["is-active", "--quiet", TIMER]);
    let job_active = success("systemctl", &["is-active", "--quiet", JOB]);
    let failed = success("systemctl", &["is-failed", "--quiet", JOB]);
    let result = property(JOB, "Result");
    let last_run = property(JOB, "InactiveExitTimestamp");
    let next_run = property(TIMER, "NextElapseUSecRealtime");
    let repo_exists = Path::new(REPOSITORY).exists();

    format!(
        "{{\"backend\":\"borg\",\"repository\":\"{}\",\"repositoryExists\":{},\"managedBy\":\"nixos\",\"mutable\":false,\"timerActive\":{},\"jobActive\":{},\"failed\":{},\"lastResult\":\"{}\",\"lastRun\":\"{}\",\"nextRun\":\"{}\",\"retention\":{{\"daily\":7,\"weekly\":4,\"monthly\":6}},\"restoreAvailableInSettings\":false}}",
        escape(REPOSITORY),
        bool_lit(repo_exists),
        bool_lit(timer_active),
        bool_lit(job_active),
        bool_lit(failed),
        escape(&result),
        escape(&last_run),
        escape(&next_run),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_does_not_expose_destructive_restore() {
        let json = status_json();
        assert!(json.contains("\"restoreAvailableInSettings\":false"));
        assert!(json.contains("\"mutable\":false"));
    }
}
