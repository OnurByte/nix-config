use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::canonical::{self, validate_svg_text};
use crate::config::Config;
use crate::discovery::alias_name;
use crate::model::InventoryItem;
use crate::state;
use crate::util::{generation_root, json_escape, safe_name, sql_escape, xdg_data_home};

pub const THEME_NAME: &str = "Vesper-Adaptive";
pub const RENDERER_REVISION: &str = "static-material-2026.08-r1";

fn manifest_value(manifest: &Path, filter: &str) -> Result<String, String> {
    crate::util::command_output("jq", &["-er", filter, &manifest.to_string_lossy()])
}

fn svg_inner(text: &str) -> Result<String, String> {
    let lower = text.to_ascii_lowercase();
    let start = lower.find("<svg").ok_or_else(|| "missing-svg-root".to_string())?;
    let open = text[start..].find('>').ok_or_else(|| "malformed-svg-root".to_string())? + start;
    let close = lower.rfind("</svg>").ok_or_else(|| "missing-svg-close".to_string())?;
    if close <= open { return Err("malformed-svg".into()); }
    Ok(text[open+1..close].to_string())
}

fn hex_ok(value: &str) -> bool {
    let v = value.trim().trim_start_matches('#');
    v.len() == 6 && v.chars().all(|c| c.is_ascii_hexdigit())
}

fn appearance(cfg: &Config) -> &str {
    if cfg.appearance == "automatic" { if cfg.scheme_mode == "dark" { "dark" } else { "default" } } else { &cfg.appearance }
}

fn matrix_for(color: &str) -> String {
    let hex = color.trim_start_matches('#');
    let c = |a: usize| u8::from_str_radix(&hex[a..a+2],16).unwrap_or(255) as f64 / 255.0;
    format!("0 0 0 0 {:.5} 0 0 0 0 {:.5} 0 0 0 0 {:.5} 0 0 0 1 0",c(0),c(2),c(4))
}

fn background_svg(strategy: &str, brand: &str, needs_enclosure: bool, cfg: &Config, material: &str) -> String {
    if !needs_enclosure && strategy == "transparent" { return String::new(); }
    let brand = if hex_ok(brand) { brand } else { &cfg.accent };
    let light = cfg.scheme_mode == "light";
    let base = match strategy {
        "system-light" => "#f3f3f5",
        "system-dark" => "#1c1c1e",
        "palette-surface" => if light { "#f4f2f6" } else { "#1d1b20" },
        "brand-solid" | "brand-gradient" | "system-brand-gradient" => brand,
        _ => if light { "#f3f3f5" } else { "#1c1c1e" },
    };
    let g = canonical::guide("enclosed");
    if material == "glass" {
        format!("<defs><linearGradient id=\"vesper-bg\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"><stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.34\"/><stop offset=\"0.48\" stop-color=\"{}\" stop-opacity=\"0.30\"/><stop offset=\"1\" stop-color=\"{}\" stop-opacity=\"0.62\"/></linearGradient></defs><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"190\" fill=\"url(#vesper-bg)\" stroke=\"#ffffff\" stroke-opacity=\"0.28\" stroke-width=\"6\"/>",base,base,g.x,g.y,g.size,g.size)
    } else if matches!(strategy,"brand-gradient"|"system-brand-gradient") {
        format!("<defs><linearGradient id=\"vesper-bg\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"><stop offset=\"0\" stop-color=\"{}\" stop-opacity=\"0.86\"/><stop offset=\"1\" stop-color=\"{}\"/></linearGradient></defs><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"190\" fill=\"url(#vesper-bg)\"/>",if light {"#ffffff"} else {base},base,g.x,g.y,g.size,g.size)
    } else {
        format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"190\" fill=\"{}\"/>",g.x,g.y,g.size,g.size,base)
    }
}

