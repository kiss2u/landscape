mod cmd;
mod env;
mod runner;
mod scripted_server;
mod scripted_server_tests;
mod standard_server_tests;

use std::fs;

fn require_root() {
    std::env::set_var("LANDSCAPE_IGNORE_CLI_ARGS", "1");

    let status = match fs::read_to_string("/proc/self/status") {
        Ok(s) => s,
        Err(_) => return,
    };
    let uid = status
        .lines()
        .find(|line| line.starts_with("Uid:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if uid != 0 {
        eprintln!("skipping test: requires root privileges (current uid: {uid})");
        std::process::exit(0);
    }
}
