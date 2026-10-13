use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct GetDaysSchemeIn {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Serialize, Deserialize)]
pub struct GetTrainingsByDateIn {
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateTrainingIn {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterUserIn {
    pub user_id: String,
    pub training_id: Uuid,
}
