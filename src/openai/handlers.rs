use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use futures::stream;
use std::convert::Infallible;
use std::fs;
use std::path::Path as StdPath;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::models::{
    ChatChoice, ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
    ChunkChoice, ChunkDelta, Model, ModelDetail, ModelList, UsageStats,
};
use crate::settings::SharedConfig;
use crate::utils::validate_model_folder;

const HARDCODED_REPLY: &str =
    "I am Open Crafter Engine. LLM integration is not yet available.";

pub async fn list_models(State(config): State<SharedConfig>) -> impl IntoResponse {
    let state = config.lock().unwrap();
    let model_path = state.config.model_path.clone();
    drop(state);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut models = Vec::new();

    // Scan the model directory for valid model folders
    if let Ok(entries) = fs::read_dir(&model_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            if validate_model_folder(&path_str) {
                // Try to get model name from metadata, fallback to folder name
                let model_id = read_model_metadata(&path).0.unwrap_or_else(|| {
                    entry.file_name().to_string_lossy().to_string()
                });

                models.push(Model {
                    id: model_id,
                    object: "model",
                    created: now,
                    owned_by: "user",
                });
            }
        }
    }

    Json(ModelList {
        object: "list",
        data: models,
    })
}

pub async fn retrieve_model(
    State(config): State<SharedConfig>,
    Path(model_id): Path<String>,
) -> impl IntoResponse {
    let state = config.lock().unwrap();
    let model_path = state.config.model_path.clone();
    drop(state);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Search for the model in the model directory
    if let Ok(entries) = fs::read_dir(&model_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            if !validate_model_folder(&path_str) {
                continue;
            }

            // Check if this is the requested model
            let folder_name = entry.file_name().to_string_lossy().to_string();
            let (metadata_name, model_version) = read_model_metadata(&path);

            if folder_name == model_id || metadata_name.as_ref() == Some(&model_id) {
                let model_name = metadata_name;

                return Json(ModelDetail {
                    id: model_id,
                    object: "model",
                    created: now,
                    owned_by: "user",
                    model_name,
                    model_version,
                })
                .into_response();
            }
        }
    }

    // Model not found
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": {
                "message": format!("Model '{}' not found", model_id),
                "type": "invalid_request_error",
                "param": "model",
                "code": "model_not_found"
            }
        })),
    )
        .into_response()
}

/// Read model name and version from metadata.json
fn read_model_metadata(path: &StdPath) -> (Option<String>, Option<String>) {
    let metadata_path = path.join("metadata.json");
    
    if let Ok(content) = fs::read_to_string(&metadata_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let model_name = json
                .get("model_name")
                .and_then(|v| v.as_str())
                .map(String::from);
            
            let model_version = json
                .get("model_version")
                .and_then(|v| v.as_str())
                .map(String::from);
            
            return (model_name, model_version);
        }
    }
    
    (None, None)
}

pub async fn chat_completions(Json(body): Json<ChatCompletionRequest>) -> impl IntoResponse {
    let id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if body.stream.unwrap_or(false) {
        let id_clone = id.clone();
        let model = body.model.clone();

        // First chunk: role announcement
        let role_chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk",
            created,
            model: model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta {
                    role: Some("assistant"),
                    content: None,
                },
                finish_reason: None,
            }],
        };

        // Content chunk
        let content_chunk = ChatCompletionChunk {
            id: id_clone.clone(),
            object: "chat.completion.chunk",
            created,
            model: model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta {
                    role: None,
                    content: Some(HARDCODED_REPLY.to_string()),
                },
                finish_reason: None,
            }],
        };

        // Final chunk: finish_reason = "stop"
        let finish_chunk = ChatCompletionChunk {
            id: id_clone,
            object: "chat.completion.chunk",
            created,
            model,
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop"),
            }],
        };

        let events: Vec<Result<Event, Infallible>> = vec![
            Ok(Event::default().data(serde_json::to_string(&role_chunk).unwrap())),
            Ok(Event::default().data(serde_json::to_string(&content_chunk).unwrap())),
            Ok(Event::default().data(serde_json::to_string(&finish_chunk).unwrap())),
            Ok(Event::default().data("[DONE]")),
        ];

        Sse::new(stream::iter(events))
            .keep_alive(KeepAlive::default())
            .into_response()
    } else {
        Json(ChatCompletionResponse {
            id,
            object: "chat.completion",
            created,
            model: body.model,
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: HARDCODED_REPLY.to_string(),
                },
                finish_reason: "stop",
            }],
            usage: UsageStats {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
        .into_response()
    }
}
