use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Default)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub is_trainer: bool,
}

#[derive(Debug, Clone, FromRow, Default)]
pub struct Training {
    pub id: Uuid,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub capacity: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, FromRow, Default)]
pub struct Registration {
    pub id: Uuid,
    pub user_id: String,
    pub training_id: Uuid,
}

#[derive(Debug, Clone, FromRow, Default)]
pub struct Notification {
    pub id: Uuid,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, FromRow)]
pub struct RegistrationFullInfo {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub full_name: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CancelTrainingAiRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

pub fn create_cancel_request(promt: String, message: String) -> serde_json::Value {
    serde_json::json!({
        "model": "nex-agi/nex-n2.5-mini:free",
        "messages": [
            {
                "role": "system",
                "content": promt
            },
            {
                "role": "user",
                "content": message
            }
        ],
        "stream": false,
        "max_tokens": 200,
        "reasoning": {
            "enabled": false
        },
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "cancel_training",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "date_from": {"type": ["string", "null"]},
                        "date_to": {"type": ["string", "null"]},
                        "time_from": {"type": ["string", "null"]},
                        "time_to": {"type": ["string", "null"]}
                    },
                    "required": [
                        "date_from",
                        "date_to",
                        "time_from",
                        "time_to"
                    ],
                    "additionalProperties": false
                }
            }
        }
    })
}

impl CancelTrainingAiRequest {
    pub fn new(promt: String, message: String) -> Self {
        Self {
            model: String::from("gpt-oss:120b-cloud"),
            messages: vec![
                Message {
                    role: String::from("system"),
                    content: promt,
                },
                Message {
                    role: String::from("user"),
                    content: message,
                },
            ],
            stream: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CancelResponse {
    pub date_from: NaiveDate,
    pub date_to: NaiveDate,
    pub time_from: Option<NaiveTime>,
    pub time_to: Option<NaiveTime>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaResponse {
    pub message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
pub struct Choices {
    pub message: OllamaMessage,
}

#[derive(Deserialize, Debug)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}
