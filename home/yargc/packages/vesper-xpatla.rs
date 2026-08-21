use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const USER_AGENT: &str = "VesperXPatla/0.1 (+local-news-research)";
const MAX_VIDEO_BYTES: u64 = 512 * 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_POSTS_PER_SOURCE: usize = 20;

#[derive(Clone, Debug)]
struct Source {
    handle: String,
    max_posts: usize,
    rights_status: String,
    ideology: String,
    tone: String,
}

struct Lock(PathBuf);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn config_path() -> PathBuf {
    env::var_os("VESPER_XPATLA_SOURCES")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home().join(".config"))
                .join("vesper/xpatla/sources.json")
        })
}

fn state_root() -> PathBuf {
    env::var_os("VESPER_XPATLA_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home().join(".local/state"))
                .join("vesper/xpatla")
        })
}

fn cache_root() -> PathBuf {
    env::var_os("VESPER_XPATLA_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home().join(".cache"))
                .join("vesper-xpatla/media")
        })
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn run(program: &str, args: &[&str], input: Option<&str>) -> Result<String, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("{program}: {error}"))?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .ok_or_else(|| format!("{program}: stdin unavailable"))?
            .write_all(input.as_bytes())
            .map_err(|error| format!("{program}: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("{program}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!("{program} failed: {detail}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn jq(input: &str, filter: &str) -> Result<String, String> {
    run("jq", &["-c", filter], Some(input))
}

fn jq_raw(input: &str, filter: &str) -> Result<String, String> {
    run("jq", &["-r", "-c", filter], Some(input))
}

fn json_quote(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() => output.push(' '),
            c => output.push(c),
        }
    }
    output.push('"');
    output
}

fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn db_path() -> PathBuf {
    state_root().join("state.sqlite3")
}

fn db_exec(sql: &str) -> Result<String, String> {
    let root = state_root();
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let path = db_path();
    let path = path.to_str().ok_or("state path is not UTF-8")?;
    run("sqlite3", &["-batch", path, sql], None)
}

fn db_query(sql: &str) -> Result<String, String> {
    db_exec(sql)
}

