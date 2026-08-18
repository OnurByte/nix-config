use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VICON_SCHEMA_VERSION: u32 = 1;
const PROMPT_REVISION: u32 = 1;

#[derive(Clone, Debug)]
struct Claim {
    key: String,
    provider: String,
    source_kind: String,
    source_path: PathBuf,
    app_ids: Vec<String>,
}

#[derive(Clone, Debug)]
struct Decomposition {
    silhouette: String,
    background: String,
    confidence: f64,
    notes: String,
    provider: String,
    model: String,
}

#[derive(Clone, Debug)]
struct InventoryItem {
    id: String,
    icon_key: String,
    fingerprint: String,
    source_kind: String,
    canonical_state: String,
    excluded: bool,
}

fn home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/nonexistent"))
}

fn state_root() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("vesper/adaptive-icons")
}

fn data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

fn config_root() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("vesper")
}

fn runtime_root() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| state_root().join("runtime"))
        .join("vesper/adaptive-icons")
}

fn canonical_root() -> PathBuf {
    data_home().join("vesper/adaptive-icons/canonical")
}

fn active_theme() -> PathBuf {
    data_home().join("icons/Vesper-Adaptive")
}

fn ai_config_path() -> PathBuf {
    config_root().join("adaptive-icons-ai.conf")
}

fn inventory_path() -> PathBuf {
    state_root().join("inventory.tsv")
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | '+') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn write_atomic(path: &Path, data: impl AsRef<[u8]>) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid path: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("vicon"),
        std::process::id()
    ));
    fs::write(&tmp, data).map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())
}

