use std::ffi::{c_char, c_int, c_long, c_void, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::Write;

use crate::config::Config;
use crate::util::{base64, cache_root, command_output, command_stdin, json_escape, safe_name};

const CURL_GLOBAL_ALL: c_long = 3;
const CURLOPT_WRITEDATA: c_int = 10001;
const CURLOPT_URL: c_int = 10002;
const CURLOPT_POSTFIELDS: c_int = 10015;
const CURLOPT_USERAGENT: c_int = 10018;
const CURLOPT_HTTPHEADER: c_int = 10023;
const CURLOPT_HEADERDATA: c_int = 10029;
const CURLOPT_POST: c_int = 47;
const CURLOPT_POSTFIELDSIZE: c_int = 60;
const CURLOPT_TIMEOUT: c_int = 13;
const CURLOPT_CONNECTTIMEOUT: c_int = 78;
const CURLOPT_WRITEFUNCTION: c_int = 20011;
const CURLOPT_HEADERFUNCTION: c_int = 20079;
const CURLINFO_RESPONSE_CODE: c_int = 0x200002;

#[repr(C)]
struct CurlSlist { _private: [u8; 0] }

#[link(name = "curl")]
extern "C" {
    fn curl_global_init(flags: c_long) -> c_int;
    fn curl_easy_init() -> *mut c_void;
    fn curl_easy_cleanup(handle: *mut c_void);
    fn curl_easy_perform(handle: *mut c_void) -> c_int;
    fn curl_easy_setopt(handle: *mut c_void, option: c_int, ...) -> c_int;
    fn curl_easy_getinfo(handle: *mut c_void, info: c_int, ...) -> c_int;
    fn curl_easy_strerror(code: c_int) -> *const c_char;
    fn curl_slist_append(list: *mut CurlSlist, value: *const c_char) -> *mut CurlSlist;
    fn curl_slist_free_all(list: *mut CurlSlist);
}

extern "C" fn collect(ptr: *mut c_char, size: usize, nmemb: usize, userdata: *mut c_void) -> usize {
    let len = size.saturating_mul(nmemb);
    if ptr.is_null() || userdata.is_null() { return 0; }
    unsafe {
        let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
        let target = &mut *(userdata as *mut Vec<u8>);
        target.extend_from_slice(bytes);
    }
    len
}

#[derive(Debug)]
pub struct HttpResponse { pub status: i64, pub body: String, pub retry_after: Option<i64> }

fn curl_error(code: c_int) -> String {
    unsafe {
        let ptr = curl_easy_strerror(code);
        if ptr.is_null() { return format!("libcurl error {code}"); }
        std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

fn post_json(url: &str, key: &str, body: &str) -> Result<HttpResponse, String> {
    unsafe {
        if curl_global_init(CURL_GLOBAL_ALL) != 0 { return Err("libcurl global init failed".into()); }
        let handle = curl_easy_init();
        if handle.is_null() { return Err("libcurl easy init failed".into()); }
        let url = CString::new(url).map_err(|_| "invalid provider url".to_string())?;
        let body_c = CString::new(body).map_err(|_| "request contains NUL".to_string())?;
        let ua = CString::new("vesper-icon-engine/0.3").unwrap();
        let auth = CString::new(format!("Authorization: Bearer {key}")).map_err(|_| "invalid credential".to_string())?;
        let content = CString::new("Content-Type: application/json").unwrap();
        let mut headers: *mut CurlSlist = std::ptr::null_mut();
        headers = curl_slist_append(headers, auth.as_ptr());
        headers = curl_slist_append(headers, content.as_ptr());
        let mut response = Vec::<u8>::new();
        let mut response_headers = Vec::<u8>::new();
        let mut set = |option: c_int, code: c_int| -> Result<(), String> { if code == 0 { Ok(()) } else { Err(format!("curl option {option}: {}", curl_error(code))) } };
        set(CURLOPT_URL, curl_easy_setopt(handle, CURLOPT_URL, url.as_ptr()))?;
        set(CURLOPT_POST, curl_easy_setopt(handle, CURLOPT_POST, 1_c_long))?;
        set(CURLOPT_POSTFIELDS, curl_easy_setopt(handle, CURLOPT_POSTFIELDS, body_c.as_ptr()))?;
        set(CURLOPT_POSTFIELDSIZE, curl_easy_setopt(handle, CURLOPT_POSTFIELDSIZE, body.len() as c_long))?;
        set(CURLOPT_HTTPHEADER, curl_easy_setopt(handle, CURLOPT_HTTPHEADER, headers))?;
        set(CURLOPT_USERAGENT, curl_easy_setopt(handle, CURLOPT_USERAGENT, ua.as_ptr()))?;
        set(CURLOPT_CONNECTTIMEOUT, curl_easy_setopt(handle, CURLOPT_CONNECTTIMEOUT, 15_c_long))?;
        set(CURLOPT_TIMEOUT, curl_easy_setopt(handle, CURLOPT_TIMEOUT, 150_c_long))?;
        set(CURLOPT_WRITEFUNCTION, curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, collect as extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize))?;
        set(CURLOPT_WRITEDATA, curl_easy_setopt(handle, CURLOPT_WRITEDATA, &mut response as *mut Vec<u8> as *mut c_void))?;
        set(CURLOPT_HEADERFUNCTION, curl_easy_setopt(handle, CURLOPT_HEADERFUNCTION, collect as extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize))?;
        set(CURLOPT_HEADERDATA, curl_easy_setopt(handle, CURLOPT_HEADERDATA, &mut response_headers as *mut Vec<u8> as *mut c_void))?;
        let code = curl_easy_perform(handle);
        let mut status: c_long = 0;
        let _ = curl_easy_getinfo(handle, CURLINFO_RESPONSE_CODE, &mut status as *mut c_long);
        curl_slist_free_all(headers);
        curl_easy_cleanup(handle);
        if code != 0 { return Err(curl_error(code)); }
        let headers_text = String::from_utf8_lossy(&response_headers);
        let retry_after = headers_text.lines().rev().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.trim().eq_ignore_ascii_case("retry-after").then(|| value.trim().parse::<i64>().ok()).flatten()
        });
        Ok(HttpResponse { status: status as i64, body: String::from_utf8_lossy(&response).into_owned(), retry_after })
    }
}

