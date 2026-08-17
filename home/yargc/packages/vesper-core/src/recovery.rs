use crate::json::{bool_lit, escape};
use crate::process::{output, success};

const BACKUP_JOB: &str = "vesper-backup.service";
const BACKUP_TIMER: &str = "vesper-backup.timer";
const CHECK_JOB: &str = "vesper-backup-check.service";
const CHECK_TIMER: &str = "vesper-backup-check.timer";

fn property(unit: &str, name: &str) -> String {
    output("systemctl", &["show", unit, "--property", name, "--value"]).unwrap_or_default()
}

fn unit_active(unit: &str) -> bool {
    success("systemctl", &["is-active", "--quiet", unit])
}

fn unit_failed(unit: &str) -> bool {
    success("systemctl", &["is-failed", "--quiet", unit])
}

fn snapper_summary(config: &str) -> String {
    output("snapper", &["-c", config, "list"])
        .map(|text| {
            let mut rows = text.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<_>>();
            if rows.len() > 5 {
                rows = rows.split_off(rows.len() - 5);
            }
            rows.join(" · ")
        })
        .unwrap_or_default()
}

fn scrub_timer_summary() -> String {
    output(
        "systemctl",
        &["list-timers", "--all", "--no-legend", "--plain", "btrfs-scrub*.timer"],
    )
    .map(|text| text.lines().take(4).collect::<Vec<_>>().join(" · "))
    .unwrap_or_default()
}

fn scrub_result_summary() -> String {
    output(
        "systemctl",
        &["list-units", "--all", "--no-legend", "--plain", "btrfs-scrub*.service"],
    )
    .map(|text| text.lines().take(4).collect::<Vec<_>>().join(" · "))
    .unwrap_or_default()
}

pub fn run_repository_check() -> Result<(), String> {
    let load_state = property(CHECK_JOB, "LoadState");
    if load_state != "loaded" {
        return Err("Restic repository check service is not installed/loaded".to_string());
    }
    if unit_active(CHECK_JOB) {
        return Ok(());
    }
    output("systemctl", &["start", CHECK_JOB]).map(|_| ())
}

pub fn status_json() -> String {
    let backup_timer_active = unit_active(BACKUP_TIMER);
    let backup_job_active = unit_active(BACKUP_JOB);
    let backup_failed = unit_failed(BACKUP_JOB);
    let backup_result = property(BACKUP_JOB, "Result");
    let backup_last_run = property(BACKUP_JOB, "InactiveExitTimestamp");
    let backup_next_run = property(BACKUP_TIMER, "NextElapseUSecRealtime");
    let backup_condition = property(BACKUP_JOB, "ConditionResult");

    let check_load_state = property(CHECK_JOB, "LoadState");
    let check_timer_active = unit_active(CHECK_TIMER);
    let check_job_active = unit_active(CHECK_JOB);
    let check_failed = unit_failed(CHECK_JOB);
    let check_result = property(CHECK_JOB, "Result");
    let check_last_run = property(CHECK_JOB, "InactiveExitTimestamp");
    let check_next_run = property(CHECK_TIMER, "NextElapseUSecRealtime");

    let root_snapshots = snapper_summary("root");
    let home_snapshots = snapper_summary("home");
    let scrub_next = scrub_timer_summary();
    let scrub_result = scrub_result_summary();
    let restore_ready = backup_result == "success" && check_result == "success" && !backup_failed && !check_failed;
    let safe_check_available = check_load_state == "loaded";

    format!(
        "{{\"backend\":\"restic\",\"managedBy\":\"nixos\",\"mutable\":false,\"backup\":{{\"timerActive\":{},\"jobActive\":{},\"failed\":{},\"conditionResult\":\"{}\",\"lastResult\":\"{}\",\"lastRun\":\"{}\",\"nextRun\":\"{}\"}},\"repositoryCheck\":{{\"timerActive\":{},\"jobActive\":{},\"failed\":{},\"lastResult\":\"{}\",\"lastRun\":\"{}\",\"nextRun\":\"{}\"}},\"snapper\":{{\"root\":\"{}\",\"home\":\"{}\"}},\"btrfsScrub\":{{\"next\":\"{}\",\"result\":\"{}\"}},\"retention\":{{\"daily\":7,\"weekly\":4,\"monthly\":12}},\"restoreReady\":{},\"restoreAvailableInSettings\":false,\"safeCheckActionAvailableInSettings\":{}}}",
        bool_lit(backup_timer_active),
        bool_lit(backup_job_active),
        bool_lit(backup_failed),
        escape(&backup_condition),
        escape(&backup_result),
        escape(&backup_last_run),
        escape(&backup_next_run),
        bool_lit(check_timer_active),
        bool_lit(check_job_active),
        bool_lit(check_failed),
        escape(&check_result),
        escape(&check_last_run),
        escape(&check_next_run),
        escape(&root_snapshots),
        escape(&home_snapshots),
        escape(&scrub_next),
        escape(&scrub_result),
        bool_lit(restore_ready),
        bool_lit(safe_check_available),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_does_not_expose_destructive_restore() {
        let json = status_json();
        assert!(json.contains("\"backend\":\"restic\""));
        assert!(json.contains("\"restoreAvailableInSettings\":false"));
        assert!(json.contains("\"mutable\":false"));
    }
}