fn base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut index = 0;
    while index + 3 <= data.len() {
        let n = ((data[index] as u32) << 16)
            | ((data[index + 1] as u32) << 8)
            | data[index + 2] as u32;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
        index += 3;
    }
    match data.len() - index {
        1 => {
            let n = (data[index] as u32) << 16;
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = ((data[index] as u32) << 16) | ((data[index + 1] as u32) << 8);
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}

fn output(command: &str, args: &[&str]) -> Result<String, String> {
    let result = Command::new(command)
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
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

fn jq_text(input: &[u8], filter: &str) -> Result<String, String> {
    let mut child = Command::new("jq")
        .args(["-r", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start jq: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input).map_err(|error| error.to_string())?;
    }
    let result = child.wait_with_output().map_err(|error| error.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

fn credential_lookup(provider: &str) -> Result<String, String> {
    output(
        "secret-tool",
        &["lookup", "service", "vesper-ai", "provider", provider],
    )
    .and_then(|value| {
        if value.is_empty() {
            Err(format!("missing {provider} API key"))
        } else {
            Ok(value)
        }
    })
}

fn model_for(provider: &str) -> String {
    let content = fs::read_to_string(ai_config_path()).unwrap_or_default();
    let wanted = format!("model.{provider}");
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == wanted && !value.trim().is_empty() {
            return value.trim().to_string();
        }
    }
    match provider {
        "openai" => "gpt-5".to_string(),
        "xai" => "grok-4.5".to_string(),
        "openrouter" => "openai/gpt-5".to_string(),
        "google" => "gemini-3.6-flash".to_string(),
        "anthropic" => "claude-sonnet-4-20250514".to_string(),
        _ => String::new(),
    }
}

fn load_inventory() -> Vec<InventoryItem> {
    let content = fs::read_to_string(inventory_path()).unwrap_or_default();
    let mut items = Vec::new();
    for line in content.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 9 {
            continue;
        }
        items.push(InventoryItem {
            id: parts[0].to_string(),
            icon_key: parts[1].to_string(),
            fingerprint: parts[3].to_string(),
            source_kind: parts[4].to_string(),
            canonical_state: parts[5].to_string(),
            excluded: parts[7] == "1",
        });
    }
    items
}

fn claim_job() -> Result<Option<Claim>, String> {
    let result = Command::new("vesper-icon-queue")
        .arg("claim")
        .output()
        .map_err(|error| format!("failed to claim icon job: {error}"))?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
    }
    if jq_text(&result.stdout, ".job == null")? == "true" {
        return Ok(None);
    }
    let key = jq_text(&result.stdout, ".job.key // empty")?;
    let provider = jq_text(&result.stdout, ".job.provider // empty")?;
    let source_kind = jq_text(&result.stdout, ".job.sourceKind // empty")?;
    let source_path = jq_text(&result.stdout, ".job.sourcePath // empty")?;
    let apps = jq_text(&result.stdout, ".job.appIds[]? // empty")?
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if key.is_empty() || provider.is_empty() || source_path.is_empty() || apps.is_empty() {
        return Err("claimed conversion job is incomplete".to_string());
    }
    Ok(Some(Claim {
        key,
        provider,
        source_kind,
        source_path: PathBuf::from(source_path),
        app_ids: apps,
    }))
}

fn queue_complete(key: &str) {
    let _ = Command::new("vesper-icon-queue")
        .args(["complete", key])
        .status();
}

fn queue_fail(key: &str, permanent: bool, message: &str) {
    let clean = message
        .chars()
        .map(|ch| if matches!(ch, '\n' | '\r' | '\t') { ' ' } else { ch })
        .collect::<String>();
    let _ = Command::new("vesper-icon-queue")
        .args([
            "fail",
            key,
            if permanent { "permanent" } else { "transient" },
            clean.as_str(),
        ])
        .status();
}

fn source_is_unsafe_svg(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let lower = content.to_ascii_lowercase();
    [
        "<script",
        "<foreignobject",
        "javascript:",
        "http://",
        "https://",
        "file://",
        "@import",
        "@font-face",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn normalize_source(claim: &Claim, work: &Path) -> Result<PathBuf, String> {
    let metadata = fs::metadata(&claim.source_path)
        .map_err(|error| format!("cannot read source icon: {error}"))?;
    if !metadata.is_file() {
        return Err("source icon is not a regular file".to_string());
    }
    if metadata.len() > 20 * 1024 * 1024 {
        return Err("source icon exceeds 20 MiB input limit".to_string());
    }
    if matches!(claim.source_kind.as_str(), "svg" | "svgz")
        && source_is_unsafe_svg(&claim.source_path)
    {
        return Err("unsafe SVG cannot be sent or rasterized".to_string());
    }

    fs::create_dir_all(work).map_err(|error| error.to_string())?;
    let target = work.join("input.png");
    let status = if matches!(claim.source_kind.as_str(), "svg" | "svgz") {
        Command::new("rsvg-convert")
            .args(["-w", "1024", "-h", "1024", "-o"])
            .arg(&target)
            .arg(&claim.source_path)
            .status()
    } else {
        Command::new("magick")
            .arg(&claim.source_path)
            .args([
                "-auto-orient",
                "-strip",
                "-resize",
                "1024x1024>",
                "-background",
                "none",
                "-gravity",
                "center",
                "-extent",
                "1024x1024",
            ])
            .arg(format!("PNG32:{}", target.display()))
            .status()
    }
    .map_err(|error| format!("failed to normalize icon source: {error}"))?;
    if !status.success() || !target.is_file() {
        return Err("source icon could not be normalized safely".to_string());
    }
    let target_size = fs::metadata(&target).map(|value| value.len()).unwrap_or(0);
    if target_size == 0 || target_size > 20 * 1024 * 1024 {
        return Err("normalized icon is empty or too large".to_string());
    }
    Ok(target)
}

fn decomposition_prompt() -> &'static str {
    "Analyze this installed application icon for Vesper adaptive icon canonicalization. Return only JSON. Preserve brand identity. Do not redesign the logo and do not invent 3D geometry. Classify silhouette as exactly one of enclosed, circular, glyph, irregular, full-bleed. Choose backgroundStrategy as exactly one of brand-solid, brand-gradient, system-brand-gradient, system-light, system-dark, palette-surface, transparent, artwork. Use retainRaster=true whenever reconstructing vector geometry would risk changing identity. groups must be an integer from 1 to 4 describing the minimum useful semantic depth groups. notes must be short. Required JSON keys: silhouette, backgroundStrategy, retainRaster, groups, confidence, notes. confidence is 0 to 1."
}

fn schema_json() -> &'static str {
    "{\"type\":\"object\",\"additionalProperties\":false,\"properties\":{\"silhouette\":{\"type\":\"string\",\"enum\":[\"enclosed\",\"circular\",\"glyph\",\"irregular\",\"full-bleed\"]},\"backgroundStrategy\":{\"type\":\"string\",\"enum\":[\"brand-solid\",\"brand-gradient\",\"system-brand-gradient\",\"system-light\",\"system-dark\",\"palette-surface\",\"transparent\",\"artwork\"]},\"retainRaster\":{\"type\":\"boolean\"},\"groups\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":4},\"confidence\":{\"type\":\"number\",\"minimum\":0,\"maximum\":1},\"notes\":{\"type\":\"string\"}},\"required\":[\"silhouette\",\"backgroundStrategy\",\"retainRaster\",\"groups\",\"confidence\",\"notes\"]}"
}

