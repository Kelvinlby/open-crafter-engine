use serde::Serialize;

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPageData {
    pub model_path: String,
    pub selected_model: String,
    pub available_models: Vec<String>,
    pub hyperparams: Vec<HyperparamConfig>,
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
    pub admin_role_id: String,
    pub channel_ids: Vec<String>,
}
