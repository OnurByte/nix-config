use std::{env, fs, os::unix::fs::PermissionsExt, path::PathBuf};

use crate::util::{atomic_write, env_path, jq, jq_raw, json_string, now_iso, read_or, run, run_status};

fn root() -> PathBuf {
    env_path(
        "VESPER_COMMUNICATIONS_STATE_DIR",
        "~/.local/state/vesper/communications",
    )
}

fn token_path() -> PathBuf {
    env_path("VESPER_BEEPER_TOKEN_FILE", "~/.config/vesper/beeper.token")
}

fn base_url() -> Result<String, String> {
    let value = env::var("VESPER_BEEPER_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:23373".to_string());
    let value = value.trim_end_matches('/').to_string();
    if value.starts_with("http://127.0.0.1:") || value.starts_with("http://localhost:") {
        Ok(value)
    } else {
        Err("Beeper intake is local-only; VESPER_BEEPER_BASE_URL must use localhost or 127.0.0.1".to_string())
    }
}

fn set_status(state: &str, detail: &str) -> Result<(), String> {
    let payload = format!(
        "{{\"version\":1,\"state\":{},\"detail\":{},\"checkedAt\":{}}}",
        json_string(state),
        json_string(detail),
        json_string(&now_iso())
    );
    atomic_write(&root().join("status.json"), &payload)
}

pub fn status_json() -> String {
    read_or(
        &root().join("status.json"),
        "{\"version\":1,\"state\":\"unconfigured\",\"detail\":\"Beeper intake has not run yet\"}",
    )
}

fn read_token() -> Result<Option<String>, String> {
    let path = token_path();
    if !path.is_file() {
        return Ok(None);
    }
    let metadata = fs::metadata(&path).map_err(|e| format!("Beeper token metadata: {e}"))?;
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(format!(
            "Beeper token file {} is group/world-accessible; chmod 600 it before communications intake",
            path.display()
        ));
    }
    let token = fs::read_to_string(&path).map_err(|e| format!("Beeper token: {e}"))?;
    let token = token.trim().to_string();
    if token.is_empty() {
        return Ok(None);
    }
    if token.chars().any(char::is_control) || token.contains('"') || token.contains('\\') {
        return Err("Beeper token contains unsupported characters".to_string());
    }
    Ok(Some(token))
}

fn curl_config(token: &str) -> String {
    format!(
        "silent\nshow-error\nfail-with-body\nmax-time = 20\nheader = \"Authorization: Bearer {token}\"\n"
    )
}

