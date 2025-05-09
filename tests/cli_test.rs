use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

const BIN_NAME: &str = "depup";

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Path to the project directory"))
        .stdout(predicate::str::contains("--no-backup"));
}

#[test]
fn test_custom_path() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a dummy package.json file in the temp directory
    let package_json = temp_path.join("package.json");
    let mut file = File::create(&package_json).unwrap();
    writeln!(file, "{{").unwrap();
    writeln!(file, "  \"name\": \"test-package\",").unwrap();
    writeln!(file, "  \"version\": \"1.0.0\",").unwrap();
    writeln!(file, "  \"dependencies\": {{}}").unwrap();
    writeln!(file, "}}").unwrap();

    // Create a .git directory to test the gitignore feature
    fs::create_dir(temp_path.join(".git")).unwrap();

    // Run the command with the custom path and backup flag (now default)
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    cmd.arg(temp_path.to_str().unwrap()).arg("--verbose");

    // Since we don't have npm installed in the test environment,
    // we're just checking that the program runs and mentions skipping
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting dependencies update"));
}

#[test]
fn test_no_backup_flag() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a dummy package.json file in the temp directory
    let package_json = temp_path.join("package.json");
    let mut file = File::create(&package_json).unwrap();
    writeln!(file, "{{").unwrap();
    writeln!(file, "  \"name\": \"test-package\",").unwrap();
    writeln!(file, "  \"version\": \"1.0.0\",").unwrap();
    writeln!(file, "  \"dependencies\": {{}}").unwrap();
    writeln!(file, "}}").unwrap();

    // Run the command with the --no-backup flag
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    cmd.arg(temp_path.to_str().unwrap()).arg("--no-backup");

    cmd.assert().success();

    // Check that no backup file was created (this is hard to test completely since npm
    // might not be installed in the test environment, but we can at least check the logic)
    assert!(!temp_path.join("package.json.backup").exists());
}
