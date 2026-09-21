use std::fs;

use chrono::{NaiveDate, NaiveTime};
use reqwest::Client;
use tokio::io;

use crate::{
    models::{CancelResponse, CancelTrainingAiRequest, OllamaResponse},
    service::errors::ServiceError,
};

pub struct CancelRepo {
    pub promt: String,
    pub url: String,
    pub key: String,
}

impl CancelRepo {
    pub fn new(promt_path: String, url: String, key: String) -> io::Result<Self> {
        let promt = fs::read_to_string(promt_path)?;
        Ok(Self { promt, url, key })
    }

    pub fn construct_ai_request(
        &self,
        user_message: String,
        current_date: NaiveDate,
        current_time: NaiveTime,
    ) -> serde_json::Value {
        let message = format!(
            "Сообщение тренера: {} \n Сегодняшняя дата: {} \n Время отправки сообщения: {} \n",
            user_message, current_date, current_time
        );
        let req = CancelTrainingAiRequest::new(self.promt.clone(), message);
        serde_json::to_value(req).unwrap()
    }

    pub async fn send_request_to_ai(
        &self,
        request: serde_json::Value,
    ) -> Result<CancelResponse, ServiceError> {
        let client = Client::new();
        let response = client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.key))
            .json(&request)
            .send()
            .await?;

        println!("STATUS: {}", response.status());
        let text = response.text().await?;
        println!("RESPONSE:\n{}", text);

        let response: OllamaResponse = serde_json::from_str(&text)?;
        let cancel_response: CancelResponse = serde_json::from_str(&response.message.content)?;
        Ok(cancel_response)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveTime, TimeDelta, Utc};
    use chrono_tz::Europe::Moscow;

    use crate::{config, models::CancelResponse, repo::cancel::CancelRepo};

    #[tokio::test]
    async fn test_cancel_days() {
        let config = config::Config::build();

        let repo = CancelRepo::new(config.promt_path, config.ai_api_url, config.ai_key).unwrap();
        let now = Utc::now().with_timezone(&Moscow);
        let req = repo.construct_ai_request(
            "Отмени тренировки с сегодня до 23 сентября".to_string(),
            now.date_naive(),
            now.time(),
        );
        // println!("{}", serde_json::to_string_pretty(&req).unwrap());
        let resp = repo.send_request_to_ai(req).await.unwrap();
        assert_eq!(
            resp,
            CancelResponse {
                date_from: now.date_naive(),
                date_to: now.date_naive() + TimeDelta::days(2),
                time_from: None,
                time_to: None,
            }
        );
    }

    #[tokio::test]
    async fn test_cancel_time() {
        let config = config::Config::build();
        let repo = CancelRepo::new(config.promt_path, config.ai_api_url, config.ai_key).unwrap();
        let now = Utc::now().with_timezone(&Moscow);
        let req = repo.construct_ai_request(
            "Отмени тренировку в 18:00".to_string(),
            now.date_naive(),
            now.time(),
        );
        let resp = repo.send_request_to_ai(req).await.unwrap();
        assert_eq!(
            resp,
            CancelResponse {
                date_from: now.date_naive(),
                date_to: now.date_naive(),
                time_from: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
                time_to: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            }
        );
    }

    #[tokio::test]
    async fn test_cancel_time_range() {
        let config = config::Config::build();
        let repo = CancelRepo::new(config.promt_path, config.ai_api_url, config.ai_key).unwrap();
        let now = Utc::now().with_timezone(&Moscow);
        let req = repo.construct_ai_request(
            "Отмени тренировку c трех до шести.".to_string(),
            now.date_naive(),
            now.time(),
        );
        let resp = repo.send_request_to_ai(req).await.unwrap();
        assert_eq!(
            resp,
            CancelResponse {
                date_from: now.date_naive(),
                date_to: now.date_naive(),
                time_from: Some(NaiveTime::from_hms_opt(15, 0, 0).unwrap()),
                time_to: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            }
        );
    }

    #[tokio::test]
    async fn test_cancel_exact_time_in_some_days() {
        let config = config::Config::build();
        let repo = CancelRepo::new(config.promt_path, config.ai_api_url, config.ai_key).unwrap();
        let now = Utc::now().with_timezone(&Moscow);
        let req = repo.construct_ai_request(
            "Отмени все тренировки в 17:00 в следующие три дня.".to_string(),
            now.date_naive(),
            now.time(),
        );
        let resp = repo.send_request_to_ai(req).await.unwrap();
        assert_eq!(
            resp,
            CancelResponse {
                date_from: now.date_naive() + TimeDelta::days(1),
                date_to: now.date_naive() + TimeDelta::days(3),
                time_from: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
                time_to: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
            }
        );
    }
}
