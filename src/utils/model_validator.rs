use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct HyperparamValue {
    default: serde_json::Value,
    current: serde_json::Value,
    min: Option<serde_json::Value>,
    max: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Metadata {
    model_name: String,
    model_version: String,
    model_list: std::collections::HashMap<String, String>,
    hyperparam: std::collections::HashMap<String, HyperparamValue>,
}

/// Validates that a folder is a valid model folder.
///
/// A valid model folder must contain:
/// - One or more `.pt2` files
/// - A `metadata.json` file with the required structure
///
/// # Arguments
/// * `path` - A path-like object pointing to the model folder
///
/// # Returns
/// * `true` if all checks pass
/// * `false` if any check fails
pub fn validate_model_folder<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Check if path exists and is a directory
    if !path.exists() || !path.is_dir() {
        return false;
    }

    // Read directory entries
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return false,
    };

    let mut has_pt2_file = false;
    let mut has_metadata = false;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        if file_name.ends_with(".pt2") {
            has_pt2_file = true;
        } else if file_name == "metadata.json" {
            has_metadata = true;
        }
    }

    // Must have at least one .pt2 file and a metadata.json
    if !has_pt2_file || !has_metadata {
        return false;
    }

    // Validate metadata.json structure
    let metadata_path = path.join("metadata.json");
    let metadata_content = match fs::read_to_string(metadata_path) {
        Ok(content) => content,
        Err(_) => return false,
    };

    let metadata: Metadata = match serde_json::from_str(&metadata_content) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Validate hyperparam values (default, current, min, max must be float, int, or null)
    for hyperparam in metadata.hyperparam.values() {
        if !is_valid_numeric_value(&hyperparam.default) {
            return false;
        }
        if !is_valid_numeric_value(&hyperparam.current) {
            return false;
        }
        if let Some(min) = &hyperparam.min {
            if !is_valid_numeric_value(min) {
                return false;
            }
        }
        if let Some(max) = &hyperparam.max {
            if !is_valid_numeric_value(max) {
                return false;
            }
        }
    }

    true
}

/// Checks if a JSON value is a valid numeric value (int or float)
fn is_valid_numeric_value(value: &serde_json::Value) -> bool {
    matches!(value, serde_json::Value::Number(_))
}

