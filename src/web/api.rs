use axum::{routing::get, Router};

use super::models::{DiscordPageData, HyperparamConfig, ModelPageData, RuntimePageData, SkillToolItem, UsageInfo};
use crate::utils::{gpu_utilization, ram_utilization, vram_utilization};

/// GET /api/model
async fn get_model() -> axum::Json<ModelPageData> {
    axum::Json(ModelPageData {
        model_path: "/models/llama-3-8b-instruct".to_string(),
        selected_model: "llama-3-8b-instruct".to_string(),
        available_models: vec![
            "llama-3-8b-instruct".to_string(),
            "llama-3-70b".to_string(),
            "mistral-7b-v0.3".to_string(),
            "qwen2-7b".to_string(),
            "phi-3-mini".to_string(),
        ],
        hyperparams: vec![
            HyperparamConfig {
                id: "temperature".to_string(),
                title: "Temperature".to_string(),
                value: 0.7,
                min: 0.0,
                max: 2.0,
                step: 0.05,
                default_value: 0.7,
            },
            HyperparamConfig {
                id: "top_p".to_string(),
                title: "Top P".to_string(),
                value: 0.9,
                min: 0.0,
                max: 1.0,
                step: 0.01,
                default_value: 0.9,
            },
            HyperparamConfig {
                id: "top_k".to_string(),
                title: "Top K".to_string(),
                value: 40.0,
                min: 1.0,
                max: 100.0,
                step: 1.0,
                default_value: 40.0,
            },
            HyperparamConfig {
                id: "max_tokens".to_string(),
                title: "Max Tokens".to_string(),
                value: 2048.0,
                min: 64.0,
                max: 8192.0,
                step: 64.0,
                default_value: 2048.0,
            },
            HyperparamConfig {
                id: "repeat_penalty".to_string(),
                title: "Repeat Penalty".to_string(),
                value: 1.1,
                min: 1.0,
                max: 2.0,
                step: 0.05,
                default_value: 1.1,
            },
        ],
    })
}

/// GET /api/runtime
async fn get_runtime() -> axum::Json<RuntimePageData> {
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

    let selected_device = available_devices.first().cloned().unwrap_or_else(|| "CPU".to_string());

    axum::Json(RuntimePageData {
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

/// GET /api/skills
async fn get_skills() -> axum::Json<Vec<SkillToolItem>> {
    axum::Json(vec![
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
async fn get_tools() -> axum::Json<Vec<SkillToolItem>> {
    axum::Json(vec![
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
async fn get_discord() -> axum::Json<DiscordPageData> {
    axum::Json(DiscordPageData {
        bot_token: String::new(),
        admin_role_id: String::new(),
        channel_ids: vec!["1234567890".to_string(), "0987654321".to_string()],
    })
}

/// Build the API router with all /api/* routes
pub fn router() -> Router {
    Router::new()
        .route("/model", get(get_model))
        .route("/runtime", get(get_runtime))
        .route("/skills", get(get_skills))
        .route("/tools", get(get_tools))
        .route("/discord", get(get_discord))
}