fn date_after() -> String {
    let watermark = read_or(&root().join("watermark.json"), "{}");
    let last = jq_raw(&watermark, ".lastSeenAt // \"\"")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !last.is_empty() {
        let spec = format!("{last} - 10 minutes");
        if let Ok(value) = run("date", &["-u", "-d", &spec, "+%Y-%m-%dT%H:%M:%SZ"], None) {
            let value = value.trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
        return last;
    }

    let hours = env::var("VESPER_COMMS_BOOTSTRAP_HOURS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(6)
        .clamp(1, 72);
    let spec = format!("{hours} hours ago");
    run("date", &["-u", "-d", &spec, "+%Y-%m-%dT%H:%M:%SZ"], None)
        .unwrap_or_else(|_| now_iso())
        .trim()
        .to_string()
}

fn search_page(token: &str, after: &str, cursor: Option<&str>) -> Result<String, String> {
    let url = format!("{}/v1/messages/search", base_url()?);
    let mut args = vec![
        "--config".to_string(),
        "-".to_string(),
        "--get".to_string(),
        url,
        "--data-urlencode".to_string(),
        format!("dateAfter={after}"),
        "--data-urlencode".to_string(),
        "includeMuted=true".to_string(),
        "--data-urlencode".to_string(),
        "excludeLowPriority=false".to_string(),
        "--data-urlencode".to_string(),
        "limit=20".to_string(),
    ];
    if let Some(value) = cursor {
        args.extend([
            "--data-urlencode".to_string(),
            format!("cursor={value}"),
            "--data-urlencode".to_string(),
            "direction=before".to_string(),
        ]);
    }
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let raw = run("curl", &refs, Some(&curl_config(token)))?;
    jq(
        &raw,
        "select(type == \"object\" and ((.items // []) | type == \"array\"))",
    )
}

fn normalize_pages(pages: &[String], after: &str) -> Result<String, String> {
    let combined = format!("[{}]", pages.join(","));
    let filter = format!(
        r#"
        (reduce .[] as $page ({{}}; . * ($page.chats // {{}}))) as $chats
        | {{
            status: "ready",
            fetchedAfter: {},
            chats: ($chats | with_entries(.value |= {{
                id: (.id // ""),
                accountID: (.accountID // ""),
                network: (.network // ""),
                title: (.title // ""),
                type: (.type // ""),
                isMuted: (.isMuted // false),
                isLowPriority: (.isLowPriority // false),
                participants: ((.participants.items // []) | map({{
                    id: (.id // ""),
                    fullName: (.fullName // ""),
                    username: (.username // ""),
                    email: (.email // ""),
                    phoneNumber: (.phoneNumber // ""),
                    isSelf: (.isSelf // false),
                    isNetworkBot: (.isNetworkBot // false)
                }}))
            }})),
            messages: ([
                .[] | .items[]?
                | select((.isDeleted // false) == false and (.isHidden // false) == false)
                | {{
                    id: (.id // ""),
                    accountID: (.accountID // ""),
                    chatID: (.chatID // ""),
                    senderID: (.senderID // ""),
                    senderName: (.senderName // ""),
                    isSender: (.isSender // false),
                    timestamp: (.timestamp // ""),
                    sortKey: (.sortKey // ""),
                    text: (.text // ""),
                    type: (.type // ""),
                    isUnread: (.isUnread // false),
                    attachments: ((.attachments // []) | map({{
                        type: (.type // ""),
                        fileName: (.fileName // ""),
                        mimeType: (.mimeType // ""),
                        isVoiceNote: (.isVoiceNote // false),
                        transcription: (.transcription.transcription // "")
                    }})),
                    links: ((.links // []) | map({{
                        title: (.title // ""),
                        url: (.url // ""),
                        summary: (.summary // "")
                    }}))
                }}
            ] | unique_by(.id) | sort_by(.timestamp, .sortKey))
        }}
        "#,
        json_string(after)
    );
    jq(&combined, &filter)
}

fn unavailable(detail: &str) -> Result<String, String> {
    set_status("unavailable", detail)?;
    Ok(format!(
        "{{\"status\":\"unavailable\",\"reason\":{},\"messages\":[],\"chats\":{{}}}}",
        json_string(detail)
    ))
}

pub fn prepare_batch() -> Result<String, String> {
    let pending_path = root().join("pending.json");
    let pending = read_or(&pending_path, "");
    if !pending.trim().is_empty()
        && jq_raw(&pending, ".messages | length")
            .unwrap_or_default()
            .trim()
            .parse::<usize>()
            .unwrap_or(0)
            > 0
    {
        return Ok(pending);
    }

    let token = match read_token() {
        Ok(Some(value)) => value,
        Ok(None) => {
            set_status("unconfigured", "Beeper access token file is missing or empty")?;
            return Ok("{\"status\":\"unconfigured\",\"reason\":\"Beeper access token file is missing or empty\",\"messages\":[],\"chats\":{}}".to_string());
        }
        Err(error) => return unavailable(&error),
    };

    let after = date_after();
    let first = match search_page(&token, &after, None) {
        Ok(value) => value,
        Err(error) => return unavailable(&error),
    };
    let mut pages = vec![first];
    let mut cursor = jq_raw(pages.last().unwrap(), ".oldestCursor // \"\"")?
        .trim()
        .to_string();
    let mut has_more = jq_raw(pages.last().unwrap(), ".hasMore // false")?.trim() == "true";

    for _ in 1..250 {
        if !has_more {
            break;
        }
        if cursor.is_empty() {
            return Err("Beeper pagination reported hasMore without oldestCursor".to_string());
        }
        let page = search_page(&token, &after, Some(&cursor))?;
        has_more = jq_raw(&page, ".hasMore // false")?.trim() == "true";
        cursor = jq_raw(&page, ".oldestCursor // \"\"")?.trim().to_string();
        pages.push(page);
    }
    if has_more {
        return Err("Beeper communications backlog exceeds the 5000-message safety bound; watermark was not advanced".to_string());
    }

    let normalized = normalize_pages(&pages, &after)?;
    let watermark = read_or(&root().join("watermark.json"), "{\"version\":1,\"recentIds\":[]}");
    let max_messages = env::var("VESPER_COMMS_BATCH_MESSAGES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(200)
        .clamp(20, 500);
    let wrapped = format!("{{\"batch\":{},\"watermark\":{}}}", normalized, watermark);
    let filter = format!(
        r#"
        (.watermark.recentIds // []) as $seen
        | (.batch.messages | map(select(.id as $id | (($seen | index($id)) == null)))) as $fresh
        | .batch + {{
            discoveredCount: ($fresh | length),
            backlog: (($fresh | length) > {max_messages}),
            messages: ($fresh[:{max_messages}])
        }}
        "#
    );
    let batch = jq(&wrapped, &filter)?;
    let count = jq_raw(&batch, ".messages | length")?
        .trim()
        .parse::<usize>()
        .unwrap_or(0);
    if count == 0 {
        set_status("ready", "Beeper is reachable; no new messages in the current delta")?;
        return Ok(batch);
    }

    atomic_write(&pending_path, &batch)?;
    set_status("ready", &format!("{count} messages staged for communications analysis"))?;
    Ok(batch)
}

pub fn commit_batch(batch: &str) -> Result<(), String> {
    let watermark_path = root().join("watermark.json");
    let watermark = read_or(&watermark_path, "{\"version\":1,\"recentIds\":[]}");
    let wrapped = format!("{{\"batch\":{},\"watermark\":{}}}", batch, watermark);
    let filter = format!(
        r#"
        .watermark as $old
        | .batch as $batch
        | {{
            version: 1,
            lastSeenAt: (($batch.messages | map(.timestamp) | max) // ($old.lastSeenAt // "")),
            lastPollAt: {},
            recentIds: (((($old.recentIds // []) + ($batch.messages | map(.id))) | unique) | if length > 5000 then .[-5000:] else . end)
        }}
        "#,
        json_string(&now_iso())
    );
    let updated = jq(&wrapped, &filter)?;
    atomic_write(&watermark_path, &updated)?;
    let pending = root().join("pending.json");
    if pending.exists() {
        fs::remove_file(&pending).map_err(|e| e.to_string())?;
    }
    set_status("ready", "latest communications batch analyzed and committed")
}

pub fn maybe_notify(report: &str) -> Result<(), String> {
    let mut body = jq_raw(
        report,
        r#"
        ([.alerts[]? | select((.severity // "") == "high" or (.severity // "") == "critical")][0:4]
          | map("[" + ((.severity // "high") | ascii_upcase) + "] " + (.reason // .summary // "important communication") + (if (.person // "") != "" then " · " + .person elif (.chat // "") != "" then " · " + .chat else "" end))
          | join("\n")) as $alerts
        | if $alerts != "" then $alerts
          elif ((.priority // "normal") == "high" or (.priority // "normal") == "critical") then "[" + ((.priority // "high") | ascii_upcase) + "] " + (.summary // "important communication")
          else "" end
        "#,
    )?;
    body = body.trim().chars().take(1200).collect();
    if body.is_empty() {
        return Ok(());
    }

    let last_path = root().join("last-alert.txt");
    if read_or(&last_path, "").trim() == body {
        return Ok(());
    }
    atomic_write(&last_path, &body)?;
    let _ = run_status(
        "notify-send",
        &[
            "-a",
            "Hermes",
            "-u",
            "critical",
            "Communications intelligence",
            &body,
        ],
        None,
    );
    Ok(())
}
