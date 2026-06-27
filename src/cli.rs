use crate::commands;
use crate::git;
use crate::log;

const HELP: &str = "gw — wraps git, switching the active GitHub CLI account per project.

Usage:
  gw <git args...>     Switch to the project's account, then run git
  gw doctor            Diagnose git/gh install and auth state
  gw which             Show the account/host resolved for this directory
  gw init [account]    Scaffold a .gitwrapper file (adds it to .gitignore;
                       pass --no-gitignore to skip)
  gw --gw-version      Print gw's version
  gw --gw-help         Show this help

Anything that is not a gw meta-command is passed straight through to git.
";

/// Route meta-commands; pass everything else through to git. Returns the
/// process exit code.
pub fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("--gw-version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            0
        }
        Some("--gw-help") => {
            print!("{HELP}");
            0
        }
        Some("doctor") => commands::doctor::doctor(),
        Some("which") => commands::which::which(),
        Some("init") => commands::init::init(&args[1..]),
        // Switch to the project's gh account, run git, then restore. Account
        // switching arrives in a later phase; today this is pure passthrough.
        _ => match git::run_git(args) {
            Ok(code) => code,
            Err(err) => {
                log::error(&format!("failed to run git: {err}"));
                1
            }
        },
    }
}
