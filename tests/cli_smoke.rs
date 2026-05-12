//! Smoke tests for wind CLI

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn wind(temp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("wind").unwrap();
    cmd.env("WIND_CONFIG_PATH", temp.path().join("config.json"));
    cmd
}

fn workspace(temp: &TempDir) -> std::path::PathBuf {
    temp.path().join("workspace")
}

#[test]
fn wind_version() {
    Command::cargo_bin("wind")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("wind"));
}

#[test]
fn init_creates_workspace_directory() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace initialized"));

    assert!(root.is_dir());
}

#[test]
fn nested_mkdir_and_put_work() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);

    wind(&temp).args(["init", root.to_str().unwrap()]).assert().success();
    wind(&temp).args(["mkdir", "a/b"]).assert().success();

    let mut put = wind(&temp);
    put.args(["put", "a/b/file.txt", "--stdin"])
        .write_stdin("hello nested")
        .assert()
        .success();

    wind(&temp)
        .args(["cat", "a/b/file.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello nested"));
}

#[cfg(unix)]
#[test]
fn ls_reports_symlink_entries_without_following() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(temp.path().join("outside.txt"), "secret").unwrap();
    symlink(temp.path().join("outside.txt"), root.join("link.txt")).unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"type\": \"symlink\""));
}

#[test]
fn path_traversal_is_rejected_as_json_error() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "cat", "../secret.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_TRAVERSAL"))
        .stderr(predicate::str::contains("exitCode"));
}

#[test]
fn windlocal_rejects_unknown_params() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/readme.md"), "ok").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Extra --cmd argument is rejected at CLI layer (unknown argument)
    wind(&temp)
        .args(["--json", "open", "file", "docs/readme.md", "--cmd", "launch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected")
            .or(predicate::str::contains("argument")));
}

#[cfg(unix)]
#[test]
fn symlink_targets_are_rejected_for_read() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(temp.path().join("outside.txt"), "secret").unwrap();
    symlink(temp.path().join("outside.txt"), root.join("link.txt")).unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "cat", "link.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SYMLINK_NOT_SUPPORTED"));
}

#[test]
fn cat_enforces_ten_mb_limit() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("big.txt"), vec![b'x'; 10 * 1024 * 1024 + 1]).unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "cat", "big.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FILE_TOO_LARGE"));
}