fn init_db() -> Result<(), String> {
    db_exec(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS sources (
           handle TEXT PRIMARY KEY,
           enabled INTEGER NOT NULL DEFAULT 1,
           rights_status TEXT NOT NULL DEFAULT 'unknown',
           ideology TEXT NOT NULL DEFAULT '',
           tone TEXT NOT NULL DEFAULT '',
           followers INTEGER,
           statuses INTEGER,
           updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS observed_posts (
           post_id TEXT PRIMARY KEY,
           source_handle TEXT NOT NULL,
           status_url TEXT NOT NULL,
           text TEXT NOT NULL,
           created_at TEXT NOT NULL DEFAULT '',
           created_timestamp INTEGER NOT NULL DEFAULT 0,
           likes INTEGER NOT NULL DEFAULT 0,
           replies INTEGER NOT NULL DEFAULT 0,
           reposts INTEGER NOT NULL DEFAULT 0,
           quotes INTEGER NOT NULL DEFAULT 0,
           views INTEGER NOT NULL DEFAULT 0,
           media_count INTEGER NOT NULL DEFAULT 0,
           media_kinds TEXT NOT NULL DEFAULT '',
           sensitive INTEGER NOT NULL DEFAULT 0,
           raw_json TEXT NOT NULL,
           observed_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS media_assets (
           media_id TEXT NOT NULL,
           post_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           media_url TEXT NOT NULL,
           publisher_handle TEXT NOT NULL DEFAULT '',
           duration REAL,
           width INTEGER,
           height INTEGER,
           formats_json TEXT NOT NULL DEFAULT '[]',
           fingerprint TEXT NOT NULL,
           selected INTEGER NOT NULL DEFAULT 0,
           PRIMARY KEY (media_id, post_id)
         );
         CREATE TABLE IF NOT EXISTS opportunities (
           post_id TEXT PRIMARY KEY,
           score REAL NOT NULL,
           score_reason TEXT NOT NULL,
           state TEXT NOT NULL DEFAULT 'candidate',
           updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS runs (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           started_at INTEGER NOT NULL,
           finished_at INTEGER NOT NULL,
           source_count INTEGER NOT NULL,
           posts_seen INTEGER NOT NULL,
           posts_new INTEGER NOT NULL,
           errors TEXT NOT NULL DEFAULT '',
           status TEXT NOT NULL
         );",
    )?;
    // Keep existing local databases usable when source-profile fields grow.
    let _ = db_exec("ALTER TABLE sources ADD COLUMN ideology TEXT NOT NULL DEFAULT '';");
    Ok(())
}

fn load_sources() -> Result<Vec<Source>, String> {
    let path = config_path();
    let text = fs::read_to_string(&path).map_err(|error| {
        format!(
            "source config unavailable at {}: {error}; create a JSON file with a top-level sources array",
            path.display()
        )
    })?;
    parse_sources(&text)
}

fn parse_sources(text: &str) -> Result<Vec<Source>, String> {
    let rows = jq_raw(
        &text,
        ".sources[]? | select(.enabled != false) | [(.handle // \"\"), ((.maxPosts // 20) | tostring), (.rightsStatus // \"unknown\"), (.profile.ideology // \"\"), (.profile.tone // \"\")] | @tsv",
    )?;
    let mut sources = Vec::new();
    for row in rows.lines() {
        let fields = row.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            continue;
        }
        let handle = fields[0].trim().trim_start_matches('@').to_string();
        if !valid_handle(&handle) {
            continue;
        }
        sources.push(Source {
            handle,
            // FxTwitter's profile timeline is bounded to one page here. Keep
            // the scan predictable; pagination can be added when a real need
            // for deeper history appears.
            max_posts: fields[1]
                .parse::<usize>()
                .unwrap_or(MAX_POSTS_PER_SOURCE)
                .clamp(1, MAX_POSTS_PER_SOURCE),
            rights_status: fields[2].to_string(),
            ideology: fields[3].to_string(),
            tone: fields[4].to_string(),
        });
    }
    Ok(sources)
}

fn valid_handle(handle: &str) -> bool {
    !handle.is_empty()
        && handle.len() <= 15
        && handle
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn fetch_profile_statuses(source: &Source, since: i64) -> Result<String, String> {
    let endpoint = if since > 0 {
        format!(
            "https://api.fxtwitter.com/2/profile/{}/statuses?count={}&since={}",
            source.handle, source.max_posts, since
        )
    } else {
        format!(
            "https://api.fxtwitter.com/2/profile/{}/statuses?count={}",
            source.handle, source.max_posts
        )
    };
    let endpoint = endpoint.as_str();
    run(
        "curl",
        &[
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--user-agent",
            USER_AGENT,
            endpoint,
        ],
        None,
    )
}

fn fetch_status(_handle: &str, id: &str) -> Result<String, String> {
    let endpoint = format!("https://api.fxtwitter.com/2/status/{id}");
    let endpoint = endpoint.as_str();
    run(
        "curl",
        &[
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--user-agent",
            USER_AGENT,
            endpoint,
        ],
        None,
    )
}

fn field(raw: &str, filter: &str) -> String {
    jq_raw(raw, filter).unwrap_or_default().trim().to_string()
}

fn field_json(raw: &str, filter: &str) -> String {
    jq(raw, filter)
        .unwrap_or_else(|_| "null".to_string())
        .trim()
        .to_string()
}

fn first_i64(raw: &str, filter: &str) -> i64 {
    field(raw, filter).parse::<i64>().unwrap_or(0)
}

fn first_f64(raw: &str, filter: &str) -> f64 {
    field(raw, filter).parse::<f64>().unwrap_or(0.0)
}

fn optional_i64(raw: &str, filter: &str) -> Option<i64> {
    let value = field(raw, filter);
    let value = value.trim();
    if value.is_empty() || value == "null" {
        None
    } else {
        value.parse::<i64>().ok()
    }
}

fn media_kind_summary(raw: &str) -> String {
    field(raw, "[(.tweet.media.all // [])[]?.type // \"\"] | map(select(. != \"\")) | unique | join(\",\")")
}

fn score_post(
    likes: i64,
    replies: i64,
    reposts: i64,
    quotes: i64,
    views: i64,
    age_minutes: i64,
    media_count: i64,
    sensitive: bool,
) -> (f64, String) {
    let weighted = likes.max(0) as f64 * 0.5
        + replies.max(0) as f64 * 5.0
        + reposts.max(0) as f64
        + quotes.max(0) as f64 * 5.0;
    let velocity = weighted / (age_minutes.max(15) as f64 / 60.0);
    let view_signal = (views.max(0) as f64 + 1.0).ln().min(16.0);
    let media_signal = if media_count > 0 { 5.0 } else { 0.0 };
    let penalty = if sensitive { 100.0 } else { 0.0 };
    let score =
        ((velocity + 1.0).ln() * 14.0 + view_signal + media_signal - penalty).clamp(0.0, 100.0);
    let reason =
        format!("velocity={velocity:.2};views={views};media={media_count};sensitive={sensitive}");
    (score, reason)
}

fn upsert_source(source: &Source, raw: &str) -> Result<(), String> {
    let followers = optional_i64(raw, ".results[0].author.followers")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "NULL".to_string());
    let statuses = optional_i64(raw, ".results[0].author.statuses")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "NULL".to_string());
    let sql = format!(
        "INSERT INTO sources(handle,enabled,rights_status,ideology,tone,followers,statuses,updated_at) VALUES ({},1,{},{},{},{},{},{}) ON CONFLICT(handle) DO UPDATE SET rights_status=excluded.rights_status,ideology=excluded.ideology,tone=excluded.tone,followers=COALESCE(excluded.followers,sources.followers),statuses=COALESCE(excluded.statuses,sources.statuses),updated_at=excluded.updated_at;",
        sql_quote(&source.handle),
        sql_quote(&source.rights_status),
        sql_quote(&source.ideology),
        sql_quote(&source.tone),
        followers,
        statuses,
        timestamp()
    );
    db_exec(&sql).map(|_| ())
}

fn insert_media(post_id: &str, item: &str) -> Result<(), String> {
    let media_id = field(item, ".id // \"\"");
    if media_id.is_empty() {
        return Ok(());
    }
    let kind = field(item, ".type // \"\"");
    let url = field(item, ".url // \"\"");
    let publisher = field(item, ".publisher.screen_name // \"\"");
    let duration = first_f64(item, ".duration // 0");
    let width = first_i64(item, ".width // 0");
    let height = first_i64(item, ".height // 0");
    let formats = field_json(item, ".formats // []");
    let fingerprint = format!("{kind}:{media_id}");
    let sql = format!(
        "INSERT OR IGNORE INTO media_assets(media_id,post_id,kind,media_url,publisher_handle,duration,width,height,formats_json,fingerprint) VALUES ({},{},{},{},{},{},{},{},{},{});",
        sql_quote(&media_id),
        sql_quote(post_id),
        sql_quote(&kind),
        sql_quote(&url),
        sql_quote(&publisher),
        duration,
        width,
        height,
        sql_quote(&formats),
        sql_quote(&fingerprint)
    );
    db_exec(&sql).map(|_| ())
}

fn ingest_status(source: &Source, raw: &str) -> Result<bool, String> {
    let post_id = field(raw, ".tweet.id // \"\"");
    if post_id.is_empty() {
        return Ok(false);
    }
    let source_handle = field(raw, ".tweet.author.screen_name // \"\"");
    let source_handle = if source_handle.is_empty() {
        source.handle.as_str()
    } else {
        source_handle.as_str()
    };
    let status_url = field(raw, ".tweet.url // \"\"");
    let text = field(raw, ".tweet.text // \"\"");
    let created_at = field(raw, ".tweet.created_at // \"\"");
    let created_timestamp = first_i64(raw, ".tweet.created_timestamp // 0");
    let likes = first_i64(raw, ".tweet.likes // 0");
    let replies = first_i64(raw, ".tweet.replies // 0");
    let reposts = first_i64(raw, ".tweet.reposts // 0");
    let quotes = first_i64(raw, ".tweet.quotes // 0");
    let views = first_i64(raw, ".tweet.views // 0");
    let media_count = first_i64(raw, "((.tweet.media.all // []) | length)");
    let media_kinds = media_kind_summary(raw);
    let sensitive = field(raw, ".tweet.possibly_sensitive // false") == "true";
    let raw_json = field_json(raw, ".tweet");
    let sql = format!(
        "INSERT OR IGNORE INTO observed_posts(post_id,source_handle,status_url,text,created_at,created_timestamp,likes,replies,reposts,quotes,views,media_count,media_kinds,sensitive,raw_json,observed_at) VALUES ({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{});",
        sql_quote(&post_id),
        sql_quote(source_handle),
        sql_quote(&status_url),
        sql_quote(&text),
        sql_quote(&created_at),
        created_timestamp,
        likes,
        replies,
        reposts,
        quotes,
        views,
        media_count,
        sql_quote(&media_kinds),
        if sensitive { 1 } else { 0 },
        sql_quote(&raw_json),
        timestamp()
    );
    let existed = db_query(&format!(
        "SELECT COUNT(*) FROM observed_posts WHERE post_id={};",
        sql_quote(&post_id)
    ))?
    .trim()
    .parse::<i64>()
    .unwrap_or(0)
        > 0;
    db_exec(&sql)?;
    if existed {
        return Ok(false);
    }
    let age_minutes = (timestamp() - created_timestamp).max(0) / 60;
    let (score, reason) = score_post(
        likes,
        replies,
        reposts,
        quotes,
        views,
        age_minutes,
        media_count,
        sensitive,
    );
    let opportunity = format!(
        "INSERT OR REPLACE INTO opportunities(post_id,score,score_reason,state,updated_at) VALUES ({},{},{},'candidate',{});",
        sql_quote(&post_id),
        score,
        sql_quote(&reason),
        timestamp()
    );
    db_exec(&opportunity)?;
    let media_items = jq_raw(&raw_json, ".media.all[]?").unwrap_or_default();
    for item in media_items.lines() {
        insert_media(&post_id, item)?;
    }
    Ok(true)
}

fn source_watermark(handle: &str) -> i64 {
    db_query(&format!(
        "SELECT COALESCE(MAX(created_timestamp),0) FROM observed_posts WHERE source_handle={};",
        sql_quote(handle)
    ))
    .ok()
    .and_then(|value| value.trim().parse::<i64>().ok())
    .unwrap_or(0)
}

fn scan() -> Result<String, String> {
    init_db()?;
    let lock_path = state_root().join("scan.lock");
    let _lock_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .map_err(|error| format!("scan lock {}: {error}", lock_path.display()))?;
    let _lock = Lock(lock_path);
    let sources = load_sources()?;
    let started = timestamp();
    let mut seen = 0i64;
    let mut new_posts = 0i64;
    let mut errors = Vec::new();
    for source in &sources {
        let since = source_watermark(&source.handle);
        match fetch_profile_statuses(source, since) {
            Ok(raw) => {
                if let Err(error) = upsert_source(source, &raw) {
                    errors.push(format!("{} source metadata: {error}", source.handle));
                }
                let results =
                    jq_raw(&raw, ".results[]? | select(.type == \"status\")").unwrap_or_default();
                let mut source_seen = 0usize;
                for encoded in results.lines() {
                    if source_seen >= source.max_posts {
                        break;
                    }
                    source_seen += 1;
                    let item = encoded.trim().to_string();
                    if item.is_empty() {
                        continue;
                    }
                    seen += 1;
                    // The profile response already contains the complete status and
                    // media object. Hydrate only explicit/manual status URLs so a
                    // 50-account scan does not turn into hundreds of duplicate
                    // requests every three minutes.
                    match ingest_status(source, &format!("{{\"tweet\":{item}}}")) {
                        Ok(true) => new_posts += 1,
                        Ok(false) => {}
                        Err(error) => errors.push(format!("{}: {error}", source.handle)),
                    }
                }
            }
            Err(error) => errors.push(format!("{} feed: {error}", source.handle)),
        }
    }
    let status = if errors.is_empty() { "ok" } else { "partial" };
    let error_text = errors.join(" | ");
    let sql = format!(
        "INSERT INTO runs(started_at,finished_at,source_count,posts_seen,posts_new,errors,status) VALUES ({},{},{},{},{},{},{});",
        started,
        timestamp(),
        sources.len(),
        seen,
        new_posts,
        sql_quote(&error_text),
        sql_quote(status)
    );
    db_exec(&sql)?;
    Ok(format!(
        "{{\"status\":{},\"sourceCount\":{},\"postsSeen\":{},\"postsNew\":{},\"errors\":{}}}",
        json_quote(status),
        sources.len(),
        seen,
        new_posts,
        json_quote(&error_text)
    ))
}

// FxTwitter's profile endpoint returns a result envelope. Keep only the tweet
// object before feeding the common ingest path so a status URL and a profile
// timeline use exactly the same provenance schema.
fn detail_field(raw: &str) -> String {
    field_json(raw, ".tweet // .status // .results[0] // null")
}

fn host_allowed(url: &str) -> bool {
    url.strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .map(|host| host == "video.twimg.com" || host == "pbs.twimg.com")
        .unwrap_or(false)
}

fn media_limit(url: &str) -> u64 {
    if url.contains("video.twimg.com") {
        MAX_VIDEO_BYTES
    } else {
        MAX_IMAGE_BYTES
    }
}

fn media_extension(url: &str) -> &'static str {
    if url.contains("video.twimg.com") {
        return "mp4";
    }
    if let Some(format) = url.split('?').nth(1).and_then(|query| {
        query
            .split('&')
            .find_map(|part| part.strip_prefix("format="))
    }) {
        match format.to_ascii_lowercase().as_str() {
            "png" => return "png",
            "webp" => return "webp",
            "jpg" | "jpeg" => return "jpg",
            _ => {}
        }
    }
    match url
        .split('?')
        .next()
        .unwrap_or(url)
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "png",
        "webp" => "webp",
        _ => "jpg",
    }
}

