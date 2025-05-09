use crate::utils;
use std::collections::HashMap;
use std::fs::{copy, read_to_string, write};
use std::process::Command;
use toml_edit::{DocumentMut, Formatted, Item, Value};

// Latest versions of common Rust packages
// In a real implementation, these could be fetched from crates.io API
const LATEST_VERSIONS: &[(&str, &str)] = &[
    ("colored", "3.0.0"),
    ("toml_edit", "0.22.26"),
    ("serde", "1.0.190"),
    ("serde_json", "1.0.107"),
    ("clap", "4.4.8"),
    ("tempfile", "3.8.1"),
    ("assert_cmd", "2.0.12"),
    ("predicates", "3.0.4"),
];

pub fn update_cargo(backup: bool) {
    utils::info("Updating Cargo dependencies...");

    // Create backups first if enabled
    if backup {
        create_backups();
    }

    // Read the Cargo.toml file
    let cargo_toml = match read_to_string("Cargo.toml") {
        Ok(content) => content,
        Err(e) => {
            utils::error(&format!("Failed to read Cargo.toml: {}", e));
            return;
        }
    };

    // Parse the TOML file
    let mut document = match cargo_toml.parse::<DocumentMut>() {
        Ok(doc) => doc,
        Err(e) => {
            utils::error(&format!("Failed to parse Cargo.toml: {}", e));
            return;
        }
    };

    // Find outdated packages
    let outdated_packages = find_outdated_packages(&document);
    if outdated_packages.is_empty() {
        utils::info("No outdated cargo packages found.");
        return;
    }

    // Track if we've made any changes
    let mut updated = false;

    // Update dependencies section
    if let Some(deps) = document.get_mut("dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            for (name, (current_version, latest_version)) in &outdated_packages {
                if let Some(dep) = deps_table.get_mut(name) {
                    let version_str = format!("^{}", latest_version);
                    utils::info(&format!(
                        "Updating {} from {} to {}",
                        name, current_version, latest_version
                    ));

                    // Handle different dependency specification formats
                    match dep {
                        Item::Value(val) if val.is_str() => {
                            *val = to_formatted_string(&version_str);
                            updated = true;
                        }
                        Item::Table(table) => {
                            if let Some(ver) = table.get_mut("version") {
                                *ver = Item::Value(to_formatted_string(&version_str));
                                updated = true;
                            }
                        }
                        _ => {
                            utils::warning(&format!(
                                "Could not update {} - unsupported dependency format",
                                name
                            ));
                        }
                    }
                }
            }
        }
    }

    // Update dev-dependencies section
    if let Some(dev_deps) = document.get_mut("dev-dependencies") {
        if let Some(dev_deps_table) = dev_deps.as_table_mut() {
            for (name, (current_version, latest_version)) in &outdated_packages {
                if let Some(dep) = dev_deps_table.get_mut(name) {
                    let version_str = format!("^{}", latest_version);
                    utils::info(&format!(
                        "Updating {} from {} to {}",
                        name, current_version, latest_version
                    ));

                    // Handle different dependency specification formats
                    match dep {
                        Item::Value(val) if val.is_str() => {
                            *val = to_formatted_string(&version_str);
                            updated = true;
                        }
                        Item::Table(table) => {
                            if let Some(ver) = table.get_mut("version") {
                                *ver = Item::Value(to_formatted_string(&version_str));
                                updated = true;
                            }
                        }
                        _ => {
                            utils::warning(&format!(
                                "Could not update {} - unsupported dependency format",
                                name
                            ));
                        }
                    }
                }
            }
        }
    }

    if updated {
        // Write updated Cargo.toml
        if let Err(e) = write("Cargo.toml", document.to_string()) {
            utils::error(&format!("Failed to write updated Cargo.toml: {}", e));
            return;
        }

        utils::info("Cargo.toml updated with latest dependencies");

        // Run cargo update to update the lock file
        utils::info("Running cargo update...");
        run_cargo_update();
    } else {
        utils::info("No changes needed in Cargo.toml");
    }
}

// Helper function to create formatted TOML strings
fn to_formatted_string(s: &str) -> Value {
    Value::String(Formatted::new(s.to_string()))
}

// Extract version constraints without the ^ or ~ prefix
fn extract_version(version_str: &str) -> String {
    version_str
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches('=')
        .trim_start_matches(' ')
        .to_string()
}

fn find_outdated_packages(document: &DocumentMut) -> HashMap<String, (String, String)> {
    let mut outdated = HashMap::new();
    utils::debug("Checking for outdated cargo packages");

    // Check dependencies section
    check_section(document, "dependencies", &mut outdated);

    // Check dev-dependencies section
    check_section(document, "dev-dependencies", &mut outdated);

    if outdated.is_empty() {
        utils::debug("No outdated cargo packages found");
    } else {
        utils::info(&format!("Found {} outdated cargo packages", outdated.len()));
        for (name, (current, latest)) in &outdated {
            utils::debug(&format!("  {} {} -> {}", name, current, latest));
        }
    }

    outdated
}

fn check_section(
    document: &DocumentMut,
    section_name: &str,
    outdated: &mut HashMap<String, (String, String)>,
) {
    if let Some(section) = document.get(section_name) {
        if let Some(table) = section.as_table() {
            for (name, item) in table.iter() {
                let current_version = match item {
                    // Simple version string: "package = "1.0""
                    Item::Value(val) if val.is_str() => {
                        val.as_str().map(extract_version).unwrap_or_default()
                    }
                    // Table format: "package = { version = "1.0", features = ["derive"] }"
                    Item::Table(table) => table
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(extract_version)
                        .unwrap_or_default(),
                    _ => continue,
                };

                // Skip if we couldn't determine the current version
                if current_version.is_empty() {
                    continue;
                }

                // Check if we have a known latest version for this package
                if let Some((_, latest_version)) =
                    LATEST_VERSIONS.iter().find(|(pkg, _)| pkg == &name)
                {
                    // Compare versions
                    if !is_up_to_date(&current_version, latest_version) {
                        utils::info(&format!(
                            "Found outdated package: {} current: {} latest: {}",
                            name, current_version, latest_version
                        ));

                        outdated.insert(
                            name.to_string(),
                            (current_version, latest_version.to_string()),
                        );
                    }
                }
            }
        }
    }
}

// Simple version comparison - in a real implementation, this would be more sophisticated
fn is_up_to_date(current: &str, latest: &str) -> bool {
    // This is a simplified version check - in reality we would use semver parsing
    current == latest
}

fn create_backups() {
    // Create backup of Cargo.toml
    if copy("Cargo.toml", "Cargo.toml.backup").is_ok() {
        utils::debug("Created backup: Cargo.toml.backup");
    } else {
        utils::warning("Failed to create Cargo.toml backup");
    }

    // Create backup of Cargo.lock if it exists
    if std::path::Path::new("Cargo.lock").exists() {
        if copy("Cargo.lock", "Cargo.lock.backup").is_ok() {
            utils::debug("Created backup: Cargo.lock.backup");
        } else {
            utils::warning("Failed to create Cargo.lock backup");
        }
        utils::info("Created backups of Cargo files");
    } else {
        utils::info("Created backup of Cargo.toml");
    }
}

fn run_cargo_update() {
    let mut cmd = Command::new("cargo");
    cmd.arg("update");

    // Pass --verbose to cargo if our verbose mode is enabled
    if utils::is_verbose() {
        cmd.arg("--verbose");
    }

    cmd.status().unwrap();
}
