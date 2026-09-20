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
    pub format: serde_json::Value,
}

impl CancelTrainingAiRequest {
    pub fn new(promt: String, message: String) -> Self {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "date_from": {
                    "type": ["string", "null"],
                    "description": "Дата начала периода в формате YYYY-MM-DD"
                },
                "date_to": {
                    "type": ["string", "null"],
                    "description": "Дата конца периода в формате YYYY-MM-DD"
                },
                "time_from": {
                    "type": ["string", "null"],
                    "description": "Время начала периода в формате HH:MM"
                },
                "time_to": {
                    "type": ["string", "null"],
                    "description": "Время конца периода в формате HH:MM"
                }
            },
            "required": [
                "date_from",
                "date_to",
                "time_from",
                "time_to"
            ]
        });

        Self {
            model: String::from("qwen3:1.7b"),
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
            format: schema,
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
    pub time_from: Option<NaiveDate>,
    pub time_to: Option<NaiveTime>,
}