fn http_post(
    url: &str,
    headers: &[String],
    body: &Path,
    response: &Path,
    response_headers: &Path,
) -> Result<u16, String> {
    let mut child = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--max-time",
            "240",
            "--request",
            "POST",
            "--header",
            "@-",
            "--dump-header",
        ])
        .arg(response_headers)
        .args(["--output"])
        .arg(response)
        .args(["--write-out", "%{http_code}", "--data-binary"])
        .arg(format!("@{}", body.display()))
        .arg(url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start curl: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        for header in headers {
            stdin.write_all(header.as_bytes()).map_err(|error| error.to_string())?;
            stdin.write_all(b"\n").map_err(|error| error.to_string())?;
        }
    }
    let result = child.wait_with_output().map_err(|error| error.to_string())?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "provider transport failed".to_string()
        } else {
            stderr
        });
    }
    String::from_utf8_lossy(&result.stdout)
        .trim()
        .parse::<u16>()
        .map_err(|_| "provider returned invalid HTTP status".to_string())
}

fn strip_code_fence(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let mut lines = trimmed.lines();
    let _ = lines.next();
    let mut body = lines.collect::<Vec<_>>();
    if body.last().map(|line| line.trim()) == Some("```") {
        body.pop();
    }
    body.join("\n").trim().to_string()
}

