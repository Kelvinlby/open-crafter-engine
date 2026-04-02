use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::{get, post}, Json, Router};

use super::models::{
    DiscordPageData, HyperparamConfig, ModelOption, ModelPageData, RuntimePageData,
    SaveDiscordConfigRequest, SaveHyperparamRequest, SaveModelConfigRequest,
    SaveRuntimeConfigRequest, ScanModelsRequest, SkillToolItem, UsageInfo,
};
use crate::settings::{self, SharedConfig};
use crate::utils::{gpu_utilization, ram_utilization, validate_model_folder, vram_utilization};

/// GET /api/model
async fn get_model(State(config): State<SharedConfig>) -> Json<ModelPageData> {
    let state = config.lock().unwrap();
    let model_path = state.config.model_path.clone();
    let selected_model = state.config.selected_model.clone();
    drop(state);

    // Scan for available models at the stored path
    let available_models = scan_model_dir(&model_path);

    // Load hyperparams from the selected model's metadata.json
    let hyperparams = load_hyperparams(&selected_model);

    Json(ModelPageData {
        model_path,
        selected_model,
        available_models,
        hyperparams,
    })
}

/// Scan a directory for valid model subfolders.
/// Returns ModelOption with `folder` as the full absolute path.
fn scan_model_dir(base: &str) -> Vec<ModelOption> {
    let base_path = std::path::Path::new(base);
    let mut models = Vec::new();

    let entries = match std::fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(_) => return models,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !validate_model_folder(&path) {
            continue;
        }

        let metadata_path = path.join("metadata.json");
        let content = match std::fs::read_to_string(&metadata_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let model_name = match json.get("model_name").and_then(|v| v.as_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        models.push(ModelOption {
            folder: path.to_string_lossy().to_string(),
            name: model_name,
        });
    }

    models
}

/// Load hyperparameters from a model's metadata.json.
/// `model_folder` is the full path to the model folder.
fn load_hyperparams(model_folder: &str) -> Vec<HyperparamConfig> {
    if model_folder.is_empty() {
        return Vec::new();
    }

    let metadata_path = std::path::Path::new(model_folder).join("metadata.json");

    let content = match std::fs::read_to_string(&metadata_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let hyperparam = match json.get("hyperparam").and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => return Vec::new(),
    };

    let mut configs: Vec<HyperparamConfig> = Vec::new();
    for (id, val) in hyperparam {
        let current = val.get("current").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let default = val.get("default").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let min = val.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let max = val.get("max").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let step = val.get("step").and_then(|v| v.as_f64()).unwrap_or(0.01);

        // Use the id as title with first letter capitalized and underscores replaced
        let title = id
            .replace('_', " ")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        configs.push(HyperparamConfig {
            id: id.clone(),
            title,
            value: current,
            min,
            max,
            step,
            default_value: default,
        });
    }

    configs
}

/// POST /api/model/scan — scan a directory for valid model subfolders
async fn scan_models(Json(body): Json<ScanModelsRequest>) -> Json<Vec<ModelOption>> {
    Json(scan_model_dir(&body.model_path))
}

/// POST /api/model/save — save model path and selection to config
async fn save_model_config(
    State(config): State<SharedConfig>,
    Json(body): Json<SaveModelConfigRequest>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let mut state = config.lock().unwrap();
    state.config.model_path = body.model_path;
    state.config.selected_model = body.selected_model;
    state.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json("ok"))
}

/// POST /api/model/hyperparam — save a hyperparameter value to the model's metadata.json
async fn save_hyperparam(
    State(config): State<SharedConfig>,
    Json(body): Json<SaveHyperparamRequest>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let state = config.lock().unwrap();
    let selected_model = state.config.selected_model.clone();
    drop(state);

    if selected_model.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "no model selected".to_string()));
    }

    settings::save_model_hyperparam(&selected_model, &body.param_id, body.value)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json("ok"))
}

/// GET /api/runtime
async fn get_runtime(State(config): State<SharedConfig>) -> Json<RuntimePageData> {
    // RAM
    let ram_pct = ram_utilization();
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let total_ram = sys.total_memory() as f64 / (1024.0_f64.powi(3));
    let used_ram = sys.used_memory() as f64 / (1024.0_f64.powi(3));
    let ram_detail = format!("{:.1} / {:.1} GB", used_ram, total_ram);

    // VRAM
    let vram_pct = vram_utilization();
    let vram_detail = if let Ok(nvml) = nvml_wrapper::Nvml::init() {
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(mem) = device.memory_info() {
                let total_vram = mem.total as f64 / (1024.0_f64.powi(3));
                let used_vram = mem.used as f64 / (1024.0_f64.powi(3));
                format!("{:.1} / {:.1} GB", used_vram, total_vram)
            } else {
                "N/A".to_string()
            }
        } else {
            "N/A".to_string()
        }
    } else {
        "N/A".to_string()
    };

    // GPU
    let gpu_pct = gpu_utilization();
    let gpu_detail = format!("{:.0}% utilization", gpu_pct);

    // Available devices
    let mut available_devices: Vec<String> = Vec::new();
    if let Ok(nvml) = nvml_wrapper::Nvml::init() {
        if let Ok(count) = nvml.device_count() {
            for i in 0..count {
                if let Ok(device) = nvml.device_by_index(i) {
                    if let Ok(name) = device.name() {
                        available_devices.push(format!("CUDA:{} ({})", i, name));
                    }
                }
            }
        }
    }
    available_devices.push("CPU".to_string());

    // Use saved device selection, fall back to first available
    let state = config.lock().unwrap();
    let selected_device = if available_devices.contains(&state.config.inference_device) {
        state.config.inference_device.clone()
    } else {
        available_devices.first().cloned().unwrap_or_else(|| "CPU".to_string())
    };

    Json(RuntimePageData {
        ram: UsageInfo {
            label: "RAM".to_string(),
            value: ram_pct,
            detail: ram_detail,
        },
        vram: UsageInfo {
            label: "VRAM".to_string(),
            value: vram_pct,
            detail: vram_detail,
        },
        gpu: UsageInfo {
            label: "GPU".to_string(),
            value: gpu_pct,
            detail: gpu_detail,
        },
        selected_device,
        available_devices,
    })
}

