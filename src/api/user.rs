use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;

use crate::{
    api::{errors::ApiError, schemes::RegisterUserIn, state::ApiState},
    models::{Training, User},
};

pub async fn get_user(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    let user = state.user_service.get(id).await?;
    Ok((StatusCode::OK, Json(user)))
}

pub async fn get_user_trainings(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Vec<Training>>), ApiError> {
    let trainings = state
        .training_service
        .get_registered_trainings_for_user(id)
        .await?;
    Ok((StatusCode::OK, Json(trainings)))
}

pub async fn register_user_to_training(
    State(state): State<Arc<ApiState>>,
    Json(input): Json<RegisterUserIn>,
) -> Result<StatusCode, ApiError> {
    let _ = state.user_service.get(input.user_id.clone()).await?;
    let _ = state
        .training_service
        .register_to_training(input.user_id, input.training_id)
        .await?;
    Ok(StatusCode::CREATED)
}

pub async fn delete_user_to_training(
    State(state): State<Arc<ApiState>>,
    Json(input): Json<RegisterUserIn>,
) -> Result<StatusCode, ApiError> {
    let _ = state.user_service.get(input.user_id.clone()).await?;
    let _ = state
        .training_service
        .cancel_training(input.training_id, input.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
