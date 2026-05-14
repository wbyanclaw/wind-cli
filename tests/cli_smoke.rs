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
fn open_file_works() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("readme.md"), "hello").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "open", "--file", "readme.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"page\""))
        .stdout(predicate::str::contains("\"kind\": \"file\""));
}

#[test]
fn open_search_works() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "open", "--search", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"page\""))
        .stdout(predicate::str::contains("\"kind\": \"search\""));
}

#[test]
fn open_requires_argument() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["--json", "open"])
        .assert()
        .failure();
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

// C-2: Write default deny tests
#[test]
fn put_rejects_existing_file_by_default() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("existing.txt"), "original content").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Try to overwrite without --overwrite flag - should fail
    wind(&temp)
        .args(["--json", "put", "existing.txt", "--stdin"])
        .write_stdin("new content")
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_EXISTS"));

    // Original file should be unchanged
    let content = fs::read_to_string(root.join("existing.txt")).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn put_allows_overwrite_with_flag() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("existing.txt"), "original content").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Overwrite with --overwrite flag - should succeed
    wind(&temp)
        .args(["put", "existing.txt", "--stdin", "--overwrite"])
        .write_stdin("new content")
        .assert()
        .success();

    // Original file should be changed
    let content = fs::read_to_string(root.join("existing.txt")).unwrap();
    assert_eq!(content, "new content");
}

#[test]
fn put_works_for_new_file_without_overwrite() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Create new file without --overwrite - should succeed
    wind(&temp)
        .args(["put", "newfile.txt", "--stdin"])
        .write_stdin("brand new content")
        .assert()
        .success();

    // File should exist
    let content = fs::read_to_string(root.join("newfile.txt")).unwrap();
    assert_eq!(content, "brand new content");
}

// D-2: tools write risk escalation tests
#[test]
fn tools_write_new_file_with_overwrite_no_force() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Write new file with overwrite=true but no force - should succeed (D-2 fix)
    // Output structure: {ok: true, result: {ok: true, result: {...}}}}
    wind(&temp)
        .args([
            "tools",
            "--call", "write",
            "--args", r#"{"path": "new.txt", "content": "hello", "overwrite": true}"#
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result\":{\"ok\":true"));

    // File should exist
    let content = fs::read_to_string(root.join("new.txt")).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn tools_write_existing_file_requires_force() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("existing.txt"), "original").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Try to overwrite without force - should fail with error in result
    wind(&temp)
        .args([
            "tools",
            "--call", "write",
            "--args", r#"{"path": "existing.txt", "content": "new", "overwrite": true}"#
        ])
        .assert()
        .success() // CLI succeeds, but result contains error
        .stdout(predicate::str::contains("\"ok\":false"))
        .stdout(predicate::str::contains("HIGH_RISK_REQUIRED_FORCE"));

    // Original file should be unchanged
    let content = fs::read_to_string(root.join("existing.txt")).unwrap();
    assert_eq!(content, "original");
}

#[test]
fn tools_write_existing_file_with_force_succeeds() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("existing.txt"), "original").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Overwrite with force=true - should succeed
    wind(&temp)
        .args([
            "tools",
            "--call", "write",
            "--args", r#"{"path": "existing.txt", "content": "new", "overwrite": true, "force": true}"#
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result\":{\"ok\":true"));

    // File should be changed
    let content = fs::read_to_string(root.join("existing.txt")).unwrap();
    assert_eq!(content, "new");
}

#[test]
fn tools_workspace_info_works() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["tools", "--call", "workspace_info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result\":{\"ok\":true"))
        .stdout(predicate::str::contains("workspace_root"));
}

#[test]
fn tools_version_works() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["tools", "--call", "version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result\":{\"ok\":true"))
        .stdout(predicate::str::contains("version"));
}

#[test]
fn tools_list_shows_all_tools() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["tools", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ls"))
        .stdout(predicate::str::contains("read"))
        .stdout(predicate::str::contains("write"))
        .stdout(predicate::str::contains("mkdir"))
        .stdout(predicate::str::contains("rm"))
        .stdout(predicate::str::contains("workspace_info"))
        .stdout(predicate::str::contains("version"));
}

// B-iv: wft command tests
#[test]
fn wft_file_works() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("readme.md"), "hello").unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "wft", "file", "readme.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"page\""))
        .stdout(predicate::str::contains("\"kind\": \"file\""));
}

#[test]
fn wft_search_works() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["--json", "wft", "search", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"page\""))
        .stdout(predicate::str::contains("\"kind\": \"search\""));
}

#[test]
fn wft_app_works() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["--json", "wft", "app"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("show_app"));
}

#[test]
fn wft_settings_works() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["--json", "wft", "settings"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("show_settings"));
}

#[test]
fn wft_workspace_works() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["--json", "wft", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("show_workspace"));
}

#[test]
fn wft_upgrade_works() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["--json", "wft", "upgrade"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""))
        .stdout(predicate::str::contains("check_upgrade"));
}

#[test]
fn wft_url_works() {
    let temp = TempDir::new().unwrap();

    wind(&temp)
        .args(["--json", "wft", "url", "windlocal://command?id=show_workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"type\": \"command\""));
}

#[test]
fn open_shows_deprecation_warning() {
    let temp = TempDir::new().unwrap();
    let root = workspace(&temp);
    fs::create_dir_all(&root).unwrap();

    wind(&temp)
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    wind(&temp)
        .args(["open", "--file", "readme.md"])
        .assert()
        .success()
        .stderr(predicate::str::contains("deprecated"));
}
