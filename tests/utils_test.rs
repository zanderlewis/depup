use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

// Import the utils module from the main crate
use depup::utils;

#[test]
fn test_is_command_available() {
    // Test a command that should definitely exist
    assert!(utils::is_command_available("ls"));

    // Test a command that should definitely not exist
    assert!(!utils::is_command_available(
        "this_command_does_not_exist_12345"
    ));
}

#[test]
fn test_ensure_backups_in_gitignore() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a fake .git directory
    fs::create_dir(temp_path.join(".git")).unwrap();

    // Test creating .gitignore when it doesn't exist
    assert!(utils::ensure_backups_in_gitignore(temp_path).is_ok());

    // Check that .gitignore was created with *.backup
    let gitignore_content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("*.backup"));

    // Test adding to existing .gitignore
    let mut file = File::create(temp_path.join(".gitignore")).unwrap();
    writeln!(file, "node_modules/").unwrap();

    // Run ensure_backups_in_gitignore again
    assert!(utils::ensure_backups_in_gitignore(temp_path).is_ok());

    // Check that .gitignore still contains *.backup
    let gitignore_content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("node_modules/"));
    assert!(gitignore_content.contains("*.backup"));

    // Test with non-git directory
    let non_git_dir = tempdir().unwrap();
    assert!(utils::ensure_backups_in_gitignore(non_git_dir.path()).is_ok());
    assert!(!Path::new(&non_git_dir.path().join(".gitignore")).exists());
}
