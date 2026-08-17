use std::{env, fs};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const ZAPRET_STATE: &str = "/var/lib/vesper-zapret/profile";

fn home() -> PathBuf { env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/nonexistent")) }
fn config_root() -> PathBuf { env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config")).join("vesper") }
fn runtime_root() -> PathBuf { env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|| home().join(".local/state/vesper/runtime")).join("vesper") }
fn die(message: &str) -> ! { eprintln!("{message}"); std::process::exit(1) }

fn output(command: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(command).args(args).output().map_err(|e| format!("failed to run {command}: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() { format!("{command} failed") } else { err });
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
fn success(command: &str, args: &[&str]) -> bool {
    Command::new(command).args(args).stdout(Stdio::null()).stderr(Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
}
fn escape(value: &str) -> String { value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r") }
fn radio(kind: &str) -> bool { output("nmcli", &["radio", kind]).map(|v| v == "enabled").unwrap_or(false) }
fn bluetooth() -> bool { output("bluetoothctl", &["show"]).map(|v| v.lines().any(|l| l.trim() == "Powered: yes")).unwrap_or(false) }
fn set_radio(kind: &str, on: bool) -> Result<(), String> { if success("nmcli", &["radio", kind, if on { "on" } else { "off" }]) { Ok(()) } else { Err(format!("NetworkManager rejected {kind} radio change")) } }
fn set_bluetooth(on: bool) { let _ = Command::new("bluetoothctl").args(["power", if on { "on" } else { "off" }]).stdout(Stdio::null()).stderr(Stdio::null()).status(); }

fn active_wifi() -> Option<String> {
    let text = output("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show", "--active"]).ok()?;
    text.lines().find_map(|line| {
        let (name, kind) = line.rsplit_once(':')?;
        (kind == "802-11-wireless" || kind == "wifi").then(|| name.replace("\\:", ":"))
    })
}
fn status() {
    let wifi = radio("wifi"); let wwan = radio("wwan"); let bt = bluetooth();
    let connection = active_wifi().unwrap_or_default();
    let zapret = success("systemctl", &["is-active", "--quiet", "nfqws2@default.service"]);
    let proxy = config_root().join("proxy.env").exists();
    println!("{{\"airplane\":{},\"wifi\":{},\"wwan\":{},\"bluetooth\":{},\"connection\":\"{}\",\"zapret\":{},\"proxy\":{}}}", !wifi && !wwan && !bt, wifi, wwan, bt, escape(&connection), zapret, proxy);
}

fn airplane_path() -> PathBuf { runtime_root().join("airplane-state") }
fn save_airplane(wifi: bool, wwan: bool, bt: bool) -> Result<(), String> {
    fs::create_dir_all(runtime_root()).map_err(|e| e.to_string())?;
    fs::write(airplane_path(), format!("wifi={}\nwwan={}\nbluetooth={}\n", wifi as u8, wwan as u8, bt as u8)).map_err(|e| e.to_string())
}
fn load_airplane() -> Option<(bool, bool, bool)> {
    let text = fs::read_to_string(airplane_path()).ok()?;
    let get = |key: &str| text.lines().find_map(|l| l.strip_prefix(&format!("{key}=")).map(|v| v == "1"));
    Some((get("wifi")?, get("wwan")?, get("bluetooth")?))
}
fn airplane(on: bool) -> Result<(), String> {
    if on {
        save_airplane(radio("wifi"), radio("wwan"), bluetooth())?;
        set_radio("wifi", false)?; set_radio("wwan", false)?; set_bluetooth(false); return Ok(());
    }
    let (wifi, wwan, bt) = load_airplane().unwrap_or((true, true, true));
    set_radio("wifi", wifi)?; set_radio("wwan", wwan)?; set_bluetooth(bt);
    match fs::remove_file(airplane_path()) { Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(e.to_string()) }
}

fn wifi_escape(v: &str) -> String { v.replace('\\', "\\\\").replace(';', "\\;").replace(',', "\\,").replace(':', "\\:") }
fn wifi_qr() -> Result<PathBuf, String> {
    let connection = active_wifi().ok_or_else(|| "no active Wi-Fi connection".to_string())?;
    let values = output("nmcli", &["-s", "-g", "802-11-wireless.ssid,802-11-wireless-security.key-mgmt,802-11-wireless-security.psk", "connection", "show", &connection])?;
    let mut lines = values.lines(); let ssid = lines.next().unwrap_or("").trim(); let key = lines.next().unwrap_or("").trim(); let password = lines.next().unwrap_or("").trim();
    if ssid.is_empty() { return Err("active Wi-Fi has no SSID".to_string()); }
    let payload = format!("WIFI:T:{};S:{};P:{};;", if key.is_empty() || key == "none" { "nopass" } else { "WPA" }, wifi_escape(ssid), wifi_escape(password));
    fs::create_dir_all(runtime_root()).map_err(|e| e.to_string())?;
    let path = runtime_root().join("wifi-share.svg"); let path_s = path.to_string_lossy().into_owned();
    let mut child = Command::new("qrencode").args(["-t", "SVG", "-o", &path_s, "-m", "2"]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() { stdin.write_all(payload.as_bytes()).map_err(|e| e.to_string())?; }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).trim().to_string()); }
    Ok(path)
}

fn parse_token(token: &str) -> Option<(u8, &'static str)> {
    let (r, s) = token.strip_prefix('r')?.split_once('-')?;
    let repeats = match r { "1" => 1, "2" => 2, "4" => 4, "6" => 6, _ => return None };
    let split = match s { "default" => "default", "method" => "method", "sni" => "sni", _ => return None };
    Some((repeats, split))
}
fn zapret_status() {
    let runtime = fs::read_to_string(ZAPRET_STATE).ok().map(|v| v.trim().to_string()).filter(|v| parse_token(v).is_some());
    let token = runtime.as_deref().unwrap_or("r1-default"); let (repeats, split) = parse_token(token).unwrap();
    let split_pos = match split { "method" => "method+2,midsld", "sni" => "1,sniext+1,midsld", _ => "1,midsld" };
    let active = success("systemctl", &["is-active", "--quiet", "nfqws2@default.service"]);
    println!("{{\"active\":{},\"repeats\":{},\"split\":\"{}\",\"splitPos\":\"{}\",\"runtimeOverride\":{},\"scope\":\"TCP 443 · first 16 packets · adaptive hostlist\"}}", active, repeats, split, split_pos, runtime.is_some());
}
fn start_unit(unit: &str) -> Result<(), String> {
    let out = Command::new("systemctl").args(["start", unit]).output().map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { let e = String::from_utf8_lossy(&out.stderr).trim().to_string(); Err(if e.is_empty() { "systemd rejected the change".to_string() } else { e }) }
}
fn zapret_set(repeats: &str, split: &str) -> Result<(), String> {
    let token = format!("r{repeats}-{split}"); if parse_token(&token).is_none() { return Err("unsupported Zapret2 tuning value".to_string()); }
    start_unit(&format!("vesper-zapret-profile@{token}.service"))
}

fn usage() -> ! { eprintln!("vesper-network: status | airplane on|off | wifi-qr | zapret status | zapret set <1|2|4|6> <default|method|sni> | zapret reset"); std::process::exit(2) }
fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [c] if c == "status" => status(),
        [c, v] if c == "airplane" => airplane(v == "on").unwrap_or_else(|e| die(&e)),
        [c] if c == "wifi-qr" => println!("{}", wifi_qr().unwrap_or_else(|e| die(&e)).display()),
        [g, a] if g == "zapret" && a == "status" => zapret_status(),
        [g, a, r, s] if g == "zapret" && a == "set" => zapret_set(r, s).unwrap_or_else(|e| die(&e)),
        [g, a] if g == "zapret" && a == "reset" => start_unit("vesper-zapret-profile@reset.service").unwrap_or_else(|e| die(&e)),
        _ => usage(),
    }
}
