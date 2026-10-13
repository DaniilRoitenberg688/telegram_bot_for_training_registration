use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    api::{
        errors::ApiError,
        schemes::{GetDaysSchemeIn, GetTrainingsByDateIn},
        state::ApiState,
    },
    models::Training,
};

pub async fn get_trainigs_between_days(
    State(state): State<Arc<ApiState>>,
    Query(body): Query<GetDaysSchemeIn>,
) -> Result<(StatusCode, Json<Vec<Training>>), ApiError> {
    let trainings = state
        .training_service
        .get_week_traings(body.from, body.to)
        .await;
    Ok((StatusCode::OK, Json(trainings)))
}

pub async fn get_trainings_by_date(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<GetTrainingsByDateIn>,
) -> Result<(StatusCode, Json<Vec<Training>>), ApiError> {
    let trainings = state.training_service.get_training_by_date(q.date).await;
    Ok((StatusCode::OK, Json(trainings)))
}

pub async fn get_training_by_id(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Training>), ApiError> {
    match state.training_service.get(id).await {
        None => Err(ApiError::NotFound),
        Some(training) => Ok((StatusCode::OK, Json(training))),
    }
}
