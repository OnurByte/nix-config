use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const LISTEN_ADDR: &str = "127.0.0.1:3210";
const LOOPBACK_ORIGIN: &str = "http://127.0.0.1:3210";
const DREAD_URL: &str = "https://dreadytofatroptsdj6io7l3xptbet6onoyno2yv7jicoxknyazubrad.onion";
const PITCH_URL: &str = "https://pitchzzzoot5i4cpsblu2d5poifsyixo5r4litxkukstre5lrbjakxid.onion/";
const MAX_REQUEST_BYTES: usize = 16 * 1024;
const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;
const MAX_STATIC_BYTES: u64 = 8 * 1024 * 1024;

const HELIUM_SQL: &str = r#"
WITH recent AS (
  SELECT
    urls.title AS title,
    urls.url AS url,
    ((visits.visit_time / 1000) - 11644473600000) AS visited_at
  FROM visits
  JOIN urls ON urls.id = visits.url
  WHERE (lower(urls.url) LIKE 'http://%' OR lower(urls.url) LIKE 'https://%')
    AND lower(urls.url) NOT LIKE '%.onion%'
  ORDER BY visits.visit_time DESC
  LIMIT 16
), stats AS (
  SELECT
    (SELECT count(*) FROM visits
      JOIN urls ON urls.id = visits.url
      WHERE (lower(urls.url) LIKE 'http://%' OR lower(urls.url) LIKE 'https://%')
        AND lower(urls.url) NOT LIKE '%.onion%') AS total_visits,
    (SELECT count(DISTINCT urls.url) FROM urls
      WHERE (lower(urls.url) LIKE 'http://%' OR lower(urls.url) LIKE 'https://%')
        AND lower(urls.url) NOT LIKE '%.onion%') AS unique_urls,
    (SELECT count(*) FROM visits
      JOIN urls ON urls.id = visits.url
      WHERE (lower(urls.url) LIKE 'http://%' OR lower(urls.url) LIKE 'https://%')
        AND lower(urls.url) NOT LIKE '%.onion%'
        AND date((visits.visit_time / 1000000) - 11644473600, 'unixepoch', 'localtime')
          = date('now', 'localtime')) AS today_visits
)
SELECT json_object(
  'available', 1,
  'stats', json_object(
    'totalVisits', stats.total_visits,
    'uniqueUrls', stats.unique_urls,
    'todayVisits', stats.today_visits
  ),
  'items', COALESCE((SELECT json_group_array(json_object(
    'title', COALESCE(recent.title, recent.url),
    'url', recent.url,
    'visitedAt', datetime(recent.visited_at / 1000, 'unixepoch') || 'Z'
  )) FROM recent), json('[]'))
) AS result
FROM stats;
"#;

const ZEN_SQL: &str = r#"
WITH recent AS (
  SELECT
    moz_places.title AS title,
    moz_places.url AS url,
    moz_historyvisits.visit_date AS visited_at
  FROM moz_historyvisits
  JOIN moz_places ON moz_places.id = moz_historyvisits.place_id
  WHERE (lower(moz_places.url) LIKE 'http://%' OR lower(moz_places.url) LIKE 'https://%')
    AND lower(moz_places.url) NOT LIKE '%.onion%'
  ORDER BY moz_historyvisits.visit_date DESC
  LIMIT 16
), stats AS (
  SELECT
    (SELECT count(*) FROM moz_historyvisits
      JOIN moz_places ON moz_places.id = moz_historyvisits.place_id
      WHERE (lower(moz_places.url) LIKE 'http://%' OR lower(moz_places.url) LIKE 'https://%')
        AND lower(moz_places.url) NOT LIKE '%.onion%') AS total_visits,
    (SELECT count(DISTINCT moz_places.url) FROM moz_places
      WHERE (lower(moz_places.url) LIKE 'http://%' OR lower(moz_places.url) LIKE 'https://%')
        AND lower(moz_places.url) NOT LIKE '%.onion%') AS unique_urls,
    (SELECT count(*) FROM moz_historyvisits
      JOIN moz_places ON moz_places.id = moz_historyvisits.place_id
      WHERE (lower(moz_places.url) LIKE 'http://%' OR lower(moz_places.url) LIKE 'https://%')
        AND lower(moz_places.url) NOT LIKE '%.onion%'
        AND date(moz_historyvisits.visit_date / 1000000, 'unixepoch', 'localtime')
        = date('now', 'localtime')) AS today_visits
)
SELECT json_object(
  'available', 1,
  'stats', json_object(
    'totalVisits', stats.total_visits,
    'uniqueUrls', stats.unique_urls,
    'todayVisits', stats.today_visits
  ),
  'items', COALESCE((SELECT json_group_array(json_object(
    'title', COALESCE(recent.title, recent.url),
    'url', recent.url,
    'visitedAt', datetime(recent.visited_at / 1000000, 'unixepoch') || 'Z'
  )) FROM recent), json('[]'))
) AS result
FROM stats;
"#;

