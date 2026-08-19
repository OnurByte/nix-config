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
        def mixed_latin_confusable_script($s):
          (($s // "") | explode) as $cp
          | (([$cp[] | select((. >= 65 and . <= 90) or (. >= 97 and . <= 122))] | length) > 0)
            and (([$cp[] | select((. >= 880 and . <= 1023) or (. >= 1024 and . <= 1327))] | length) > 0);

        def suspicious_double_extension($name):
          (($name // "") | ascii_downcase
            | test("\\.(pdf|doc|docx|xls|xlsx|ppt|pptx|txt|jpg|jpeg|png|gif|webp|zip|rar)\\.(exe|scr|com|bat|cmd|ps1|js|jse|vbs|vbe|sh|apk)$"));

        def presentation_signals($m):
          ([($m.text // "")]
            + [($m.links // [])[]? | (.title // ""), (.summary // ""), (.url // ""), (.originalURL // "")]
            + [($m.attachments // [])[]? | (.fileName // ""), (.transcription.transcription // "")]
            | map(select(type == "string"))
            | join("\n")
            | explode) as $cp
          | [
              if ([$cp[] | select(. == 8203 or . == 8204 or . == 8205 or . == 8288 or . == 65279)] | length) > 0
              then "zero_width_unicode" else empty end,
              if ([$cp[] | select(. == 8206 or . == 8207 or (. >= 8234 and . <= 8238) or (. >= 8294 and . <= 8297))] | length) > 0
              then "bidi_control_unicode" else empty end,
              if ([($m.links // [])[]? | select((.originalURL // .url // "") != (.url // ""))] | length) > 0
              then "redirected_link" else empty end,
              if ([($m.links // [])[]? | (.url // ""), (.originalURL // "") | select(mixed_latin_confusable_script(.))] | length) > 0
              then "mixed_script_link" else empty end,
              if ([($m.links // [])[]? | (.url // ""), (.originalURL // "") | ascii_downcase | select(contains("xn--"))] | length) > 0
              then "punycode_link" else empty end,
              if ([($m.attachments // [])[]? | .fileName // "" | select(suspicious_double_extension(.))] | length) > 0
              then "suspicious_double_extension" else empty end
            ];

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
                | . as $m
                | {{
                    id: ($m.id // ""),
                    accountID: ($m.accountID // ""),
                    chatID: ($m.chatID // ""),
                    senderID: ($m.senderID // ""),
                    senderName: ($m.senderName // ""),
                    isSender: ($m.isSender // false),
                    timestamp: ($m.timestamp // ""),
                    editedTimestamp: ($m.editedTimestamp // ""),
                    sortKey: ($m.sortKey // ""),
                    text: ($m.text // ""),
                    type: ($m.type // ""),
                    isUnread: ($m.isUnread // false),
                    linkedMessageID: ($m.linkedMessageID // ""),
                    mentions: ($m.mentions // []),
                    presentationSignals: presentation_signals($m),
                    attachments: (($m.attachments // []) | map({{
                        type: (.type // ""),
                        fileName: (.fileName // ""),
                        fileSize: (.fileSize // 0),
                        mimeType: (.mimeType // ""),
                        isVoiceNote: (.isVoiceNote // false),
                        transcription: (.transcription.transcription // "")
                    }})),
                    links: (($m.links // []) | map({{
                        title: (.title // ""),
                        url: (.url // ""),
                        originalURL: (.originalURL // ""),
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
        "{{\"status\":\"unavailable\",\"reason\":{},\"messages\":[],\"contextMessages\":[],\"chats\":{{}}}}",
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
            return Ok("{\"status\":\"unconfigured\",\"reason\":\"Beeper access token file is missing or empty\",\"messages\":[],\"contextMessages\":[],\"chats\":{}}".to_string());
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
        | ($fresh[:{max_messages}]) as $selected
        | ($selected | map(.chatID) | unique) as $selectedChats
        | (.batch.messages
            | map(select(
                (.id as $id | (($seen | index($id)) != null))
                and (.chatID as $chat | (($selectedChats | index($chat)) != null))
              ))
            | if length > 400 then .[-400:] else . end) as $context
        | .batch + {{
            discoveredCount: ($fresh | length),
            backlog: (($fresh | length) > {max_messages}),
            contextMessages: $context,
            messages: $selected
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

#[cfg(test)]
mod tests {
    use super::normalize_pages;
    use crate::util::jq_raw;

    #[test]
    fn presentation_preflight_flags_deceptive_surfaces_without_scoring_them() {
        let page = r#"{
          "chats": {
            "chat-1": {
              "id": "chat-1",
              "accountID": "whatsapp",
              "network": "whatsapp",
              "title": "Test",
              "type": "single",
              "participants": {"items": []}
            }
          },
          "items": [
            {
              "id": "m1",
              "accountID": "whatsapp",
              "chatID": "chat-1",
              "senderID": "person-1",
              "senderName": "Person",
              "timestamp": "2026-08-19T12:00:00Z",
              "sortKey": "1",
              "text": "pay\u200bnow",
              "type": "TEXT",
              "links": [{
                "title": "portal",
                "url": "https://exаmple.com/final",
                "originalURL": "https://xn--exmple-cua.example/x",
                "summary": "open portal"
              }],
              "attachments": [{
                "type": "file",
                "fileName": "invoice.pdf.exe",
                "mimeType": "application/octet-stream"
              }]
            },
            {
              "id": "m2",
              "accountID": "whatsapp",
              "chatID": "chat-1",
              "senderID": "person-1",
              "timestamp": "2026-08-19T12:01:00Z",
              "sortKey": "2",
              "text": "ordinary text",
              "type": "TEXT"
            }
          ]
        }"#;

        let normalized = normalize_pages(&[page.to_string()], "2026-08-19T11:00:00Z")
            .expect("communications fixture should normalize");
        let first = jq_raw(&normalized, ".messages[0].presentationSignals | sort | join(\",\")")
            .expect("first signal list should parse");
        assert_eq!(
            first.trim(),
            "mixed_script_link,punycode_link,redirected_link,suspicious_double_extension,zero_width_unicode"
        );
        let second = jq_raw(&normalized, ".messages[1].presentationSignals | length")
            .expect("second signal list should parse");
        assert_eq!(second.trim(), "0");
    }
}
