use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn reports_no_config_in_empty_dir() {
    let dir = tempdir().unwrap();
    Command::cargo_bin("gw")
        .unwrap()
        .arg("which")
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No .gitwrapper found"));
}

#[test]
fn reports_malformed_config() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitwrapper"), "{ not json").unwrap();
    Command::cargo_bin("gw")
        .unwrap()
        .arg("which")
        .current_dir(&dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid JSON"));
}

#[test]
fn reports_resolved_account() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitwrapper"), r#"{"account":"alice"}"#).unwrap();
    Command::cargo_bin("gw")
        .unwrap()
        .arg("which")
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("account: alice"))
        .stdout(predicate::str::contains("host:    github.com"));
}
