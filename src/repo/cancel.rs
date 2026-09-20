use std::fs;

use chrono::{NaiveDate, NaiveTime};
use reqwest::Client;
use tokio::io;

use crate::models::{CancelResponse, CancelTrainingAiRequest};

pub struct CancelRepo {
    pub promt: String,
    pub url: String,
}

impl CancelRepo {
    pub fn new(promt_path: String, url: String) -> io::Result<Self> {
        let promt = fs::read_to_string(promt_path)?;
        Ok(Self { promt, url })
    }

    pub fn construct_ai_request(
        &self,
        user_message: String,
        current_date: NaiveDate,
        current_time: NaiveTime,
    ) -> CancelTrainingAiRequest {
        let message = format!(
            "Сообщение тренера: '{}' \n Сегодняшняя дата: {} \n Время отправки сообщения: {} \n Часовой пояс: Europe/Moscow",
            user_message, current_date, current_time
        );
        CancelTrainingAiRequest::new(self.promt.clone(), message)
    }

    pub async fn send_request_to_ai(
        &self,
        request: CancelTrainingAiRequest,
    ) -> Result<CancelResponse, reqwest::Error> {
        let client = Client::new();
        let response = client.post(&self.url).json(&request).send().await?;
        let cancel_response = response.json().await?;
        Ok(cancel_response)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, Utc};
    use chrono_tz::Europe::Moscow;

    use crate::{models::CancelResponse, repo::cancel::CancelRepo};

    #[tokio::test]
    async fn check_simple_case() {
        let repo = CancelRepo::new(
            "promts/promt.txt".to_string(),
            "http://77.83.85.190/api/chat".to_string(),
        )
        .unwrap();
        let now = Utc::now().with_timezone(&Moscow);
        let req = repo.construct_ai_request(
            "Отмени тренировки с сегодня до 23 сентября".to_string(),
            now.date_naive(),
            now.time(),
        );
        println!("{:?}", req);
        let resp = repo.send_request_to_ai(req).await.unwrap();
        assert_eq!(
            resp,
            CancelResponse {
                date_from: now.date_naive(),
                date_to: now.date_naive() + TimeDelta::days(3),
                time_from: None,
                time_to: None,
            }
        );
    }
}
