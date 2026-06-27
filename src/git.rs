use std::io;

use crate::types::Runner;

/// Run `git <args>` with inherited stdio, returning git's exit code.
pub fn run_git(args: &[String], runner: &dyn Runner) -> io::Result<i32> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    runner.passthrough("git", &refs)
}

/// Whether the `git` binary is available on PATH.
pub fn is_git_installed(runner: &dyn Runner) -> bool {
    matches!(runner.capture("git", &["--version"]), Ok(r) if r.code == 0)
}

/// Whether the current directory is inside a git work tree.
pub fn is_inside_work_tree(runner: &dyn Runner) -> bool {
    match runner.capture("git", &["rev-parse", "--is-inside-work-tree"]) {
        Ok(r) => r.code == 0 && r.stdout.trim() == "true",
        Err(_) => false,
    }
}

/// Apply repo-local git identity. Best-effort; ignores failures.
pub fn set_local_identity(name: Option<&str>, email: Option<&str>, runner: &dyn Runner) {
    if let Some(name) = name {
        let _ = runner.capture("git", &["config", "--local", "user.name", name]);
    }
    if let Some(email) = email {
        let _ = runner.capture("git", &["config", "--local", "user.email", email]);
    }
}

/// URL of the `origin` remote, or `None` if there is none.
pub fn origin_url(runner: &dyn Runner) -> Option<String> {
    match runner.capture("git", &["remote", "get-url", "origin"]) {
        Ok(r) if r.code == 0 => {
            let url = r.stdout.trim();
            (!url.is_empty()).then(|| url.to_string())
        }
        _ => None,
    }
}

/// Heuristic: does this remote URL use SSH rather than HTTPS?
pub fn is_ssh_remote(url: &str) -> bool {
    url.starts_with("git@") || url.starts_with("ssh://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ssh_remotes() {
        assert!(is_ssh_remote("git@github.com:acme/repo.git"));
        assert!(is_ssh_remote("ssh://git@github.com/acme/repo.git"));
        assert!(!is_ssh_remote("https://github.com/acme/repo.git"));
    }
}