pub fn render_package(package: &Path, cfg: &Config, forced_appearance: Option<&str>, forced_material: Option<&str>) -> Result<String, String> {
    let manifest = package.join("manifest.json");
    let strategy = manifest_value(&manifest,".background.strategy // \"transparent\"")?;
    let brand = manifest_value(&manifest,".background.brandColor // \"\"")?;
    let needs = manifest_value(&manifest,".normalization.needsEnclosure // false")? == "true";
    let app = forced_appearance.unwrap_or_else(|| appearance(cfg));
    let material = forced_material.unwrap_or(&cfg.material);
    let mut group_dirs = fs::read_dir(package.join("groups")).map_err(|e| e.to_string())?.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect::<Vec<_>>();
    group_dirs.sort();
    let mut groups = String::new();
    for (index, dir) in group_dirs.iter().enumerate() {
        let layer = dir.join("layers/01.svg");
        let text = fs::read_to_string(&layer).map_err(|e| e.to_string())?;
        validate_svg_text(&text,true)?;
        let inner = svg_inner(&text)?;
        let effects = fs::read_to_string(dir.join("group.json")).unwrap_or_default().contains("\"effects\":true");
        let depth = if material == "glass" && effects { format!(" filter=\"url(#vesper-depth-{})\"",index+1) } else { String::new() };
        groups.push_str(&format!("<g{depth}>{inner}</g>"));
    }
    let mut defs = String::new();
    if material == "glass" {
        defs.push_str("<defs>");
        for index in 0..group_dirs.len() { defs.push_str(&format!("<filter id=\"vesper-depth-{}\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\"><feDropShadow dx=\"0\" dy=\"{}\" stdDeviation=\"{}\" flood-color=\"#000000\" flood-opacity=\"0.16\"/></filter>",index+1,4+index*2,3+index)); }
        defs.push_str("</defs>");
    }
    if matches!(app,"tinted"|"clear") {
        let color = if app == "tinted" { &cfg.accent } else if cfg.scheme_mode == "light" { "#202124" } else { "#ffffff" };
        let id = if app == "tinted" { "vesper-tint" } else { "vesper-clear" };
        defs.push_str(&format!("<defs><filter id=\"{id}\" color-interpolation-filters=\"sRGB\"><feColorMatrix type=\"matrix\" values=\"{}\"/></filter></defs>",matrix_for(color)));
        groups = format!("<g filter=\"url(#{id})\">{groups}</g>");
    }
    let mut render_cfg = cfg.clone();
    if app == "dark" { render_cfg.scheme_mode = "dark".into(); }
    if app == "default" { render_cfg.scheme_mode = "light".into(); }
    let background = background_svg(&strategy,&brand,needs,&render_cfg,material);
    let svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\">{defs}{background}{groups}</svg>\n");
    validate_svg_text(&svg,true)?;
    Ok(svg)
}

fn legacy_svg(source: &Path) -> Result<String,String> {
    let text = fs::read_to_string(source).map_err(|e| e.to_string())?;
    validate_svg_text(&text,false)?;
    // Legacy fallback intentionally does not synthesize another enclosure. It preserves identity until AI canonicalization succeeds.
    let lower=text.to_ascii_lowercase(); let start=lower.find("<svg").ok_or("missing-svg")?; let open=text[start..].find('>').ok_or("missing-svg-open")?+start; let close=lower.rfind("</svg>").ok_or("missing-svg-close")?;
    let body=&text[open+1..close];
    let tag=&text[start..=open];
    let viewbox = {
        let l=tag.to_ascii_lowercase();
        if let Some(pos)=l.find("viewbox=") { let tail=&tag[pos+8..].trim_start(); let q=tail.chars().next().unwrap_or('"'); let rest=&tail[1..]; rest.find(q).map(|e| rest[..e].to_string()).unwrap_or_else(||"0 0 1024 1024".into()) } else { "0 0 1024 1024".into() }
    };
    let g=canonical::guide("irregular");
    let svg=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\"><svg x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"{}\" preserveAspectRatio=\"xMidYMid meet\">{}</svg></svg>\n",g.x,g.y,g.size,g.size,json_escape(&viewbox),body);
    validate_svg_text(&svg,true)?;
    Ok(svg)
}

