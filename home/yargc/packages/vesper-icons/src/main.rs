mod canonical;
mod config;
mod discovery;
mod export;
mod model;
mod provider;
mod state;
mod theme;
mod util;

use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use config::Config;
use model::InventoryItem;
use util::{hash_text, json_escape, state_root};

fn retire_prototype_queue() {
    let old = state_root().join("queue");
    if !old.exists() {
        return;
    }
    let retired = state_root().join("queue.prototype-retired");
    if retired.exists() {
        let _ = fs::remove_dir_all(old);
    } else {
        let _ = fs::rename(old, retired);
    }
}

fn work_key(source: &model::Source, cfg: &Config) -> Result<String, String> {
    hash_text(&format!(
        "{}|schema={}|prompt={}|validator={}|provider={}|model={}",
        source.fingerprint,
        canonical::SCHEMA_VERSION,
        canonical::PROMPT_REVISION,
        canonical::VALIDATOR_REVISION,
        cfg.provider,
        cfg.model
    ))
}

fn build_inventory(cfg: &Config) -> Result<Vec<InventoryItem>, String> {
    let records = discovery::desktops();
    let sources = discovery::resolve_sources(&records);
    let exclusions = state::load_exclusions();
    let provider_ready = provider::credential_ready(cfg);
    let mut items = Vec::with_capacity(records.len());

    for desktop in records {
        let identity = discovery::identity(&desktop);
        let source = sources.get(&desktop.id).cloned().flatten();
        let excluded = exclusions.contains(&desktop.id);
        let mut item = InventoryItem {
            desktop,
            identity,
            source,
            excluded,
            tier: "original-fallback".into(),
            ..Default::default()
        };

        if item.excluded {
            item.queue_state = "cancelled".into();
        } else if let Some(source) = &item.source {
            item.work_key = work_key(source, cfg)?;
            if state::canonical_valid(&item.work_key) {
                item.tier = "canonical-ai".into();
                item.queue_state = "succeeded".into();
            } else {
                item.tier = if source.kind == "svg" {
                    "legacy-auto-fit".into()
                } else {
                    "original-fallback".into()
                };
                item.queue_state = state::ensure_job(&item, cfg, provider_ready)?;
            }
        } else {
            item.tier = "unresolved-source".into();
            item.error = "source-icon-not-found".into();
        }

        state::upsert_app(&item)?;
        items.push(item);
    }

    let seen = items
        .iter()
        .map(|item| item.desktop.id.clone())
        .collect::<Vec<_>>();
    state::remove_stale_apps(&seen)?;
    state::cancel_unreferenced_jobs()?;
    state::refresh_blocked(cfg, provider_ready)?;
    Ok(items)
}

fn reconcile(compile: bool) -> Result<String, String> {
    state::init()?;
    retire_prototype_queue();
    let mut cfg = config::load();
    config::sync_palette(&mut cfg);
    let _ = config::save(&cfg);
    let mut items = build_inventory(&cfg)?;
    let active = if compile {
        theme::compile_theme(&mut items, &cfg)?
    } else {
        0
    };
    let counts = state::counts()?;
    Ok(format!(
        "discovered={} canonical={} pending={} blocked={} active={}",
        counts.0,
        counts.1,
        counts.2,
        counts.6,
        if compile { active as i64 } else { counts.7 }
    ))
}

fn process_one(cfg: &Config) -> Result<bool, String> {
    if !cfg.automatic
        || cfg.queue_paused
        || !cfg.remote_consent
        || !provider::credential_ready(cfg)
    {
        return Ok(false);
    }

    let Some(job) = state::lease_next()? else {
        return Ok(false);
    };
    if !job.source_path.is_file() {
        state::job_failure(&job, "source-disappeared", None, true)?;
        return Ok(true);
    }

    match provider::canonicalize(cfg, &job.source_path, &job.source_kind, &job.work_key) {
        Ok(proposal) => {
            let fingerprint = util::sha256(&job.source_path).unwrap_or_default();
            match canonical::build_package(
                &job.work_key,
                &job.source_path,
                &job.source_kind,
                &fingerprint,
                &proposal,
                &cfg.provider,
                &cfg.model,
            ) {
                Ok(package) => state::job_success(&job, &package)?,
                Err(error) => state::job_failure(
                    &job,
                    &format!("canonical-validation: {error}"),
                    None,
                    true,
                )?,
            }
        }
        Err((error, retry_after, permanent)) => {
            state::job_failure(&job, &error, retry_after, permanent)?
        }
    }
    Ok(true)
}

