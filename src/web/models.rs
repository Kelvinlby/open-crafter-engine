use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HyperparamConfig {
    pub id: String,
    pub title: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default_value: f64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelOption {
    pub folder: String,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPageData {
    pub model_path: String,
    pub selected_model: String,
    pub available_models: Vec<ModelOption>,
    pub hyperparams: Vec<HyperparamConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanModelsRequest {
    pub model_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageInfo {
    pub label: String,
    pub value: f64,
    pub detail: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePageData {
    pub ram: UsageInfo,
    pub vram: UsageInfo,
    pub gpu: UsageInfo,
    pub selected_device: String,
    pub available_devices: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillToolItem {
    pub id: String,
    pub title: String,
    pub version: String,
    pub description: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordPageData {
    pub bot_token: String,
    pub admin_channel_id: String,
    pub log_channel_id: String,
    pub user_channel_ids: Vec<String>,
}

// --- Save request models ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveModelConfigRequest {
    pub model_path: String,
    pub selected_model: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHyperparamRequest {
    pub param_id: String,
    pub value: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRuntimeConfigRequest {
    pub inference_device: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDiscordConfigRequest {
    pub bot_token: String,
    pub admin_channel_id: String,
    pub log_channel_id: String,
    pub user_channel_ids: Vec<String>,
}
