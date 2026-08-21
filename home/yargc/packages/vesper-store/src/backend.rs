use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CATALOG_SCHEMA_VERSION: u32 = 1;
const REQUIRED_TABLES: u32 = 8;
const REQUIRED_COLUMNS: u32 = 44;

const CATALOG_SCHEMA_QUERY: &str = r#"
PRAGMA user_version;
WITH required_tables(name) AS (
    VALUES
        ('apps'),
        ('variants'),
        ('categories'),
        ('app_categories'),
        ('screenshots'),
        ('keywords'),
        ('aliases'),
        ('apps_fts')
), required_columns(table_name, column_name) AS (
    VALUES
        ('apps', 'id'), ('apps', 'name'), ('apps', 'generic_name'),
        ('apps', 'summary'), ('apps', 'description'), ('apps', 'appstream_id'),
        ('apps', 'desktop_id'), ('apps', 'homepage'), ('apps', 'icon_key'),
        ('apps', 'primary_category'),
        ('variants', 'id'), ('variants', 'app_id'), ('variants', 'source_kind'),
        ('variants', 'source_id'), ('variants', 'package_attr'),
        ('variants', 'package_version'), ('variants', 'flatpak_id'),
        ('variants', 'license'), ('variants', 'sandbox_kind'),
        ('variants', 'install_kind'), ('variants', 'supported'),
        ('variants', 'broken'), ('variants', 'insecure'),
        ('categories', 'id'), ('categories', 'name'),
        ('app_categories', 'app_id'), ('app_categories', 'category_id'),
        ('screenshots', 'id'), ('screenshots', 'app_id'), ('screenshots', 'url'),
        ('screenshots', 'caption'), ('screenshots', 'position'),
        ('keywords', 'app_id'), ('keywords', 'keyword'),
        ('aliases', 'app_id'), ('aliases', 'alias'),
        ('apps_fts', 'app_id'), ('apps_fts', 'name'),
        ('apps_fts', 'generic_name'), ('apps_fts', 'aliases'),
        ('apps_fts', 'keywords'), ('apps_fts', 'package_attr'),
        ('apps_fts', 'summary'), ('apps_fts', 'description')
)
SELECT
    (SELECT count(*) FROM required_tables r
     WHERE EXISTS (
         SELECT 1 FROM sqlite_master m
         WHERE m.type = 'table' AND m.name = r.name
     )),
    (SELECT count(*) FROM required_columns c
     WHERE EXISTS (
         SELECT 1 FROM pragma_table_info(c.table_name) p
         WHERE p.name = c.column_name
     )),
    (SELECT count(*) FROM sqlite_master
     WHERE type = 'table' AND name = 'apps_fts'
       AND lower(sql) LIKE '%using fts5%');
"#;

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

fn catalog_metadata_path(catalog: &Path) -> PathBuf {
    env::var_os("VESPER_STORE_CATALOG_META")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            catalog
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("catalog-meta.json")
        })
}

fn inspect_metadata(path: &Path, schema_version: u32) -> Result<(), String> {
    if !path.is_file() {
        return Err("catalogue metadata file does not exist".to_string());
    }

    let expected_system = env::var("VESPER_STORE_EXPECTED_SYSTEM").unwrap_or_default();
    let output = Command::new("jq")
        .args([
            "-e",
            "--argjson",
            "expectedSchema",
            &schema_version.to_string(),
            "--arg",
            "expectedSystem",
            &expected_system,
            r#"
                if type != "object"
                   or (.schemaVersion | type) != "number"
                   or (.schemaVersion != $expectedSchema)
                   or (.system | type) != "string"
                   or (.system == "")
                   or ($expectedSystem != "" and .system != $expectedSystem)
                   or (.nixpkgsRevision | type) != "string"
                   or (.nixpkgsRevision | test("^[0-9a-fA-F]{7,64}$") | not)
                   or (.generatedAt | type) != "string"
                   or (.generatedAt == "")
                then error("catalogue metadata contract mismatch")
                else true
                end
            "#,
        ])
        .arg(path)
        .output()
        .map_err(|error| format!("could not run jq: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if error.is_empty() {
        "catalogue metadata contract mismatch".to_string()
    } else {
        error
    })
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
        .arg(CATALOG_SCHEMA_QUERY)
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
    let inventory = lines
        .next()
        .ok_or_else(|| "catalogue table inventory is missing".to_string())?
        .split('|')
        .map(str::trim)
        .map(|value| value.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "catalogue table inventory is invalid".to_string())?;

    if version != CATALOG_SCHEMA_VERSION {
        return Err(format!(
            "catalogue schema mismatch: expected {CATALOG_SCHEMA_VERSION}, got {version}"
        ));
    }
    if inventory.len() != 3 {
        return Err("catalogue schema inventory is incomplete".to_string());
    }
    if inventory[0] != REQUIRED_TABLES {
        return Err("catalogue is missing required tables".to_string());
    }
    if inventory[1] != REQUIRED_COLUMNS {
        return Err("catalogue is missing required columns".to_string());
    }
    if inventory[2] != 1 {
        return Err("catalogue FTS table is missing or invalid".to_string());
    }

    inspect_metadata(&catalog_metadata_path(&canonical), version)
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