fn provider_request(claim: &Claim, image: &Path, work: &Path) -> Result<Decomposition, String> {
    let key = credential_lookup(&claim.provider)?;
    let model = model_for(&claim.provider);
    if model.is_empty() {
        return Err(format!("unsupported provider: {}", claim.provider));
    }
    let image_bytes = fs::read(image).map_err(|error| error.to_string())?;
    let image_b64 = base64(&image_bytes);
    let prompt = decomposition_prompt();
    let body_path = work.join("request.json");
    let response_path = work.join("response.json");
    let headers_path = work.join("headers.txt");

    let (url, headers, body, filter) = match claim.provider.as_str() {
        "openai" => (
            "https://api.openai.com/v1/responses".to_string(),
            vec![
                format!("Authorization: Bearer {key}"),
                "Content-Type: application/json".to_string(),
            ],
            format!(
                "{{\"model\":\"{}\",\"store\":false,\"input\":[{{\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",\"text\":\"{}\"}},{{\"type\":\"input_image\",\"image_url\":\"data:image/png;base64,{}\",\"detail\":\"high\"}}]}}],\"text\":{{\"format\":{{\"type\":\"json_schema\",\"name\":\"vesper_icon_decomposition\",\"strict\":true,\"schema\":{}}}}}}}}",
                json_escape(&model),
                json_escape(prompt),
                image_b64,
                schema_json(),
            ),
            "[.. | objects | select(.type? == \"output_text\") | .text?] | map(select(. != null)) | join(\"\\n\")",
        ),
        "xai" => (
            "https://api.x.ai/v1/responses".to_string(),
            vec![
                format!("Authorization: Bearer {key}"),
                "Content-Type: application/json".to_string(),
            ],
            format!(
                "{{\"model\":\"{}\",\"store\":false,\"input\":[{{\"role\":\"user\",\"content\":[{{\"type\":\"input_image\",\"image_url\":\"data:image/png;base64,{}\",\"detail\":\"high\"}},{{\"type\":\"input_text\",\"text\":\"{}\"}}]}}]}}",
                json_escape(&model),
                image_b64,
                json_escape(prompt),
            ),
            "[.. | objects | select(.type? == \"output_text\") | .text?] | map(select(. != null)) | join(\"\\n\")",
        ),
        "openrouter" => (
            "https://openrouter.ai/api/v1/chat/completions".to_string(),
            vec![
                format!("Authorization: Bearer {key}"),
                "Content-Type: application/json".to_string(),
                "X-OpenRouter-Title: Vesper Adaptive Icons".to_string(),
            ],
            format!(
                "{{\"model\":\"{}\",\"stream\":false,\"response_format\":{{\"type\":\"json_object\"}},\"messages\":[{{\"role\":\"user\",\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}},{{\"type\":\"image_url\",\"image_url\":{{\"url\":\"data:image/png;base64,{}\"}}}}]}}]}}",
                json_escape(&model),
                json_escape(prompt),
                image_b64,
            ),
            ".choices[0].message.content // empty",
        ),
        "anthropic" => (
            "https://api.anthropic.com/v1/messages".to_string(),
            vec![
                format!("x-api-key: {key}"),
                "anthropic-version: 2023-06-01".to_string(),
                "Content-Type: application/json".to_string(),
            ],
            format!(
                "{{\"model\":\"{}\",\"max_tokens\":1200,\"messages\":[{{\"role\":\"user\",\"content\":[{{\"type\":\"image\",\"source\":{{\"type\":\"base64\",\"media_type\":\"image/png\",\"data\":\"{}\"}}}},{{\"type\":\"text\",\"text\":\"{}\"}}]}}]}}",
                json_escape(&model),
                image_b64,
                json_escape(prompt),
            ),
            "[.content[]? | select(.type == \"text\") | .text] | join(\"\\n\")",
        ),
        "google" => (
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model
            ),
            vec![
                format!("x-goog-api-key: {key}"),
                "Content-Type: application/json".to_string(),
            ],
            format!(
                "{{\"contents\":[{{\"parts\":[{{\"inline_data\":{{\"mime_type\":\"image/png\",\"data\":\"{}\"}}}},{{\"text\":\"{}\"}}]}}],\"generationConfig\":{{\"responseMimeType\":\"application/json\"}}}}",
                image_b64,
                json_escape(prompt),
            ),
            "[.candidates[0].content.parts[]? | .text?] | map(select(. != null)) | join(\"\\n\")",
        ),
        _ => return Err(format!("unsupported provider: {}", claim.provider)),
    };

    fs::write(&body_path, body).map_err(|error| error.to_string())?;
    let status = http_post(&url, &headers, &body_path, &response_path, &headers_path)?;
    if !(200..300).contains(&status) {
        let response = fs::read_to_string(&response_path).unwrap_or_default();
        let concise = response.chars().take(800).collect::<String>();
        if matches!(status, 400 | 401 | 403 | 404) {
            return Err(format!("permanent: provider HTTP {status}: {concise}"));
        }
        return Err(format!("provider HTTP {status}: {concise}"));
    }

    let response = fs::read(&response_path).map_err(|error| error.to_string())?;
    let text = strip_code_fence(&jq_text(&response, filter)?);
    if text.is_empty() {
        return Err("provider returned no decomposition text".to_string());
    }
    let decomposition_path = work.join("decomposition.json");
    fs::write(&decomposition_path, &text).map_err(|error| error.to_string())?;
    let data = fs::read(&decomposition_path).map_err(|error| error.to_string())?;
    let silhouette = jq_text(&data, ".silhouette // empty")?;
    let background = jq_text(&data, ".backgroundStrategy // empty")?;
    let confidence = jq_text(&data, ".confidence // 0")?
        .parse::<f64>()
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let notes = jq_text(&data, ".notes // empty")?;

    if !matches!(
        silhouette.as_str(),
        "enclosed" | "circular" | "glyph" | "irregular" | "full-bleed"
    ) {
        return Err("provider returned invalid silhouette classification".to_string());
    }
    if !matches!(
        background.as_str(),
        "brand-solid"
            | "brand-gradient"
            | "system-brand-gradient"
            | "system-light"
            | "system-dark"
            | "palette-surface"
            | "transparent"
            | "artwork"
    ) {
        return Err("provider returned invalid background strategy".to_string());
    }
    if confidence < 0.45 {
        return Err(format!("provider confidence too low: {confidence:.2}"));
    }

    Ok(Decomposition {
        silhouette,
        background,
        confidence,
        notes,
        provider: claim.provider.clone(),
        model,
    })
}

fn canonical_dir(app_id: &str, fingerprint: &str) -> PathBuf {
    canonical_root()
        .join(safe_name(app_id))
        .join(fingerprint)
}

fn appearance_json(name: &str) -> String {
    format!(
        "{{\"schemaVersion\":{},\"name\":\"{}\",\"inherits\":\"default\",\"material\":\"standard\"}}\n",
        VICON_SCHEMA_VERSION,
        json_escape(name)
    )
}

