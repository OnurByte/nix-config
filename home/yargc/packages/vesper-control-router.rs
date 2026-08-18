use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn exec(command: &str, args: &[String]) -> ! {
    let error = Command::new(command).args(args).exec();
    eprintln!("failed to exec {command}: {error}");
    std::process::exit(1);
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if matches!(args.first().map(String::as_str), Some("icon")) {
        exec("vesper-icon-engine", &args[1..]);
    }
    exec("vesper-control-core", &args);
}