fn first_field(raw: &str, filter: &str) -> String {
    field(raw, filter).lines().next().unwrap_or("").to_string()
}

fn frame_rate(value: &str) -> Option<f64> {
    if let Some((numerator, denominator)) = value.split_once('/') {
        let numerator = numerator.parse::<f64>().ok()?;
        let denominator = denominator.parse::<f64>().ok()?;
        return (denominator > 0.0).then_some(numerator / denominator);
    }
    value.parse::<f64>().ok()
}

fn validate_media_file(path: &str, extension: &str) -> Result<String, String> {
    let mime = run("file", &["--brief", "--mime-type", path], None)?
        .trim()
        .to_string();
    if extension == "mp4" {
        if mime != "video/mp4" {
            return Err(format!("downloaded video has unsupported MIME {mime}"));
        }
        let probe = run(
            "ffprobe",
            &[
                "-v",
                "error",
                "-show_entries",
                "format=duration:stream=codec_type,codec_name,width,height,r_frame_rate,pix_fmt,profile",
                "-of",
                "json",
                path,
            ],
            None,
        )?;
        let video_codec = first_field(
            &probe,
            ".streams[]? | select(.codec_type == \"video\") | .codec_name // \"\"",
        );
        if video_codec != "h264" {
            return Err(format!("video codec {video_codec:?} is not H.264"));
        }
        let duration = first_f64(&probe, ".format.duration // 0");
        if !(0.5..=140.0).contains(&duration) {
            return Err(format!("video duration {duration:.3}s is outside 0.5-140s"));
        }
        let fps_text = first_field(
            &probe,
            ".streams[]? | select(.codec_type == \"video\") | .r_frame_rate // \"\"",
        );
        let fps = frame_rate(&fps_text).unwrap_or(0.0);
        if fps > 60.0 {
            return Err(format!("video frame rate {fps:.2}fps exceeds 60fps"));
        }
        let pixel_format = first_field(
            &probe,
            ".streams[]? | select(.codec_type == \"video\") | .pix_fmt // \"\"",
        );
        if !matches!(pixel_format.as_str(), "yuv420p" | "yuvj420p") {
            return Err(format!("video pixel format {pixel_format:?} is not 4:2:0"));
        }
        let audio_codecs = field(
            &probe,
            "[.streams[]? | select(.codec_type == \"audio\") | .codec_name // \"\"] | unique | join(\",\")",
        );
        if !audio_codecs.is_empty() && audio_codecs != "aac" {
            return Err(format!("audio codec {audio_codecs:?} is not AAC"));
        }
        Ok(probe)
    } else {
        let expected = match extension {
            "png" => "image/png",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };
        if mime != expected {
            return Err(format!(
                "downloaded image has MIME {mime}, expected {expected}"
            ));
        }
        Ok("{}".to_string())
    }
}

