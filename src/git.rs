use std::io;

use crate::runner::RealRunner;
use crate::types::Runner;

/// Run `git <args>` with inherited stdio, returning git's exit code.
pub fn run_git(args: &[String]) -> io::Result<i32> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    RealRunner.passthrough("git", &refs)
}

// TODO(phase 3): is_git_installed, is_inside_work_tree, set_local_identity,
// origin_url, is_ssh_remote.
