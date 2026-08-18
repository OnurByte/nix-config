use std::env;
use std::process::{Child, Command, ExitStatus};
use std::thread;
use std::time::Duration;

fn run(command: &str, args: &[String]) -> Result<ExitStatus, String> {
    Command::new(command)
        .args(args)
        .status()
        .map_err(|error| format!("failed to run {command}: {error}"))
}

fn sync_queue() -> Result<(), String> {
    let status = Command::new("vesper-icon-queue")
        .arg("sync")
        .status()
        .map_err(|error| format!("failed to run vesper-icon-queue: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "vesper-icon-queue sync exited with {}",
            status.code().unwrap_or(-1)
        ))
    }
}

fn retry_queue_app(id: &str) {
    let _ = Command::new("vesper-icon-queue")
        .args(["retry-app", id])
        .status();
}

fn stop(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn daemon() -> Result<(), String> {
    let mut engine = Command::new("vesper-icon-engine-core")
        .arg("daemon")
        .spawn()
        .map_err(|error| format!("failed to start icon engine: {error}"))?;
    let mut queue = match Command::new("vesper-icon-queue").arg("daemon").spawn() {
        Ok(child) => child,
        Err(error) => {
            stop(&mut engine);
            return Err(format!("failed to start icon queue: {error}"));
        }
    };

    loop {
        if let Some(status) = engine
            .try_wait()
            .map_err(|error| format!("failed to poll icon engine: {error}"))?
        {
            stop(&mut queue);
            return Err(format!(
                "icon engine exited with {}",
                status.code().unwrap_or(-1)
            ));
        }
        if let Some(status) = queue
            .try_wait()
            .map_err(|error| format!("failed to poll icon queue: {error}"))?
        {
            stop(&mut engine);
            return Err(format!(
                "icon queue exited with {}",
                status.code().unwrap_or(-1)
            ));
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if matches!(args.as_slice(), [command] if command == "daemon") {
        if let Err(error) = daemon() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    if matches!(args.as_slice(), [command] if command == "queue-status") {
        let status = run("vesper-icon-queue", &["status".to_string()])
            .unwrap_or_else(|error| {
                eprintln!("{error}");
                std::process::exit(1);
            });
        std::process::exit(status.code().unwrap_or(1));
    }

    let status = run("vesper-icon-engine-core", &args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    if matches!(args.as_slice(), [command, id] if command == "app-retry") {
        retry_queue_app(id);
    } else if matches!(
        args.first().map(String::as_str),
        Some("reconcile" | "enable" | "disable" | "provider" | "app-exclude" | "rebuild-canonical" | "ensure-theme")
    ) {
        if let Err(error) = sync_queue() {
            eprintln!("warning: {error}");
        }
    }
}
