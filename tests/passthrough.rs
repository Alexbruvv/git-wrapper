use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn passes_through_to_git_without_config() {
    let dir = tempdir().unwrap();
    // No .gitwrapper and not a git repo: git itself errors, and gw forwards it
    // verbatim (transparent passthrough, no account switching).
    Command::cargo_bin("gw")
        .unwrap()
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(&dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));
}

#[test]
fn propagates_git_failure_exit_code() {
    let dir = tempdir().unwrap();
    Command::cargo_bin("gw")
        .unwrap()
        .arg("definitely-not-a-git-command")
        .current_dir(&dir)
        .assert()
        .failure();
}

#[test]
fn version_flag_is_not_passed_to_git() {
    // `--gw-version` is ours; `--version` would fall through to git.
    let dir = tempdir().unwrap();
    Command::cargo_bin("gw")
        .unwrap()
        .arg("--gw-version")
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d+\.\d+\.\d+\n$").unwrap());
}
