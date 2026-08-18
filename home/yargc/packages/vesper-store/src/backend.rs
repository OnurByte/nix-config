use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CATALOG_SCHEMA_VERSION: u32 = 1;
const REQUIRED_TABLES: u32 = 3;

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn catalog_path() -> Option<PathBuf> {
    env::var_os("VESPER_STORE_CATALOG")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn inspect_catalog(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("catalogue file does not exist".to_string());
    }

    let canonical = path
        .canonicalize()
        .map_err(|error| format!("could not resolve catalogue path: {error}"))?;
    let output = Command::new("sqlite3")
        .args(["-readonly", "-batch", "-noheader"])
        .arg(&canonical)
        .arg(
            "PRAGMA user_version; \
             SELECT count(*) FROM sqlite_master \
             WHERE type = 'table' AND name IN ('apps', 'variants', 'apps_fts');",
        )
        .output()
        .map_err(|error| format!("could not run sqlite3: {error}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if error.is_empty() {
            "sqlite3 rejected catalogue".to_string()
        } else {
            error
        });
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|_| "sqlite3 returned non-UTF-8 output".to_string())?;
    let mut lines = stdout.lines().map(str::trim).filter(|line| !line.is_empty());
    let version = lines
        .next()
        .ok_or_else(|| "catalogue schema version is missing".to_string())?
        .parse::<u32>()
        .map_err(|_| "catalogue schema version is invalid".to_string())?;
    let required_tables = lines
        .next()
        .ok_or_else(|| "catalogue table inventory is missing".to_string())?
        .parse::<u32>()
        .map_err(|_| "catalogue table inventory is invalid".to_string())?;

    if version != CATALOG_SCHEMA_VERSION {
        return Err(format!(
            "catalogue schema mismatch: expected {CATALOG_SCHEMA_VERSION}, got {version}"
        ));
    }
    if required_tables != REQUIRED_TABLES {
        return Err("catalogue is missing required tables".to_string());
    }

    Ok(())
}

fn print_catalog_status() {
    match catalog_path() {
        Some(path) => {
            let display = json_escape(&path.to_string_lossy());
            match inspect_catalog(&path) {
                Ok(()) => println!(
                    "{{\"schemaVersion\":{CATALOG_SCHEMA_VERSION},\"available\":true,\"path\":\"{display}\",\"error\":\"\"}}"
                ),
                Err(error) => println!(
                    "{{\"schemaVersion\":{CATALOG_SCHEMA_VERSION},\"available\":false,\"path\":\"{display}\",\"error\":\"{}\"}}",
                    json_escape(&error)
                ),
            }
        }
        None => println!(
            "{{\"schemaVersion\":{CATALOG_SCHEMA_VERSION},\"available\":false,\"path\":\"\",\"error\":\"catalogue path not configured\"}}"
        ),
    }
}

fn print_sources() {
    println!(
        "{}",
        r#"{"schemaVersion":1,"nixpkgs":{"enabled":true,"default":true},"flathub":{"enabled":false,"default":false}}"#
    );
}

fn usage() -> ! {
    eprintln!("usage: vesper-store-core <catalog-status|sources>");
    std::process::exit(2);
}

fn main() {
    match env::args().nth(1).as_deref() {
        Some("catalog-status") => print_catalog_status(),
        Some("sources") => print_sources(),
        _ => usage(),
    }
}
