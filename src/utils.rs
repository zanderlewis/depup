use colored::*;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs;

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(verbose: bool) {
    VERBOSE.store(verbose, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

pub fn info(message: &str) {
    println!("{} {}", "[INFO]".blue(), message);
}

pub fn success(message: &str) {
    println!("{} {}", "[SUCCESS]".green(), message);
}

pub fn error(message: &str) {
    println!("{} {}", "[ERROR]".red(), message);
}

pub fn warning(message: &str) {
    println!("{} {}", "[WARNING]".yellow(), message);
}

pub fn debug(message: &str) {
    if is_verbose() {
        println!("{} {}", "[DEBUG]".purple(), message);
    }
}

// Add all *.backup files to .gitignore if in a git repository
pub fn ensure_backups_in_gitignore(project_path: &Path) -> Result<(), std::io::Error> {
    // Check if we're in a git repository
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        // Not a git repo, so we don't need to do anything
        return Ok(());
    }

    let gitignore_path = project_path.join(".gitignore");
    let backup_pattern = "*.backup";

    // Create .gitignore if it doesn't exist
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, "")?;
        debug("Created .gitignore file");
    }

    // Read contents of .gitignore
    let contents = std::fs::read_to_string(&gitignore_path)?;
    let lines: Vec<&str> = contents.lines().collect();

    // Check if *.backup pattern is already in .gitignore
    if !lines.contains(&backup_pattern) {
        // Add *.backup to .gitignore
        let mut new_content = contents;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(backup_pattern);
        new_content.push('\n');
        std::fs::write(&gitignore_path, new_content)?;
        debug("Added *.backup to .gitignore");
    }

    Ok(())
}

// Check if a command is available in the system
pub fn is_command_available(command: &str) -> bool {
    let status = if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(command)
            .output()
            .map(|output| output.status.success())
    } else {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
    };

    matches!(status, Ok(true))
}

// Check if backup files exist
pub fn check_backups_exist(project_path: &Path) -> bool {
    
    // Look for any .backup files in the project directory
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".backup") {
                    return true;
                }
            }
        }
    }
    
    false
}

// Revert changes using backup files
pub fn revert_changes(project_path: &Path) -> Result<(), std::io::Error> {
    // Look for any .backup files in the project directory
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".backup") {
                    let original_file_name = file_name.trim_end_matches(".backup");
                    let original_file_path = project_path.join(original_file_name);
                    let backup_file_path = project_path.join(file_name);

                    // Restore the original file from the backup
                    fs::rename(backup_file_path, original_file_path)?;
                }
            }
        }
    }

    Ok(())
}