fn install_vicon_dir(stage: &Path, target: &Path) -> Result<(), String> {
    let parent = target.parent().ok_or_else(|| "invalid vicon target".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let old = parent.join(format!(".icon.vicon.old.{}", std::process::id()));
    if old.exists() {
        fs::remove_dir_all(&old).map_err(|error| error.to_string())?;
    }
    if target.exists() {
        fs::rename(target, &old).map_err(|error| error.to_string())?;
    }
    if let Err(error) = fs::rename(stage, target) {
        if old.exists() {
            let _ = fs::rename(&old, target);
        }
        return Err(error.to_string());
    }
    if old.exists() {
        let _ = fs::remove_dir_all(old);
    }
    Ok(())
}

fn raster_compat_svg(png: &[u8]) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\"><image x=\"0\" y=\"0\" width=\"1024\" height=\"1024\" preserveAspectRatio=\"xMidYMid meet\" href=\"data:image/png;base64,{}\"/></svg>\n",
        base64(png)
    )
}

fn build_raster_vicon(claim: &Claim, normalized: &Path, decomposition: &Decomposition) -> Result<(), String> {
    let png = fs::read(normalized).map_err(|error| error.to_string())?;
    for app_id in &claim.app_ids {
        let dir = canonical_dir(app_id, &claim.key);
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let stage = dir.join(format!(".icon.vicon.{}.tmp", std::process::id()));
        if stage.exists() {
            fs::remove_dir_all(&stage).map_err(|error| error.to_string())?;
        }
        let layers = stage.join("groups/01-primary/layers");
        fs::create_dir_all(&layers).map_err(|error| error.to_string())?;
        fs::create_dir_all(stage.join("appearances")).map_err(|error| error.to_string())?;
        fs::write(layers.join("01.png"), &png).map_err(|error| error.to_string())?;
        fs::write(
            stage.join("groups/01-primary/group.json"),
            "{\"id\":\"primary\",\"role\":\"primary\",\"renderMode\":\"combined\",\"depth\":1}\n",
        )
        .map_err(|error| error.to_string())?;
        fs::write(stage.join("appearances/default.json"), appearance_json("default"))
            .map_err(|error| error.to_string())?;
        fs::write(stage.join("appearances/dark.json"), appearance_json("dark"))
            .map_err(|error| error.to_string())?;
        fs::write(stage.join("appearances/mono.json"), appearance_json("mono"))
            .map_err(|error| error.to_string())?;

        let apps = claim
            .app_ids
            .iter()
            .map(|id| format!("\"{}\"", json_escape(id)))
            .collect::<Vec<_>>()
            .join(",");
        let manifest = format!(
            "{{\"schemaVersion\":{},\"canvas\":{{\"width\":1024,\"height\":1024,\"masked\":false}},\"sourceFingerprint\":\"{}\",\"applicationIds\":[{}],\"provenance\":{{\"kind\":\"ai-semantic-retained-raster\",\"provider\":\"{}\",\"model\":\"{}\",\"promptRevision\":{}}},\"silhouette\":\"{}\",\"background\":{{\"strategy\":\"{}\"}},\"groups\":[{{\"id\":\"primary\",\"role\":\"primary\",\"renderMode\":\"combined\",\"layers\":[{{\"id\":\"preserved-artwork\",\"assetType\":\"raster\",\"asset\":\"groups/01-primary/layers/01.png\",\"effects\":\"limited\"}}]}}],\"appearances\":[\"default\",\"dark\",\"mono\"],\"validation\":{{\"identityConfidence\":{:.3},\"status\":\"passed\",\"notes\":\"{}\"}}}}\n",
            VICON_SCHEMA_VERSION,
            json_escape(&claim.key),
            apps,
            json_escape(&decomposition.provider),
            json_escape(&decomposition.model),
            PROMPT_REVISION,
            json_escape(&decomposition.silhouette),
            json_escape(&decomposition.background),
            decomposition.confidence,
            json_escape(&decomposition.notes),
        );
        fs::write(stage.join("manifest.json"), manifest).map_err(|error| error.to_string())?;
        install_vicon_dir(&stage, &dir.join("icon.vicon"))?;

        let compatibility = raster_compat_svg(&png);
        write_atomic(&dir.join("canonical.svg"), compatibility.as_bytes())?;
        let metadata = format!(
            "{{\"schemaVersion\":{},\"desktopId\":\"{}\",\"sourceFingerprint\":\"{}\",\"sourcePath\":\"{}\",\"sourceKind\":\"{}\",\"provenance\":\"ai-semantic-retained-raster\",\"compatibilityDerived\":true,\"vicon\":\"icon.vicon\",\"validation\":\"passed\"}}\n",
            VICON_SCHEMA_VERSION,
            json_escape(app_id),
            json_escape(&claim.key),
            json_escape(&claim.source_path.to_string_lossy()),
            json_escape(&claim.source_kind),
        );
        write_atomic(&dir.join("metadata.json"), metadata.as_bytes())?;
    }
    Ok(())
}