fn fallback_themes(cfg: &Config) -> &'static str { if cfg.scheme_mode=="light" { "Papirus-Light,Papirus,hicolor" } else { "Papirus-Dark,Papirus,hicolor" } }

fn sync_shadow(item: &InventoryItem, alias: &str, enabled: bool) -> Result<(),String> {
    if !Path::new(&item.desktop.icon).is_absolute() { return Ok(()); }
    let user_apps=xdg_data_home().join("applications");
    let shadow=user_apps.join(&item.desktop.id);
    if !enabled {
        if fs::read_to_string(&shadow).map(|t|t.contains("X-Vesper-Generated=true")).unwrap_or(false) { let _=fs::remove_file(shadow); }
        return Ok(());
    }
    if item.desktop.exec.split_whitespace().any(|v|v.contains("%i")) { return Ok(()); }
    if item.desktop.path.starts_with(&user_apps) { return Ok(()); }
    if shadow.exists() && !fs::read_to_string(&shadow).map(|t|t.contains("X-Vesper-Generated=true")).unwrap_or(false) { return Ok(()); }
    let source=fs::read_to_string(&item.desktop.path).map_err(|e|e.to_string())?;
    let mut out=String::new(); let mut in_main=false; let mut replaced=false; let mut marker=false;
    for raw in source.lines() {
        let line=raw.trim();
        if line.starts_with('[')&&line.ends_with(']') { if in_main && !marker { out.push_str("X-Vesper-Generated=true\n"); marker=true; } in_main=line=="[Desktop Entry]"; }
        if in_main && line.starts_with("Icon=") { out.push_str(&format!("Icon={alias}\n")); replaced=true; continue; }
        if in_main && line.starts_with("X-Vesper-Generated=") { if !marker { out.push_str("X-Vesper-Generated=true\n"); marker=true; } continue; }
        out.push_str(raw); out.push('\n');
    }
    if !replaced { return Err("absolute-icon-shadow-main-icon-not-found".into()); }
    if in_main && !marker { out.push_str("X-Vesper-Generated=true\n"); }
    fs::create_dir_all(&user_apps).map_err(|e|e.to_string())?;
    crate::util::write_atomic(&shadow,out)
}

fn generation_id() -> String { format!("{}-{}",crate::util::now_epoch(),std::process::id()) }

