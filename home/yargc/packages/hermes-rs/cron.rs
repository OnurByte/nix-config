use std::{collections::HashSet, fs, process::Command};

use crate::{
    prompts::ALL_TASKS,
    util::{
        command_exists, hermes_home, jq, jq_raw, output_allow_failure, read_or, registry_path, run,
        run_status,
    },
};

#[derive(Clone, Debug)]
pub struct Job {
    pub name: String,
    pub schedule: String,
    pub mode: String,
    pub task: String,
    pub enabled: bool,
    pub deliver: String,
    pub cron_name: String,
    pub script: String,
}

pub fn jobs() -> Result<Vec<Job>, String> {
    let text = fs::read_to_string(registry_path()).map_err(|e| format!("job registry: {e}"))?;
    let rows = jq_raw(
        &text,
        r#"to_entries[] | [
            .key,
            (.value.schedule // ""),
            (.value.mode // "dispatch"),
            (.value.task // .key),
            ((.value.enabled // true) | tostring),
            (.value.deliver // "local"),
            (.value.cronName // ("vesper:" + .key)),
            (.value.script // ("vesper-" + .key + ".sh"))
        ] | @tsv"#,
    )?;
    let mut out = Vec::new();
    for line in rows.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 8 {
            return Err(format!("invalid registry row: {line}"));
        }
        out.push(Job {
            name: fields[0].to_string(),
            schedule: fields[1].to_string(),
            mode: fields[2].to_string(),
            task: fields[3].to_string(),
            enabled: fields[4] == "true",
            deliver: fields[5].to_string(),
            cron_name: fields[6].to_string(),
            script: fields[7].to_string(),
        });
    }
    Ok(out)
}

pub fn validate_registry() -> Result<usize, String> {
    let jobs = jobs()?;
    if jobs.is_empty() {
        return Err("registry is empty".to_string());
    }
    let mut errors = Vec::new();
    let mut cron_names = HashSet::new();
    let mut scripts = HashSet::new();
    for job in &jobs {
        if job.schedule.split_whitespace().count() != 5 {
            errors.push(format!("{}: schedule must have 5 cron fields", job.name));
        }
        if !ALL_TASKS.contains(&job.task.as_str()) {
            errors.push(format!("{}: unknown task {}", job.name, job.task));
        }
        match job.mode.as_str() {
            "dispatch" => {
                if job.deliver != "local" {
                    errors.push(format!("{}: dispatch delivery must be local", job.name));
                }
            }
            "watchdog" => {
                if job.deliver == "local" {
                    errors.push(format!("{}: watchdog needs a non-local alert target", job.name));
                }
            }
            other => errors.push(format!("{}: unsupported mode {other}", job.name)),
        }
        if !cron_names.insert(job.cron_name.clone()) {
            errors.push(format!("{}: duplicate cron name {}", job.name, job.cron_name));
        }
        if !scripts.insert(job.script.clone()) {
            errors.push(format!("{}: duplicate script {}", job.name, job.script));
        }
    }
    if errors.is_empty() {
        Ok(jobs.len())
    } else {
        Err(errors.join("\n"))
    }
}

fn jobs_store() -> String {
    read_or(&hermes_home().join("cron/jobs.json"), "[]")
}

fn existing_ref(store: &str, cron_name: &str) -> String {
    let escaped = crate::util::json_string(cron_name);
    jq_raw(
        store,
        &format!("(.jobs? // . // [])[]? | select((.name // \"\") == {escaped}) | (.id // .name)"),
    )
    .unwrap_or_default()
    .lines()
    .next()
    .unwrap_or("")
    .trim()
    .to_string()
}

fn legacy_morning_ref(store: &str) -> String {
    jq_raw(
        store,
        r#"(.jobs? // . // [])[]?
          | select(
              (((.name // "") | ascii_downcase) as $n
                | ($n == "sabah check" or $n == "sabah-check" or $n == "morning check" or $n == "morning-check"))
              or ((.script // "") | endswith("sabah-check-deliver.sh"))
              or ((.script // "") | endswith("morning-check-deliver.sh"))
            )
          | (.id // .name)"#,
    )
    .unwrap_or_default()
    .lines()
    .next()
    .unwrap_or("")
    .trim()
    .to_string()
}

fn record_by_ref(store: &str, reference: &str) -> String {
    if reference.is_empty() {
        return "{}".to_string();
    }
    let escaped = crate::util::json_string(reference);
    jq(
        store,
        &format!(
            "first((.jobs? // . // [])[]? | select((.id // \"\") == {escaped} or (.name // \"\") == {escaped})) // {{}}"
        ),
    )
    .unwrap_or_else(|_| "{}".to_string())
}

fn record_enabled(record: &str) -> bool {
    jq_raw(record, ".enabled // true").unwrap_or_else(|_| "true".to_string()).trim() == "true"
}

pub fn sync_cron(prune: bool) -> Result<String, String> {
    validate_registry()?;
    let desired = jobs()?;
    let store = jobs_store();
    let mut created = Vec::new();
    let mut updated = Vec::new();
    let mut resumed = Vec::new();
    let mut paused = Vec::new();
    let mut removed = Vec::new();

    for job in &desired {
        let script_path = hermes_home().join("scripts").join(&job.script);
        if !script_path.is_file() {
            return Err(format!("{}: script missing at {}", job.name, script_path.display()));
        }
        let script = script_path.to_string_lossy().to_string();
        let mut reference = existing_ref(&store, &job.cron_name);
        if reference.is_empty() && job.name == "morning-check" {
            reference = legacy_morning_ref(&store);
        }
        let existing = record_by_ref(&store, &reference);
        let prompt = "Run the declarative Vesper Hermes trigger.";

        if reference.is_empty() {
            let code = run_status(
                "hermes",
                &[
                    "cron", "create", &job.schedule, prompt, "--name", &job.cron_name, "--deliver",
                    &job.deliver, "--script", &script, "--no-agent",
                ],
                None,
            )?;
            if code != 0 {
                return Err(format!("{}: Hermes cron create failed", job.name));
            }
            created.push(job.name.clone());
        } else {
            let code = run_status(
                "hermes",
                &[
                    "cron", "edit", &reference, "--name", &job.cron_name, "--schedule", &job.schedule,
                    "--prompt", prompt, "--deliver", &job.deliver, "--script", &script, "--no-agent",
                ],
                None,
            )?;
            if code != 0 {
                return Err(format!("{}: Hermes cron edit failed", job.name));
            }
            updated.push(job.name.clone());
        }

        let currently = if reference.is_empty() { true } else { record_enabled(&existing) };
        if job.enabled != currently {
            let action = if job.enabled { "resume" } else { "pause" };
            let target = if reference.is_empty() { &job.cron_name } else { &reference };
            if run_status("hermes", &["cron", action, target], None)? != 0 {
                return Err(format!("{}: Hermes cron {action} failed", job.name));
            }
            if job.enabled {
                resumed.push(job.name.clone());
            } else {
                paused.push(job.name.clone());
            }
        }
    }

    if prune {
        let wanted: HashSet<String> = desired.iter().map(|job| job.cron_name.clone()).collect();
        let names = jq_raw(&store, r#"(.jobs? // . // [])[]? | .name // empty"#).unwrap_or_default();
        for name in names.lines().map(str::trim).filter(|name| name.starts_with("vesper:")) {
            if wanted.contains(name) {
                continue;
            }
            let reference = existing_ref(&store, name);
            let target = if reference.is_empty() { name } else { &reference };
            if run_status("hermes", &["cron", "remove", target], None)? == 0 {
                removed.push(name.to_string());
            }
        }
    }

    Ok(format!(
        "{{\"ok\":true,\"created\":{},\"updated\":{},\"resumed\":{},\"paused\":{},\"removed\":{}}}",
        json_array(&created), json_array(&updated), json_array(&resumed), json_array(&paused), json_array(&removed)
    ))
}

fn json_array(items: &[String]) -> String {
    format!(
        "[{}]",
        items.iter().map(|item| crate::util::json_string(item)).collect::<Vec<_>>().join(",")
    )
}

fn unit_name(task: &str) -> String {
    let safe = task
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>();
    format!("vesper-hermes-{safe}")
}

pub fn dispatch(task: &str) -> Result<(), String> {
    let unit = unit_name(task);
    let service = format!("{unit}.service");
    if run_status("systemctl", &["--user", "is-active", "--quiet", &service], None).unwrap_or(1) == 0 {
        return Ok(());
    }

    let unit_arg = format!("--unit={unit}");
    let code = run_status(
        "systemd-run",
        &[
            "--user", "--no-block", "--collect", "--quiet", &unit_arg, "--property=Nice=10",
            "--property=IOSchedulingClass=best-effort", "--property=KillMode=mixed",
            "vesper-hermes-automations", "execute", task,
        ],
        None,
    )?;
    if code == 0 {
        return Ok(());
    }
    if run_status("systemctl", &["--user", "is-active", "--quiet", &service], None).unwrap_or(1) == 0 {
        return Ok(());
    }
    Err(format!("failed to dispatch {task}"))
}

fn failed_units(scope: &str) -> String {
    if scope == "user" {
        output_allow_failure("systemctl", &["--user", "--failed", "--no-legend", "--plain"])
    } else {
        output_allow_failure("systemctl", &["--failed", "--no-legend", "--plain"])
    }
}

fn disk_percent() -> u32 {
    let output = output_allow_failure("df", &["-P", "/"]);
    output
        .lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().nth(4))
        .and_then(|value| value.trim_end_matches('%').parse::<u32>().ok())
        .unwrap_or(0)
}

pub fn health_watch() -> String {
    let mut problems = Vec::new();
    if command_exists("vesper-doctor") {
        let output = Command::new("vesper-doctor").arg("--json").output();
        match output {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                if jq_raw(&text, ".healthy // false").unwrap_or_default().trim() != "true" {
                    problems.push("vesper-doctor reports unhealthy state".to_string());
                }
            }
            _ => problems.push("vesper-doctor failed".to_string()),
        }
    } else {
        problems.push("vesper-doctor is not available".to_string());
    }
    for scope in ["user", "system"] {
        let units = failed_units(scope);
        let meaningful = units
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.contains("0 loaded units listed"))
            .take(8)
            .collect::<Vec<_>>();
        if !meaningful.is_empty() {
            problems.push(format!("{scope} failed units: {}", meaningful.join(" | ")));
        }
    }
    let threshold = std::env::var("VESPER_DISK_ALERT_PERCENT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(90)
        .clamp(1, 99);
    let used = disk_percent();
    if used >= threshold {
        problems.push(format!("disk /: {used}% used (threshold {threshold}%)"));
    }
    if problems.is_empty() {
        String::new()
    } else {
        format!(
            "[Hermes health]\n{}",
            problems.into_iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n")
        )
    }
}

pub fn cron_integrity_watch() -> String {
    let mut problems = Vec::new();
    if let Err(error) = validate_registry() {
        problems.push(error);
    }
    let store = jobs_store();
    for job in jobs().unwrap_or_default() {
        let reference = existing_ref(&store, &job.cron_name);
        if reference.is_empty() {
            problems.push(format!("missing job {}", job.cron_name));
            continue;
        }
        let record = record_by_ref(&store, &reference);
        if record_enabled(&record) != job.enabled {
            problems.push(format!("enabled-state drift {}", job.cron_name));
        }
        let actual_script = jq_raw(&record, ".script // \"\"").unwrap_or_default();
        if !actual_script.trim().ends_with(&job.script) {
            problems.push(format!("script drift {}", job.cron_name));
        }
        let no_agent = jq_raw(&record, ".no_agent // .noAgent // false").unwrap_or_default();
        if no_agent.trim() != "true" {
            problems.push(format!("mode drift {}: expected no_agent=true", job.cron_name));
        }
        let script_path = hermes_home().join("scripts").join(&job.script);
        if !script_path.is_file() {
            problems.push(format!("missing script {}", script_path.display()));
        }
    }
    let status = output_allow_failure("hermes", &["cron", "status"]);
    if status.contains("will NOT fire") || status.contains("STALLED") {
        problems.push("Hermes cron scheduler/gateway is unhealthy".to_string());
    }
    if problems.is_empty() {
        String::new()
    } else {
        format!(
            "[Hermes cron integrity]\n{}",
            problems.into_iter().take(20).map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n")
        )
    }
}

pub fn watch(task: &str) -> Result<String, String> {
    match task {
        "vesper-health-watch" => Ok(health_watch()),
        "cron-skill-integrity-watch" | "cron-integrity-watch" => Ok(cron_integrity_watch()),
        _ => Err(format!("unknown watchdog: {task}")),
    }
}

fn onion_host(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else { return false };
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let authority = rest.split('/').next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or("");
    let host = host_port.split(':').next().unwrap_or("").trim_end_matches('.').to_ascii_lowercase();
    host.ends_with(".onion")
}

pub fn tor_fetch(url: &str, max_chars: usize) -> Result<String, String> {
    if !onion_host(url) {
        return Err("tor-fetch only accepts http(s) .onion URLs".to_string());
    }
    let output = run(
        "curl",
        &[
            "--fail", "--location", "--silent", "--show-error", "--max-time", "45",
            "--socks5-hostname", "127.0.0.1:9050", "--user-agent", "Mozilla/5.0 Vesper-Hermes/2", url,
        ],
        None,
    )?;
    let mut text = output;
    if text.len() > max_chars {
        text.truncate(max_chars);
    }
    Ok(format!(
        "{{\"url\":{},\"transport\":\"tor-socks5\",\"chars\":{},\"content\":{}}}",
        crate::util::json_string(url),
        text.len(),
        crate::util::json_string(&text)
    ))
}

pub fn job_for(name: &str) -> Result<Job, String> {
    jobs()?
        .into_iter()
        .find(|job| job.name == name)
        .ok_or_else(|| format!("unknown Hermes job: {name}"))
}