fn prepare_media(url: &str) -> Result<String, String> {
    if !host_allowed(url) {
        return Err("media host is not in the FxTwitter media allowlist".to_string());
    }
    let cache = cache_root();
    fs::create_dir_all(&cache).map_err(|error| error.to_string())?;
    let temporary = cache.join(format!(
        ".download.{}.{}.part",
        std::process::id(),
        timestamp()
    ));
    let temporary_string = temporary.to_str().ok_or("temporary path is not UTF-8")?;
    let limit = media_limit(url).to_string();
    let url_static = url.to_string();
    if let Err(error) = run(
        "curl",
        &[
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "120",
            "--max-filesize",
            &limit,
            "--user-agent",
            USER_AGENT,
            "--output",
            temporary_string,
            &url_static,
        ],
        None,
    ) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    let bytes = fs::metadata(&temporary)
        .map_err(|error| error.to_string())?
        .len();
    if bytes == 0 || bytes > media_limit(url) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("media size {bytes} is outside the accepted limit"));
    }
    let hash = match run("sha256sum", &[temporary_string], None) {
        Ok(value) => value.split_whitespace().next().unwrap_or("").to_string(),
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
    };
    if hash.len() != 64 {
        let _ = fs::remove_file(&temporary);
        return Err("sha256sum did not return a valid digest".to_string());
    }
    let extension = media_extension(url);
    let probe = match validate_media_file(temporary_string, extension) {
        Ok(probe) => probe,
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
    };
    let final_path = cache.join(format!("{hash}.{extension}"));
    if !final_path.exists() {
        fs::rename(&temporary, &final_path).map_err(|error| error.to_string())?;
    } else {
        let _ = fs::remove_file(&temporary);
    }
    let final_string = final_path.to_string_lossy();
    Ok(format!(
        "{{\"path\":{},\"sha256\":{},\"bytes\":{},\"format\":{},\"probe\":{}}}",
        json_quote(&final_string),
        json_quote(&hash),
        bytes,
        json_quote(extension),
        probe
    ))
}

