use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    state: PathBuf,
    data: PathBuf,
    config: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "vesper-icon-consent-test-{}-{id}",
            std::process::id()
        ));
        let state = root.join("state");
        let data = root.join("data");
        let config = root.join("config");
        fs::create_dir_all(state.join("vesper/adaptive-icons")).unwrap();
        fs::create_dir_all(&data).unwrap();
        fs::create_dir_all(config.join("vesper")).unwrap();
        Self {
            root,
            state,
            data,
            config,
        }
    }

    fn pending_raster(&self) {
        fs::write(
            self.state.join("vesper/adaptive-icons/inventory.tsv"),
            format!(
                "fixture.desktop\tfixture\t/tmp/fixture.png\t{}\tpng\tpending-ai\t0\t0\t\n",
                "d".repeat(64)
            ),
        )
        .unwrap();
    }

    fn process_once(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_vesper-icon-worker"))
            .arg("process-once")
            .env("HOME", &self.root)
            .env("XDG_STATE_HOME", &self.state)
            .env("XDG_DATA_HOME", &self.data)
            .env("XDG_CONFIG_HOME", &self.config)
            .env("PATH", "/nonexistent")
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn worker_never_claims_remote_work_without_consent() {
    let fixture = Fixture::new();
    fixture.pending_raster();
    fs::write(
        fixture.config.join("vesper/adaptive-icons.conf"),
        "enabled=1\nprovider=openai\nremoteConsent=0\n",
    )
    .unwrap();

    let output = fixture.process_once();
    assert!(
        output.status.success(),
        "worker failed instead of respecting consent: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "idle");
}

#[test]
fn missing_consent_key_defaults_to_denied() {
    let fixture = Fixture::new();
    fixture.pending_raster();
    fs::write(
        fixture.config.join("vesper/adaptive-icons.conf"),
        "enabled=1\nprovider=openai\n",
    )
    .unwrap();

    let output = fixture.process_once();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "idle");
}
