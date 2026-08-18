use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::provider::Proposal;
use crate::state;
use crate::util::{cache_root, command_stdin, json_escape, safe_name, write_atomic};

pub const SCHEMA_VERSION: u32 = 2;
pub const GRID_REVISION: &str = "vesper-grid-2026.08-r1";
pub const VALIDATOR_REVISION: &str = "validator-v2";
pub const PROMPT_REVISION: &str = "vicon-semantic-v2";

#[derive(Clone, Copy)]
pub struct Guide { pub x: f64, pub y: f64, pub size: f64 }

pub fn guide(shape: &str) -> Guide {
    // Versioned Vesper optical calibration. Circular artwork deliberately has its own guide.
    match shape {
        "enclosed" | "full-bleed" => Guide { x: 92.0, y: 92.0, size: 840.0 },
        "circular" => Guide { x: 146.0, y: 146.0, size: 732.0 },
        "glyph" => Guide { x: 156.0, y: 156.0, size: 712.0 },
        _ => Guide { x: 144.0, y: 144.0, size: 736.0 },
    }
}

fn unsafe_svg_reason(text: &str) -> Option<&'static str> {
    if text.len() > 2_500_000 { return Some("svg-byte-budget"); }
    let lower = text.to_ascii_lowercase();
    for (needle, reason) in [
        ("<script", "script"), ("<foreignobject", "foreign-object"), ("javascript:", "javascript-url"),
        ("data:image", "embedded-raster"), ("http://", "external-url"), ("https://", "external-url"),
        ("file://", "external-file"), ("@import", "css-import"), ("@font-face", "external-font"),
        ("<iframe", "foreign-frame"), (" onload=", "event-handler"), (" onclick=", "event-handler"),
        (" onerror=", "event-handler"), ("<image", "embedded-or-external-image")
    ] { if lower.contains(needle) { return Some(reason); } }
    if lower.matches('<').count() > 15_000 { return Some("svg-node-budget"); }
    None
}

fn root_tag(text: &str) -> Option<&str> {
    let lower = text.to_ascii_lowercase();
    let start = lower.find("<svg")?;
    let end = text[start..].find('>')? + start;
    Some(&text[start..=end])
}

fn attr(tag: &str, key: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = key.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(pos) = lower[offset..].find(&needle) {
        let pos = offset + pos;
        let after = pos + needle.len();
        let rest = &tag[after..];
        let eq = rest.find('=')?;
        if !rest[..eq].trim().is_empty() { offset = after; continue; }
        let tail = rest[eq+1..].trim_start();
        let quote = tail.chars().next()?;
        if quote != '"' && quote != '\'' { return None; }
        let value = &tail[1..];
        let end = value.find(quote)?;
        return Some(value[..end].to_string());
    }
    None
}

fn viewbox(text: &str) -> Option<(f64,f64,f64,f64)> {
    let tag = root_tag(text)?;
    if let Some(v) = attr(tag, "viewBox") {
        let p: Vec<f64> = v.replace(',', " ").split_whitespace().filter_map(|v| v.parse().ok()).collect();
        if p.len() == 4 && p[2] > 0.0 && p[3] > 0.0 && p[2] <= 32768.0 && p[3] <= 32768.0 { return Some((p[0],p[1],p[2],p[3])); }
    }
    let parse = |v: String| -> Option<f64> { v.chars().take_while(|c| c.is_ascii_digit() || matches!(c,'.'|'-'|'+')).collect::<String>().parse().ok() };
    let w = parse(attr(tag,"width")?)?;
    let h = parse(attr(tag,"height")?)?;
    (w > 0.0 && h > 0.0 && w <= 32768.0 && h <= 32768.0).then_some((0.0,0.0,w,h))
}

fn inner_svg(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let start = lower.find("<svg")?;
    let open = text[start..].find('>')? + start;
    let close = lower.rfind("</svg>")?;
    (close > open).then(|| text[open+1..close].to_string())
}

