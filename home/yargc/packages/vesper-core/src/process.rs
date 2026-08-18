use std::env;
use std::process::{Command, Stdio};

fn override_name(command: &str) -> String {
    let suffix = command
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_uppercase() } else { '_' })
        .collect::<String>();
    format!("VESPER_CMD_{suffix}")
}

pub fn binary(command: &str) -> String {
    env::var(override_name(command)).unwrap_or_else(|_| command.to_string())
}

pub fn output(command: &str, args: &[&str]) -> Result<String, String> {
    let binary = binary(command);
    let result = Command::new(&binary)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run {command}: {error}"))?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{command} exited with {}", result.status.code().unwrap_or(-1))
        } else {
            stderr
        });
    }
    String::from_utf8(result.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|error| format!("invalid UTF-8 from {command}: {error}"))
}

pub fn success(command: &str, args: &[&str]) -> bool {
    let binary = binary(command);
    Command::new(binary)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
