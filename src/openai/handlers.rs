use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use futures::stream;
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::models::{
    ChatChoice, ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
    ChunkChoice, ChunkDelta, UsageStats,
};

const HARDCODED_REPLY: &str =
    "I am Open Crafter Engine. LLM integration is not yet available.";

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
