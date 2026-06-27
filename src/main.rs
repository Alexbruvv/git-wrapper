// Crate-wide allow during the staged Rust port: shared types and stub modules
// are defined ahead of the phases that consume them. TODO(v1): remove at
// cutover, once every module is wired up.
#![allow(dead_code)]

mod app;
mod cli;
mod commands;
mod config;
mod gh;
mod git;
mod log;
mod runner;
#[cfg(test)]
mod testutil;
mod types;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(cli::run(&args));
}
