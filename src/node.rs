use crate::utils;
use serde_json::Value;
use std::fs::{copy, read_to_string, write};
use std::process::Command;

pub fn update_npm(backup: bool) {
    utils::info("Updating npm dependencies...");

    // Create backups first if enabled
    if backup {
        create_backups();
    }

    // Read package.json
    let content = match read_to_string("package.json") {
        Ok(c) => c,
        Err(_) => {
            utils::error("Error reading package.json");
            return;
        }
    };

    let mut json: Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => {
            utils::error("Invalid package.json file");
            return;
        }
    };

    // Get outdated packages
    let outdated = get_outdated_packages();
    if outdated.is_empty() {
        utils::info("No outdated npm packages found.");
        return;
    }

    let mut updates = 0;

    // Update dependencies and devDependencies
    for key in ["dependencies", "devDependencies"] {
        if let Some(deps) = json.get_mut(key).and_then(|v| v.as_object_mut()) {
            for (name, (current_version, latest_version)) in &outdated {
                if deps.contains_key(name) {
                    let new_ver = format!("^{}", latest_version);
                    utils::info(&format!(
                        "Updating {} from {} to {}",
                        name, current_version, latest_version
                    ));
                    deps.insert(name.to_string(), Value::String(new_ver));
                    updates += 1;
                }
            }
        }
    }

    if updates > 0 {
        // Write the updated package.json
        if let Err(e) = write("package.json", serde_json::to_string_pretty(&json).unwrap()) {
            utils::error(&format!("Failed to write updated package.json: {}", e));
            return;
        }

        utils::info(&format!("Updated {} package(s) in package.json", updates));

        // Run npm update to update the lock file
        utils::info("Running npm update...");
        let mut cmd = Command::new("npm");
        cmd.arg("update");

        // Add --verbose flag if verbose mode is enabled
        if utils::is_verbose() {
            cmd.arg("--verbose");
        }

        cmd.status().unwrap();
    } else {
        utils::info("No changes needed in package.json");
    }
}

fn get_outdated_packages() -> Vec<(String, (String, String))> {
    let mut outdated = Vec::new();

    let output = Command::new("npm").args(["outdated", "--json"]).output();

    match output {
        Ok(out) => {
            if !out.stdout.is_empty() {
                match serde_json::from_slice::<Value>(&out.stdout) {
                    Ok(json) => {
                        if let Some(obj) = json.as_object() {
                            for (name, details) in obj {
                                if let (Some(current), Some(latest)) = (
                                    details.get("current").and_then(|c| c.as_str()),
                                    details.get("latest").and_then(|l| l.as_str()),
                                ) {
                                    if current != latest {
                                        outdated.push((
                                            name.clone(),
                                            (current.to_string(), latest.to_string()),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        utils::warning("Failed to parse npm outdated output");
                    }
                }
            }
        }
        Err(e) => {
            utils::warning(&format!("Failed to run npm outdated: {}", e));
        }
    }

    outdated
}

fn create_backups() {
    // Create backup of package.json
    if copy("package.json", "package.json.backup").is_ok() {
        utils::debug("Created backup: package.json.backup");
    } else {
        utils::warning("Failed to create package.json backup");
    }

    // Create backup of package-lock.json if it exists
    if std::path::Path::new("package-lock.json").exists() {
        if copy("package-lock.json", "package-lock.json.backup").is_ok() {
            utils::debug("Created backup: package-lock.json.backup");
        } else {
            utils::warning("Failed to create package-lock.json backup");
        }
        utils::info("Created backups of npm files");
    } else {
        utils::info("Created backup of package.json");
    }
}
