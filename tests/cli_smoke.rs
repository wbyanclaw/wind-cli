//! Smoke tests for wind CLI

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn wind_version() {
    Command::cargo_bin("wind")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("wind"));
}