fn status() -> Result<(), String> {
    state::init()?;
    let cfg = config::load();
    let counts = state::counts().unwrap_or((0, 0, 0, 0, 0, 0, 0, 0));
    let provider_ready = provider::credential_ready(&cfg);
    let progress = if counts.0 > 0 {
        format!("{} / {} canonicalized", counts.1, counts.0)
    } else {
        "0 / 0 canonicalized".into()
    };

    println!(
        "{{\"schemaVersion\":3,\"enabled\":{},\"automatic\":{},\"remoteConsent\":{},\
         \"appearance\":\"{}\",\"material\":\"{}\",\"provider\":\"{}\",\"model\":\"{}\",\
         \"providerConfigured\":{},\"followPalette\":{},\"schemeMode\":\"{}\",\"accent\":\"{}\",\
         \"theme\":\"{}\",\"gridRevision\":\"{}\",\"rendererRevision\":\"{}\",\"queuePaused\":{},\
         \"discovered\":{},\"canonical\":{},\"pending\":{},\"running\":{},\"retry\":{},\"failed\":{},\
         \"blocked\":{},\"active\":{},\"progress\":\"{}\",\"aiTransport\":\"responses-structured\"}}",
        cfg.enabled,
        cfg.automatic,
        cfg.remote_consent,
        json_escape(&cfg.appearance),
        json_escape(&cfg.material),
        json_escape(&cfg.provider),
        json_escape(&cfg.model),
        provider_ready,
        cfg.follow_palette,
        json_escape(&cfg.scheme_mode),
        json_escape(&cfg.accent),
        theme::THEME_NAME,
        canonical::GRID_REVISION,
        theme::RENDERER_REVISION,
        cfg.queue_paused,
        counts.0,
        counts.1,
        counts.2,
        counts.3,
        counts.4,
        counts.5,
        counts.6,
        counts.7,
        json_escape(&progress)
    );
    Ok(())
}

fn mutate<F>(f: F) -> Result<(), String>
where
    F: FnOnce(&mut Config) -> Result<(), String>,
{
    let mut cfg = config::load();
    f(&mut cfg)?;
    config::save(&cfg)?;
    let _ = reconcile(true)?;
    Ok(())
}

fn set_enabled(value: bool) -> Result<(), String> {
    mutate(|cfg| {
        cfg.enabled = value;
        Ok(())
    })
}

fn set_automatic(value: bool) -> Result<(), String> {
    mutate(|cfg| {
        cfg.automatic = value;
        Ok(())
    })
}

fn set_consent(value: bool) -> Result<(), String> {
    mutate(|cfg| {
        cfg.remote_consent = value;
        Ok(())
    })
}

fn set_appearance(value: &str) -> Result<(), String> {
    if !config::valid_appearance(value) {
        return Err(format!("unsupported appearance: {value}"));
    }
    mutate(|cfg| {
        cfg.appearance = value.into();
        Ok(())
    })
}

fn set_material(value: &str) -> Result<(), String> {
    if !config::valid_material(value) {
        return Err(format!("unsupported material: {value}"));
    }
    mutate(|cfg| {
        cfg.material = value.into();
        Ok(())
    })
}

fn set_provider(value: &str) -> Result<(), String> {
    if !config::valid_provider(value) {
        return Err(format!("unsupported provider: {value}"));
    }
    mutate(|cfg| {
        cfg.provider = value.into();
        Ok(())
    })
}

fn set_model(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("model cannot be empty".into());
    }
    mutate(|cfg| {
        cfg.model = value.into();
        Ok(())
    })
}

fn set_follow(value: bool) -> Result<(), String> {
    mutate(|cfg| {
        cfg.follow_palette = value;
        config::sync_palette(cfg);
        Ok(())
    })
}

fn sync_theme(value: &str) -> Result<(), String> {
    if !config::valid_scheme(value) {
        return Err(format!("unsupported scheme mode: {value}"));
    }
    mutate(|cfg| {
        cfg.scheme_mode = value.into();
        config::sync_palette(cfg);
        Ok(())
    })
}

fn queue_pause(value: bool) -> Result<(), String> {
    mutate(|cfg| {
        cfg.queue_paused = value;
        state::pause(value)?;
        Ok(())
    })
}

fn app_status(id: &str) -> Result<(), String> {
    state::init()?;
    println!("{}", state::app_json(id)?);
    Ok(())
}

fn app_exclude(id: &str, value: bool) -> Result<(), String> {
    state::set_excluded(id, value)?;
    let _ = reconcile(true)?;
    Ok(())
}

fn app_retry(id: &str) -> Result<(), String> {
    state::retry_failed(Some(id))?;
    let _ = reconcile(true)?;
    Ok(())
}

fn rebuild_canonical() -> Result<(), String> {
    let root = util::canonical_root();
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|e| e.to_string())?;
    }
    state::sql(
        "BEGIN IMMEDIATE; \
         UPDATE canonical_work SET state='missing',package_path='',validation=''; \
         UPDATE jobs SET state='pending',attempts=0,retry_at=0,lease_until=0,last_error=''; \
         COMMIT;",
    )?;
    let _ = reconcile(true)?;
    Ok(())
}