pub fn credential_ready(cfg: &Config) -> bool {
    if cfg.provider != "openai" { return false; }
    Command::new("secret-tool").args(["lookup", "service", "vesper-ai", "provider", &cfg.provider]).stdout(Stdio::null()).stderr(Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
}

fn credential(cfg: &Config) -> Result<String, String> {
    if cfg.provider != "openai" { return Err(format!("adaptive icon transport is not implemented for {}", cfg.provider)); }
    let output = command_output("secret-tool", &["lookup", "service", "vesper-ai", "provider", &cfg.provider])?;
    if output.trim().is_empty() { Err("provider credential is empty".into()) } else { Ok(output.trim().to_string()) }
}

fn mime(kind: &str) -> &'static str {
    match kind { "jpg" | "jpeg" => "image/jpeg", "webp" => "image/webp", _ => "image/png" }
}

fn preview(source: &Path, kind: &str, key: &str) -> Result<(PathBuf, String), String> {
    let metadata = fs::metadata(source).map_err(|e| e.to_string())?;
    if metadata.len() > 12 * 1024 * 1024 { return Err("source-byte-budget".into()); }
    let root = cache_root().join("provider-previews");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let target = root.join(format!("{}.png", safe_name(key)));
    if matches!(kind, "png" | "jpg" | "jpeg" | "webp") && metadata.len() <= 8 * 1024 * 1024 {
        let dimensions = command_output("magick", &["-limit","memory","64MiB","-limit","map","128MiB","identify","-format","%w %h", &source.to_string_lossy()])?;
        let dims: Vec<u64> = dimensions.split_whitespace().filter_map(|v| v.parse().ok()).collect();
        if dims.len() != 2 || dims[0] == 0 || dims[1] == 0 || dims[0].saturating_mul(dims[1]) > 40_000_000 { return Err("source-pixel-budget".into()); }
        if kind == "png" { return Ok((source.to_path_buf(), "image/png".into())); }
        if kind == "jpg" || kind == "jpeg" || kind == "webp" { return Ok((source.to_path_buf(), mime(kind).into())); }
    }
    let status = Command::new("magick").args(["-limit","memory","64MiB","-limit","map","128MiB", &source.to_string_lossy(), "-background","none", "-alpha","on", "-resize","1024x1024>", &target.to_string_lossy()]).status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("provider-preview-normalization-failed".into()); }
    Ok((target, "image/png".into()))
}

fn structural_summary(source: &Path, kind: &str) -> String {
    if kind != "svg" { return format!("raster source format: {kind}"); }
    let Ok(text) = fs::read_to_string(source) else { return "svg source; structural read failed".into(); };
    let clipped: String = text.chars().take(24_000).collect();
    format!("sanitized local SVG source follows for geometry reference. Preserve exact official curves when practical:\n{clipped}")
}

