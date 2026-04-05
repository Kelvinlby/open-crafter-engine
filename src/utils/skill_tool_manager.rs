use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::web::models::SkillToolItem;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkillInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

/// Validates that a path is a valid skill folder.
///
/// A valid skill folder must contain:
/// - Exactly one `skill.md` file
/// - Exactly one `info.json` file with keys: name, version, enabled (no extras)
pub fn validate_skill_folder<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    if !path.is_dir() {
        return false;
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return false,
    };

    let mut skill_md_count = 0usize;
    let mut info_json_count = 0usize;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        match entry.file_name().to_string_lossy().as_ref() {
            "skill.md" => skill_md_count += 1,
            "info.json" => info_json_count += 1,
            _ => {}
        }
    }

    if skill_md_count != 1 || info_json_count != 1 {
        return false;
    }

    let content = match fs::read_to_string(path.join("info.json")) {
        Ok(c) => c,
        Err(_) => return false,
    };

    serde_json::from_str::<SkillInfo>(&content).is_ok()
}

/// Validates that a path is a valid tool JSON file.
///
/// A valid tool file must be a `.json` file with keys: name, version, enabled (no extras).
pub fn validate_tool_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    if !path.is_file() {
        return false;
    }

    if path.extension().and_then(|e| e.to_str()) != Some("json") {
        return false;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    serde_json::from_str::<ToolInfo>(&content).is_ok()
}

/// Scans `base_dir` for valid skill subfolders and returns their data.
/// Silently skips invalid entries. Returns empty Vec if `base_dir` is unreadable.
pub fn scan_skills(base_dir: &Path) -> Vec<SkillToolItem> {
    let entries = match fs::read_dir(base_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() || !validate_skill_folder(&path) {
            continue;
        }

        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let info: SkillInfo = match fs::read_to_string(path.join("info.json"))
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
        {
            Some(i) => i,
            None => continue,
        };

        let description = match fs::read_to_string(path.join("skill.md")) {
            Ok(c) => c,
            Err(_) => continue,
        };

        items.push(SkillToolItem {
            id,
            title: info.name,
            version: info.version,
            description,
            enabled: info.enabled,
        });
    }

    items
}

/// Scans `base_dir` for valid tool JSON files and returns their data.
/// Silently skips invalid entries. Returns empty Vec if `base_dir` is unreadable.
pub fn scan_tools(base_dir: &Path) -> Vec<SkillToolItem> {
    let entries = match fs::read_dir(base_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() || !validate_tool_file(&path) {
            continue;
        }

        let id = match path.file_stem().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let info: ToolInfo = match fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
        {
            Some(i) => i,
            None => continue,
        };

        items.push(SkillToolItem {
            id,
            title: info.name,
            version: info.version,
            description: info.description,
            enabled: info.enabled,
        });
    }

    items
}

/// Updates the `enabled` field in a skill's `info.json` file.
pub fn toggle_skill(base_dir: &Path, id: &str, enabled: bool) -> Result<(), String> {
    let info_path = base_dir.join(id).join("info.json");

    let content = fs::read_to_string(&info_path)
        .map_err(|e| format!("failed to read {}: {e}", info_path.display()))?;

    let info: SkillInfo = serde_json::from_str(&content)
        .map_err(|e| format!("invalid info.json for skill '{id}': {e}"))?;

    let updated = SkillInfo { name: info.name, version: info.version, enabled };

    let json = serde_json::to_string_pretty(&updated)
        .map_err(|e| format!("serialization error: {e}"))?;

    fs::write(&info_path, json)
        .map_err(|e| format!("failed to write {}: {e}", info_path.display()))?;

    Ok(())
}

/// Updates the `enabled` field in a tool's JSON file.
pub fn toggle_tool(base_dir: &Path, id: &str, enabled: bool) -> Result<(), String> {
    let tool_path = base_dir.join(format!("{id}.json"));

    let content = fs::read_to_string(&tool_path)
        .map_err(|e| format!("failed to read {}: {e}", tool_path.display()))?;

    let info: ToolInfo = serde_json::from_str(&content)
        .map_err(|e| format!("invalid json for tool '{id}': {e}"))?;

    let updated = ToolInfo { name: info.name, version: info.version, description: info.description, enabled };

    let json = serde_json::to_string_pretty(&updated)
        .map_err(|e| format!("serialization error: {e}"))?;

    fs::write(&tool_path, json)
        .map_err(|e| format!("failed to write {}: {e}", tool_path.display()))?;

    Ok(())
}