fn local_silhouette(svg: &str) -> &'static str {
    let lower = svg.to_ascii_lowercase();
    let circles = lower.matches("<circle").count() + lower.matches("<ellipse").count();
    let rects = lower.matches("<rect").count();
    if circles > 0 && rects == 0 {
        "circular"
    } else if rects > 0 {
        "enclosed"
    } else {
        "irregular"
    }
}

fn ensure_local_vector_vicon(item: &InventoryItem) -> Result<(), String> {
    if item.excluded || item.canonical_state != "validated" || item.fingerprint.is_empty() {
        return Ok(());
    }
    let dir = canonical_dir(&item.id, &item.fingerprint);
    let canonical = dir.join("canonical.svg");
    let target = dir.join("icon.vicon");
    if target.join("manifest.json").is_file() || !canonical.is_file() {
        return Ok(());
    }
    let svg = fs::read_to_string(&canonical).map_err(|error| error.to_string())?;
    let stage = dir.join(format!(".icon.vicon.{}.tmp", std::process::id()));
    if stage.exists() {
        fs::remove_dir_all(&stage).map_err(|error| error.to_string())?;
    }
    let layers = stage.join("groups/01-primary/layers");
    fs::create_dir_all(&layers).map_err(|error| error.to_string())?;
    fs::create_dir_all(stage.join("appearances")).map_err(|error| error.to_string())?;
    fs::write(layers.join("01.svg"), &svg).map_err(|error| error.to_string())?;
    fs::write(
        stage.join("groups/01-primary/group.json"),
        "{\"id\":\"primary\",\"role\":\"primary\",\"renderMode\":\"combined\",\"depth\":1}\n",
    )
    .map_err(|error| error.to_string())?;
    fs::write(stage.join("appearances/default.json"), appearance_json("default"))
        .map_err(|error| error.to_string())?;
    fs::write(stage.join("appearances/dark.json"), appearance_json("dark"))
        .map_err(|error| error.to_string())?;
    fs::write(stage.join("appearances/mono.json"), appearance_json("mono"))
        .map_err(|error| error.to_string())?;
    let manifest = format!(
        "{{\"schemaVersion\":{},\"canvas\":{{\"width\":1024,\"height\":1024,\"masked\":false}},\"sourceFingerprint\":\"{}\",\"applicationIds\":[\"{}\"],\"provenance\":{{\"kind\":\"local-vector\"}},\"silhouette\":\"{}\",\"background\":{{\"strategy\":\"transparent\"}},\"groups\":[{{\"id\":\"primary\",\"role\":\"primary\",\"renderMode\":\"combined\",\"layers\":[{{\"id\":\"official-vector\",\"assetType\":\"vector\",\"asset\":\"groups/01-primary/layers/01.svg\"}}]}}],\"appearances\":[\"default\",\"dark\",\"mono\"],\"validation\":{{\"status\":\"passed\"}}}}\n",
        VICON_SCHEMA_VERSION,
        json_escape(&item.fingerprint),
        json_escape(&item.id),
        local_silhouette(&svg),
    );
    fs::write(stage.join("manifest.json"), manifest).map_err(|error| error.to_string())?;
    install_vicon_dir(&stage, &target)
}

fn sync_local_vicons() -> Result<(), String> {
    for item in load_inventory() {
        if let Err(error) = ensure_local_vector_vicon(&item) {
            eprintln!("vicon package for {} failed: {error}", item.id);
        }
    }
    Ok(())
}

fn process_once() -> Result<bool, String> {
    sync_local_vicons()?;
    let Some(claim) = claim_job()? else {
        return Ok(false);
    };
    let work = runtime_root().join(format!("{}-{}", &claim.key[..claim.key.len().min(16)], std::process::id()));
    if work.exists() {
        let _ = fs::remove_dir_all(&work);
    }
    fs::create_dir_all(&work).map_err(|error| error.to_string())?;

    let result = (|| {
        let normalized = normalize_source(&claim, &work)?;
        let decomposition = provider_request(&claim, &normalized, &work)?;
        build_raster_vicon(&claim, &normalized, &decomposition)?;
        Ok::<(), String>(())
    })();

    match result {
        Ok(()) => {
            queue_complete(&claim.key);
            let _ = Command::new("vesper-icon-engine-core").arg("reconcile").status();
        }
        Err(error) => {
            let permanent = error.starts_with("permanent:")
                || error.contains("unsafe SVG")
                || error.contains("regular file")
                || error.contains("20 MiB")
                || error.contains("normalized icon is empty")
                || error.contains("unsupported provider");
            queue_fail(&claim.key, permanent, &error);
        }
    }
    let _ = fs::remove_dir_all(work);
    Ok(true)
}

