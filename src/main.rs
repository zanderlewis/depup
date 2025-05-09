mod cargo;
mod node;
mod php;
mod utils;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "dup",
    about = "Dependency Update Tool",
    version,
    author = "Chris L",
    long_about = "A utility for updating dependencies in various project types (Node.js, PHP, Rust)."
)]
struct Cli {
    /// Skip creating backups of package files before updating
    #[arg(short = 'B', long = "no-backup")]
    no_backup: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Path to the project directory
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    // Set global config for utils
    utils::set_verbose(cli.verbose);

    utils::info("Starting dependencies update...");

    // Determine if we should create backups (default is true, unless --no-backup is specified)
    let create_backups = !cli.no_backup;

    // If backups are enabled, ensure *.backup is in .gitignore
    if create_backups {
        if let Err(e) = utils::ensure_backups_in_gitignore(&cli.path) {
            utils::warning(&format!("Could not update .gitignore: {}", e));
        }
    }

    let mut packages_found = false;

    // Change to the specified directory if needed
    let original_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cli.path != PathBuf::from(".") {
        if let Err(e) = std::env::set_current_dir(&cli.path) {
            utils::error(&format!(
                "Failed to change to directory {}: {}",
                cli.path.display(),
                e
            ));
            return;
        }
        utils::debug(&format!(
            "Changed working directory to {}",
            cli.path.display()
        ));
    }

    // Check for composer.json
    if std::path::Path::new("composer.json").exists() {
        if utils::is_command_available("composer") {
            php::update_composer(create_backups);
            packages_found = true;
        } else {
            utils::warning(
                "composer.json found but composer is not installed. Skipping PHP dependencies.",
            );
        }
    }

    // Check for package.json
    if std::path::Path::new("package.json").exists() {
        if utils::is_command_available("npm") {
            node::update_npm(create_backups);
            packages_found = true;
        } else {
            utils::warning(
                "package.json found but npm is not installed. Skipping Node.js dependencies.",
            );
        }
    }

    // Check for Cargo.toml
    if std::path::Path::new("Cargo.toml").exists() {
        if utils::is_command_available("cargo") {
            cargo::update_cargo(create_backups);
            packages_found = true;
        } else {
            utils::warning(
                "Cargo.toml found but cargo is not installed. Skipping Rust dependencies.",
            );
        }
    }

    // Change back to the original directory
    if cli.path != PathBuf::from(".") {
        if let Err(e) = std::env::set_current_dir(&original_dir) {
            utils::warning(&format!(
                "Failed to change back to original directory: {}",
                e
            ));
        }
    }

    if packages_found {
        utils::success("Dependency update completed.");
    } else {
        utils::warning("No supported dependency files found or no package managers installed.");
    }
}