fn media_plan(post_id: &str) -> Result<String, String> {
    init_db()?;
    let source_handle = db_query(&format!(
        "SELECT source_handle FROM observed_posts WHERE post_id={};",
        sql_quote(post_id)
    ))?
    .trim()
    .to_string();
    let rights_status = db_query(&format!(
        "SELECT COALESCE(rights_status,'unknown') FROM sources WHERE handle={};",
        sql_quote(&source_handle)
    ))
    .ok()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| "unknown".to_string());
    let raw = db_query(&format!(
        "SELECT raw_json FROM observed_posts WHERE post_id={};",
        sql_quote(post_id)
    ))?;
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(format!("post {post_id} is not in the observed state"));
    }
    let candidates = jq(
        raw,
        r#"
        ([.media.photos[]? | {
          id: (.id // ""),
          kind: "photo",
          url: (.url // ""),
          publisher: (.publisher.screen_name // ""),
          width: (.width // 0),
          height: (.height // 0),
          duration: null,
          bitrate: null,
          quality: (((.width // 0) * (.height // 0)) | tonumber)
        }])
        +
        ([.media.videos[]? as $video
          | [($video.formats // [])[]
             | select((.container // "") == "mp4")
             | select((.codec // "") == "h264")
             | select((.url // "") | startswith("https://video.twimg.com/"))]
          | sort_by((.bitrate // 0)) | reverse | .[0] as $format
          | select($format != null)
          | {
              id: ($video.id // ""),
              kind: "video",
              url: $format.url,
              publisher: ($video.publisher.screen_name // ""),
              width: ($video.width // 0),
              height: ($video.height // 0),
              duration: ($video.duration // 0),
              bitrate: ($format.bitrate // 0),
              quality: (($format.bitrate // 0) + ((($video.width // 0) * ($video.height // 0)) / 1000))
            }
        ])
        | map(select(.url != ""))
        | sort_by(.quality) | reverse
        | {candidates: ., recommended: (.[0] // null)}
        "#,
    )?;
    Ok(format!(
        "{{\"postId\":{},\"source\":{},\"rightsStatus\":{},\"requiresManualReview\":{},\"plan\":{}}}",
        json_quote(post_id),
        json_quote(&source_handle),
        json_quote(&rights_status),
        if rights_status == "cleared" { "false" } else { "true" },
        candidates
    ))
}

fn status_json() -> Result<String, String> {
    init_db()?;
    let config = config_path();
    let configured_sources = load_sources().map(|sources| sources.len()).unwrap_or(0);
    let sql = "SELECT json_object(
      'sourcesConfigured',%CONFIGURED%,
      'sourcesObserved',(SELECT COUNT(*) FROM sources),
      'followersTotal',(SELECT SUM(followers) FROM sources),
      'postsObserved',(SELECT COUNT(*) FROM observed_posts),
      'postsLast24h',(SELECT COUNT(*) FROM observed_posts WHERE observed_at >= strftime('%s','now')-86400),
      'mediaAssets',(SELECT COUNT(*) FROM media_assets),
      'opportunities',(SELECT COUNT(*) FROM opportunities WHERE state='candidate'),
      'recentPosts',json((SELECT COALESCE(json_group_array(json_object(
        'postId',post_id,
        'source',source_handle,
        'url',status_url,
        'text',text,
        'media',media_kinds,
        'score',COALESCE(score,0),
        'observedAt',observed_at
      )), '[]') FROM (
        SELECT observed_posts.*, opportunities.score
        FROM observed_posts
        LEFT JOIN opportunities ON opportunities.post_id=observed_posts.post_id
        ORDER BY observed_posts.observed_at DESC
        LIMIT 20
      ))),
      'lastRun',(SELECT json_object('status',status,'finishedAt',finished_at,'sourceCount',source_count,'postsSeen',posts_seen,'postsNew',posts_new,'errors',errors) FROM runs ORDER BY id DESC LIMIT 1)
    );"
    .replace("%CONFIGURED%", &configured_sources.to_string());
    let summary = db_query(&sql)?.trim().to_string();
    let summary = if summary.is_empty() {
        "{}".to_string()
    } else {
        summary
    };
    Ok(format!(
        "{{\"configPath\":{},\"statePath\":{},\"cachePath\":{},\"summary\":{}}}",
        json_quote(&config.to_string_lossy()),
        json_quote(&db_path().to_string_lossy()),
        json_quote(&cache_root().to_string_lossy()),
        summary
    ))
}

fn manual_status(url: &str) -> Result<String, String> {
    let parts = url.split('/').collect::<Vec<_>>();
    let status_index = parts
        .iter()
        .position(|part| *part == "status")
        .ok_or("URL has no /status/<id>")?;
    let handle = parts
        .get(status_index.wrapping_sub(1))
        .ok_or("URL has no author")?;
    let id = parts
        .get(status_index + 1)
        .ok_or("URL has no status ID")?
        .split('?')
        .next()
        .unwrap_or("");
    if !valid_handle(handle) || id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return Err("unsupported X status URL".to_string());
    }
    let source = Source {
        handle: handle.to_string(),
        max_posts: 20,
        rights_status: "unknown".to_string(),
        ideology: "manual".to_string(),
        tone: "manual".to_string(),
    };
    init_db()?;
    let raw = fetch_status(handle, id)?;
    let tweet = detail_field(&raw);
    let inserted = ingest_status(&source, &format!("{{\"tweet\":{tweet}}}"))?;
    Ok(format!(
        "{{\"status\":{},\"postId\":{},\"sourceUrl\":{}}}",
        json_quote(if inserted { "observed" } else { "duplicate" }),
        json_quote(id),
        json_quote(url)
    ))
}

fn usage() {
    eprintln!("vesper-xpatla scan|status [--json]|sources|manual <x-status-url>|media-plan <post-id>|prepare-media <fx-media-url>");
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("scan") => scan(),
        Some("status") | None => status_json(),
        Some("sources") => load_sources().map(|sources| {
            format!(
                "{{\"count\":{},\"handles\":[{}]}}",
                sources.len(),
                sources
                    .iter()
                    .map(|source| json_quote(&source.handle))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }),
        Some("manual") => args
            .get(1)
            .ok_or_else(|| "manual requires a status URL".to_string())
            .and_then(|url| manual_status(url)),
        Some("media-plan") => args
            .get(1)
            .ok_or_else(|| "media-plan requires a post ID".to_string())
            .and_then(|post_id| media_plan(post_id)),
        Some("prepare-media") => args
            .get(1)
            .ok_or_else(|| "prepare-media requires a media URL".to_string())
            .and_then(|url| prepare_media(url)),
        _ => {
            usage();
            Err("unknown command".to_string())
        }
    };
    match result {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("vesper-xpatla: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_are_dynamic_but_validated() {
        assert!(valid_handle("bpthaber"));
        assert!(valid_handle("ZamHaberAjans"));
        assert!(!valid_handle("@bpthaber"));
        assert!(!valid_handle("bad-handle"));
    }

    #[test]
    fn source_config_count_follows_enabled_entries() {
        let sources = parse_sources(
            r#"{"sources":[
                {"handle":"bpthaber","enabled":true},
                {"handle":"ntv","enabled":false},
                {"handle":"Medyascope"}
            ]}"#,
        )
        .expect("source config should parse");
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].handle, "bpthaber");
        assert_eq!(sources[1].handle, "Medyascope");
    }

    #[test]
    fn scoring_penalises_sensitive_media() {
        let (safe, _) = score_post(100, 5, 2, 1, 1000, 20, 1, false);
        let (sensitive, _) = score_post(100, 5, 2, 1, 1000, 20, 1, true);
        assert!(safe > 0.0);
        assert_eq!(sensitive, 0.0);
    }

    #[test]
    fn media_allowlist_is_narrow() {
        assert!(host_allowed("https://video.twimg.com/a.mp4"));
        assert!(host_allowed("https://pbs.twimg.com/media/a.jpg"));
        assert!(!host_allowed("https://example.invalid/a.mp4"));
        assert!(!host_allowed("http://video.twimg.com/a.mp4"));
    }

    #[test]
    fn media_formats_and_frame_rates_are_conservative() {
        assert_eq!(
            media_extension("https://pbs.twimg.com/media/a.png?name=small"),
            "png"
        );
        assert_eq!(
            media_extension("https://pbs.twimg.com/media/a?format=webp&name=small"),
            "webp"
        );
        assert_eq!(
            media_extension("https://pbs.twimg.com/media/a.webp"),
            "webp"
        );
        assert_eq!(media_extension("https://video.twimg.com/a.mp4"), "mp4");
        assert_eq!(frame_rate("30000/1001").unwrap_or_default().round(), 30.0);
        assert!(frame_rate("not-a-rate").is_none());
    }

    #[test]
    fn sql_quote_handles_news_apostrophes() {
        assert_eq!(sql_quote("İzmir'de"), "'İzmir''de'");
    }
}
