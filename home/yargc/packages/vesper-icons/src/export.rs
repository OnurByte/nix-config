use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::state;
use crate::theme::{render_package, RENDERER_REVISION};
use crate::canonical::GRID_REVISION;
use crate::util::{export_root, json_escape, now_epoch, safe_name, write_atomic};

fn copy_tree(source: &Path, target: &Path) -> Result<(),String> {
    fs::create_dir_all(target.parent().unwrap_or(target)).map_err(|e|e.to_string())?;
    let status=Command::new("cp").args(["-a",&source.to_string_lossy(),&target.to_string_lossy()]).status().map_err(|e|e.to_string())?;
    if status.success(){Ok(())}else{Err("copy-export-failed".into())}
}

fn rows() -> Result<Vec<(String,String,String)>,String> {
    let raw=state::sql("SELECT canonical_app_id,work_key,tier FROM applications WHERE excluded=0 AND work_key<>'' ORDER BY canonical_app_id;")?;
    Ok(raw.lines().filter_map(|line|{let p:Vec<&str>=line.split('|').collect();(p.len()>=3).then(||(p[0].to_string(),p[1].to_string(),p[2].to_string()))}).collect())
}

fn render_svg_png(svg:&str,svg_path:&Path,png_path:Option<&Path>)->Result<(),String>{
    write_atomic(svg_path,svg)?;
    if let Some(png)=png_path {
        let status=Command::new("rsvg-convert").args(["-w","1024","-h","1024","-o",&png.to_string_lossy(),&svg_path.to_string_lossy()]).stdout(Stdio::null()).stderr(Stdio::null()).status().map_err(|e|e.to_string())?;
        if !status.success(){return Err("export-png-render-failed".into());}
    }
    Ok(())
}

pub fn export_all(kind:&str,cfg:&Config,destination:Option<&str>)->Result<PathBuf,String>{
    if !matches!(kind,"current-svg"|"current-png"|"all-appearances"|"canonical"|"complete"){return Err(format!("unsupported export type: {kind}"));}
    fs::create_dir_all(export_root()).map_err(|e|e.to_string())?;
    let stamp=now_epoch();
    let staging=export_root().join(format!(".export-{stamp}-{}.staging",std::process::id()));
    if staging.exists(){fs::remove_dir_all(&staging).map_err(|e|e.to_string())?;}
    fs::create_dir_all(&staging).map_err(|e|e.to_string())?;
    let apps=rows()?; let mut failures=Vec::new(); let mut exported=0usize;
    for (app,work,tier) in &apps {
        let package=state::canonical_path(work);
        if !package.join("manifest.json").is_file(){continue;}
        let name=safe_name(app);
        let result:Result<(),String>=(||{
            if matches!(kind,"canonical"|"complete"){let target=staging.join("canonical").join(format!("{name}.vicon"));fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;copy_tree(&package,&target)?;}
            if matches!(kind,"current-svg"|"current-png"|"complete"){
                let dir=staging.join("current");fs::create_dir_all(dir.join("svg")).map_err(|e|e.to_string())?;if matches!(kind,"current-png"|"complete"){fs::create_dir_all(dir.join("png")).map_err(|e|e.to_string())?;}
                let svg=render_package(&package,cfg,None,None)?;let png=matches!(kind,"current-png"|"complete").then(||dir.join("png").join(format!("{name}.png")));
                render_svg_png(&svg,&dir.join("svg").join(format!("{name}.svg")),png.as_deref())?;
            }
            if matches!(kind,"all-appearances"|"complete"){
                for appearance in ["default","dark","clear","tinted"] {for material in ["standard","glass"] {let folder=format!("{appearance}-{material}");let dir=staging.join("appearances").join(&folder);fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let svg=render_package(&package,cfg,Some(appearance),Some(material))?;render_svg_png(&svg,&dir.join(format!("{name}.svg")),None)?;}}
            }
            Ok(())
        })();
        match result{Ok(())=>exported+=1,Err(e)=>failures.push(format!("{{\"appId\":\"{}\",\"error\":\"{}\"}}",json_escape(app),json_escape(&e)))}
        let _=tier;
    }
    let manifest=format!("{{\"schemaVersion\":1,\"createdAt\":{},\"rendererRevision\":\"{}\",\"gridRevision\":\"{}\",\"appearance\":\"{}\",\"material\":\"{}\",\"exported\":{},\"failed\":{},\"failures\":[{}]}}\n",stamp,RENDERER_REVISION,GRID_REVISION,json_escape(&cfg.appearance),json_escape(&cfg.material),exported,failures.len(),failures.join(","));
    write_atomic(&staging.join("manifest.json"),manifest)?;
    let default_name=format!("vesper-icons-export-{stamp}");
    let target=destination.map(PathBuf::from).unwrap_or_else(||export_root().join(&default_name));
    if kind=="complete" {
        let archive=if target.extension().is_some(){target}else{target.with_extension("tar.gz")};
        if let Some(parent)=archive.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
        let status=Command::new("tar").args(["-C",&staging.to_string_lossy(),"-czf",&archive.to_string_lossy(),"."]).status().map_err(|e|e.to_string())?;
        if !status.success(){return Err("archive-export-failed".into());}
        let _=fs::remove_dir_all(&staging); Ok(archive)
    } else {
        if target.exists(){return Err(format!("export destination already exists: {}",target.display()));}
        fs::rename(&staging,&target).map_err(|e|e.to_string())?; Ok(target)
    }
}

pub fn export_app(id:&str,cfg:&Config,destination:Option<&str>)->Result<PathBuf,String>{
    let row=state::sql(&format!("SELECT canonical_app_id,work_key FROM applications WHERE id='{}' LIMIT 1;",crate::util::sql_escape(id)))?;
    let p:Vec<&str>=row.trim().split('|').collect();if p.len()!=2{return Err("application has no canonical package".into());}
    let package=state::canonical_path(p[1]);if !package.join("manifest.json").is_file(){return Err("application has no canonical package".into());}
    let target=destination.map(PathBuf::from).unwrap_or_else(||export_root().join(format!("{}-{}.svg",safe_name(p[0]),now_epoch())));
    if let Some(parent)=target.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
    let svg=render_package(&package,cfg,None,None)?;render_svg_png(&svg,&target,None)?;Ok(target)
}
