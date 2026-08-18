use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::json::{bool_lit, escape};
use crate::paths::{config_root, home};

fn canonical_root() -> PathBuf { home().join(".agents/skills") }
fn draft_root() -> PathBuf {
    std::env::var_os("VESPER_SKILL_DRAFT_DIR").map(PathBuf::from).unwrap_or_else(|| home().join(".local/share/vesper/skill-drafts"))
}
fn disabled_root() -> PathBuf { config_root().join("ai/skills-disabled") }
fn agent_roots() -> Vec<PathBuf> { vec![home().join(".codex/skills"), home().join(".claude/skills"), home().join(".config/opencode/skills")] }
fn valid_name(name: &str) -> bool { !name.is_empty() && name.len() <= 80 && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) }
fn runtime_owned(path: &Path) -> bool { path.is_dir() && path.join(".vesper-owned").is_file() }
fn nix_managed(path: &Path) -> bool { fs::read_link(path).ok().map(|target| target.to_string_lossy().starts_with("/nix/store/")).unwrap_or(false) }

fn copy_tree(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
        let source = entry.path();
        let target = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&source).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() { return Err("skill drafts may not contain symlinks".to_string()); }
        if meta.is_dir() { copy_tree(&source, &target)?; }
        else if meta.is_file() {
            if meta.len() > 2_000_000 { return Err("skill draft file exceeds 2 MiB".to_string()); }
            fs::copy(&source, &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn remove_agent_link(root: &Path, name: &str, canonical: &Path) -> Result<(), String> {
    let link = root.join(name);
    if let Ok(target) = fs::read_link(&link) {
        let absolute = if target.is_absolute() { target } else { root.join(target) };
        if absolute == canonical { fs::remove_file(link).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

fn set_links(name: &str, enabled: bool) -> Result<(), String> {
    let canonical = canonical_root().join(name);
    for root in agent_roots() {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let link = root.join(name);
        if enabled {
            if link.exists() || fs::symlink_metadata(&link).is_ok() {
                if fs::read_link(&link).ok().map(|t| if t.is_absolute() { t } else { root.join(t) }) == Some(canonical.clone()) { continue; }
                return Err(format!("agent skill path already owned by another source: {}", link.display()));
            }
            symlink(&canonical, &link).map_err(|e| e.to_string())?;
        } else { remove_agent_link(&root, name, &canonical)?; }
    }
    fs::create_dir_all(disabled_root()).map_err(|e| e.to_string())?;
    let marker = disabled_root().join(name);
    if enabled { let _ = fs::remove_file(marker); } else { fs::write(marker, b"disabled\n").map_err(|e| e.to_string())?; }
    Ok(())
}

pub fn promote(name: &str) -> Result<(), String> {
    if !valid_name(name) { return Err("invalid skill draft name".to_string()); }
    let source = draft_root().join(name);
    if !source.join("SKILL.md").is_file() { return Err("draft must contain SKILL.md".to_string()); }
    let target = canonical_root().join(name);
    if target.exists() || fs::symlink_metadata(&target).is_ok() { return Err("canonical skill name already exists".to_string()); }
    fs::create_dir_all(canonical_root()).map_err(|e| e.to_string())?;
    let staging = canonical_root().join(format!(".{name}.staging-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    copy_tree(&source, &staging)?;
    fs::write(staging.join(".vesper-owned"), b"runtime skill\n").map_err(|e| e.to_string())?;
    fs::rename(&staging, &target).map_err(|e| e.to_string())?;
    if let Err(error) = set_links(name, true) { let _ = fs::remove_dir_all(&target); return Err(error); }
    Ok(())
}

pub fn set_enabled(name: &str, enabled: bool) -> Result<(), String> {
    if !valid_name(name) { return Err("invalid skill name".to_string()); }
    let path = canonical_root().join(name);
    if nix_managed(&path) { return Err("Nix-managed skills are immutable at runtime; change the Nix source instead".to_string()); }
    if !runtime_owned(&path) { return Err("only Vesper runtime-owned skills can be enabled or disabled here".to_string()); }
    set_links(name, enabled)
}

pub fn remove(name: &str) -> Result<(), String> {
    if !valid_name(name) { return Err("invalid skill name".to_string()); }
    let path = canonical_root().join(name);
    if nix_managed(&path) { return Err("Nix-managed skills cannot be removed at runtime".to_string()); }
    if !runtime_owned(&path) { return Err("skill is not Vesper runtime-owned".to_string()); }
    set_links(name, false)?;
    fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(disabled_root().join(name));
    Ok(())
}

pub fn status_json() -> String {
    let mut skills = Vec::new();
    if let Ok(entries) = fs::read_dir(canonical_root()) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            let path = entry.path();
            let ownership = if nix_managed(&path) { "nix" } else if runtime_owned(&path) { "runtime" } else { "external" };
            let enabled = if ownership == "runtime" { !disabled_root().join(&name).exists() } else { true };
            skills.push(format!("{{\"name\":\"{}\",\"ownership\":\"{}\",\"enabled\":{},\"mutable\":{}}}", escape(&name), ownership, bool_lit(enabled), bool_lit(ownership == "runtime")));
        }
    }
    skills.sort();
    let mut drafts = Vec::new();
    if let Ok(entries) = fs::read_dir(draft_root()) {
        for entry in entries.flatten() {
            if entry.path().join("SKILL.md").is_file() { drafts.push(format!("\"{}\"", escape(&entry.file_name().to_string_lossy()))); }
        }
    }
    drafts.sort();
    format!("{{\"skills\":[{}],\"drafts\":[{}]}}", skills.join(","), drafts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn names_are_path_safe() { assert!(valid_name("research-orchestrator")); assert!(!valid_name("../escape")); assert!(!valid_name("a/b")); }
}
