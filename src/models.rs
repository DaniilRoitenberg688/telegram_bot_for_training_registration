use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;
use sqlx::prelude::FromRow;


#[derive(Debug, Clone, FromRow, Default)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub is_trainer: bool
}


#[derive(Debug, Clone, FromRow, Default)]
pub struct Training {
    pub id: Uuid,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub capacity: u32,
    pub enabled: bool
}


#[derive(Debug, Clone, FromRow, Default)]
pub struct Registration {
    pub id: Uuid,
    pub user_id: String,
    pub training_id: Uuid,
}