pub fn validate_svg_text(text: &str, require_1024: bool) -> Result<(), String> {
    if let Some(reason) = unsafe_svg_reason(text) { return Err(reason.into()); }
    if !text.to_ascii_lowercase().contains("</svg>") { return Err("missing-svg-close".into()); }
    let (_,_,w,h) = viewbox(text).ok_or_else(|| "invalid-viewbox".to_string())?;
    if require_1024 && ((w - 1024.0).abs() > 0.01 || (h - 1024.0).abs() > 0.01) { return Err("canonical-viewbox-must-be-1024".into()); }
    let root = cache_root().join("validation");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let input = root.join(format!("validate-{}.svg", std::process::id()));
    write_atomic(&input, text)?;
    let xml = Command::new("xmllint").args(["--noout","--nonet"]).arg(&input).stdout(Stdio::null()).stderr(Stdio::null()).status().map_err(|e| e.to_string())?;
    if !xml.success() { let _ = fs::remove_file(&input); return Err("malformed-xml".into()); }
    for size in [16,24,32,48,64,128,256] {
        let out = root.join(format!("validate-{}-{size}.png", std::process::id()));
        let status = Command::new("rsvg-convert").args(["-w",&size.to_string(),"-h",&size.to_string(),"-o",&out.to_string_lossy(),&input.to_string_lossy()]).stdout(Stdio::null()).stderr(Stdio::null()).status().map_err(|e| e.to_string())?;
        let good = status.success() && fs::metadata(&out).map(|m| m.len() > 64).unwrap_or(false);
        let _ = fs::remove_file(out);
        if !good { let _ = fs::remove_file(&input); return Err(format!("render-{size}-failed")); }
    }
    let _ = fs::remove_file(input);
    Ok(())
}

fn normalize_source_svg(path: &Path, shape: &str, offset_x: f64, offset_y: f64) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    validate_svg_text(&text, false)?;
    let (vx,vy,vw,vh) = viewbox(&text).ok_or_else(|| "invalid-source-viewbox".to_string())?;
    let inner = inner_svg(&text).ok_or_else(|| "invalid-source-svg".to_string())?;
    let g = guide(shape);
    let ox = (offset_x.clamp(-0.15,0.15) * g.size).round();
    let oy = (offset_y.clamp(-0.15,0.15) * g.size).round();
    Ok(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\"><svg x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"xMidYMid meet\">{}</svg></svg>\n", g.x+ox,g.y+oy,g.size,g.size,vx,vy,vw,vh,inner))
}

fn jq(json: &str, filter: &str) -> Result<String, String> { command_stdin("jq", &["-er", filter], json) }