fn daemon() -> Result<(), String> {
    loop {
        match process_once() {
            Ok(true) => {}
            Ok(false) => thread::sleep(Duration::from_secs(3)),
            Err(error) => {
                eprintln!("adaptive icon worker error: {error}");
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    if source.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(source, target).map_err(|error| error.to_string())?;
        return Ok(());
    }
    if !source.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(target).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        copy_tree(&entry.path(), &target.join(entry.file_name()))?;
    }
    Ok(())
}

fn read_accent() -> String {
    let content = fs::read_to_string(config_root().join("adaptive-icons.conf")).unwrap_or_default();
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "accent" {
                let value = value.trim();
                if value.starts_with('#') && value.len() == 7 {
                    return value.to_string();
                }
            }
        }
    }
    "#7aa2f7".to_string()
}

fn render_appearance(canonical: &[u8], mode: &str, accent: &str) -> String {
    let encoded = base64(canonical);
    let image = format!(
        "<image x=\"136\" y=\"136\" width=\"752\" height=\"752\" preserveAspectRatio=\"xMidYMid meet\" href=\"data:image/svg+xml;base64,{}\"/>",
        encoded
    );
    let body = match mode {
        "original" => format!(
            "<image x=\"0\" y=\"0\" width=\"1024\" height=\"1024\" preserveAspectRatio=\"xMidYMid meet\" href=\"data:image/svg+xml;base64,{}\"/>",
            encoded
        ),
        "light" => format!(
            "<rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"#f7f7f8\" stroke=\"#ffffff\" stroke-width=\"10\"/>{image}"
        ),
        "dark" => format!(
            "<rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"#171719\" stroke=\"#38383d\" stroke-width=\"10\"/>{image}"
        ),
        "tinted" => format!(
            "<defs><filter id=\"mono\"><feColorMatrix type=\"saturate\" values=\"0\"/></filter></defs><rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"{accent}\" fill-opacity=\"0.24\"/><g filter=\"url(#mono)\" opacity=\"0.92\">{image}</g>"
        ),
        "clear" => format!(
            "<rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"#ffffff\" fill-opacity=\"0.10\" stroke=\"#ffffff\" stroke-opacity=\"0.36\" stroke-width=\"8\"/>{image}"
        ),
        "glass" => format!(
            "<defs><linearGradient id=\"g\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.38\"/><stop offset=\"0.48\" stop-color=\"{accent}\" stop-opacity=\"0.18\"/><stop offset=\"1\" stop-color=\"#000000\" stop-opacity=\"0.16\"/></linearGradient><filter id=\"shadow\"><feDropShadow dx=\"0\" dy=\"18\" stdDeviation=\"24\" flood-opacity=\"0.24\"/></filter></defs><rect x=\"100\" y=\"100\" width=\"824\" height=\"824\" rx=\"188\" fill=\"url(#g)\" stroke=\"#ffffff\" stroke-opacity=\"0.45\" stroke-width=\"8\" filter=\"url(#shadow)\"/>{image}"
        ),
        _ => image,
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\">{body}</svg>\n"
    )
}

fn export_root() -> Result<PathBuf, String> {
    let downloads = home().join("Downloads");
    fs::create_dir_all(&downloads).map_err(|error| error.to_string())?;
    Ok(downloads.join(format!("Vesper-Adaptive-Icons-{}", now_ms())))
}

