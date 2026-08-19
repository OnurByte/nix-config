use std::{env, fs, path::PathBuf};

use crate::util::{atomic_write, env_path, jq, jq_raw, json_string, now_iso, read_or, run, run_status};

const TRANSPORT: &str = "agent-messenger";
const PLATFORMS: &[&str] = &["whatsapp", "telegram", "instagram", "discord"];

struct SourceResult {
    status_json: String,
    message_arrays: Vec<String>,
    configured: bool,
    readable: bool,
    degraded: bool,
}

fn root() -> PathBuf {
    env_path(
        "VESPER_COMMUNICATIONS_STATE_DIR",
        "~/.local/state/vesper/communications",
    )
}

fn config_root() -> PathBuf {
    env_path(
        "AGENT_MESSENGER_CONFIG_DIR",
        "~/.config/agent-messenger",
    )
}

fn set_status(state: &str, detail: &str) -> Result<(), String> {
    let payload = format!(
        "{{\"version\":2,\"transport\":\"agent-messenger\",\"state\":{},\"detail\":{},\"checkedAt\":{}}}",
        json_string(state),
        json_string(detail),
        json_string(&now_iso())
    );
    atomic_write(&root().join("status.json"), &payload)
}

pub fn status_json() -> String {
    read_or(
        &root().join("status.json"),
        "{\"version\":2,\"transport\":\"agent-messenger\",\"state\":\"unconfigured\",\"detail\":\"Agent Messenger intake has not run yet\"}",
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

fn agent_read(args: &[&str]) -> Result<String, String> {
    run("vesper-agent-messenger-read", args, None)
}

fn source_status(platform: &str, state: &str, detail: &str) -> String {
    format!(
        "{{\"platform\":{},\"state\":{},\"detail\":{}}}",
        json_string(platform),
        json_string(state),
        json_string(detail)
    )
}

fn chat_refs(raw: &str, after: &str, limit: usize) -> Result<Vec<(String, String)>, String> {
    let filter = format!(
        r#"
        if type != "array" then [] else
          [ .[]?
            | . as $chat
            | (($chat.last_message.timestamp // $chat.last_message.date // "") | tostring) as $last
            | select((($chat.unread_count // 0) > 0) or ($last != "" and $last >= {}))
            | {{
                id: ((.id // "") | tostring),
                title: (.name // .title // ((.id // "") | tostring))
              }}
          ][0:{}]
        end
        "#,
        json_string(after),
        limit
    );
    let refs = jq(raw, &filter)?;
    let count = jq_raw(&refs, "length")?
        .trim()
        .parse::<usize>()
        .unwrap_or(0);
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let id = jq_raw(&refs, &format!(".[{index}].id // \"\""))?
            .trim()
            .to_string();
        if id.is_empty() {
            continue;
        }
        let title_json = jq(&refs, &format!(".[{index}].title // \"\""))?
            .trim()
            .to_string();
        out.push((id, title_json));
    }
    Ok(out)
}

fn discord_dm_refs(raw: &str, limit: usize) -> Result<Vec<(String, String)>, String> {
    let filter = format!(
        r#"
        [(.channels // [])[]?
          | {{
              id: ((.id // "") | tostring),
              title: (
                if ((.name // "") | length) > 0 then .name
                else (([.recipients[]?.username // empty] | join(", ")) // "Discord DM")
                end
              )
            }}
        ][0:{}]
        "#,
        limit
    );
    let refs = jq(raw, &filter)?;
    let count = jq_raw(&refs, "length")?
        .trim()
        .parse::<usize>()
        .unwrap_or(0);
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let id = jq_raw(&refs, &format!(".[{index}].id // \"\""))?
            .trim()
            .to_string();
        if id.is_empty() {
            continue;
        }
        let title_json = jq(&refs, &format!(".[{index}].title // \"Discord DM\""))?
            .trim()
            .to_string();
        out.push((id, title_json));
    }
    Ok(out)
}

fn normalize_messages(
    network: &str,
    chat_id: &str,
    chat_title_json: &str,
    raw: &str,
    after: &str,
) -> Result<String, String> {
    let wrapped = format!(
        "{{\"items\":{},\"network\":{},\"chat\":{},\"chatTitle\":{},\"after\":{}}}",
        raw,
        json_string(network),
        json_string(chat_id),
        chat_title_json,
        json_string(after)
    );
    jq(
        &wrapped,
        r#"
        def mixed_latin_confusable_script($s):
          (($s // "") | explode) as $cp
          | (([$cp[] | select((. >= 65 and . <= 90) or (. >= 97 and . <= 122))] | length) > 0)
            and (([$cp[] | select((. >= 880 and . <= 1023) or (. >= 1024 and . <= 1327))] | length) > 0);
        def presentation_signals($text; $media):
          (($text // "") + "\n" + ($media // "")) as $surface
          | ($surface | explode) as $cp
          | [
              if ([$cp[] | select(. == 8203 or . == 8204 or . == 8205 or . == 8288 or . == 65279)] | length) > 0
              then "zero_width_unicode" else empty end,
              if ([$cp[] | select(. == 1564 or . == 8206 or . == 8207 or (. >= 8234 and . <= 8238) or (. >= 8294 and . <= 8297))] | length) > 0
              then "bidi_control_unicode" else empty end,
              if ([$cp[] | select(. >= 917504 and . <= 917631)] | length) > 0
              then "unicode_tag_payload" else empty end,
              if (($surface | ascii_downcase) | contains("xn--"))
              then "punycode_link" else empty end,
              if (($media // "") != "" and mixed_latin_confusable_script($media))
              then "mixed_script_link" else empty end
            ];
        def source_id($n; $m): (($m.id // "") | tostring);
        def source_chat($n; $m; $fallback):
          if $n == "instagram" then (($m.thread_id // $fallback) | tostring)
          elif $n == "discord" then $fallback
          else (($m.chat_id // $fallback) | tostring)
          end;
        def sender_id($n; $m):
          if $n == "telegram" then (($m.sender.id // "") | tostring)
          elif $n == "discord" then ""
          else (($m.from // "") | tostring)
          end;
        def sender_name($n; $m):
          if $n == "discord" then ($m.author // "")
          elif $n == "telegram" then ""
          else ($m.from_name // "")
          end;
        def msg_timestamp($n; $m):
          if $n == "telegram" then ($m.date // "") else ($m.timestamp // "") end;
        def msg_text($n; $m):
          if $n == "discord" then ($m.content // "") else ($m.text // "") end;
        def msg_type($n; $m):
          if $n == "telegram" then ($m.content_type // "")
          elif $n == "discord" then "message"
          else ($m.type // "")
          end;
        def outgoing($n; $m):
          if $n == "discord" then false else ($m.is_outgoing // false) end;
        def media_url($n; $m): if $n == "instagram" then ($m.media_url // "") else "" end;

        .network as $network
        | .chat as $chat
        | .chatTitle as $title
        | .after as $after
        | [(.items // [])[]?
            | . as $m
            | source_id($network; $m) as $sourceId
            | source_chat($network; $m; $chat) as $sourceChat
            | msg_timestamp($network; $m) as $timestamp
            | msg_text($network; $m) as $text
            | media_url($network; $m) as $media
            | select($sourceId != "" and $timestamp != "" and $timestamp >= $after)
            | {
                id: ($network + ":" + $sourceId),
                sourceMessageId: $sourceId,
                network: $network,
                chatID: ($network + ":" + $sourceChat),
                sourceChatID: $sourceChat,
                chatTitle: $title,
                senderID: sender_id($network; $m),
                senderName: sender_name($network; $m),
                isSender: outgoing($network; $m),
                timestamp: $timestamp,
                editedTimestamp: "",
                sortKey: ($timestamp + ":" + $sourceId),
                text: $text,
                type: msg_type($network; $m),
                isUnread: false,
                linkedMessageID: "",
                mentions: [],
                mediaURL: $media,
                presentationSignals: presentation_signals($text; $media),
                attachments: (if $media != "" then [{type: "media", url: $media}] else [] end),
                links: []
              }
          ]
        "#,
    )
}

fn normalize_discord_mentions(raw: &str, after: &str) -> Result<String, String> {
    let wrapped = format!("{{\"raw\":{},\"after\":{}}}", raw, json_string(after));
    jq(
        &wrapped,
        r#"
        def presentation_signals($text):
          (($text // "") | explode) as $cp
          | [
              if ([$cp[] | select(. == 8203 or . == 8204 or . == 8205 or . == 8288 or . == 65279)] | length) > 0
              then "zero_width_unicode" else empty end,
              if ([$cp[] | select(. == 1564 or . == 8206 or . == 8207 or (. >= 8234 and . <= 8238) or (. >= 8294 and . <= 8297))] | length) > 0
              then "bidi_control_unicode" else empty end,
              if ([$cp[] | select(. >= 917504 and . <= 917631)] | length) > 0
              then "unicode_tag_payload" else empty end,
              if ((($text // "") | ascii_downcase) | contains("xn--"))
              then "punycode_link" else empty end
            ];
        .after as $after
        | [(.raw.mentions // [])[]?
            | . as $m
            | (($m.id // "") | tostring) as $sourceId
            | (($m.channel_id // "") | tostring) as $sourceChat
            | ($m.timestamp // "") as $timestamp
            | ($m.content // "") as $text
            | select($sourceId != "" and $sourceChat != "" and $timestamp != "" and $timestamp >= $after)
            | {
                id: ("discord:" + $sourceId),
                sourceMessageId: $sourceId,
                network: "discord",
                chatID: ("discord:" + $sourceChat),
                sourceChatID: $sourceChat,
                chatTitle: "Discord mention",
                senderID: "",
                senderName: ($m.author // ""),
                isSender: false,
                timestamp: $timestamp,
                editedTimestamp: "",
                sortKey: ($timestamp + ":" + $sourceId),
                text: $text,
                type: "mention",
                isUnread: true,
                linkedMessageID: "",
                mentions: ($m.mentioned_users // []),
                guildID: (($m.guild_id // "") | tostring),
                mediaURL: "",
                presentationSignals: presentation_signals($text),
                attachments: [],
                links: []
              }
          ]
        "#,
    )
}

fn collect_chat_platform(
    platform: &str,
    after: &str,
    chat_limit: usize,
    per_chat_limit: usize,
) -> SourceResult {
    if agent_read(&[platform, "auth", "status"]).is_err() {
        return SourceResult {
            status_json: source_status(platform, "unconfigured", "authentication is not ready"),
            message_arrays: Vec::new(),
            configured: false,
            readable: false,
            degraded: true,
        };
    }

    let chat_limit_string = chat_limit.to_string();
    let raw_chats = match agent_read(&[platform, "chat", "list", "--limit", &chat_limit_string]) {
        Ok(value) => value,
        Err(_) => {
            return SourceResult {
                status_json: source_status(platform, "unavailable", "chat list failed"),
                message_arrays: Vec::new(),
                configured: true,
                readable: false,
                degraded: true,
            };
        }
    };

    let refs = match chat_refs(&raw_chats, after, chat_limit) {
        Ok(value) => value,
        Err(_) => {
            return SourceResult {
                status_json: source_status(platform, "unavailable", "chat list JSON was invalid"),
                message_arrays: Vec::new(),
                configured: true,
                readable: false,
                degraded: true,
            };
        }
    };

    let per_chat_limit_string = per_chat_limit.to_string();
    let mut arrays = Vec::new();
    let mut failures = 0usize;
    for (chat_id, title_json) in &refs {
        let raw = match agent_read(&[
            platform,
            "message",
            "list",
            chat_id,
            "--limit",
            &per_chat_limit_string,
        ]) {
            Ok(value) => value,
            Err(_) => {
                failures += 1;
                continue;
            }
        };
        match normalize_messages(platform, chat_id, title_json, &raw, after) {
            Ok(value) => arrays.push(value),
            Err(_) => failures += 1,
        }
    }

    let degraded = failures > 0;
    let state = if degraded { "degraded" } else { "ready" };
    let detail = if degraded {
        format!(
            "{} recent chats selected; {} read failures",
            refs.len(), failures
        )
    } else {
        format!("{} recent chats selected", refs.len())
    };
    SourceResult {
        status_json: source_status(platform, state, &detail),
        message_arrays: arrays,
        configured: true,
        readable: true,
        degraded,
    }
}

fn collect_discord(after: &str, chat_limit: usize, per_chat_limit: usize) -> SourceResult {
    let platform = "discord";
    if agent_read(&[platform, "auth", "status"]).is_err() {
        return SourceResult {
            status_json: source_status(platform, "unconfigured", "authentication is not ready"),
            message_arrays: Vec::new(),
            configured: false,
            readable: false,
            degraded: true,
        };
    }

    let chat_limit_string = chat_limit.to_string();
    let mention_limit = (chat_limit * 4).clamp(50, 400).to_string();
    let dm_raw = agent_read(&[platform, "dm", "unread", "--limit", &chat_limit_string]);
    let mention_raw = agent_read(&[platform, "mention", "unread", "--limit", &mention_limit]);

    if dm_raw.is_err() && mention_raw.is_err() {
        return SourceResult {
            status_json: source_status(platform, "unavailable", "unread DM and mention queries both failed"),
            message_arrays: Vec::new(),
            configured: true,
            readable: false,
            degraded: true,
        };
    }

    let mut arrays = Vec::new();
    let mut failures = 0usize;
    let mut dm_count = 0usize;
    let mut completeness_issue = false;

    if let Ok(raw) = dm_raw {
        completeness_issue |= jq_raw(&raw, ".complete // true")
            .map(|value| value.trim() != "true")
            .unwrap_or(true);
        match discord_dm_refs(&raw, chat_limit) {
            Ok(refs) => {
                dm_count = refs.len();
                let per_chat_limit_string = per_chat_limit.to_string();
                for (chat_id, title_json) in refs {
                    match agent_read(&[
                        platform,
                        "message",
                        "list",
                        &chat_id,
                        "--limit",
                        &per_chat_limit_string,
                    ]) {
                        Ok(messages) => match normalize_messages(
                            platform,
                            &chat_id,
                            &title_json,
                            &messages,
                            after,
                        ) {
                            Ok(value) => arrays.push(value),
                            Err(_) => failures += 1,
                        },
                        Err(_) => failures += 1,
                    }
                }
            }
            Err(_) => failures += 1,
        }
    } else {
        failures += 1;
    }

    let mut mention_count = 0usize;
    if let Ok(raw) = mention_raw {
        completeness_issue |= jq_raw(&raw, ".complete // true")
            .map(|value| value.trim() != "true")
            .unwrap_or(true);
        mention_count = jq_raw(&raw, ".mentions | length")
            .ok()
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        match normalize_discord_mentions(&raw, after) {
            Ok(value) => arrays.push(value),
            Err(_) => failures += 1,
        }
    } else {
        failures += 1;
    }

    let degraded = failures > 0 || completeness_issue;
    let state = if degraded { "degraded" } else { "ready" };
    let detail = format!(
        "{} unread DM channels; {} unread mentions{}",
        dm_count,
        mention_count,
        if degraded { "; partial read" } else { "" }
    );
    SourceResult {
        status_json: source_status(platform, state, &detail),
        message_arrays: arrays,
        configured: true,
        readable: true,
        degraded,
    }
}

fn combine_message_arrays(arrays: &[String]) -> Result<String, String> {
    if arrays.is_empty() {
        return Ok("[]".to_string());
    }
    let wrapped = format!("[{}]", arrays.join(","));
    jq(
        &wrapped,
        "[.[][]?] | unique_by(.id) | sort_by(.timestamp, .sortKey, .id)",
    )
}

fn empty_batch(state: &str, reason: &str, after: &str, sources: &str) -> String {
    format!(
        "{{\"status\":{},\"transport\":\"agent-messenger\",\"reason\":{},\"fetchedAfter\":{},\"sources\":{},\"degraded\":true,\"messages\":[],\"contextMessages\":[],\"chats\":[]}}",
        json_string(state),
        json_string(reason),
        json_string(after),
        sources
    )
}

pub fn prepare_batch() -> Result<String, String> {
    let pending_path = root().join("pending.json");
    let pending = read_or(&pending_path, "");
    if !pending.trim().is_empty() {
        let transport = jq_raw(&pending, ".transport // \"\"")
            .unwrap_or_default()
            .trim()
            .to_string();
        let count = jq_raw(&pending, ".messages | length")
            .unwrap_or_default()
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        if transport == TRANSPORT && count > 0 {
            return Ok(pending);
        }
        let _ = fs::remove_file(&pending_path);
    }

    let after = date_after();
    let chat_limit = env::var("VESPER_COMMS_CHAT_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(80)
        .clamp(10, 200);
    let per_chat_limit = env::var("VESPER_COMMS_MESSAGES_PER_CHAT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50)
        .clamp(10, 100);

    let mut results = Vec::new();
    for platform in ["whatsapp", "telegram", "instagram"] {
        results.push(collect_chat_platform(
            platform,
            &after,
            chat_limit,
            per_chat_limit,
        ));
    }
    results.push(collect_discord(&after, chat_limit, per_chat_limit));

    let configured = results.iter().filter(|result| result.configured).count();
    let readable = results.iter().filter(|result| result.readable).count();
    let degraded = results.iter().any(|result| result.degraded);
    let sources = format!(
        "[{}]",
        results
            .iter()
            .map(|result| result.status_json.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );

    if readable == 0 {
        let state = if configured == 0 && !config_root().exists() {
            "unconfigured"
        } else {
            "unavailable"
        };
        let reason = if state == "unconfigured" {
            "No Agent Messenger accounts are configured"
        } else {
            "Agent Messenger accounts exist but no configured source is currently readable"
        };
        set_status(state, reason)?;
        return Ok(empty_batch(state, reason, &after, &sources));
    }

    let mut arrays = Vec::new();
    for result in &results {
        arrays.extend(result.message_arrays.iter().cloned());
    }
    let messages = combine_message_arrays(&arrays)?;
    let normalized = format!(
        "{{\"status\":\"ready\",\"transport\":\"agent-messenger\",\"fetchedAfter\":{},\"sources\":{},\"degraded\":{},\"messages\":{},\"chats\":[]}}",
        json_string(&after),
        sources,
        if degraded { "true" } else { "false" },
        messages
    );

    let watermark = read_or(
        &root().join("watermark.json"),
        "{\"version\":2,\"transport\":\"agent-messenger\",\"recentIds\":[]}",
    );
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
    let discovered = jq_raw(&batch, ".discoveredCount // 0")?
        .trim()
        .parse::<usize>()
        .unwrap_or(count);
    let intake_state = if degraded { "degraded" } else { "ready" };

    if count == 0 {
        set_status(
            intake_state,
            &format!(
                "Agent Messenger: {readable}/{} sources readable; no new messages in the current delta",
                PLATFORMS.len()
            ),
        )?;
        return Ok(batch);
    }

    atomic_write(&pending_path, &batch)?;
    set_status(
        intake_state,
        &format!(
            "Agent Messenger: {readable}/{} sources readable; {count} of {discovered} new messages staged",
            PLATFORMS.len()
        ),
    )?;
    Ok(batch)
}

pub fn commit_batch(batch: &str) -> Result<(), String> {
    let watermark_path = root().join("watermark.json");
    let watermark = read_or(
        &watermark_path,
        "{\"version\":2,\"transport\":\"agent-messenger\",\"recentIds\":[]}",
    );
    let wrapped = format!("{{\"batch\":{},\"watermark\":{}}}", batch, watermark);
    let filter = format!(
        r#"
        .watermark as $old
        | .batch as $batch
        | {{
            version: 2,
            transport: "agent-messenger",
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
    let degraded = jq_raw(batch, ".degraded // false")?.trim() == "true";
    if degraded {
        set_status(
            "degraded",
            "latest Agent Messenger communications batch analyzed and committed with partial source coverage",
        )
    } else {
        set_status("ready", "latest Agent Messenger communications batch analyzed and committed")
    }
}

pub fn sanitize_report(batch: &str, report: &str) -> Result<String, String> {
    let wrapped = format!("{{\"batch\":{},\"report\":{}}}", batch, report);
    jq(
        &wrapped,
        r#"
        def valid_evidence($valid):
          [(.evidenceMessageIds // [])[]?
            | select(type == "string" and length > 0)
            | . as $id
            | select(($valid | index($id)) != null)]
          | unique;
        def evidence_bound($valid):
          .evidenceMessageIds = valid_evidence($valid);
        def semantic_grounds:
          [(.semanticGrounds // [])[]?
            | select(type == "string")
            | . as $ground
            | select((["direct_request","deadline","credential_request","money_request","impersonation","coercion","threat","boundary_pressure","material_contradiction","sensitive_account_action","material_decision","other"] | index($ground)) != null)]
          | unique;

        (((.batch.messages // []) + (.batch.contextMessages // []))
          | map(.id // empty)
          | map(select(type == "string" and length > 0))
          | unique) as $valid
        | (.report.alerts // [] | length) as $alertsBefore
        | (.report.manipulationSignals // [] | length) as $manipulationBefore
        | .report
        | .alerts = [(.alerts // [])[]?
            | select(type == "object")
            | evidence_bound($valid)
            | .semanticGrounds = semantic_grounds
            | select((.evidenceMessageIds | length) > 0)
            | select((.semanticGrounds | length) > 0)
            | select((.basis // "") == "semantic" or (.basis // "") == "fused")
            | select((.severity // "") == "high" or (.severity // "") == "critical")]
        | .strategy = [(.strategy // [])[]?
            | select(type == "object")
            | evidence_bound($valid)
            | select((.evidenceMessageIds | length) > 0)]
        | .commitments = [(.commitments // [])[]?
            | select(type == "object")
            | evidence_bound($valid)
            | select((.evidenceMessageIds | length) > 0)]
        | .manipulationSignals = [(.manipulationSignals // [])[]?
            | select(type == "object")
            | evidence_bound($valid)
            | select((.evidenceMessageIds | length) > 0)]
        | .topics = [(.topics // [])[]?
            | select(type == "object")
            | evidence_bound($valid)
            | select((.evidenceMessageIds | length) > 0)]
        | .people = [(.people // [])[]?
            | select(type == "object")
            | .facts = [(.facts // [])[]?
                | select(type == "object")
                | evidence_bound($valid)
                | select((.evidenceMessageIds | length) > 0)]
            | .riskSignals = [(.riskSignals // [])[]?
                | select(type == "object")
                | evidence_bound($valid)
                | select((.evidenceMessageIds | length) > 0)]
            | select(((.facts | length) + (.riskSignals | length) + ((.openLoops // []) | length)) > 0)]
        | if ((.priority // "normal") == "high" or (.priority // "normal") == "critical")
             and ((.alerts // []) | length) == 0
          then .priority = "normal"
          else .
          end
        | .validation = ((.validation // {}) + {
            evidenceGate: {
              validMessageIds: ($valid | length),
              droppedAlerts: ($alertsBefore - ((.alerts // []) | length)),
              droppedManipulationSignals: ($manipulationBefore - ((.manipulationSignals // []) | length)),
              priorityRequiresValidatedAlert: true,
              alertRequiresSemanticGrounds: true
            }
          })
        "#,
    )
}

pub fn maybe_notify(report: &str) -> Result<(), String> {
    let mut body = jq_raw(
        report,
        r#"
        [.alerts[]? | select((.severity // "") == "high" or (.severity // "") == "critical")][0:4]
        | map("[" + ((.severity // "high") | ascii_upcase) + "] " + (.reason // .summary // "important communication") + (if (.person // "") != "" then " · " + .person elif (.chat // "") != "" then " · " + .chat else "" end))
        | join("\n")
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
    use super::{normalize_messages, sanitize_report};
    use crate::util::jq_raw;

    #[test]
    fn agent_messenger_normalization_prefixes_identity_and_flags_unicode() {
        let messages = r#"[
          {
            "id":"m1",
            "chat_id":"chat-1",
            "from":"person-1",
            "from_name":"Person",
            "timestamp":"2026-08-19T12:00:00Z",
            "is_outgoing":false,
            "type":"text",
            "text":"pay\u200bnow\udb40\udc01"
          },
          {
            "id":"m2",
            "chat_id":"chat-1",
            "from":"person-1",
            "timestamp":"2026-08-19T12:01:00Z",
            "is_outgoing":false,
            "type":"text",
            "text":"ordinary text"
          }
        ]"#;

        let normalized = normalize_messages(
            "whatsapp",
            "chat-1",
            "\"Test\"",
            messages,
            "2026-08-19T11:00:00Z",
        )
        .expect("Agent Messenger fixture should normalize");
        assert_eq!(jq_raw(&normalized, ".[0].id").unwrap().trim(), "whatsapp:m1");
        assert_eq!(
            jq_raw(&normalized, ".[0].presentationSignals | sort | join(\",\")")
                .unwrap()
                .trim(),
            "unicode_tag_payload,zero_width_unicode"
        );
        assert_eq!(
            jq_raw(&normalized, ".[1].presentationSignals | length")
                .unwrap()
                .trim(),
            "0"
        );
    }

    #[test]
    fn agent_messenger_normalization_drops_messages_before_overlap_window() {
        let messages = r#"[
          {"id":1,"chat_id":7,"date":"2026-08-19T10:00:00Z","is_outgoing":false,"sender":{"type":"user","id":9},"content_type":"messageText","text":"old"},
          {"id":2,"chat_id":7,"date":"2026-08-19T12:00:00Z","is_outgoing":false,"sender":{"type":"user","id":9},"content_type":"messageText","text":"new"}
        ]"#;
        let normalized = normalize_messages(
            "telegram",
            "7",
            "\"Telegram chat\"",
            messages,
            "2026-08-19T11:00:00Z",
        )
        .expect("Telegram fixture should normalize");
        assert_eq!(jq_raw(&normalized, "length").unwrap().trim(), "1");
        assert_eq!(jq_raw(&normalized, ".[0].id").unwrap().trim(), "telegram:2");
    }

    #[test]
    fn evidence_gate_drops_unknown_ids_and_downgrades_unproven_priority() {
        let batch = r#"{
          "messages":[{"id":"m1"}],
          "contextMessages":[{"id":"m0"}]
        }"#;
        let report = r#"{
          "title":"test",
          "summary":"test",
          "priority":"critical",
          "alerts":[{"severity":"critical","reason":"invented","basis":"semantic","semanticGrounds":["credential_request"],"evidenceMessageIds":["missing"]}],
          "strategy":[{"action":"verify","evidenceMessageIds":["m1","missing"]}],
          "manipulationSignals":[{"kind":"unicode_obfuscation","evidenceMessageIds":["missing"]}],
          "people":[{"identityKey":"p1","facts":[{"claim":"supported","evidenceMessageIds":["m0","missing"]}],"riskSignals":[]}]
        }"#;

        let clean = sanitize_report(batch, report).expect("report should sanitize");
        assert_eq!(jq_raw(&clean, ".priority").unwrap().trim(), "normal");
        assert_eq!(jq_raw(&clean, ".alerts | length").unwrap().trim(), "0");
        assert_eq!(
            jq_raw(&clean, ".strategy[0].evidenceMessageIds | join(\",\")")
                .unwrap()
                .trim(),
            "m1"
        );
        assert_eq!(jq_raw(&clean, ".manipulationSignals | length").unwrap().trim(), "0");
        assert_eq!(
            jq_raw(&clean, ".people[0].facts[0].evidenceMessageIds | join(\",\")")
                .unwrap()
                .trim(),
            "m0"
        );
        assert_eq!(jq_raw(&clean, ".validation.evidenceGate.droppedAlerts").unwrap().trim(), "1");
    }

    #[test]
    fn evidence_gate_rejects_presentation_only_alerts() {
        let batch = r#"{"messages":[{"id":"m1","presentationSignals":["zero_width_unicode"]}],"contextMessages":[]}"#;
        let report = r#"{
          "title":"test",
          "summary":"unicode only",
          "priority":"high",
          "alerts":[{"severity":"high","reason":"invisible unicode","basis":"fused","semanticGrounds":[],"evidenceMessageIds":["m1"]}],
          "manipulationSignals":[{"kind":"unicode_obfuscation","observation":"zero width","evidenceMessageIds":["m1"]}]
        }"#;

        let clean = sanitize_report(batch, report).expect("report should sanitize");
        assert_eq!(jq_raw(&clean, ".priority").unwrap().trim(), "normal");
        assert_eq!(jq_raw(&clean, ".alerts | length").unwrap().trim(), "0");
        assert_eq!(jq_raw(&clean, ".manipulationSignals | length").unwrap().trim(), "1");
    }

    #[test]
    fn evidence_gate_keeps_valid_high_alert() {
        let batch = r#"{"messages":[{"id":"m1"}],"contextMessages":[]}"#;
        let report = r#"{
          "title":"test",
          "summary":"test",
          "priority":"high",
          "alerts":[{"severity":"high","reason":"credential request","basis":"semantic","semanticGrounds":["credential_request"],"evidenceMessageIds":["m1"]}]
        }"#;

        let clean = sanitize_report(batch, report).expect("report should sanitize");
        assert_eq!(jq_raw(&clean, ".priority").unwrap().trim(), "high");
        assert_eq!(jq_raw(&clean, ".alerts | length").unwrap().trim(), "1");
        assert_eq!(
            jq_raw(&clean, ".alerts[0].semanticGrounds | join(\",\")")
                .unwrap()
                .trim(),
            "credential_request"
        );
    }
}
