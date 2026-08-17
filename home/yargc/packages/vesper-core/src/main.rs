mod apps;
mod json;
mod network;
mod paths;
mod process;
mod wellbeing;

use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn fail(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    std::process::exit(1);
}

fn legacy(args: &[String]) -> ! {
    let current = env::current_exe().unwrap_or_else(|error| fail(format!("cannot resolve vesper-control path: {error}")));
    let legacy = current.with_file_name("vesper-control-legacy");
    let error = Command::new(&legacy).args(args).exec();
    fail(format!("failed to execute {}: {error}", legacy.display()));
}

fn on_off(value: &str) -> bool {
    match value {
        "on" => true,
        "off" => false,
        _ => fail("expected on or off"),
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [group, action] if group == "network" && action == "status" => {
            println!("{}", network::status_json());
        }
        [group, action, value] if group == "network" && action == "airplane" => {
            network::set_airplane(on_off(value)).unwrap_or_else(|error| fail(error));
        }
        [command, id] if command == "app-status" => {
            println!("{}", apps::status_json(id));
        }
        [command, id, permission, value] if command == "app-permission" => {
            apps::set_permission(id, permission, on_off(value)).unwrap_or_else(|error| fail(error));
        }
        [command, id] if command == "app-reset-permissions" => {
            apps::reset_all(id).unwrap_or_else(|error| fail(error));
        }
        [command, id, filesystem, value] if command == "app-filesystem" => {
            apps::set_filesystem(id, filesystem, on_off(value)).unwrap_or_else(|error| fail(error));
        }
        [command, id, bus, name, access] if command == "app-dbus" => {
            apps::set_dbus(id, bus, name, access).unwrap_or_else(|error| fail(error));
        }
        [command] if command == "control-version" => {
            println!("0.3.0");
        }
        _ => legacy(&args),
    }
}
