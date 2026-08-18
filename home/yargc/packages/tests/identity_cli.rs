use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    state: PathBuf,
    data: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "vesper-icon-identity-test-{}-{id}",
            std::process::id()
        ));
        let state = root.join("state");
        let data = root.join("data");
        fs::create_dir_all(state.join("vesper/adaptive-icons")).unwrap();
        fs::create_dir_all(data.join("applications")).unwrap();
        Self { root, state, data }
    }

    fn desktop(&self, id: &str, body: &str, icon: &str) {
        fs::write(self.data.join("applications").join(id), body).unwrap();
        let inventory = self.state.join("vesper/adaptive-icons/inventory.tsv");
        let mut existing = fs::read_to_string(&inventory).unwrap_or_default();
        existing.push_str(&format!(
            "{id}\t{icon}\t/tmp/source.svg\t{}\tsvg\tvalidated\t1\t0\t\n",
            "a".repeat(64)
        ));
        fs::write(inventory, existing).unwrap();
    }

    fn resolve(&self, runtime_id: &str) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_vesper-icon-identity"))
            .args(["resolve", runtime_id])
            .env("HOME", &self.root)
            .env("XDG_STATE_HOME", &self.state)
            .env("XDG_DATA_HOME", &self.data)
            .env("XDG_DATA_DIRS", &self.data)
            .env_remove("NIX_PROFILES")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "resolver failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn desktop(exec: &str, extra: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Fixture\nIcon=fixture\nExec={exec}\n{extra}"
    )
}

#[test]
fn steam_games_do_not_collapse_into_client() {
    let fixture = Fixture::new();
    fixture.desktop(
        "steam-730.desktop",
        &desktop("steam -applaunch 730", ""),
        "steam-730",
    );
    fixture.desktop(
        "steam-570.desktop",
        &desktop("steam steam://rungameid/570", ""),
        "steam-570",
    );

    let cs = fixture.resolve("steam:730");
    assert!(cs.contains("\"desktopId\":\"steam-730.desktop\""));
    let dota = fixture.resolve("steam:570");
    assert!(dota.contains("\"desktopId\":\"steam-570.desktop\""));
    assert!(fixture.resolve("steam").contains("\"resolved\":false"));
}

#[test]
fn two_browser_pwas_remain_distinct() {
    let fixture = Fixture::new();
    fixture.desktop(
        "chrome-mail.desktop",
        &desktop("google-chrome --app-id=mail-pwa", ""),
        "chrome-mail",
    );
    fixture.desktop(
        "chrome-chat.desktop",
        &desktop("google-chrome --app-id=chat-pwa", ""),
        "chrome-chat",
    );

    assert!(fixture
        .resolve("mail-pwa")
        .contains("\"desktopId\":\"chrome-mail.desktop\""));
    assert!(fixture
        .resolve("chat-pwa")
        .contains("\"desktopId\":\"chrome-chat.desktop\""));
}

#[test]
fn startup_wm_class_casefold_is_exact_not_fuzzy() {
    let fixture = Fixture::new();
    fixture.desktop(
        "Example.desktop",
        &desktop("/opt/example/bin/example", "StartupWMClass=Example.App\n"),
        "example",
    );

    assert!(fixture
        .resolve("example.app")
        .contains("\"desktopId\":\"Example.desktop\""));
    assert!(fixture.resolve("example.ap").contains("\"resolved\":false"));
    assert!(fixture.resolve("Example App").contains("\"resolved\":false"));
}

#[test]
fn generic_electron_and_wine_runtimes_are_not_aliases() {
    let fixture = Fixture::new();
    fixture.desktop(
        "electron-one.desktop",
        &desktop("electron --class=Electron.One /opt/one/app.asar", ""),
        "one",
    );
    fixture.desktop(
        "wine-tool.desktop",
        &desktop("wine /home/test/.wine/drive_c/tool.exe", ""),
        "tool",
    );

    assert!(fixture.resolve("electron").contains("\"resolved\":false"));
    assert!(fixture.resolve("wine").contains("\"resolved\":false"));
    assert!(fixture
        .resolve("electron.one")
        .contains("\"desktopId\":\"electron-one.desktop\""));
}

#[test]
fn similar_desktop_prefixes_never_choose_alphabetically() {
    let fixture = Fixture::new();
    fixture.desktop(
        "thunar.desktop",
        &desktop("thunar", "StartupWMClass=Thunar\n"),
        "thunar",
    );
    fixture.desktop(
        "thunar-bulk-rename.desktop",
        &desktop("thunar-bulk-rename", "StartupWMClass=ThunarBulkRename\n"),
        "thunar-bulk-rename",
    );

    assert!(fixture
        .resolve("Thunar")
        .contains("\"desktopId\":\"thunar.desktop\""));
    assert!(fixture
        .resolve("ThunarBulkRename")
        .contains("\"desktopId\":\"thunar-bulk-rename.desktop\""));
    assert!(fixture.resolve("thunar-bulk").contains("\"resolved\":false"));
}
