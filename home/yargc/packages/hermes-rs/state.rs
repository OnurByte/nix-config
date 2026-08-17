use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::util::{
    atomic_write, briefing_root, date_path, jq, jq_raw, json_files, json_string, now_iso, read_or,
    state_root, timestamp,
};

fn report_array() -> Result<String, String> {
    let mut rows: Vec<(String, String)> = Vec::new();
    for path in json_files(&briefing_root()) {
        let Ok(text) = fs::read_to_string(&path) else { continue };
        if jq(&text, "select(type == \"object\" and .id)").is_err() {
            continue;
        }
        let markdown = path.with_extension("md");
        let markdown_text = if markdown.exists() {
            markdown.to_string_lossy().into_owned()
        } else {
            String::new()
        };
        let filter = format!(
            ". + {{_jsonPath:{},_markdownPath:{}}}",
            json_string(&path.to_string_lossy()),
            json_string(&markdown_text)
        );
        let value = jq(&text, &filter)?;
        let created = jq_raw(&value, ".createdAt // \"\"")?;
        rows.push((created.trim().to_string(), value.trim().to_string()));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    jq(&format!("[{}]", rows.into_iter().map(|(_, v)| v).collect::<Vec<_>>().join(",")), ".")
}

pub fn rebuild_index() -> Result<String, String> {
    let array = report_array()?;
    atomic_write(&briefing_root().join("index.json"), &array)?;
    Ok(array)
}

pub fn status_json() -> Result<String, String> {
    let reports = rebuild_index()?;
    jq(
        &reports,
        r#"{
            count:length,
            unread:([.[] | select(.unread == true)] | length),
            high:([.[] | select(.unread == true and ((.priority // "normal") == "high" or (.priority // "normal") == "critical"))] | length),
            latestTitle:(.[0].title // "No briefings yet"),
            latestLane:(.[0].lane // ""),
            class:(if ([.[] | select(.unread == true and ((.priority // "normal") == "high" or (.priority // "normal") == "critical"))] | length) > 0 then "attention" elif ([.[] | select(.unread == true)] | length) > 0 then "unread" else "idle" end)
        }"#,
    )
}

pub fn list_json() -> Result<String, String> {
    rebuild_index()
}

fn find_report(id: &str) -> Result<(PathBuf, String), String> {
    for path in json_files(&briefing_root()) {
        let Ok(text) = fs::read_to_string(&path) else { continue };
        if jq_raw(&text, ".id // \"\"").unwrap_or_default().trim() == id {
            return Ok((path, text));
        }
    }
    Err(format!("unknown briefing id: {id}"))
}

pub fn mark_read(id: &str) -> Result<String, String> {
    let (path, text) = find_report(id)?;
    let updated = jq(&text, ".unread = false")?;
    atomic_write(&path, &updated)?;
    rebuild_index()?;
    Ok(updated)
}

pub fn mark_all_read() -> Result<(), String> {
    for path in json_files(&briefing_root()) {
        let Ok(text) = fs::read_to_string(&path) else { continue };
        if jq_raw(&text, ".unread // false").unwrap_or_default().trim() != "true" {
            continue;
        }
        let updated = jq(&text, ".unread = false")?;
        atomic_write(&path, &updated)?;
    }
    rebuild_index()?;
    Ok(())
}

pub fn report_text(report: &str) -> Result<String, String> {
    jq_raw(
        report,
        r#""# \(.title // "Untitled")\n\nlane: \(.lane // "unknown") · priority: \(.priority // "normal") · confidence: \(.confidence // "unknown")\n\n\(.summary // "")\n\n\(.body // "")\n\nSources:\n\((.sources // []) | map(if type == "string" then . else ((.title // .url // "source") + (if .url then " — " + .url else "" end)) end) | map("- " + .) | join("\n"))""#,
    )
}

pub fn read_report(id: &str) -> Result<String, String> {
    let report = mark_read(id)?;
    report_text(&report)
}

pub fn task_context(task: &str, max_chars: usize) -> String {
    let path = state_root().join(task).join("latest.json");
    let mut text = read_or(&path, "");
    if text.len() > max_chars {
        text.truncate(max_chars);
    }
    text
}

pub fn frontier_context(max_chars: usize) -> String {
    let mut out = String::new();
    for task in crate::prompts::FRONTIER_TASKS {
        out.push_str(&format!("\n--- {task} ---\n{}\n", task_context(task, max_chars / 4)));
    }
    if out.len() > max_chars {
        out.truncate(max_chars);
    }
    out
}

pub fn recent_briefings(max_chars: usize) -> String {
    let reports = rebuild_index().unwrap_or_else(|_| "[]".to_string());
    let paths = jq_raw(
        &reports,
        r#".[0:20][] | select((._markdownPath // "") != "") | ._markdownPath"#,
    )
    .unwrap_or_default();
    let mut out = String::new();
    for path in paths.lines() {
        if let Ok(text) = fs::read_to_string(path) {
            out.push_str("\n--- briefing ---\n");
            out.push_str(&text);
            out.push('\n');
        }
        if out.len() >= max_chars {
            break;
        }
    }
    if out.len() > max_chars {
        out.truncate(max_chars);
    }
    out
}

pub fn save_report(task: &str, report: &str) -> Result<String, String> {
    let id = format!("{task}-{}", timestamp());
    let created = now_iso();
    let filter = format!(
        ". + {{id:{},lane:{},createdAt:{},unread:true,priority:(.priority // \"normal\"),confidence:(.confidence // 0.5),sources:(.sources // [])}}",
        json_string(&id),
        json_string(task),
        json_string(&created)
    );
    let normalized = jq(report, &filter)?;

    let latest = state_root().join(task).join("latest.json");
    atomic_write(&latest, &normalized)?;

    let day = briefing_root().join(date_path());
    atomic_write(&day.join(format!("{id}.json")), &jq(&normalized, ".")?)?;
    let markdown = jq_raw(
        &normalized,
        r#""# \(.title // "Hermes briefing")\n\n- lane: `\(.lane // "unknown")`\n- priority: `\(.priority // "normal")`\n- confidence: `\(.confidence // 0.5)`\n- created: `\(.createdAt // "")`\n\n\(.summary // "")\n\n\(.body // "")\n\n## Sources\n\n\((.sources // []) | map(if type == "string" then "- " + . else "- [" + (.title // .url // "source") + "](" + (.url // "") + ")" end) | join("\n"))""#,
    )?;
    atomic_write(&day.join(format!("{id}.md")), &markdown)?;
    reinforce_sources(&normalized)?;
    rebuild_index()?;
    Ok(normalized)
}

pub fn source_registry_text() -> String {
    read_or(
        &state_root().join("unknown-frontier-ai/source-registry.json"),
        "{\"version\":3,\"sources\":{}}",
    )
}

fn source_urls(report: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let raw = jq_raw(report, r#"(.sources // [])[]? | if type == "string" then . else (.url // empty) end"#)
        .unwrap_or_default();
    for url in raw.lines() {
        let value = url.trim();
        if value.starts_with("http://") || value.starts_with("https://") {
            urls.push(value.to_string());
        }
    }
    urls.sort();
    urls.dedup();
    urls
}

pub fn reinforce_sources(report: &str) -> Result<(), String> {
    let urls = source_urls(report);
    if urls.is_empty() {
        return Ok(());
    }
    let path = state_root().join("unknown-frontier-ai/source-registry.json");
    let mut current = source_registry_text();
    let observed = now_iso();
    for url in urls {
        let filter = format!(
            ".version = 3 | .sources = (.sources // {{}}) | .sources[{}] = ((.sources[{}] // {{url:{},tier:\"probation\",score:0,hits:0,failures:0,firstSeen:{},origin:\"report-evidence\"}}) | .hits += 1 | .score += 1 | .lastSeen = {} | .lastUseful = {} | .tier = (if .hits >= 4 then \"promoted\" elif .hits >= 2 then \"trusted\" else \"probation\" end))",
            json_string(&url),
            json_string(&url),
            json_string(&url),
            json_string(&observed),
            json_string(&observed),
            json_string(&observed)
        );
        current = jq(&current, &filter)?;
    }
    atomic_write(&path, &current)
}

pub fn source_records() -> Result<String, String> {
    let registry = source_registry_text();
    jq(
        &registry,
        r#"{schemaVersion:1,count:((.sources // {})|length),sources:((.sources // {})|to_entries|map(.value + {id:.key})|sort_by(-(.score // 0),-(.hits // 0)))}"#,
    )
}

pub fn coverage_summary() -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for task in crate::prompts::FRONTIER_TASKS {
        let text = task_context(task, 100_000);
        let value = |key: &str| {
            jq_raw(&text, &format!(".coverage.{key} // 0"))
                .unwrap_or_default()
                .trim()
                .parse::<i64>()
                .unwrap_or(0)
        };
        *out.entry("candidatesInspected".to_string()).or_default() += value("candidatesInspected");
        *out.entry("canonicalCandidates".to_string()).or_default() += value("canonicalCandidates");
        *out.entry("deepReads".to_string()).or_default() += value("deepReads");
        *out.entry("primaryVerifications".to_string()).or_default() += value("primaryVerifications");
    }
    out
}
