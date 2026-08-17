use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn env_path(name: &str, default: &str) -> PathBuf {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    if let Some(rest) = value.strip_prefix("~/") {
        home().join(rest)
    } else {
        PathBuf::from(value)
    }
}

pub fn state_root() -> PathBuf {
    env_path("VESPER_RESEARCH_STATE_DIR", "~/.local/state/vesper/research")
}

pub fn briefing_root() -> PathBuf {
    env_path("VESPER_BRIEFING_DIR", "~/.local/share/vesper/briefings")
}

pub fn skill_draft_root() -> PathBuf {
    env_path("VESPER_SKILL_DRAFT_DIR", "~/.local/share/vesper/skill-drafts")
}

pub fn registry_path() -> PathBuf {
    env_path("VESPER_HERMES_JOB_REGISTRY", "~/.config/vesper/hermes-jobs.json")
}

pub fn hermes_home() -> PathBuf {
    env_path("HERMES_HOME", "~/.hermes")
}

pub fn run(program: &str, args: &[&str], input: Option<&str>) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().map_err(|e| format!("{program}: {e}"))?;
    if let Some(text) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
        return Err(format!("{program} failed: {detail}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_status(program: &str, args: &[&str], input: Option<&str>) -> Result<i32, String> {
    let mut command = Command::new(program);
    command.args(args);
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().map_err(|e| format!("{program}: {e}"))?;
    if let Some(text) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
    }
    Ok(child.wait().map_err(|e| e.to_string())?.code().unwrap_or(1))
}

pub fn output_allow_failure(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .map(|o| {
            let mut text = String::from_utf8_lossy(&o.stdout).to_string();
            if !o.stderr.is_empty() {
                text.push_str(&String::from_utf8_lossy(&o.stderr));
            }
            text
        })
        .unwrap_or_default()
}

pub fn jq(input: &str, filter: &str) -> Result<String, String> {
    run("jq", &["-c", filter], Some(input))
}

pub fn jq_raw(input: &str, filter: &str) -> Result<String, String> {
    run("jq", &["-r", filter], Some(input))
}

pub fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn now_iso() -> String {
    run("date", &["--iso-8601=seconds"], None)
        .unwrap_or_else(|_| "unknown\n".to_string())
        .trim()
        .to_string()
}

pub fn timestamp() -> String {
    run("date", &["+%Y%m%dT%H%M%S"], None)
        .unwrap_or_else(|_| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        })
        .trim()
        .to_string()
}

pub fn date_path() -> String {
    run("date", &["+%Y/%m/%d"], None)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

pub fn atomic_write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_or(path: &Path, default: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| default.to_string())
}

pub fn json_files(root: &Path) -> Vec<PathBuf> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(path) else { return };
        for entry in entries.flatten() {
            let candidate = entry.path();
            if candidate.is_dir() {
                walk(&candidate, out);
            } else if candidate.extension().and_then(|x| x.to_str()) == Some("json")
                && candidate.file_name().and_then(|x| x.to_str()) != Some("index.json")
            {
                out.push(candidate);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

pub fn research_skill() -> String {
    let candidates = [
        home().join(".agents/skills/hermes-research-radar/SKILL.md"),
        hermes_home().join("skills/vesper/hermes-research-radar/SKILL.md"),
    ];
    for path in candidates {
        if let Ok(text) = fs::read_to_string(path) {
            return text;
        }
    }
    "Use Vesper's persistent research contract. Prefer primary evidence, broad exploration and honest coverage.".to_string()
}

pub fn second_brain_skill() -> String {
    let candidates = [
        home().join(".agents/skills/vesper-obsidian-second-brain/SKILL.md"),
        hermes_home().join("skills/vesper/vesper-obsidian-second-brain/SKILL.md"),
    ];
    for path in candidates {
        if let Ok(text) = fs::read_to_string(path) {
            return text;
        }
    }
    String::new()
}

pub fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {} >/dev/null 2>&1", name)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
