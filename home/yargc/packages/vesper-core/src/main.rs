mod json;
mod network;
mod paths;
mod process;

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

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [group, action] if group == "network" && action == "status" => {
            println!("{}", network::status_json());
        }
        [group, action, value] if group == "network" && action == "airplane" => {
            let enabled = match value.as_str() {
                "on" => true,
                "off" => false,
                _ => fail("network airplane expects on or off"),
            };
            network::set_airplane(enabled).unwrap_or_else(|error| fail(error));
        }
        [command] if command == "control-version" => {
            println!("0.3.0");
        }
        _ => legacy(&args),
    }
}
