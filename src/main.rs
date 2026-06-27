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
