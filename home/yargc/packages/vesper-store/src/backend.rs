use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CATALOG_SCHEMA_VERSION: u32 = 1;
const REQUIRED_TABLES: u32 = 8;
const REQUIRED_COLUMNS: u32 = 44;
const SEARCH_LIMIT: u32 = 24;

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
    let expected_revision =
        env::var("VESPER_STORE_EXPECTED_NIXPKGS_REVISION").unwrap_or_default();
    let output = Command::new("jq")
        .args([
            "-e",
            "--argjson",
            "expectedSchema",
            &schema_version.to_string(),
            "--arg",
            "expectedSystem",
            &expected_system,
            "--arg",
            "expectedRevision",
            &expected_revision,
            r#"
                if type != "object"
                   or (.schemaVersion | type) != "number"
                   or (.schemaVersion != $expectedSchema)
                   or (.system | type) != "string"
                   or (.system == "")
                   or ($expectedSystem != "" and .system != $expectedSystem)
                   or (.nixpkgsRevision | type) != "string"
                   or (.nixpkgsRevision | test("^[0-9a-fA-F]{7,64}$") | not)
                   or ($expectedRevision != "" and .nixpkgsRevision != $expectedRevision)
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

fn normalized_search_terms(query: &str) -> Result<Vec<String>, String> {
    if query.chars().count() > 128 {
        return Err("search query is too long".to_string());
    }

    let mut terms = Vec::new();
    let mut current = String::new();
    for character in query.chars() {
        if character.is_alphanumeric() || matches!(character, '_' | '-' | '.') {
            current.push(character);
        } else if !current.is_empty() {
            terms.push(std::mem::take(&mut current));
        }
        if terms.len() == 8 {
            break;
        }
    }
    if !current.is_empty() && terms.len() < 8 {
        terms.push(current);
    }

    for term in &mut terms {
        *term = term.chars().take(48).collect();
    }
    terms.retain(|term| !term.is_empty());
    Ok(terms)
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn search_catalog(path: &Path, query: &str) -> Result<String, String> {
    inspect_catalog(path)?;
    let terms = normalized_search_terms(query)?;
    if terms.is_empty() {
        return Ok("[]".to_string());
    }

    let fts_query = terms
        .iter()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ");
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("could not resolve catalogue path: {error}"))?;
    let sql = format!(
        "SELECT apps.id AS id, apps.name AS name, apps.summary AS summary, \
                COALESCE((SELECT variants.source_kind FROM variants \
                          WHERE variants.app_id = apps.id ORDER BY variants.id LIMIT 1), '') AS source, \
                COALESCE((SELECT variants.package_attr FROM variants \
                          WHERE variants.app_id = apps.id ORDER BY variants.id LIMIT 1), '') AS packageAttr \
         FROM apps_fts \
         JOIN apps ON apps.id = apps_fts.app_id \
         WHERE apps_fts MATCH {} \
         ORDER BY bm25(apps_fts, 0.0, 10.0, 4.0, 9.0, 7.0, 6.0, 2.0, 1.0), apps.name \
         LIMIT {SEARCH_LIMIT};",
        sql_literal(&fts_query)
    );
    let output = Command::new("sqlite3")
        .args(["-readonly", "-batch", "-json"])
        .arg(canonical)
        .arg(sql)
        .output()
        .map_err(|error| format!("could not run sqlite3: {error}"))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if error.is_empty() {
            "catalogue search failed".to_string()
        } else {
            error
        });
    }

    let result = String::from_utf8(output.stdout)
        .map_err(|_| "catalogue search returned non-UTF-8 output".to_string())?;
    let result = result.trim();
    if result.is_empty() {
        return Ok("[]".to_string());
    }
    Ok(result.to_string())
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

fn print_search(query: &str) {
    let escaped_query = json_escape(query);
    match catalog_path() {
        Some(path) => match search_catalog(&path, query) {
            Ok(results) => println!(
                "{{\"available\":true,\"query\":\"{escaped_query}\",\"results\":{results}}}"
            ),
            Err(error) => println!(
                "{{\"available\":false,\"query\":\"{escaped_query}\",\"results\":[],\"error\":\"{}\"}}",
                json_escape(&error)
            ),
        },
        None => println!(
            "{{\"available\":false,\"query\":\"{escaped_query}\",\"results\":[],\"error\":\"catalogue path not configured\"}}"
        ),
    }
}

fn usage() -> ! {
    eprintln!("usage: vesper-store-core <catalog-status|sources|search QUERY>");
    std::process::exit(2);
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("catalog-status") => print_catalog_status(),
        Some("sources") => print_sources(),
        Some("search") => print_search(&args.collect::<Vec<_>>().join(" ")),
        _ => usage(),
    }
}
