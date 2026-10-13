use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;

use crate::{
    api::{errors::ApiError, state::ApiState},
    models::User,
};

pub async fn get_user(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    match state.user_service.get(id).await {
        None => Err(ApiError::NotFound),
        Some(user) => Ok((StatusCode::OK, Json(user))),
    }
}