fn export_all(kind: &str) -> Result<PathBuf, String> {
    sync_local_vicons()?;
    let root = export_root()?;
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let accent = read_accent();

    if matches!(kind, "current-svg" | "archive") {
        copy_tree(&active_theme(), &root.join("current-svg"))?;
    }

    if matches!(kind, "current-png" | "archive") {
        let source = active_theme();
        let target = root.join("current-png");
        if source.is_dir() {
            for entry in walk_files(&source)? {
                if entry.extension().and_then(|value| value.to_str()) != Some("svg") {
                    continue;
                }
                let relative = entry.strip_prefix(&source).unwrap_or(&entry);
                let mut out = target.join(relative);
                out.set_extension("png");
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                let status = Command::new("rsvg-convert")
                    .args(["-w", "512", "-h", "512", "-o"])
                    .arg(&out)
                    .arg(&entry)
                    .status()
                    .map_err(|error| error.to_string())?;
                if !status.success() {
                    return Err(format!("failed to export {}", entry.display()));
                }
            }
        }
    }

    if matches!(kind, "all-appearances" | "archive") {
        let target = root.join("all-appearances");
        for item in load_inventory() {
            if item.excluded || item.canonical_state != "validated" || item.fingerprint.is_empty() {
                continue;
            }
            let canonical = canonical_dir(&item.id, &item.fingerprint).join("canonical.svg");
            if !canonical.is_file() {
                continue;
            }
            let bytes = fs::read(&canonical).map_err(|error| error.to_string())?;
            let app_dir = target.join(safe_name(&item.id));
            fs::create_dir_all(&app_dir).map_err(|error| error.to_string())?;
            for mode in ["original", "light", "dark", "tinted", "clear", "glass"] {
                fs::write(app_dir.join(format!("{mode}.svg")), render_appearance(&bytes, mode, &accent))
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    if matches!(kind, "canonical" | "archive") {
        let target = root.join("canonical-vicon");
        for item in load_inventory() {
            if item.fingerprint.is_empty() {
                continue;
            }
            let source = canonical_dir(&item.id, &item.fingerprint).join("icon.vicon");
            if source.is_dir() {
                copy_tree(&source, &target.join(format!("{}.vicon", safe_name(&item.id))))?;
            }
        }
    }

    if kind == "archive" {
        let archive = root.with_extension("tar.gz");
        let parent = root.parent().ok_or_else(|| "invalid export path".to_string())?;
        let name = root
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "invalid export name".to_string())?;
        let status = Command::new("tar")
            .arg("-C")
            .arg(parent)
            .args(["-czf"])
            .arg(&archive)
            .arg(name)
            .status()
            .map_err(|error| format!("failed to start tar: {error}"))?;
        if !status.success() {
            return Err("failed to create icon export archive".to_string());
        }
        return Ok(archive);
    }

    if !matches!(kind, "current-svg" | "current-png" | "all-appearances" | "canonical") {
        return Err(format!("unknown export format: {kind}"));
    }
    Ok(root)
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn export_app(id: &str) -> Result<PathBuf, String> {
    sync_local_vicons()?;
    let item = load_inventory()
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| format!("application not in adaptive icon inventory: {id}"))?;
    if item.fingerprint.is_empty() {
        return Err("application has no resolved icon fingerprint".to_string());
    }
    let root = export_root()?.join(safe_name(id));
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let dir = canonical_dir(id, &item.fingerprint);
    let canonical = dir.join("canonical.svg");
    if canonical.is_file() {
        let bytes = fs::read(&canonical).map_err(|error| error.to_string())?;
        let accent = read_accent();
        for mode in ["original", "light", "dark", "tinted", "clear", "glass"] {
            fs::write(root.join(format!("{mode}.svg")), render_appearance(&bytes, mode, &accent))
                .map_err(|error| error.to_string())?;
        }
    }
    if dir.join("icon.vicon").is_dir() {
        copy_tree(&dir.join("icon.vicon"), &root.join(format!("{}.vicon", safe_name(id))))?;
    }
    fs::write(
        root.join("source.txt"),
        format!(
            "desktopId={}\niconKey={}\nsourceKind={}\nfingerprint={}\n",
            item.id, item.icon_key, item.source_kind, item.fingerprint
        ),
    )
    .map_err(|error| error.to_string())?;
    Ok(root)
}

fn usage() -> ! {
    eprintln!(
        "vesper-icon-worker\n\
         commands:\n\
           process-once\n\
           sync-vicons\n\
           export-all current-svg|current-png|all-appearances|canonical|archive\n\
           export-app <desktop-id>\n\
           daemon"
    );
    std::process::exit(2);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [command] if command == "process-once" => process_once().map(|processed| {
            println!("{}", if processed { "processed" } else { "idle" });
        }),
        [command] if command == "sync-vicons" => sync_local_vicons(),
        [command, kind] if command == "export-all" => export_all(kind).map(|path| println!("{}", path.display())),
        [command, id] if command == "export-app" => export_app(id).map(|path| println!("{}", path.display())),
        [command] if command == "daemon" => daemon(),
        _ => usage(),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