const SCHEMA: &str = r#"{
  "type":"object","additionalProperties":false,
  "required":["schemaVersion","sourceAssessment","normalization","background","groups","appearances"],
  "properties":{
    "schemaVersion":{"type":"integer","const":2},
    "sourceAssessment":{"type":"object","additionalProperties":false,"required":["shapeClass","confidence","identityRisk"],"properties":{"shapeClass":{"type":"string","enum":["enclosed","circular","glyph","irregular","full-bleed"]},"confidence":{"type":"number","minimum":0,"maximum":1},"identityRisk":{"type":"string","enum":["low","medium","high"]}}},
    "normalization":{"type":"object","additionalProperties":false,"required":["needsEnclosure","opticalOffsetX","opticalOffsetY"],"properties":{"needsEnclosure":{"type":"boolean"},"opticalOffsetX":{"type":"number","minimum":-0.15,"maximum":0.15},"opticalOffsetY":{"type":"number","minimum":-0.15,"maximum":0.15}}},
    "background":{"type":"object","additionalProperties":false,"required":["strategy","brandColor"],"properties":{"strategy":{"type":"string","enum":["brand-solid","brand-gradient","system-brand-gradient","system-light","system-dark","palette-surface","transparent","artwork"]},"brandColor":{"type":"string"}}},
    "groups":{"type":"array","minItems":1,"maxItems":4,"items":{"type":"object","additionalProperties":false,"required":["id","z","renderMode","blendMode","effects","reuseWholeSource","svg"],"properties":{"id":{"type":"string"},"z":{"type":"integer","minimum":1,"maximum":4},"renderMode":{"type":"string","enum":["combined","individual"]},"blendMode":{"type":"string","enum":["auto","normal","multiply","screen","darken","lighten","plus-lighter","plus-darker"]},"effects":{"type":"boolean"},"reuseWholeSource":{"type":"boolean"},"svg":{"type":"string"}}}},
    "appearances":{"type":"object","additionalProperties":false,"required":["default","dark","mono"],"properties":{"default":{"type":"object"},"dark":{"type":"object"},"mono":{"type":"object"}}}
  }
}"#;

pub struct Proposal { pub json: String, pub retry_after: Option<i64> }

pub fn canonicalize(cfg: &Config, source: &Path, kind: &str, work_key: &str) -> Result<Proposal, (String, Option<i64>, bool)> {
    let key = credential(cfg).map_err(|e| (e, None, false))?;
    let (preview_path, preview_mime) = preview(source, kind, work_key).map_err(|e| (e, None, true))?;
    let bytes = fs::read(&preview_path).map_err(|e| (e.to_string(), None, true))?;
    if bytes.len() > 10 * 1024 * 1024 { return Err(("normalized-preview-byte-budget".into(), None, true)); }
    let data_url = format!("data:{};base64,{}", preview_mime, base64(&bytes));
    let summary = structural_summary(source, kind);
    let prompt = format!("Decompose this installed application icon into Vesper's canonical layered icon schema. Preserve application identity and recognizable geometry. Do not invent letters, symbols or decoration. Canonical artwork is material-neutral: no glass, drop shadow, specular highlight, glow, bevel, refraction or final rounded-square mask. Use one to four semantic foreground depth groups. If this is a clean official SVG and one whole-source group is semantically sufficient, set reuseWholeSource=true and leave svg empty so local code preserves exact geometry. Otherwise return standalone safe SVG for each group on a 0 0 1024 1024 viewBox. Mono must remain recognizable without brand hue. {summary}");
    let payload = format!("{{\"model\":\"{}\",\"input\":[{{\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",\"text\":\"{}\"}},{{\"type\":\"input_image\",\"image_url\":\"{}\",\"detail\":\"high\"}}]}}],\"text\":{{\"format\":{{\"type\":\"json_schema\",\"name\":\"vesper_vicon_v2\",\"strict\":true,\"schema\":{}}}}}}}", json_escape(&cfg.model), json_escape(&prompt), json_escape(&data_url), SCHEMA);
    let response = post_json("https://api.openai.com/v1/responses", &key, &payload).map_err(|e| (e, None, false))?;
    if response.status == 429 || response.status >= 500 { return Err((format!("provider-http-{}", response.status), response.retry_after, false)); }
    if !(200..300).contains(&response.status) {
        let message = command_stdin("jq", &["-r", ".error.message // .error.code // \"provider request failed\""], &response.body).unwrap_or_else(|_| format!("provider-http-{}", response.status));
        return Err((message, response.retry_after, response.status >= 400 && response.status < 500));
    }
    let json = command_stdin("jq", &["-er", "[.output[]?.content[]? | select(.type == \"output_text\") | .text] | join(\"\")"], &response.body).map_err(|e| (format!("provider-response: {e}"), None, false))?;
    command_stdin("jq", &["-e", ".schemaVersion == 2 and (.groups|length)>=1 and (.groups|length)<=4 and .sourceAssessment.identityRisk != \"high\""], &json).map_err(|e| (format!("identity-or-schema-validation: {e}"), None, true))?;
    Ok(Proposal { json, retry_after: response.retry_after })
}
