use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetDaysSchemeIn {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Serialize, Deserialize)]
pub struct GetTrainingsByDateIn {
    pub date: NaiveDate,
}
