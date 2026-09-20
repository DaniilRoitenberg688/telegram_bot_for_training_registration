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
            "{} \n Сообщение тренера: {} \n Сегодняшняя дата: {} \n Время отправки сообщения: {}",
            self.promt, user_message, current_date, current_time
        );
        CancelTrainingAiRequest { message }
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