pub fn build_package(work_key: &str, source: &Path, source_kind: &str, source_fingerprint: &str, proposal: &Proposal, provider: &str, model: &str) -> Result<PathBuf, String> {
    let shape = jq(&proposal.json, ".sourceAssessment.shapeClass")?;
    let risk = jq(&proposal.json, ".sourceAssessment.identityRisk")?;
    if risk == "high" { return Err("identity-risk-high".into()); }
    let offset_x = jq(&proposal.json, ".normalization.opticalOffsetX")?.parse::<f64>().unwrap_or(0.0);
    let offset_y = jq(&proposal.json, ".normalization.opticalOffsetY")?.parse::<f64>().unwrap_or(0.0);
    let count = jq(&proposal.json, ".groups|length")?.parse::<usize>().map_err(|_| "invalid-group-count".to_string())?;
    if !(1..=4).contains(&count) { return Err("group-count-out-of-bounds".into()); }

    let final_dir = state::canonical_path(work_key);
    let staging = final_dir.with_extension(format!("vicon.staging-{}", std::process::id()));
    if staging.exists() { fs::remove_dir_all(&staging).map_err(|e| e.to_string())?; }
    fs::create_dir_all(staging.join("groups")).map_err(|e| e.to_string())?;
    fs::create_dir_all(staging.join("appearances")).map_err(|e| e.to_string())?;

    let mut manifest_groups = Vec::new();
    for index in 0..count {
        let id = jq(&proposal.json, &format!(".groups[{index}].id"))?;
        let z = jq(&proposal.json, &format!(".groups[{index}].z"))?.parse::<i64>().unwrap_or((index+1) as i64);
        let render_mode = jq(&proposal.json, &format!(".groups[{index}].renderMode"))?;
        let blend_mode = jq(&proposal.json, &format!(".groups[{index}].blendMode"))?;
        let effects = jq(&proposal.json, &format!(".groups[{index}].effects"))? == "true";
        let reuse = jq(&proposal.json, &format!(".groups[{index}].reuseWholeSource"))? == "true";
        let group_name = format!("{:02}-{}", index + 1, safe_name(&id));
        let group_dir = staging.join("groups").join(&group_name);
        let layers = group_dir.join("layers");
        fs::create_dir_all(&layers).map_err(|e| e.to_string())?;
        let svg = if reuse {
            if source_kind != "svg" { let _ = fs::remove_dir_all(&staging); return Err("raster-cannot-reuse-whole-source-as-vector".into()); }
            normalize_source_svg(source, &shape, offset_x, offset_y)?
        } else {
            let svg = jq(&proposal.json, &format!(".groups[{index}].svg"))?;
            if svg.trim().is_empty() { let _ = fs::remove_dir_all(&staging); return Err("empty-generated-svg".into()); }
            svg
        };
        validate_svg_text(&svg, true)?;
        write_atomic(&layers.join("01.svg"), &svg)?;
        write_atomic(&group_dir.join("group.json"), format!("{{\"id\":\"{}\",\"z\":{},\"renderMode\":\"{}\",\"blendMode\":\"{}\",\"effects\":{},\"layers\":[{{\"id\":\"primary\",\"assetType\":\"vector\",\"asset\":\"layers/01.svg\"}}]}}\n", json_escape(&id),z,json_escape(&render_mode),json_escape(&blend_mode),effects))?;
        manifest_groups.push(format!("{{\"id\":\"{}\",\"z\":{},\"path\":\"groups/{}/group.json\"}}",json_escape(&id),z,json_escape(&group_name)));
    }

    for name in ["default","dark","mono"] {
        let value = command_stdin("jq", &["-c", &format!(".appearances.{name}")], &proposal.json).unwrap_or_else(|_| "{}".into());
        write_atomic(&staging.join("appearances").join(format!("{name}.json")), format!("{value}\n"))?;
    }
    let background = command_stdin("jq", &["-c", ".background"], &proposal.json).unwrap_or_else(|_| "{\"strategy\":\"transparent\",\"brandColor\":\"\"}".into());
    let needs_enclosure = jq(&proposal.json, ".normalization.needsEnclosure").unwrap_or_else(|_| "false".into()) == "true";
    let manifest = format!("{{\"schemaVersion\":{},\"canvas\":{{\"width\":1024,\"height\":1024,\"masked\":false}},\"workKey\":\"{}\",\"sourceFingerprint\":\"{}\",\"sourceKind\":\"{}\",\"gridRevision\":\"{}\",\"validatorRevision\":\"{}\",\"promptRevision\":\"{}\",\"provider\":\"{}\",\"modelFamily\":\"{}\",\"sourceAssessment\":{{\"shapeClass\":\"{}\",\"identityRisk\":\"{}\"}},\"normalization\":{{\"needsEnclosure\":{},\"opticalOffsetX\":{},\"opticalOffsetY\":{}}},\"background\":{},\"groups\":[{}],\"appearances\":{{\"default\":\"appearances/default.json\",\"dark\":\"appearances/dark.json\",\"mono\":\"appearances/mono.json\"}}}}\n",
        SCHEMA_VERSION,json_escape(work_key),json_escape(source_fingerprint),json_escape(source_kind),GRID_REVISION,VALIDATOR_REVISION,PROMPT_REVISION,json_escape(provider),json_escape(model),json_escape(&shape),json_escape(&risk),needs_enclosure,offset_x,offset_y,background,manifest_groups.join(","));
    write_atomic(&staging.join("manifest.json"), manifest)?;
    write_atomic(&staging.join("proposal.json"), format!("{}\n", proposal.json))?;
    if final_dir.exists() { fs::remove_dir_all(&final_dir).map_err(|e| e.to_string())?; }
    fs::rename(&staging, &final_dir).map_err(|e| e.to_string())?;
    Ok(final_dir)
}

pub fn grid_json() -> String {
    let variants = ["enclosed","circular","glyph","irregular","full-bleed"].iter().map(|name| { let g=guide(name); format!("\"{}\":{{\"x\":{},\"y\":{},\"size\":{}}}",name,g.x,g.y,g.size) }).collect::<Vec<_>>().join(",");
    format!("{{\"revision\":\"{}\",\"canvas\":1024,\"guides\":{{{}}}}}\n", GRID_REVISION, variants)
}
