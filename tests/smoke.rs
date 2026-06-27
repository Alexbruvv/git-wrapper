use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prints_version() {
    Command::cargo_bin("gw")
        .unwrap()
        .arg("--gw-version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d+\.\d+\.\d+\n$").unwrap());
}

#[test]
fn help_mentions_usage() {
    Command::cargo_bin("gw")
        .unwrap()
        .arg("--gw-help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}