const MERGE_HISTORY_JQ: &str = r#"
reduce .[] as $item
  ({available: false, stats: {totalVisits: 0, uniqueUrls: 0, todayVisits: 0}, items: []};
    .available = (.available or ($item.available // false))
    | .stats.totalVisits += ($item.stats.totalVisits // 0)
    | .stats.uniqueUrls += ($item.stats.uniqueUrls // 0)
    | .stats.todayVisits += ($item.stats.todayVisits // 0)
    | .items += ($item.items // []))
| .items |= sort_by(.visitedAt) | .items |= reverse | .items |= .[:16]
"#;

const SHORTCUTS_JQ: &str = r#"
.custom_links.list // []
| map(select((.url // "") | type == "string"))
| map(select((.url | test("^https?://"; "i")) and ((.url | test("\\.onion"; "i")) | not)))
| map({
    title: ((.title // .url) | tostring),
    url: .url,
    domain: (.url | sub("^https?://"; "") | split("/")[0] | split(":")[0])
  })
"#;

const HERMES_JQ: &str = r#"
.[0] as $index
| .[1] as $registry
| (if ($index | type) == "array" then $index else ($index.briefings // []) end) as $briefings
| ($registry.sources // {}) as $source_map
| ($source_map | to_entries | map(.value + {id: .key})) as $sources
| {
    available: true,
    stats: {
      totalBriefings: ($briefings | length),
      unread: ([$briefings[]? | select((.unread // false) == true)] | length),
      highPriority: ([$briefings[]? | select(((.priority // "") | tostring | ascii_downcase) == "high")] | length),
      sourceCount: ($sources | length)
    },
    briefings: ([$briefings[]?
      | {
          id: ((.id // .slug // .title // "briefing") | tostring),
          lane: ((.lane // .topic // "general") | tostring),
          title: ((.title // "Untitled briefing") | tostring),
          summary: ((.summary // .abstract // .description // "") | tostring),
          priority: ((.priority // "normal") | tostring),
          confidence: ((.confidence // "unknown") | tostring),
          createdAt: ((.createdAt // .created_at // .updatedAt // "") | tostring),
          unread: ((.unread // false) == true),
          sourceCount: ((.sources // []) | length)
        }
    ] | sort_by(.createdAt) | reverse | .[:6]),
    tiers: ($sources
      | map({tier: ((.tier // "unknown") | tostring)})
      | group_by(.tier)
      | map({tier: .[0].tier, count: length})),
    coverage: (reduce ($briefings[]? | (.coverage // {}) | to_entries[]) as $entry
      ({}; .[$entry.key] = ((.[$entry.key] // 0) + ($entry.value // 0))))
  }
"#;

const TOR_JQ: &str = r#"
.[0] as $registry
| ($registry.sources // {})
| to_entries
| map(
    . as $entry
    | ($entry.value.url // $entry.key) as $url
    | select(($url | test("^https?://[a-z2-7]{56}\\.onion(?:/.*)?$")))
    | ($url | capture("^(?<scheme>https?)://(?<host>[a-z2-7]{56}\\.onion)")) as $match
    | {
        id: ($entry.key | tostring),
        url: $url,
        name: (($entry.value.title // $entry.value.name // "Hermes onion") | tostring),
        source: "hermes",
        tier: (($entry.value.tier // "unknown") | tostring),
        hits: ($entry.value.hits // null),
        lastUseful: (($entry.value.lastUseful // $entry.value.last_useful // "") | tostring),
        displayHost: $match.host
      }
  ) as $dynamic
| [
    {id: "dread", url: $dread, name: "Dread", source: "pinned", displayHost: ("dreadytofatroptsdj6io7l3xptbet6onoyno2yv7jicoxknyazubrad.onion")},
    {id: "pitch", url: $pitch, name: "Pitch", source: "pinned", displayHost: ("pitchzzzoot5i4cpsblu2d5poifsyixo5r4litxkukstre5lrbjakxid.onion")}
  ] as $static
| ($dynamic + $static)
| group_by(.url)
| map(reduce .[] as $item ({}; . * $item))
| map(del(.url))
| {available: true, items: .}
"#;

#[derive(Clone, Debug)]
struct Config {
    web_root: PathBuf,
    helium_history: PathBuf,
    helium_preferences: PathBuf,
    zen_histories: Vec<PathBuf>,
    briefings_index: PathBuf,
    source_registry: PathBuf,
    tor_browser: PathBuf,
}

#[derive(Debug)]
struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
}

fn main() {
    let config = Arc::new(parse_args());
    let listener = socket_activated_listener()
        .unwrap_or_else(|| TcpListener::bind(LISTEN_ADDR).expect("bind startpage listener"));
    eprintln!("vesper-startpage listening on {LISTEN_ADDR}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config = Arc::clone(&config);
                thread::spawn(move || handle_stream(stream, &config));
            }
            Err(error) => eprintln!("startpage accept failed: {error}"),
        }
    }
}

fn socket_activated_listener() -> Option<TcpListener> {
    let pid = env::var("LISTEN_PID").ok()?.parse::<u32>().ok()?;
    let fds = env::var("LISTEN_FDS").ok()?.parse::<u32>().ok()?;
    if pid != process::id() || fds != 1 {
        return None;
    }
    env::remove_var("LISTEN_PID");
    env::remove_var("LISTEN_FDS");
    env::remove_var("LISTEN_FDNAMES");
    // systemd passes the first Accept=no stream socket at fd 3.
    Some(unsafe { TcpListener::from_raw_fd(3) })
}

fn parse_args() -> Config {
    let mut args = env::args().skip(1);
    let mut config = Config {
        web_root: PathBuf::from("."),
        helium_history: PathBuf::new(),
        helium_preferences: PathBuf::new(),
        zen_histories: Vec::new(),
        briefings_index: PathBuf::new(),
        source_registry: PathBuf::new(),
        tor_browser: PathBuf::from("tor-browser"),
    };

    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .unwrap_or_else(|| panic!("missing value for {flag}"))
        };
        match flag.as_str() {
            "--web-root" => config.web_root = PathBuf::from(value()),
            "--helium-history" => config.helium_history = PathBuf::from(value()),
            "--helium-preferences" => config.helium_preferences = PathBuf::from(value()),
            "--zen-history" => config.zen_histories.push(PathBuf::from(value())),
            "--briefings-index" => config.briefings_index = PathBuf::from(value()),
            "--source-registry" => config.source_registry = PathBuf::from(value()),
            "--tor-browser" => config.tor_browser = PathBuf::from(value()),
            other => panic!("unknown argument {other}"),
        }
    }

    config
}

fn handle_stream(mut stream: TcpStream, config: &Config) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let response = match read_request(&mut stream).and_then(|request| route(request, config)) {
        Ok(response) => response,
        Err((status, body)) => response(status, "application/json; charset=utf-8", body.as_bytes()),
    };
    let _ = stream.write_all(&response);
}

fn read_request(stream: &mut TcpStream) -> Result<Request, (u16, String)> {
    let mut buffer = Vec::with_capacity(2048);
    let mut chunk = [0_u8; 2048];
    let end = loop {
        let read = stream
            .read(&mut chunk)
            .map_err(|_| (408, error_json("timeout")))?;
        if read == 0 {
            return Err((400, error_json("empty request")));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.len() > MAX_REQUEST_BYTES {
            return Err((413, error_json("request too large")));
        }
        if let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break end;
        }
    };

    let head = String::from_utf8(buffer[..end].to_vec())
        .map_err(|_| (400, error_json("invalid request")))?;
    let mut lines = head.split("\r\n");
    let request_line = lines.next().ok_or((400, error_json("invalid request")))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let target = request_parts.next().unwrap_or_default().to_owned();
    let version = request_parts.next().unwrap_or_default();
    if target.is_empty() || version != "HTTP/1.1" || request_parts.next().is_some() {
        return Err((400, error_json("invalid request")));
    }

    let mut headers = BTreeMap::new();
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err((400, error_json("invalid headers")));
        };
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }

    Ok(Request {
        method,
        target,
        headers,
    })
}

fn route(request: Request, config: &Config) -> Result<Vec<u8>, (u16, String)> {
    if request.headers.get("host").map(String::as_str) != Some(LISTEN_ADDR) {
        return Err((421, error_json("loopback host required")));
    }
    if let Some(length) = request.headers.get("content-length") {
        if length.parse::<u64>().unwrap_or(1) != 0 {
            return Err((413, error_json("request body is not accepted")));
        }
    }
    if request.headers.contains_key("transfer-encoding") {
        return Err((413, error_json("request body is not accepted")));
    }

    let path = request.target.split('?').next().unwrap_or_default();
    if request.method == "POST" {
        if request.headers.get("origin").map(String::as_str) != Some(LOOPBACK_ORIGIN) {
            return Err((403, error_json("loopback origin required")));
        }
        if let Some(id) = path.strip_prefix("/api/tor/open/") {
            let id = percent_decode(id).ok_or((400, error_json("invalid tor id")))?;
            return open_tor(&id, config);
        }
        return Err((405, error_json("method not allowed")));
    }
    if request.method != "GET" {
        return Err((405, error_json("method not allowed")));
    }

    match path {
        "/health" => Ok(response(
            200,
            "application/json; charset=utf-8",
            br#"{"ok":true}"#,
        )),
        "/api/history/helium" => Ok(json_response(history_json(&[(
            &config.helium_history,
            "Helium",
            HELIUM_SQL,
        )]))),
        "/api/history/zen" => {
            let paths = config
                .zen_histories
                .iter()
                .map(|path| (path, "Zen", ZEN_SQL))
                .collect::<Vec<_>>();
            Ok(json_response(history_json(&paths)))
        }
        "/api/shortcuts" => Ok(json_response(shortcuts_json(config))),
        "/api/hermes" => Ok(json_response(hermes_json(config))),
        "/api/tor" => Ok(json_response(tor_json(config))),
        _ => static_response(config, path),
    }
}

fn static_response(config: &Config, path: &str) -> Result<Vec<u8>, (u16, String)> {
    if path.contains("..") || !path.starts_with('/') {
        return Err((404, error_json("not found")));
    }
    let relative = if path == "/" {
        "index.html"
    } else {
        &path[1..]
    };
    let candidate = config.web_root.join(relative);
    let root = fs::canonicalize(&config.web_root).map_err(|_| (404, error_json("not found")))?;
    let file = fs::canonicalize(candidate).map_err(|_| (404, error_json("not found")))?;
    if !file.starts_with(&root) || !file.is_file() {
        return Err((404, error_json("not found")));
    }
    let metadata = fs::metadata(&file).map_err(|_| (404, error_json("not found")))?;
    if metadata.len() > MAX_STATIC_BYTES {
        return Err((413, error_json("file too large")));
    }
    let body = fs::read(file).map_err(|_| (404, error_json("not found")))?;
    Ok(response(200, content_type(relative), &body))
}

fn history_json(paths: &[(&PathBuf, &str, &str)]) -> String {
    // ponytail: profile stats are summed across readable databases; use a
    // cross-database URL set only if multiple active Zen profiles become real.
    let mut documents = Vec::new();
    let mut existing_database = false;
    for (path, browser, sql) in paths {
        existing_database |= path.is_file();
        if let Some(document) = history_for_db(path, browser, sql) {
            documents.push(document);
        }
    }
    if documents.is_empty() {
        return unavailable_history_json(if existing_database {
            "unreadable"
        } else {
            "missing"
        });
    }
    let input = format!("[{}]", documents.join(","));
    run_jq(MERGE_HISTORY_JQ, &input, &[]).unwrap_or_else(|| unavailable_history_json("invalid"))
}

fn history_for_db(path: &Path, browser: &str, sql: &str) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let database = format!("file:{}?immutable=1", path.display());
    let output = Command::new("sqlite3")
        .args(["-readonly", "-json", "-batch", &database, sql])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sqlite_json = String::from_utf8(output.stdout).ok()?;
    run_jq(
        r#".[0].result | fromjson | .items |= map(. + {browser: $browser, domain: ((.url | sub("^https?://"; "") | split("/")[0] | split(":")[0]))})"#,
        &sqlite_json,
        &[("browser", browser)],
    )
}

fn hermes_json(config: &Config) -> String {
    let (index_present, index_valid, index) = read_json(&config.briefings_index, "[]");
    let (registry_present, registry_valid, registry) =
        read_json(&config.source_registry, r#"{"sources":{}}"#);
    if !index_present && !registry_present {
        return unavailable_hermes_json("missing");
    }
    if !index_valid || !registry_valid {
        return unavailable_hermes_json("unreadable");
    }
    let input = format!("[{},{}]", index, registry);
    run_jq(HERMES_JQ, &input, &[]).unwrap_or_else(|| unavailable_hermes_json("invalid"))
}

fn shortcuts_json(config: &Config) -> String {
    let (present, valid, preferences) = read_json(&config.helium_preferences, "{}");
    if !present {
        return unavailable_shortcuts_json("missing");
    }
    if !valid {
        return unavailable_shortcuts_json("unreadable");
    }
    let Some(items) = run_jq(SHORTCUTS_JQ, &preferences, &[]) else {
        return unavailable_shortcuts_json("invalid");
    };
    format!(r#"{{"available":true,"items":{items}}}"#)
}

fn tor_json(config: &Config) -> String {
    let (_, _, registry) = read_json(&config.source_registry, r#"{"sources":{}}"#);
    run_jq(
        TOR_JQ,
        &format!("[{}]", registry),
        &[("dread", DREAD_URL), ("pitch", PITCH_URL)],
    )
    .unwrap_or_else(|| unavailable_tor_json("invalid"))
}

fn open_tor(id: &str, config: &Config) -> Result<Vec<u8>, (u16, String)> {
    let url = match id {
        "dread" => Some(DREAD_URL.to_owned()),
        "pitch" => Some(PITCH_URL.to_owned()),
        _ => resolve_registry_url(id, config),
    };
    let Some(url) = url else {
        return Err((404, error_json("tor link not found")));
    };
    if !valid_onion_url(&url) {
        return Err((400, error_json("invalid tor link")));
    }
    Command::new(&config.tor_browser)
        .arg(&url)
        .spawn()
        .map_err(|_| (503, error_json("tor browser unavailable")))?;
    Ok(response(204, "application/json; charset=utf-8", b""))
}

fn resolve_registry_url(id: &str, config: &Config) -> Option<String> {
    if !config.source_registry.is_file() {
        return None;
    }
    let registry = fs::read_to_string(&config.source_registry).ok()?;
    run_jq_raw(
        r#"if (.sources // {}) | has($id) then (.sources[$id].url // $id) else empty end"#,
        &registry,
        &[("id", id)],
    )
    .and_then(|value| value.lines().next().map(str::to_owned))
}

fn read_json(path: &Path, fallback: &str) -> (bool, bool, String) {
    if !path.is_file() {
        return (false, true, fallback.to_owned());
    }
    match fs::metadata(path).ok().map(|metadata| metadata.len()) {
        Some(size) if size <= MAX_JSON_BYTES as u64 => {}
        _ => return (true, false, fallback.to_owned()),
    }
    let Ok(content) = fs::read_to_string(path) else {
        return (true, false, fallback.to_owned());
    };
    if run_jq(".", &content, &[]).is_some() {
        (true, true, content)
    } else {
        (true, false, fallback.to_owned())
    }
}

fn run_jq(filter: &str, input: &str, args: &[(&str, &str)]) -> Option<String> {
    run_jq_with_mode(filter, input, args, false)
}

fn run_jq_raw(filter: &str, input: &str, args: &[(&str, &str)]) -> Option<String> {
    run_jq_with_mode(filter, input, args, true)
}

fn run_jq_with_mode(filter: &str, input: &str, args: &[(&str, &str)], raw: bool) -> Option<String> {
    let mut command = Command::new("jq");
    command.arg(if raw { "-r" } else { "-c" });
    for (name, value) in args {
        command.args(["--arg", name, value]);
    }
    let mut child = command
        .arg(filter)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(input.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    Some(value.trim().to_owned())
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let high = hex(bytes[index + 1])?;
            let low = hex(bytes[index + 2])?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn valid_onion_url(value: &str) -> bool {
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let authority = rest.split(['/', '?']).next().unwrap_or_default();
    if authority.contains(['@', ':']) || authority.len() != 62 || !authority.ends_with(".onion") {
        return false;
    }
    authority[..56]
        .bytes()
        .all(|byte| matches!(byte, b'a'..=b'z' | b'2'..=b'7'))
}

fn error_json(reason: &str) -> String {
    format!(r#"{{"available":false,"reason":"{reason}"}}"#)
}

fn unavailable_history_json(reason: &str) -> String {
    format!(
        r#"{{"available":false,"reason":"{reason}","stats":{{"totalVisits":0,"uniqueUrls":0,"todayVisits":0}},"items":[]}}"#
    )
}

fn unavailable_shortcuts_json(reason: &str) -> String {
    format!(r#"{{"available":false,"reason":"{reason}","items":[]}}"#)
}

fn unavailable_hermes_json(reason: &str) -> String {
    format!(
        r#"{{"available":false,"reason":"{reason}","stats":{{"totalBriefings":0,"unread":0,"highPriority":0,"sourceCount":0}},"briefings":[],"tiers":[],"coverage":{{}}}}"#
    )
}

fn unavailable_tor_json(reason: &str) -> String {
    format!(r#"{{"available":false,"reason":"{reason}","items":[]}}"#)
}

fn json_response(body: String) -> Vec<u8> {
    response(200, "application/json; charset=utf-8", body.as_bytes())
}

fn response(status: u16, content_type: &str, body: &[u8]) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Payload Too Large",
        421 => "Misdirected Request",
        503 => "Service Unavailable",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes()
    .into_iter()
    .chain(body.iter().copied())
    .collect()
}

fn content_type(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        content_type, resolve_registry_url, run_jq, valid_onion_url, Config, DREAD_URL, HERMES_JQ,
        PITCH_URL, SHORTCUTS_JQ, TOR_JQ,
    };

    #[test]
    fn accepts_pinned_onion_links() {
        assert!(valid_onion_url(DREAD_URL));
        assert!(valid_onion_url(PITCH_URL));
    }

    #[test]
    fn rejects_non_onion_authorities() {
        assert!(!valid_onion_url("https://example.com/"));
        assert!(!valid_onion_url(
            "https://user:pass@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion/"
        ));
        assert!(!valid_onion_url(
            "https://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion:443/"
        ));
        assert!(!valid_onion_url("https://short.onion/"));
    }

    #[test]
    fn normalizes_hermes_records_without_invoking_hermes() {
        let input = r#"[
          [{"id":"briefing-1","lane":"frontier","title":"Edge report","summary":"A bounded summary","priority":"high","confidence":0.8,"createdAt":"2026-08-21T12:00:00Z","unread":true,"sources":["s1"],"coverage":{"github":2}}],
          {"sources":{"s1":{"url":"https://example.com","tier":"trusted"}}}
        ]"#;
        let result = run_jq(HERMES_JQ, input, &[]).expect("Hermes fixture should normalize");
        assert!(result.contains("\"totalBriefings\":1"));
        assert!(result.contains("\"highPriority\":1"));
        assert!(result.contains("\"sourceCount\":1"));
        assert!(result.contains("\"coverage\":{\"github\":2}"));
    }

    #[test]
    fn normalizes_helium_custom_links() {
        let input = r#"{
          "custom_links": {"list": [
            {"title":"GitHub","url":"https://github.com/maxrave-dev/SimpMusic"},
            {"title":"Onion","url":"https://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion/"},
            {"title":"Local","url":"file:///tmp/notes"}
          ]}
        }"#;
        let result = run_jq(SHORTCUTS_JQ, input, &[]).expect("shortcut fixture should normalize");
        assert!(result.contains(r#""title":"GitHub""#));
        assert!(result.contains(r#""domain":"github.com""#));
        assert!(!result.contains("onion"));
        assert!(!result.contains("file:///"));
    }

    #[test]
    fn root_static_content_type_is_html() {
        assert_eq!(content_type("index.html"), "text/html; charset=utf-8");
    }

    #[test]
    fn exposes_only_onion_metadata_to_the_tor_api() {
        let host = "a".repeat(56);
        let url = format!("https://{host}.onion/research");
        let input = format!(
            r#"[{{"sources":{{"hermes-1":{{"url":"{url}","title":"Onion report","tier":"probation"}}}}}}]"#
        );
        let result = run_jq(
            TOR_JQ,
            &input,
            &[("dread", DREAD_URL), ("pitch", PITCH_URL)],
        )
        .expect("Tor fixture should normalize");
        assert!(result.contains("\"hermes-1\""));
        assert!(result.contains("\"Onion report\""));
        assert!(!result.contains(&url));
    }

    #[test]
    fn resolves_registry_ids_as_raw_urls() {
        let path =
            std::env::temp_dir().join(format!("vesper-startpage-registry-{}", std::process::id()));
        let host = "b".repeat(56);
        let url = format!("https://{host}.onion/entry");
        fs::write(
            &path,
            format!(r#"{{"sources":{{"source-1":{{"url":"{url}"}}}}}}"#),
        )
        .expect("write registry fixture");
        let config = Config {
            web_root: std::path::PathBuf::new(),
            helium_history: std::path::PathBuf::new(),
            helium_preferences: std::path::PathBuf::new(),
            zen_histories: Vec::new(),
            briefings_index: std::path::PathBuf::new(),
            source_registry: path.clone(),
            tor_browser: std::path::PathBuf::from("tor-browser"),
        };
        assert_eq!(
            resolve_registry_url("source-1", &config).as_deref(),
            Some(url.as_str())
        );
        let _ = fs::remove_file(path);
    }
}
