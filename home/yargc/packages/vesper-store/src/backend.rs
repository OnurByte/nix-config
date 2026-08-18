use std::env;
use std::path::PathBuf;

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn catalog_path() -> Option<PathBuf> {
    env::var_os("VESPER_STORE_CATALOG")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn print_catalog_status() {
    match catalog_path() {
        Some(path) => {
            let display = path.to_string_lossy();
            println!(
                "{{\"schemaVersion\":1,\"available\":{},\"path\":\"{}\"}}",
                path.is_file(),
                json_escape(&display)
            );
        }
        None => println!("{}", r#"{"schemaVersion":1,"available":false,"path":""}"#),
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
