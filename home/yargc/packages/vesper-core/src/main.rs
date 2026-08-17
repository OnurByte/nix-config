mod apps;
mod consumers;
mod icons;
mod json;
mod mcp;
mod network;
mod notifications;
mod paths;
mod privacy;
mod process;
mod providers;
mod recovery;
mod skills;
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
        [group, action] if group == "network" && action == "status" => println!("{}", network::status_json()),
        [group, action, value] if group == "network" && action == "airplane" => network::set_airplane(on_off(value)).unwrap_or_else(|error| fail(error)),
        [group, action] if group == "privacy" && action == "status" => println!("{}", privacy::status_json()),
        [group, action] if group == "recovery" && action == "status" => println!("{}", recovery::status_json()),
        [command, id] if command == "app-status" => println!("{}", apps::status_json(id)),
        [command, id, permission, value] if command == "app-permission" => apps::set_permission(id, permission, on_off(value)).unwrap_or_else(|error| fail(error)),
        [command, id] if command == "app-reset-permissions" => apps::reset_all(id).unwrap_or_else(|error| fail(error)),
        [command, id, filesystem, value] if command == "app-filesystem" => apps::set_filesystem(id, filesystem, on_off(value)).unwrap_or_else(|error| fail(error)),
        [command, id, bus, name, access] if command == "app-dbus" => apps::set_dbus(id, bus, name, access).unwrap_or_else(|error| fail(error)),

        [group, action] if group == "notifications" && action == "status" => println!("{}", notifications::status_json()),
        [group, action, id] if group == "notifications" && action == "get" => println!("{}", notifications::policy_for(id)),
        [group, action, id, name, policy] if group == "notifications" && action == "set" => notifications::set_policy(id, name, policy).unwrap_or_else(|error| fail(error)),

        [group, action] if group == "consumer" && action == "status" => println!("{}", consumers::status_json()),
        [group, action, consumer] if group == "consumer" && action == "credential" => println!("{}", consumers::credential_for(consumer).unwrap_or_else(|error| fail(error))),
        [group, action, consumer, credential] if group == "consumer" && action == "set" => consumers::set_credential(consumer, credential).unwrap_or_else(|error| fail(error)),

        [group, action] if group == "provider" && action == "status" => println!("{}", providers::status_json(false)),
        [group, action, test] if group == "provider" && action == "status" && test == "test" => println!("{}", providers::status_json(true)),
        [group, action, id, name, url, credential] if group == "provider" && action == "add" => providers::add(id, name, url, credential).unwrap_or_else(|error| fail(error)),
        [group, action, id] if group == "provider" && action == "remove" => providers::remove(id).unwrap_or_else(|error| fail(error)),
        [group, action, id, field, value] if group == "provider" && action == "set" => providers::set(id, field, value).unwrap_or_else(|error| fail(error)),
        [group, action, default_provider, default_model, fallbacks] if group == "provider" && action == "routing" => providers::set_routing(default_provider, default_model, fallbacks).unwrap_or_else(|error| fail(error)),

        [group, action] if group == "skills" && action == "status" => println!("{}", skills::status_json()),
        [group, action, name] if group == "skills" && action == "promote" => skills::promote(name).unwrap_or_else(|error| fail(error)),
        [group, action, name] if group == "skills" && action == "enable" => skills::set_enabled(name, true).unwrap_or_else(|error| fail(error)),
        [group, action, name] if group == "skills" && action == "disable" => skills::set_enabled(name, false).unwrap_or_else(|error| fail(error)),
        [group, action, name] if group == "skills" && action == "remove" => skills::remove(name).unwrap_or_else(|error| fail(error)),
        [group, action] if group == "mcp" && action == "status" => println!("{}", mcp::status_json()),

        [group, action] if group == "wellbeing" && action == "status" => println!("{}", if wellbeing::enabled() { "on" } else { "off" }),
        [group, action] if group == "wellbeing" && (action == "on" || action == "off") => wellbeing::set_enabled(action == "on").unwrap_or_else(|error| fail(error)),
        [group, action, value] if group == "wellbeing" && action == "focus" => wellbeing::set_focus(on_off(value)).unwrap_or_else(|error| fail(error)),
        [group, action] if group == "wellbeing" && action == "report" => println!("{}", wellbeing::report_json()),
        [group, action, id] if group == "wellbeing" && action == "app" => println!("{}", wellbeing::app_json(id)),
        [group, action, id, field, value] if group == "wellbeing" && action == "app-set" => wellbeing::set_app_policy(id, field, value).unwrap_or_else(|error| fail(error)),
        [group, action, seconds] if group == "wellbeing" && action == "goal" => {
            let seconds = seconds.parse::<u64>().unwrap_or_else(|_| fail("wellbeing goal expects seconds"));
            wellbeing::set_daily_goal(seconds).unwrap_or_else(|error| fail(error));
        }
        [group, action, scope] if group == "wellbeing" && action == "reset" => wellbeing::reset(scope).unwrap_or_else(|error| fail(error)),
        [command] if command == "wellbeing-daemon" => wellbeing::daemon().unwrap_or_else(|error| fail(error)),
        [command] if command == "wellbeing-summary" => println!("{}", wellbeing::summary_json()),

        [group, action] if group == "icons" && action == "status" => println!("{}", icons::status_json()),
        [group, action] if group == "icons" && action == "reconcile" => icons::reconcile().unwrap_or_else(|error| fail(error)),
        [group, action, id] if group == "icons" && action == "regenerate" => icons::regenerate(id).unwrap_or_else(|error| fail(error)),
        [group, action, key, value] if group == "icons" && action == "set" => icons::set_config(key, value).unwrap_or_else(|error| fail(error)),

        [command] if command == "control-version" => println!("0.8.0"),
        _ => legacy(&args),
    }
}
