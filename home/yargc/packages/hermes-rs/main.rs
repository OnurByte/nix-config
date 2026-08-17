mod cron;
mod prompts;
mod state;
mod util;

use std::{env, fs, path::Path, process::Command};

use cron::{dispatch, job_for, sync_cron, tor_fetch, validate_registry, watch};
use prompts::{adhoc_contract, research_contract, ALL_TASKS, FRONTIER_TASKS};
use state::{
    coverage_summary, frontier_context, list_json, mark_all_read, mark_read, read_report, rebuild_index,
    recent_briefings, save_report, source_records, source_registry_text, status_json, task_context,
};
use util::{
    hermes_home, jq, jq_raw, json_string, now_iso, output_allow_failure, research_skill, run,
    run_status, second_brain_skill, state_root,
};

fn extract_json_object(text: &str) -> Result<String, String> {
    if let Ok(value) = jq(text, "select(type == \"object\" and .title and has(\"summary\"))") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }

    let bytes = text.as_bytes();
    for start in 0..bytes.len() {
        if bytes[start] != b'{' {
            continue;
        }
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;
        for end in start..bytes.len() {
            let byte = bytes[end];
            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_string = false;
                }
                continue;
            }
            match byte {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        let candidate = &text[start..=end];
                        if let Ok(value) = jq(candidate, "select(type == \"object\" and .title and has(\"summary\"))") {
                            if !value.trim().is_empty() {
                                return Ok(value);
                            }
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    Err("Hermes did not return a valid report JSON object".to_string())
}

fn invoke_agent(prompt: &str, web_only: bool) -> Result<String, String> {
    let provider = env::var("HERMES_RESEARCH_PROVIDER").unwrap_or_else(|_| "xai-oauth".to_string());
    let model = env::var("HERMES_RESEARCH_MODEL").unwrap_or_else(|_| "grok-4.5".to_string());
    let mut args = vec!["-z", prompt, "--provider", &provider, "-m", &model, "--yolo"];
    if web_only {
        args.extend(["-t", "web"]);
    }
    let output = run("hermes", &args, None)?;
    extract_json_object(&output)
}

fn task_extra(task: &str) -> String {
    match task {
        "unknown-frontier-github" => "Coverage budget: target about 180 distinct canonical candidates and about 15 strong deep reads. Expand outside the known repository map.".to_string(),
        "unknown-frontier-reddit" => format!(
            "Coverage budget: target about 150 distinct canonical candidates and about 12 strong deep reads. Reddit seed communities from VESPER_REDDIT_SEEDS/VESPER_REDDIT_COMMENT_SEEDS are bootstrap hints only.\nAdaptive source state:\n{}",
            source_registry_text()
        ),
        "unknown-frontier-x" => format!(
            "Coverage budget: target about 150 distinct canonical candidates and about 12 strong deep reads. Direct X is preferred when usable; XCancel/Nitter-compatible mirrors are transport fallbacks, not independent sources.\nAdaptive source state:\n{}",
            source_registry_text()
        ),
        "unknown-frontier-web" => format!(
            "Coverage budget: target about 120 distinct canonical candidates and about 9 strong deep reads. For .onion content use shell access to `vesper-hermes-automations tor-fetch URL` so access goes through the local Tor SOCKS proxy. Never claim a normal clearnet web tool reached an onion.\nAdaptive source state:\n{}",
            source_registry_text()
        ),
        "unknown-frontier-synthesis" => {
            let coverage = coverage_summary();
            format!(
                "Fresh scout envelopes:\n{}\n\nMeasured scout coverage summary: {:?}. Daily candidate target is 600 and deep-read target is 48. Explicitly report shortfall instead of inventing counts.",
                frontier_context(120_000), coverage
            )
        }
        "morning-check" => format!(
            "Recent durable Hermes briefings:\n{}\n\nInspect local Git/project/todo state with shell tools where useful. Return the final Telegram-ready message in the report body.",
            recent_briefings(55_000)
        ),
        "second-brain-dream" => format!(
            "Second-brain skill:\n{}\n\nRecent durable Hermes research:\n{}\n\nSkill drafts belong under {}. Do not invent an Obsidian vault if none exists.",
            second_brain_skill(),
            recent_briefings(80_000),
            util::skill_draft_root().display()
        ),
        "project-archaeologist" => "Inspect only bounded local roots such as ~/Documents, ~/Projects, ~/Code and ~/src. Use Git status/log/branches directly; ignore node_modules, target, vendor, .cache, .direnv and virtual environments.".to_string(),
        "ai-usage-economist" => format!(
            "Local accounting snapshots (commands may be unavailable):\nccusage:\n{}\nCodexBar:\n{}\nTurnLens help:\n{}",
            output_allow_failure("ccusage", &["--json"]),
            output_allow_failure("codexbar-status", &[]),
            output_allow_failure("turnlens", &["--help"])
        ),
        _ => String::new(),
    }
}

fn is_web_only(task: &str) -> bool {
    !matches!(
        task,
        "project-archaeologist" | "ai-usage-economist" | "second-brain-dream" | "morning-check"
    )
}

fn run_single_task(task: &str) -> Result<String, String> {
    if task == "frontier-daily" {
        for scout in FRONTIER_TASKS {
            run_single_task(scout)?;
        }
        return run_single_task("unknown-frontier-synthesis");
    }
    if !ALL_TASKS.contains(&task) || matches!(task, "vesper-health-watch" | "cron-skill-integrity-watch") {
        return Err(format!("unknown research task: {task}"));
    }
    let durable = task_context(task, 42_000);
    let prompt = research_contract(task, &research_skill(), &durable, &task_extra(task));
    let raw = invoke_agent(&prompt, is_web_only(task))?;
    let report = save_report(task, &raw)?;

    if task == "morning-check" {
        let body = jq_raw(&report, ".body // .summary // \"\"")?;
        if body.trim().len() < 40 {
            return Err("Morning Check output too short".to_string());
        }
        if run_status("hermes", &["send", "--to", "telegram", "--quiet"], Some(&body))? != 0 {
            return Err("Telegram delivery failed".to_string());
        }
    }
    Ok(report)
}

fn record_run(task: &str, status: &str, started: &str, error: &str) -> Result<(), String> {
    let finished = now_iso();
    let payload = format!(
        "{{\"job\":{},\"status\":{},\"startedAt\":{},\"finishedAt\":{},\"error\":{}}}",
        json_string(task), json_string(status), json_string(started), json_string(&finished), json_string(error)
    );
    let root = state_root().join("runs").join(task);
    util::atomic_write(&root.join(format!("{}.json", util::timestamp())), &payload)?;
    util::atomic_write(&root.join("latest.json"), &payload)
}

fn execute(task: &str) -> Result<String, String> {
    let started = now_iso();
    match run_single_task(task) {
        Ok(report) => {
            record_run(task, "ok", &started, "")?;
            Ok(report)
        }
        Err(error) => {
            let _ = record_run(task, "error", &started, &error);
            let _ = run_status(
                "notify-send",
                &["-a", "Hermes", "Hermes automation failed", &format!("{task}: {error}")],
                None,
            );
            Err(error)
        }
    }
}

fn automation_cli(args: &[String]) -> Result<i32, String> {
    let command = args.first().map(String::as_str).unwrap_or("jobs");
    match command {
        "jobs" => {
            let text = fs::read_to_string(util::registry_path()).map_err(|e| e.to_string())?;
            print!("{}", run("jq", &["."], Some(&text))?);
        }
        "validate-registry" => {
            let count = validate_registry()?;
            println!("ok: {count} declarative Hermes jobs");
        }
        "sync-cron" => {
            let prune = args.iter().any(|arg| arg == "--prune");
            println!("{}", sync_cron(prune)?);
        }
        "dispatch" => {
            let task = args.get(1).ok_or("dispatch requires task")?;
            dispatch(task)?;
        }
        "execute" => {
            let task = args.get(1).ok_or("execute requires task")?;
            println!("{}", execute(task)?);
        }
        "watch" => {
            let task = args.get(1).ok_or("watch requires task")?;
            let output = watch(task)?;
            if !output.is_empty() {
                println!("{output}");
            }
        }
        "trigger" => {
            let name = args.get(1).ok_or("trigger requires job")?;
            let job = job_for(name)?;
            if job.mode == "watchdog" {
                let output = watch(&job.task)?;
                if !output.is_empty() {
                    println!("{output}");
                }
            } else {
                dispatch(&job.task)?;
            }
        }
        "links" => {
            println!("{}", source_records()?);
        }
        "tor-fetch" => {
            let url = args.get(1).ok_or("tor-fetch requires URL")?;
            let mut max_chars = 50_000usize;
            if let Some(index) = args.iter().position(|arg| arg == "--max-chars") {
                if let Some(value) = args.get(index + 1) {
                    max_chars = value.parse::<usize>().unwrap_or(max_chars).clamp(1, 250_000);
                }
            }
            println!("{}", tor_fetch(url, max_chars)?);
        }
        _ => return Err(format!("unknown automation command: {command}")),
    }
    Ok(0)
}

fn runtime_cli(args: &[String]) -> Result<i32, String> {
    let command = args.first().map(String::as_str).unwrap_or("status");
    match command {
        "run" => {
            let lane = args.get(1).ok_or("run requires lane/task")?;
            let task = if lane == "unknown-frontier-ai" { "frontier-daily" } else { lane };
            println!("{}", execute(task)?);
        }
        "daily" => {
            for task in ["frontier-daily", "free-ai-radar", "agenda"] {
                execute(task)?;
            }
            println!("{}", status_json()?);
        }
        "status" => {
            let value = status_json()?;
            if args.iter().any(|arg| arg == "--json") {
                println!("{value}");
            } else {
                println!(
                    "{}",
                    jq_raw(&value, r#""Hermes · \(.unread) unread · \(.high) high priority · \(.count) total\nlatest: \(.latestTitle)""#)?
                );
            }
        }
        "list" => {
            let value = list_json()?;
            if args.iter().any(|arg| arg == "--json") {
                println!("{value}");
            } else {
                println!(
                    "{}",
                    jq_raw(&value, r#".[] | (if .unread then "●" else "○" end) + " " + (.id // "") + " · " + (.priority // "normal") + " · " + (.title // "Untitled")"#)?
                );
            }
        }
        "read" => println!("{}", read_report(args.get(1).ok_or("read requires id")?)?),
        "mark-read" => {
            mark_read(args.get(1).ok_or("mark-read requires id")?)?;
        }
        "mark-all-read" => mark_all_read()?,
        "reindex" => {
            rebuild_index()?;
        }
        "tui" => {
            println!("VESPER · HERMES BRIEFINGS\n");
            let value = list_json()?;
            println!(
                "{}",
                jq_raw(&value, r#".[0:30][] | (if .unread then "●" else "○" end) + " " + ((.priority // "normal") | ascii_upcase) + " · " + (.lane // "unknown") + "\n  " + (.title // "Untitled") + "\n  " + (.summary // "") + "\n  id: " + (.id // "") + "\n""#)?
            );
        }
        "inbox" => {
            return Ok(run_status("ghostty", &["--class=vesper-hermes-inbox", "-e", "vesper-hermes", "tui"], None)?);
        }
        _ => return Err(format!("unknown runtime command: {command}")),
    }
    Ok(0)
}

fn research_cli(args: &[String]) -> Result<i32, String> {
    if args.first().map(String::as_str) == Some("sources") {
        println!("{}", source_records()?);
        return Ok(0);
    }
    let mut index = 0usize;
    if args.first().map(String::as_str) == Some("run") {
        index = 1;
    }
    let query = args.get(index).ok_or("usage: vesper-research [run] \"query\" [--pages N] [--deep-reads N]")?;
    let value_after = |flag: &str| -> Option<&String> {
        args.iter().position(|arg| arg == flag).and_then(|i| args.get(i + 1))
    };
    let pages = value_after("--pages").and_then(|v| v.parse::<usize>().ok()).unwrap_or(600).clamp(50, 2000);
    let deep_reads = value_after("--deep-reads")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| (pages / 12).clamp(12, 80))
        .clamp(1, 120);
    let prompt = adhoc_contract(query, pages, deep_reads, &research_skill(), &source_registry_text());
    let raw = invoke_agent(&prompt, false)?;
    println!("{}", save_report("adhoc-research", &raw)?);
    Ok(0)
}

fn main() {
    let all: Vec<String> = env::args().collect();
    let invoked = Path::new(all.first().map(String::as_str).unwrap_or("vesper-hermes"))
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("vesper-hermes");
    let args = &all[1..];
    let result = if invoked.contains("automations") {
        automation_cli(args)
    } else if invoked.contains("vesper-research") {
        research_cli(args)
    } else {
        runtime_cli(args)
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
