use reqwest;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("failed to find object in database")]
    NotFound,
    #[error("failed to execute SQL query: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("reqwest sending error: {0}")]
    ReqwestSend(#[from] reqwest::Error),
    #[error("json parsing error: {0}")]
    Json(#[from] serde_json::Error),
}
