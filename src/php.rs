use crate::utils;
use serde_json::Value;
use std::fs::{copy, read_to_string, write};
use std::process::Command;

pub fn update_composer(backup: bool) {
    utils::info("Updating composer dependencies...");

    // Create backups first if enabled
    if backup {
        create_backups();
    }

    // Read composer.json
    let content = match read_to_string("composer.json") {
        Ok(c) => c,
        Err(_) => {
            utils::error("Error reading composer.json");
            return;
        }
    };

    let mut json: Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => {
            utils::error("Invalid composer.json file");
            return;
        }
    };

    // Get outdated packages
    let outdated = get_outdated_packages();
    if outdated.is_empty() {
        utils::info("No outdated composer packages found.");
        return;
    }

    let mut updates = 0;

    // Update both require and require-dev sections
    for section_name in ["require", "require-dev"] {
        if let Some(section) = json.get_mut(section_name).and_then(|s| s.as_object_mut()) {
            for (name, (current_version, latest_version)) in &outdated {
                if section.contains_key(name) {
                    let new_ver = format!("^{}", latest_version);
                    utils::info(&format!(
                        "Updating {} from {} to {}",
                        name, current_version, latest_version
                    ));
                    section.insert(name.to_string(), Value::String(new_ver));
                    updates += 1;
                }
            }
        }
    }

    if updates > 0 {
        // Write the updated composer.json
        if let Err(e) = write(
            "composer.json",
            serde_json::to_string_pretty(&json).unwrap(),
        ) {
            utils::error(&format!("Failed to write updated composer.json: {}", e));
            return;
        }

        utils::info(&format!("Updated {} package(s) in composer.json", updates));

        // Run composer update to update the lock file
        utils::info("Running composer update...");

        let mut cmd = Command::new("composer");
        cmd.arg("update");

        // Add -v flag if verbose mode is enabled
        if utils::is_verbose() {
            cmd.arg("-v");
        }

        cmd.status().unwrap();
    } else {
        utils::info("No changes needed in composer.json");
    }
}

fn get_outdated_packages() -> Vec<(String, (String, String))> {
    let mut outdated = Vec::new();

    let output = Command::new("composer")
        .args(["outdated", "-D", "--format=json"])
        .output();

    match output {
        Ok(out) => {
            if !out.stdout.is_empty() {
                match serde_json::from_slice::<Value>(&out.stdout) {
                    Ok(json) => {
                        if let Some(installed) = json.get("installed").and_then(|i| i.as_array()) {
                            for package in installed {
                                let name =
                                    package.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                let current = package
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let latest =
                                    package.get("latest").and_then(|l| l.as_str()).unwrap_or("");
                                let status = package
                                    .get("latest-status")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("");

                                if status != "up-to-date"
                                    && !current.is_empty()
                                    && !latest.is_empty()
                                {
                                    outdated.push((
                                        name.to_string(),
                                        (current.to_string(), latest.to_string()),
                                    ));
                                }
                            }
                        }
                    }
                    Err(_) => {
                        utils::warning("Failed to parse composer outdated output");
                    }
                }
            }
        }
        Err(e) => {
            utils::warning(&format!("Failed to run composer outdated: {}", e));
        }
    }

    outdated
}

fn create_backups() {
    // Create backup of composer.json
    if copy("composer.json", "composer.json.backup").is_ok() {
        utils::debug("Created backup: composer.json.backup");
    } else {
        utils::warning("Failed to create composer.json backup");
    }

    // Create backup of composer.lock if it exists
    if std::path::Path::new("composer.lock").exists() {
        if copy("composer.lock", "composer.lock.backup").is_ok() {
            utils::debug("Created backup: composer.lock.backup");
        } else {
            utils::warning("Failed to create composer.lock backup");
        }
        utils::info("Created backups of composer files");
    } else {
        utils::info("Created backup of composer.json");
    }
}
