use std::path::Path;

use crate::config;
use crate::gh::Gh;
use crate::git;
use crate::log;
use crate::types::Runner;

/// The full wrapped-git flow: resolve the project account, switch gh to it, run
/// the git command with the user's stdio, then restore the previous account.
/// Returns the process exit code.
pub fn run_wrapped(args: &[String], runner: &dyn Runner, cwd: &Path) -> i32 {
    // 1. Resolve config. No config → transparent passthrough.
    let config = match config::load_config(cwd) {
        Ok(Some(c)) => c,
        Ok(None) => return passthrough(args, runner),
        Err(e) => {
            log::error(&format!("{}: {}", e.path.display(), e.message));
            return 1;
        }
    };

    // 2. Preflight: required binaries.
    if !git::is_git_installed(runner) {
        log::error("git is not installed or not on PATH");
        return 1;
    }
    let gh = Gh::new(runner);
    if !gh.is_installed() {
        log::error("the GitHub CLI (gh) is not installed — see https://cli.github.com");
        return 1;
    }

    // 3. Read gh accounts.
    let accounts = match gh.status() {
        Ok(a) => a,
        Err(e) => {
            log::error(&e.to_string());
            return 1;
        }
    };

    // 4. Resolve the target account on the configured host (case-insensitive).
    let target = accounts.iter().find(|a| {
        a.host.eq_ignore_ascii_case(&config.host) && a.user.eq_ignore_ascii_case(&config.account)
    });
    let Some(target) = target else {
        log::error(&format!(
            "account \"{}\" is not logged in to {}; run `gh auth login --hostname {}`",
            config.account, config.host, config.host
        ));
        return 1;
    };

    let previous_active = accounts
        .iter()
        .find(|a| a.host.eq_ignore_ascii_case(&config.host) && a.active);
    let already_active = previous_active.is_some_and(|p| p.user == target.user);

    // 5. Switch to the project account (unless already active).
    if !already_active {
        if let Err(e) = gh.setup_git(&config.host) {
            log::error(&e.to_string());
            return 1;
        }
        if let Err(e) = gh.switch(&config.host, &target.user) {
            log::error(&e.to_string());
            return 1;
        }
        log::notice(&format!("switched to {} on {}", target.user, config.host));
    }

    // 6. Repo-local identity + SSH warning (best-effort, only in a work tree).
    if git::is_inside_work_tree(runner) {
        if config.user_name.is_some() || config.user_email.is_some() {
            git::set_local_identity(
                config.user_name.as_deref(),
                config.user_email.as_deref(),
                runner,
            );
        }
        if let Some(url) = git::origin_url(runner) {
            if git::is_ssh_remote(&url) {
                log::warn("origin uses SSH; gh account switching only affects HTTPS credentials");
            }
        }
    }

    // 7. Run git with the user's terminal attached.
    let code = passthrough(args, runner);

    // 8. Restore the previously active account.
    if !already_active && config.restore_previous {
        if let Some(prev) = previous_active {
            match gh.switch(&config.host, &prev.user) {
                Ok(()) => log::notice(&format!("restored {} on {}", prev.user, config.host)),
                Err(_) => log::warn(&format!("could not restore previous account {}", prev.user)),
            }
        }
    }

    code
}

fn passthrough(args: &[String], runner: &dyn Runner) -> i32 {
    match git::run_git(args, runner) {
        Ok(code) => code,
        Err(e) => {
            log::error(&format!("failed to run git: {e}"));
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{argv, FakeRunner};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_config(dir: &Path, json: &str) {
        fs::write(dir.join(".gitwrapper"), json).unwrap();
    }

    #[test]
    fn passthrough_when_no_config() {
        let dir = tempdir().unwrap();
        let runner = FakeRunner::new(&[]);
        let code = run_wrapped(&argv(&["status"]), &runner, dir.path());
        assert_eq!(code, 0);
        assert_eq!(
            *runner.passthroughs.borrow(),
            vec![argv(&["git", "status"])]
        );
        assert!(runner.switches.borrow().is_empty());
    }

    #[test]
    fn switches_then_restores() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), r#"{"account":"alice"}"#);
        let runner = FakeRunner::new(&[("bob", true), ("alice", false)]);
        let code = run_wrapped(&argv(&["push"]), &runner, dir.path());
        assert_eq!(code, 0);
        assert_eq!(*runner.passthroughs.borrow(), vec![argv(&["git", "push"])]);
        // switch to alice, then restore bob.
        assert_eq!(*runner.switches.borrow(), vec!["alice", "bob"]);
    }

    #[test]
    fn no_switch_when_already_active() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), r#"{"account":"alice"}"#);
        let runner = FakeRunner::new(&[("alice", true)]);
        run_wrapped(&argv(&["fetch"]), &runner, dir.path());
        assert!(runner.switches.borrow().is_empty());
    }

    #[test]
    fn matches_account_case_insensitively() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), r#"{"account":"ALICE"}"#);
        let runner = FakeRunner::new(&[("alice", true)]);
        run_wrapped(&argv(&["fetch"]), &runner, dir.path());
        assert!(runner.switches.borrow().is_empty());
    }

    #[test]
    fn errors_when_account_not_logged_in() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), r#"{"account":"carol"}"#);
        let runner = FakeRunner::new(&[("alice", true)]);
        let code = run_wrapped(&argv(&["status"]), &runner, dir.path());
        assert_eq!(code, 1);
        assert!(runner.passthroughs.borrow().is_empty());
    }

    #[test]
    fn skips_restore_when_disabled() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), r#"{"account":"alice","restorePrevious":false}"#);
        let runner = FakeRunner::new(&[("bob", true), ("alice", false)]);
        run_wrapped(&argv(&["push"]), &runner, dir.path());
        assert_eq!(*runner.switches.borrow(), vec!["alice"]);
    }

    #[test]
    fn propagates_git_exit_code() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), r#"{"account":"alice"}"#);
        let mut runner = FakeRunner::new(&[("alice", true)]);
        runner.git_code = 7;
        let code = run_wrapped(&argv(&["push"]), &runner, dir.path());
        assert_eq!(code, 7);
    }

    #[test]
    fn applies_local_identity() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"{"account":"alice","userName":"Alice","userEmail":"a@x.test"}"#,
        );
        let runner = FakeRunner::new(&[("alice", true)]);
        run_wrapped(&argv(&["commit"]), &runner, dir.path());
        let captures = runner.captures.borrow();
        assert!(captures
            .iter()
            .any(|c| c == &argv(&["git", "config", "--local", "user.name", "Alice"])));
        assert!(captures
            .iter()
            .any(|c| c == &argv(&["git", "config", "--local", "user.email", "a@x.test"])));
    }
}
