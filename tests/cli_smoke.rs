//! Smoke tests for windcli - AI-agent friendly file workspace

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn windcli_cmd(temp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("windcli").unwrap();
    cmd.env("WIND_CONFIG_PATH", temp.path().join("config.json"));
    cmd
}

fn workspace_path(temp: &TempDir) -> std::path::PathBuf {
    temp.path().join("workspace")
}

#[test]
fn version_command() {
    Command::cargo_bin("windcli")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("windcli"));
}

#[test]
fn init_creates_workspace() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));

    assert!(root.is_dir());
}

#[test]
fn write_and_read_file() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Write via stdin (AI-friendly)
    let mut write = windcli_cmd(&temp);
    write.args(["write", "notes/test.txt", "--stdin"])
        .write_stdin("hello world")
        .assert()
        .success();

    // Read file
    windcli_cmd(&temp)
        .args(["read", "notes/test.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn nested_mkdir_and_write() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["mkdir", "docs/api"])
        .assert()
        .success();

    let mut write = windcli_cmd(&temp);
    write.args(["write", "docs/api/intro.md", "--stdin"])
        .write_stdin("# API Docs\nWelcome")
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["read", "docs/api/intro.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API Docs"));
}

#[test]
fn list_shows_files() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("readme.md"), "welcome").unwrap();

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("readme.md"));
}

#[test]
fn delete_removes_file() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["write", "temp.txt", "--stdin"])
        .write_stdin("temporary")
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["delete", "temp.txt", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted"));
}

#[test]
fn path_traversal_blocked() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["--json", "read", "../secret.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_TRAVERSAL"));
}

#[test]
fn large_file_rejected() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("big.txt"), vec![b'x'; 10 * 1024 * 1024 + 1]).unwrap();

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["--json", "read", "big.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FILE_TOO_LARGE"));
}

#[test]
fn upgrade_check_works() {
    let temp = TempDir::new().unwrap();
    windcli_cmd(&temp)
        .args(["upgrade", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("version"));
}

#[test]
fn bare_upgrade_guides_to_check() {
    let temp = TempDir::new().unwrap();
    windcli_cmd(&temp)
        .arg("upgrade")
        .assert()
        .success()
        .stdout(predicate::str::contains("windcli upgrade --check"))
        .stdout(predicate::str::contains("checking for updates only"))
        .stdout(predicate::str::contains("P0").not());
}

// Write security tests
#[test]
fn write_rejects_existing_file_by_default() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("existing.txt"), "original").unwrap();

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Try to overwrite without --overwrite flag - should fail
    windcli_cmd(&temp)
        .args(["write", "existing.txt", "--content", "new content"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // Original file should be unchanged
    let content = fs::read_to_string(root.join("existing.txt")).unwrap();
    assert_eq!(content, "original");
}

#[test]
fn write_allows_overwrite_with_flag() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("existing.txt"), "original").unwrap();

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Overwrite with --overwrite flag - should succeed
    windcli_cmd(&temp)
        .args(["write", "existing.txt", "--content", "new content", "--overwrite"])
        .assert()
        .success();

    // Original file should be changed
    let content = fs::read_to_string(root.join("existing.txt")).unwrap();
    assert_eq!(content, "new content");
}

#[test]
fn write_works_for_new_file_without_overwrite() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Create new file without --overwrite - should succeed
    windcli_cmd(&temp)
        .args(["write", "newfile.txt", "--content", "brand new content"])
        .assert()
        .success();

    // File should exist
    let content = fs::read_to_string(root.join("newfile.txt")).unwrap();
    assert_eq!(content, "brand new content");
}

#[test]
fn upgrade_help_describes_check_only() {
    Command::cargo_bin("windcli")
        .unwrap()
        .args(["upgrade", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("does not install").or(predicate::str::contains("不下载或安装")))
        .stdout(predicate::str::contains("P0").not());
}

// Backward compatibility: cat/put/rm still work
#[test]
fn cat_alias_works() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["write", "test.txt", "--stdin"])
        .write_stdin("content")
        .assert()
        .success();

    // cat should work as alias for read
    windcli_cmd(&temp)
        .args(["cat", "test.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("content"));
}

// wft command tests
#[test]
fn wft_file_command() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("readme.md"), "hello").unwrap();

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["--json", "wft", "file", "readme.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"page\""))
        .stdout(predicate::str::contains("\"kind\": \"file\""));
}

#[test]
fn wft_search_command() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["--json", "wft", "search", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"page\""))
        .stdout(predicate::str::contains("\"kind\": \"search\""));
}

#[test]
fn wft_app_command() {
    let temp = TempDir::new().unwrap();

    windcli_cmd(&temp)
        .args(["--json", "wft", "app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("show_app"));
}

#[test]
fn wft_settings_command() {
    let temp = TempDir::new().unwrap();

    windcli_cmd(&temp)
        .args(["--json", "wft", "settings"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("show_settings"));
}

#[test]
fn wft_workspace_command() {
    let temp = TempDir::new().unwrap();

    windcli_cmd(&temp)
        .args(["--json", "wft", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("show_workspace"));
}

#[test]
fn wft_upgrade_command() {
    let temp = TempDir::new().unwrap();

    windcli_cmd(&temp)
        .args(["--json", "wft", "upgrade"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("check_upgrade"));
}

#[test]
fn wft_url_command() {
    let temp = TempDir::new().unwrap();

    windcli_cmd(&temp)
        .args(["--json", "wft", "url", "windlocal://command?id=show_workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""));
}

#[test]
fn open_shows_deprecation_warning() {
    let temp = TempDir::new().unwrap();
    let root = workspace_path(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("test.txt"), "content").unwrap();

    windcli_cmd(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    windcli_cmd(&temp)
        .args(["open", "--file", "test.txt"])
        .assert()
        .success()
        .stderr(predicate::str::contains("deprecated"));
}
