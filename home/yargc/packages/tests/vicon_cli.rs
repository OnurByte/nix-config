use std::fs;
use std::path::PathBuf;
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
            "vesper-vicon-test-{}-{id}",
            std::process::id()
        ));
        let state = root.join("state");
        let data = root.join("data");
        fs::create_dir_all(state.join("vesper/adaptive-icons")).unwrap();
        fs::create_dir_all(data.join("vesper/adaptive-icons/canonical")).unwrap();
        Self { root, state, data }
    }

    fn install_vector(&self, desktop_id: &str, fingerprint: &str) -> PathBuf {
        let canonical_dir = self
            .data
            .join("vesper/adaptive-icons/canonical")
            .join(desktop_id)
            .join(fingerprint);
        fs::create_dir_all(&canonical_dir).unwrap();
        fs::write(
            canonical_dir.join("canonical.svg"),
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024"><circle cx="512" cy="512" r="320" fill="#ff3366"/></svg>"##,
        )
        .unwrap();
        fs::write(
            canonical_dir.join("metadata.json"),
            format!(
                "{{\"schemaVersion\":1,\"desktopId\":\"{desktop_id}\",\"sourceFingerprint\":\"{fingerprint}\",\"sourceKind\":\"svg\",\"validation\":\"passed\"}}\n"
            ),
        )
        .unwrap();

        let db = self.state.join("vesper/adaptive-icons/state.sqlite3");
        let schema = format!(
            "CREATE TABLE application_inventory (desktop_id TEXT PRIMARY KEY, desktop_path TEXT NOT NULL, icon_key TEXT NOT NULL, source_path TEXT NOT NULL, source_fingerprint TEXT NOT NULL, source_kind TEXT NOT NULL, canonical_state TEXT NOT NULL, active INTEGER NOT NULL, excluded INTEGER NOT NULL, error TEXT NOT NULL, updated_ms INTEGER NOT NULL); INSERT INTO application_inventory VALUES ('{desktop_id}', '/tmp/{desktop_id}', 'fixture', '/tmp/fixture.svg', '{fingerprint}', 'svg', 'validated', 1, 0, '', 1);"
        );
        let output = Command::new("sqlite3")
            .arg(&db)
            .arg(schema)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "could not seed inventory db: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        canonical_dir
    }

    fn run_sync(&self) {
        let output = Command::new(env!("CARGO_BIN_EXE_vesper-icon-worker"))
            .arg("sync-vicons")
            .env("HOME", &self.root)
            .env("XDG_STATE_HOME", &self.state)
            .env("XDG_DATA_HOME", &self.data)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "sync-vicons failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn clean_svg_becomes_valid_local_vector_vicon() {
    let fixture = Fixture::new();
    let fingerprint = "b".repeat(64);
    let canonical = fixture.install_vector("fixture.desktop", &fingerprint);
    fixture.run_sync();

    let package = canonical.join("icon.vicon");
    let manifest = fs::read_to_string(package.join("manifest.json")).unwrap();
    assert!(manifest.contains("\"width\":1024"));
    assert!(manifest.contains("\"height\":1024"));
    assert!(manifest.contains("\"masked\":false"));
    assert!(manifest.contains("\"kind\":\"local-vector\""));
    assert!(manifest.contains("\"semantic\":{\"schemaVersion\":1,\"retainRaster\":false,\"groups\":1}"));
    assert!(manifest.contains("\"assetType\":\"vector\""));
    assert!(manifest.contains("groups/01-primary/layers/01.svg"));
    let group = fs::read_to_string(package.join("groups/01-primary/group.json")).unwrap();
    assert!(group.contains("\"semanticGroupCount\":1"));

    assert!(package.join("appearances/default.json").is_file());
    assert!(package.join("appearances/dark.json").is_file());
    assert!(package.join("appearances/mono.json").is_file());
    assert!(package.join("groups/01-primary/layers/01.svg").is_file());
}

#[test]
fn local_vicon_generation_is_idempotent() {
    let fixture = Fixture::new();
    let fingerprint = "c".repeat(64);
    let canonical = fixture.install_vector("idempotent.desktop", &fingerprint);
    fixture.run_sync();
    let first = fs::read(canonical.join("icon.vicon/manifest.json")).unwrap();
    fixture.run_sync();
    let second = fs::read(canonical.join("icon.vicon/manifest.json")).unwrap();
    assert_eq!(first, second);
}