fn daemon() -> Result<(), String> {
    state::init()?;
    let _ = reconcile(true);
    let mut full_scan_ticks = 0_u32;

    loop {
        let cfg = config::load();
        state::refresh_blocked(&cfg, provider::credential_ready(&cfg))?;
        if process_one(&cfg)? {
            let _ = reconcile(true);
            continue;
        }

        let paths = discovery::effective_data_dirs()
            .into_iter()
            .flat_map(|data| {
                [
                    data.join("applications"),
                    data.join("icons"),
                    data.join("pixmaps"),
                ]
            })
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();

        if !paths.is_empty() {
            let mut command = Command::new("inotifywait");
            command.args([
                "-q",
                "-r",
                "-e",
                "close_write,create,delete,move,attrib",
                "-t",
                "15",
                "--",
            ]);
            for path in &paths {
                command.arg(path);
            }
            let changed = command
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            if changed {
                thread::sleep(Duration::from_millis(700));
                let _ = reconcile(true);
                full_scan_ticks = 0;
                continue;
            }
        }

        full_scan_ticks += 1;
        if full_scan_ticks >= 40 {
            let _ = reconcile(true);
            full_scan_ticks = 0;
        }
    }
}

fn usage() -> ! {
    eprintln!(
        "vesper-icon-engine\n  status\n  enable|disable\n  automatic on|off\n  consent on|off\n  appearance automatic|default|dark|clear|tinted\n  material standard|glass\n  provider openai|anthropic|xai|openrouter|google\n  model <model>\n  follow-palette on|off\n  sync-theme light|dark\n  queue pause|resume\n  retry-failed [desktop-id]\n  reconcile\n  rebuild-canonical\n  grid-report\n  app-status <desktop-id>\n  app-exclude <desktop-id> on|off\n  app-retry <desktop-id>\n  app-export <desktop-id> [destination]\n  export-all current-svg|current-png|all-appearances|canonical|complete [destination]\n  daemon"
    );
    std::process::exit(2)
}

fn onoff(value: &str) -> Option<bool> {
    match value {
        "on" => Some(true),
        "off" => Some(false),
        _ => None,
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result: Result<(), String> = match args.as_slice() {
        [command] if command == "status" => status(),
        [command] if command == "enable" => set_enabled(true),
        [command] if command == "disable" => set_enabled(false),
        [command, value] if command == "automatic" => onoff(value).map(set_automatic).unwrap_or_else(|| usage()),
        [command, value] if command == "consent" => onoff(value).map(set_consent).unwrap_or_else(|| usage()),
        [command, value] if command == "appearance" => set_appearance(value),
        [command, value] if command == "material" => set_material(value),
        [command, value] if command == "provider" => set_provider(value),
        [command, value] if command == "model" => set_model(value),
        [command, value] if command == "follow-palette" => onoff(value).map(set_follow).unwrap_or_else(|| usage()),
        [command, value] if command == "sync-theme" => sync_theme(value),
        [command, value] if command == "queue" && value == "pause" => queue_pause(true),
        [command, value] if command == "queue" && value == "resume" => queue_pause(false),
        [command] if command == "retry-failed" => state::retry_failed(None).and_then(|_| reconcile(true).map(|_| ())),
        [command, id] if command == "retry-failed" => state::retry_failed(Some(id)).and_then(|_| reconcile(true).map(|_| ())),
        [command] if command == "reconcile" => reconcile(true).map(|value| println!("{value}")),
        [command] if command == "rebuild-canonical" => rebuild_canonical(),
        [command] if command == "grid-report" => {
            print!("{}", canonical::grid_json());
            Ok(())
        }
        [command, id] if command == "app-status" => app_status(id),
        [command, id, value] if command == "app-exclude" => onoff(value).map(|enabled| app_exclude(id, enabled)).unwrap_or_else(|| usage()),
        [command, id] if command == "app-retry" => app_retry(id),
        [command, id] if command == "app-export" => export::export_app(id, &config::load(), None).map(|path| println!("{}", path.display())),
        [command, id, destination] if command == "app-export" => export::export_app(id, &config::load(), Some(destination)).map(|path| println!("{}", path.display())),
        [command, kind] if command == "export-all" => export::export_all(kind, &config::load(), None).map(|path| println!("{}", path.display())),
        [command, kind, destination] if command == "export-all" => export::export_all(kind, &config::load(), Some(destination)).map(|path| println!("{}", path.display())),
        [command] if command == "ensure-theme" => reconcile(true).map(|_| ()),
        [command] if command == "daemon" => daemon(),
        _ => usage(),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
