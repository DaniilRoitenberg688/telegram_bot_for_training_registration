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

impl ToString for CancelResponse {
    fn to_string(&self) -> String {
        let mut res = "Отмена: ".to_string();
        if self.date_from == self.date_to {
            res += &format!("{} ", self.date_from.format("%d.%m"));
        } else {
            res += &format!(
                "{} - {} ",
                self.date_from.format("%d.%m"),
                self.date_to.format("%d.%m")
            );
        }

        if let Some(time_from) = self.time_from
            && let Some(time_to) = self.time_to
        {
            if time_to == time_from {
                res += &format!("в {}", time_from.format("%H:%M"));
            } else {
                res += &format!(
                    "c {} по {}",
                    time_from.format("%H:%M"),
                    time_to.format("%H:%M")
                );
            }
        };

        if let Some(time_from) = self.time_from
            && let None = self.time_to
        {
            res += &format!("c {} до конца дня", time_from.format("%H:%M"));
        }

        if let None = self.time_from
            && let Some(time_to) = self.time_to
        {
            res += &format!("c начал дня до {}", time_to.format("%H:%M"));
        }

        res
    }
}

#[derive(Deserialize, Debug)]
pub struct OllamaResponse {
    pub message: OllamaMessage,
}

#[derive(Deserialize, Debug)]
pub struct OllamaMessage {
    pub content: String,
}
