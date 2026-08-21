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

fn sync_vicons() -> Result<(), String> {
    let status = Command::new("vesper-icon-worker")
        .arg("sync-vicons")
        .status()
        .map_err(|error| format!("failed to run vesper-icon-worker: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "vesper-icon-worker sync-vicons exited with {}",
            status.code().unwrap_or(-1)
        ))
    }
}

fn sync_identity() -> Result<(), String> {
    let status = Command::new("vesper-icon-identity")
        .arg("sync")
        .status()
        .map_err(|error| format!("failed to run vesper-icon-identity: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "vesper-icon-identity sync exited with {}",
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

fn stop_all(children: &mut [&mut Child]) {
    for child in children {
        stop(child);
    }
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
    let mut worker = match Command::new("vesper-icon-worker").arg("daemon").spawn() {
        Ok(child) => child,
        Err(error) => {
            stop_all(&mut [&mut engine, &mut queue]);
            return Err(format!("failed to start icon worker: {error}"));
        }
    };
    let mut identity = match Command::new("vesper-icon-identity").arg("daemon").spawn() {
        Ok(child) => child,
        Err(error) => {
            stop_all(&mut [&mut engine, &mut queue, &mut worker]);
            return Err(format!("failed to start icon identity resolver: {error}"));
        }
    };

    loop {
        if let Some(status) = engine
            .try_wait()
            .map_err(|error| format!("failed to poll icon engine: {error}"))?
        {
            stop_all(&mut [&mut queue, &mut worker, &mut identity]);
            return Err(format!(
                "icon engine exited with {}",
                status.code().unwrap_or(-1)
            ));
        }
        if let Some(status) = queue
            .try_wait()
            .map_err(|error| format!("failed to poll icon queue: {error}"))?
        {
            stop_all(&mut [&mut engine, &mut worker, &mut identity]);
            return Err(format!(
                "icon queue exited with {}",
                status.code().unwrap_or(-1)
            ));
        }
        if let Some(status) = worker
            .try_wait()
            .map_err(|error| format!("failed to poll icon worker: {error}"))?
        {
            stop_all(&mut [&mut engine, &mut queue, &mut identity]);
            return Err(format!(
                "icon worker exited with {}",
                status.code().unwrap_or(-1)
            ));
        }
        if let Some(status) = identity
            .try_wait()
            .map_err(|error| format!("failed to poll icon identity resolver: {error}"))?
        {
            stop_all(&mut [&mut engine, &mut queue, &mut worker]);
            return Err(format!(
                "icon identity resolver exited with {}",
                status.code().unwrap_or(-1)
            ));
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn passthrough(command: &str, args: &[String]) -> ! {
    let status = run(command, args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    std::process::exit(status.code().unwrap_or(if status.success() { 0 } else { 1 }));
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
        passthrough("vesper-icon-queue", &["status".to_string()]);
    }
    if matches!(args.as_slice(), [command] if command == "queue-pause") {
        passthrough("vesper-icon-queue", &["pause".to_string()]);
    }
    if matches!(args.as_slice(), [command] if command == "queue-resume") {
        passthrough("vesper-icon-queue", &["resume".to_string()]);
    }
    if let [command, value] = args.as_slice() {
        if command == "queue-app-status" {
            passthrough(
                "vesper-icon-queue",
                &["app-status".to_string(), value.to_string()],
            );
        }
        if command == "identity-resolve" {
            passthrough(
                "vesper-icon-identity",
                &["resolve".to_string(), value.to_string()],
            );
        }
        if command == "export-app" {
            passthrough(
                "vesper-icon-worker",
                &["export-app".to_string(), value.to_string()],
            );
        }
    }

    let status = run("vesper-icon-engine-core", &args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    if let [command, id] = args.as_slice() {
        if command == "app-retry" {
            retry_queue_app(id);
        }
    }

    if matches!(
        args.first().map(String::as_str),
        Some(
            "reconcile"
                | "enable"
                | "disable"
                | "provider"
                | "remote-consent"
                | "app-exclude"
                | "app-retry"
                | "rebuild-canonical"
                | "ensure-theme"
                | "mode"
                | "material"
                | "follow-palette"
                | "sync-theme"
        )
    ) {
        if let Err(error) = sync_queue() {
            eprintln!("warning: {error}");
        }
        if let Err(error) = sync_vicons() {
            eprintln!("warning: {error}");
        }
        if let Err(error) = sync_identity() {
            eprintln!("warning: {error}");
        }
    }
}
