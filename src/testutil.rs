use std::cell::RefCell;
use std::io;

use crate::types::{RunResult, Runner};

/// Build a `gh auth status --json hosts` payload from `(user, active)` pairs.
pub fn accounts_json(accounts: &[(&str, bool)]) -> String {
    let entries: Vec<String> = accounts
        .iter()
        .map(|(u, a)| format!(r#"{{"login":"{u}","host":"github.com","active":{a}}}"#))
        .collect();
    format!(r#"{{"hosts":{{"github.com":[{}]}}}}"#, entries.join(","))
}

/// Convenience for building owned arg vectors in assertions.
pub fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

/// Runner that returns canned capture results and records every call, so the
/// gh/git boundary can be driven deterministically in tests.
pub struct FakeRunner {
    pub accounts_json: String,
    pub origin_url: String,
    pub git_code: i32,
    pub git_installed: bool,
    pub gh_installed: bool,
    pub captures: RefCell<Vec<Vec<String>>>,
    pub passthroughs: RefCell<Vec<Vec<String>>>,
    pub switches: RefCell<Vec<String>>,
}

impl FakeRunner {
    pub fn new(accounts: &[(&str, bool)]) -> Self {
        Self {
            accounts_json: accounts_json(accounts),
            origin_url: "https://github.com/acme/repo.git".to_string(),
            git_code: 0,
            git_installed: true,
            gh_installed: true,
            captures: RefCell::new(Vec::new()),
            passthroughs: RefCell::new(Vec::new()),
            switches: RefCell::new(Vec::new()),
        }
    }
}

impl Runner for FakeRunner {
    fn capture(&self, cmd: &str, args: &[&str]) -> io::Result<RunResult> {
        let mut full = vec![cmd.to_string()];
        full.extend(args.iter().map(|s| s.to_string()));
        self.captures.borrow_mut().push(full);

        let ok = |code: i32, s: &str| -> io::Result<RunResult> {
            Ok(RunResult {
                code,
                stdout: s.to_string(),
                stderr: String::new(),
            })
        };
        match (cmd, args.join(" ").as_str()) {
            ("git", "--version") => ok(if self.git_installed { 0 } else { 1 }, ""),
            ("git", "rev-parse --is-inside-work-tree") => ok(0, "true"),
            ("git", "remote get-url origin") => ok(0, &self.origin_url),
            ("gh", "--version") => ok(if self.gh_installed { 0 } else { 1 }, ""),
            ("gh", "auth status --json hosts") => ok(0, &self.accounts_json),
            ("gh", s) if s.starts_with("auth switch") => {
                if let Some(i) = args.iter().position(|a| *a == "--user") {
                    if let Some(u) = args.get(i + 1) {
                        self.switches.borrow_mut().push((*u).to_string());
                    }
                }
                ok(0, "")
            }
            _ => ok(0, ""),
        }
    }

    fn passthrough(&self, cmd: &str, args: &[&str]) -> io::Result<i32> {
        let mut full = vec![cmd.to_string()];
        full.extend(args.iter().map(|s| s.to_string()));
        self.passthroughs.borrow_mut().push(full);
        Ok(self.git_code)
    }
}
