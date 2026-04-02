use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::utils::validate_model_folder;

/// Persisted application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub model_path: String,
    pub selected_model: String,
    pub inference_device: String,
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub bot_token: String,
    pub admin_role_id: String,
    pub channel_ids: Vec<String>,
}

/// Shared config state wrapping the config and its file path.
pub struct ConfigState {
    pub config: AppConfig,
    pub config_path: PathBuf,
}

pub type SharedConfig = Arc<Mutex<ConfigState>>;

impl AppConfig {
    /// Build a default config using runtime information.
    fn default_with_devices(available_devices: &[String]) -> Self {
        let home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        let device = available_devices
            .first()
            .cloned()
            .unwrap_or_else(|| "CPU".to_string());

        Self {
            model_path: home,
            selected_model: String::new(),
            inference_device: device,
            discord: DiscordConfig {
                bot_token: String::new(),
                admin_role_id: String::new(),
                channel_ids: Vec::new(),
            },
        }
    }

    /// Validate and fix fields against current system state.
    fn validate(mut self, available_devices: &[String]) -> Self {
        // 1. Model path: must be an existing directory
        if self.model_path.is_empty() || !Path::new(&self.model_path).is_dir() {
            self.model_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy()
                .to_string();
        }

        // 2. Selected model: stored as full path to the model folder
        if !self.selected_model.is_empty() && !validate_model_folder(&self.selected_model) {
            self.selected_model = String::new();
        }

        // 3. Inference device: must be in available devices list
        if !available_devices.contains(&self.inference_device) {
            self.inference_device = available_devices
                .first()
                .cloned()
                .unwrap_or_else(|| "CPU".to_string());
        }

        self
    }
}

impl ConfigState {
    /// Save current config to disk.
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("failed to serialize config: {e}"))?;
        fs::write(&self.config_path, json)
            .map_err(|e| format!("failed to write config: {e}"))?;
        Ok(())
    }
}

/// Detect available inference devices (CUDA + CPU).
fn detect_devices() -> Vec<String> {
    let mut devices = Vec::new();
    if let Ok(nvml) = nvml_wrapper::Nvml::init() {
        if let Ok(count) = nvml.device_count() {
            for i in 0..count {
                if let Ok(device) = nvml.device_by_index(i) {
                    if let Ok(name) = device.name() {
                        devices.push(format!("CUDA:{i} ({name})"));
                    }
                }
            }
        }
    }
    devices.push("CPU".to_string());
    devices
}

/// Load config from disk, validate, and return shared state.
/// `exe_dir` is the directory containing the executable — config is stored at
/// `<exe_dir>/../engine-config.json`.
pub fn load(exe_dir: &Path) -> SharedConfig {
    let config_path = exe_dir.parent().unwrap_or(exe_dir).join("engine-config.json");
    let available_devices = detect_devices();

    let config = match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(cfg) => cfg.validate(&available_devices),
            Err(e) => {
                eprintln!("settings: engine config is malformed ({e}), using defaults");
                AppConfig::default_with_devices(&available_devices)
            }
        },
        Err(_) => {
            println!("settings: no engine config found, creating defaults");
            AppConfig::default_with_devices(&available_devices)
        }
    };

    let state = ConfigState {
        config,
        config_path,
    };

    // Persist the (possibly corrected) config
    if let Err(e) = state.save() {
        eprintln!("settings: failed to save config: {e}");
    }

    Arc::new(Mutex::new(state))
}

/// Update the hyperparameter `current` value inside a model's metadata.json.
/// `model_folder` is the full path to the model folder.
pub fn save_model_hyperparam(
    model_folder: &str,
    param_id: &str,
    value: f64,
) -> Result<(), String> {
    let metadata_path = Path::new(model_folder).join("metadata.json");

    let content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("failed to read metadata.json: {e}"))?;

    let mut json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("failed to parse metadata.json: {e}"))?;

    let current = json
        .get_mut("hyperparam")
        .and_then(|h| h.get_mut(param_id))
        .and_then(|p| p.get_mut("current"))
        .ok_or_else(|| format!("hyperparam '{param_id}' not found in metadata"))?;

    *current = serde_json::Value::from(value);

    let output = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("failed to serialize metadata: {e}"))?;
    fs::write(&metadata_path, output)
        .map_err(|e| format!("failed to write metadata.json: {e}"))?;

    Ok(())
}
