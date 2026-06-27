use std::path::Path;

use crate::config;
use crate::gh::Gh;
use crate::git;
use crate::runner::RealRunner;
use crate::types::{Account, Runner};

const PASS: &str = "✓";
const FAIL: &str = "✗";
const INFO: &str = "•";

/// Diagnose the environment: git/gh install, gh auth accounts, and the nearest
/// `.gitwrapper`. Prints a checklist; returns 1 if anything is actionable.
pub fn doctor() -> i32 {
    let cwd = std::env::current_dir().unwrap_or_default();
    let (lines, code) = diagnose(&cwd, &RealRunner);
    for line in lines {
        println!("{line}");
    }
    code
}

/// Build the diagnostic checklist and overall exit code. Split out from `doctor`
/// so it can be driven with a mock runner in tests.
fn diagnose(cwd: &Path, runner: &dyn Runner) -> (Vec<String>, i32) {
    let mut lines = Vec::new();
    let mut ok = true;

    // git
    let git_ok = git::is_git_installed(runner);
    lines.push(format!("{} git installed", mark(git_ok)));
    ok &= git_ok;

    // gh
    let gh = Gh::new(runner);
    let gh_ok = gh.is_installed();
    let gh_hint = if gh_ok {
        ""
    } else {
        " — see https://cli.github.com"
    };
    lines.push(format!("{} gh installed{gh_hint}", mark(gh_ok)));
    ok &= gh_ok;

    // accounts
    let mut accounts: Vec<Account> = Vec::new();
    if gh_ok {
        match gh.status() {
            Ok(found) if found.is_empty() => {
                lines.push(format!("{FAIL} no gh accounts — run `gh auth login`"));
                ok = false;
            }
            Ok(found) => {
                lines.push(format!("{INFO} gh accounts:"));
                for a in &found {
                    let active = if a.active { " (active)" } else { "" };
                    lines.push(format!("    - {} @ {}{active}", a.user, a.host));
                }
                accounts = found;
            }
            Err(e) => {
                lines.push(format!("{FAIL} gh auth status failed: {e}"));
                ok = false;
            }
        }
    }

    // config
    match config::load_config(cwd) {
        Ok(None) => lines.push(format!(
            "{INFO} no .gitwrapper found — git runs unchanged here"
        )),
        Ok(Some(cfg)) => {
            lines.push(format!("{INFO} .gitwrapper: {}", cfg.source_path.display()));
            lines.push(format!("    account: {} @ {}", cfg.account, cfg.host));
            if gh_ok && !accounts.is_empty() {
                let known = accounts.iter().any(|a| {
                    a.host.eq_ignore_ascii_case(&cfg.host)
                        && a.user.eq_ignore_ascii_case(&cfg.account)
                });
                if known {
                    lines.push(format!("    {PASS} account is logged in"));
                } else {
                    lines.push(format!(
                        "    {FAIL} account not logged in — run `gh auth login --hostname {}`",
                        cfg.host
                    ));
                    ok = false;
                }
            }
        }
        Err(e) => {
            lines.push(format!(
                "{FAIL} .gitwrapper invalid ({}): {}",
                e.path.display(),
                e.message
            ));
            ok = false;
        }
    }

    (lines, if ok { 0 } else { 1 })
}

fn mark(ok: bool) -> &'static str {
    if ok {
        PASS
    } else {
        FAIL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::FakeRunner;
    use std::fs;
    use tempfile::tempdir;

    fn joined(lines: &[String]) -> String {
        lines.join("\n")
    }

    #[test]
    fn passes_when_account_logged_in() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitwrapper"), r#"{"account":"alice"}"#).unwrap();
        let runner = FakeRunner::new(&[("alice", true)]);
        let (lines, code) = diagnose(dir.path(), &runner);
        assert_eq!(code, 0);
        assert!(joined(&lines).contains("account is logged in"));
    }

    #[test]
    fn fails_when_account_not_logged_in() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitwrapper"), r#"{"account":"carol"}"#).unwrap();
        let runner = FakeRunner::new(&[("alice", true)]);
        let (lines, code) = diagnose(dir.path(), &runner);
        assert_eq!(code, 1);
        assert!(joined(&lines).contains("account not logged in"));
    }

    #[test]
    fn fails_when_gh_missing() {
        let dir = tempdir().unwrap();
        let mut runner = FakeRunner::new(&[]);
        runner.gh_installed = false;
        let (lines, code) = diagnose(dir.path(), &runner);
        assert_eq!(code, 1);
        assert!(joined(&lines).contains("cli.github.com"));
    }

    #[test]
    fn reports_no_config() {
        let dir = tempdir().unwrap();
        let runner = FakeRunner::new(&[("alice", true)]);
        let (lines, code) = diagnose(dir.path(), &runner);
        assert_eq!(code, 0);
        assert!(joined(&lines).contains("no .gitwrapper found"));
    }
}