pub fn compile_theme(items: &mut [InventoryItem], cfg: &Config) -> Result<usize,String> {
    fs::create_dir_all(generation_root()).map_err(|e|e.to_string())?;
    let generation=generation_root().join(generation_id());
    let scalable=generation.join("scalable/apps"); let raster=generation.join("256x256/apps");
    fs::create_dir_all(&scalable).map_err(|e|e.to_string())?; fs::create_dir_all(&raster).map_err(|e|e.to_string())?;
    fs::write(generation.join("index.theme"),format!("[Icon Theme]\nName=Vesper Adaptive\nComment=Vesper application icon overlay\nInherits={}\nDirectories=scalable/apps,256x256/apps\n\n[scalable/apps]\nSize=128\nMinSize=16\nMaxSize=1024\nType=Scalable\nContext=Applications\n\n[256x256/apps]\nSize=256\nType=Fixed\nContext=Applications\n",fallback_themes(cfg))).map_err(|e|e.to_string())?;
    let mut written=BTreeSet::new(); let mut active=0usize;
    for item in items.iter_mut() {
        item.active=false;
        let mut aliases=BTreeSet::new(); aliases.insert(alias_name(&item.desktop.icon)); aliases.insert(alias_name(&item.identity.canonical_app_id)); aliases.insert(alias_name(&item.identity.launch_desktop_id)); for a in item.identity.runtime_ids.iter().chain(item.identity.icon_aliases.iter()) { aliases.insert(alias_name(a)); }
        aliases.retain(|v|!v.is_empty()&&v!="unknown");
        let primary=aliases.iter().next().cloned().unwrap_or_else(||safe_name(&item.identity.canonical_app_id));
        let _=sync_shadow(item,&primary,cfg.enabled&&!item.excluded);
        if !cfg.enabled || item.excluded { continue; }
        let canonical=(!item.work_key.is_empty()).then(||state::canonical_path(&item.work_key)).filter(|p|p.join("manifest.json").is_file());
        if let Some(package)=canonical {
            let svg=render_package(&package,cfg,None,None)?;
            for alias in aliases { if written.insert(alias.clone()) { fs::write(scalable.join(format!("{alias}.svg")),&svg).map_err(|e|e.to_string())?; } }
            item.active=true; item.tier="canonical-ai".into(); active+=1;
        } else if let Some(source)=&item.source {
            if source.kind=="svg" {
                if let Ok(svg)=legacy_svg(&source.path) { for alias in aliases { if written.insert(alias.clone()) { fs::write(scalable.join(format!("{alias}.svg")),&svg).map_err(|e|e.to_string())?; } } item.active=true; item.tier="legacy-auto-fit".into(); active+=1; }
            } else {
                let tmp=raster.join(format!("{}.png",safe_name(&item.identity.canonical_app_id)));
                let status=Command::new("magick").args(["-limit","memory","64MiB","-limit","map","128MiB",&source.path.to_string_lossy(),"-background","none","-alpha","on","-resize","256x256>",&tmp.to_string_lossy()]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                if status.map(|s|s.success()).unwrap_or(false) { for alias in aliases { if written.insert(alias.clone()) { let _=fs::copy(&tmp,raster.join(format!("{alias}.png"))); } } item.active=true; item.tier="legacy-auto-fit".into(); active+=1; }
            }
        }
    }
    let icon_root=xdg_data_home().join("icons"); fs::create_dir_all(&icon_root).map_err(|e|e.to_string())?;
    let link=icon_root.join(THEME_NAME); let next=icon_root.join(format!(".{THEME_NAME}.next-{}",std::process::id())); let _=fs::remove_file(&next); symlink(&generation,&next).map_err(|e|e.to_string())?; fs::rename(&next,&link).map_err(|e|e.to_string())?;
    let _=Command::new("gtk-update-icon-cache").args(["-f","-t",&generation.to_string_lossy()]).stdout(Stdio::null()).stderr(Stdio::null()).status();
    state::sql("UPDATE applications SET active=0;")?;
    for item in items.iter().filter(|i|i.active) { state::sql(&format!("UPDATE applications SET active=1,tier='{}' WHERE id='{}';",sql_escape(&item.tier),sql_escape(&item.desktop.id)))?; }
    state::sql(&format!("UPDATE theme_generations SET active=0; INSERT OR REPLACE INTO theme_generations(generation,path,active,created_at) VALUES('{}','{}',1,{});",sql_escape(generation.file_name().and_then(|v|v.to_str()).unwrap_or("unknown")),sql_escape(&generation.to_string_lossy()),crate::util::now_epoch()))?;
    gc(&generation);
    Ok(active)
}

fn gc(current:&Path) { let Ok(entries)=fs::read_dir(generation_root()) else{return;}; let mut dirs=entries.flatten().map(|e|e.path()).filter(|p|p.is_dir()).collect::<Vec<_>>(); dirs.sort_by_key(|p|fs::metadata(p).and_then(|m|m.modified()).ok()); dirs.reverse(); let mut kept=0; for p in dirs { if p==current||kept<2 {kept+=1;} else {let _=fs::remove_dir_all(p);} } }

pub fn disable_shadows(items:&[InventoryItem]) { for item in items { let _=sync_shadow(item,"",false); } }