/// POST /api/runtime/save — save inference device selection
async fn save_runtime_config(
    State(config): State<SharedConfig>,
    Json(body): Json<SaveRuntimeConfigRequest>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let mut state = config.lock().unwrap();
    state.config.inference_device = body.inference_device;
    state.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json("ok"))
}

/// GET /api/skills
async fn get_skills() -> Json<Vec<SkillToolItem>> {
    Json(vec![
        SkillToolItem {
            id: "pathfinding".to_string(),
            title: "Pathfinding".to_string(),
            version: "1.2.0".to_string(),
            description: "A* pathfinding with dynamic obstacle avoidance. Supports 3D navigation mesh traversal for complex terrain including water, lava, and scaffolding. Includes jump-sprint optimization and elytra flight paths.".to_string(),
        },
        SkillToolItem {
            id: "building".to_string(),
            title: "Building".to_string(),
            version: "0.8.1".to_string(),
            description: "Schematic-based building with automatic material gathering. Supports NBT structure files and litematica schematics. Includes scaffolding placement and block-by-block verification.".to_string(),
        },
        SkillToolItem {
            id: "combat".to_string(),
            title: "Combat".to_string(),
            version: "1.0.3".to_string(),
            description: "PvE combat with mob targeting, shield blocking, and bow aiming. Supports critical hits, sweep attacks, and potion usage. Includes flee behavior when health is low.".to_string(),
        },
        SkillToolItem {
            id: "farming".to_string(),
            title: "Farming".to_string(),
            version: "1.1.0".to_string(),
            description: "Automated crop farming with replanting. Supports wheat, carrots, potatoes, beetroot, and nether wart. Includes bone meal optimization and harvest timing.".to_string(),
        },
        SkillToolItem {
            id: "mining".to_string(),
            title: "Mining".to_string(),
            version: "2.0.0".to_string(),
            description: "Strip mining and branch mining with ore detection. Supports fortune and silk touch tool selection. Includes torch placement and lava/water hazard avoidance.".to_string(),
        },
    ])
}

/// GET /api/tools
async fn get_tools() -> Json<Vec<SkillToolItem>> {
    Json(vec![
        SkillToolItem {
            id: "chat".to_string(),
            title: "Chat".to_string(),
            version: "1.0.0".to_string(),
            description: "Send and receive in-game chat messages. Supports whisper, party, and global channels. Includes message formatting and command execution.".to_string(),
        },
        SkillToolItem {
            id: "inventory".to_string(),
            title: "Inventory".to_string(),
            version: "1.3.2".to_string(),
            description: "Inspect and manage player inventory. Supports item sorting, crafting recipe lookup, and container interaction (chests, furnaces, brewing stands).".to_string(),
        },
        SkillToolItem {
            id: "world".to_string(),
            title: "World Info".to_string(),
            version: "1.1.0".to_string(),
            description: "Query world state including time, weather, biome, and nearby entities. Supports block scanning in a configurable radius and structure detection.".to_string(),
        },
        SkillToolItem {
            id: "movement".to_string(),
            title: "Movement".to_string(),
            version: "0.9.5".to_string(),
            description: "Low-level movement commands: walk, sprint, jump, sneak, swim. Supports coordinate-based movement and relative direction commands.".to_string(),
        },
    ])
}

/// GET /api/discord
async fn get_discord(State(config): State<SharedConfig>) -> Json<DiscordPageData> {
    let state = config.lock().unwrap();
    Json(DiscordPageData {
        bot_token: state.config.discord.bot_token.clone(),
        admin_channel_id: state.config.discord.admin_channel_id.clone(),
        log_channel_id: state.config.discord.log_channel_id.clone(),
        user_channel_ids: state.config.discord.user_channel_ids.clone(),
    })
}

/// POST /api/discord/save — save discord configuration
async fn save_discord_config(
    State(config): State<SharedConfig>,
    Json(body): Json<SaveDiscordConfigRequest>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let mut state = config.lock().unwrap();
    state.config.discord.bot_token = body.bot_token;
    state.config.discord.admin_channel_id = body.admin_channel_id;
    state.config.discord.log_channel_id = body.log_channel_id;
    state.config.discord.user_channel_ids = body.user_channel_ids;
    state.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json("ok"))
}

/// Build the API router with all /api/* routes
pub fn router(config: SharedConfig) -> Router {
    Router::new()
        .route("/model", get(get_model))
        .route("/model/scan", post(scan_models))
        .route("/model/save", post(save_model_config))
        .route("/model/hyperparam", post(save_hyperparam))
        .route("/runtime", get(get_runtime))
        .route("/runtime/save", post(save_runtime_config))
        .route("/skills", get(get_skills))
        .route("/tools", get(get_tools))
        .route("/discord", get(get_discord))
        .route("/discord/save", post(save_discord_config))
        .with_state(config)
}
