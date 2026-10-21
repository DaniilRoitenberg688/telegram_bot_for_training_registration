use std::sync::Arc;

use axum::{
    Json, extract::{Path, State},
};
use chrono::NaiveDate;
use reqwest::StatusCode;

use crate::{
    api::{errors::ApiError, schemes::{GetDaysSchemeIn, GetFullTrainingInfoIn, RegisterUserIn}, state::ApiState}, models::{RegistrationFullInfo, Training, User},
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

pub async fn get_weeks_for_trainer(
    State(state): State<Arc<ApiState>>,
    Json(body): Json<Vec<Vec<NaiveDate>>>
    ) -> Result<(StatusCode, Json<Vec<Vec<NaiveDate>>>), ApiError> {
    let weeks = state.training_service.get_weeks_with_trainings(body).await?;
    Ok((StatusCode::OK, Json(weeks)))
}


pub async fn get_trainigs_for_trainner_between_dates(
    State(state): State<Arc<ApiState>>,
    Json(body): Json<GetDaysSchemeIn>
    ) -> Result<(StatusCode, Json<Vec<Training>>), ApiError> {
    let days = state.training_service.get_trainings_for_trainer_betweeen_dates(body.from, body.to).await?;
    Ok((StatusCode::OK, Json(days)))
}

pub async fn get_fullinfo_about_trainigs_for_trainer(
    State(state): State<Arc<ApiState>>,
    Json(body): Json<GetFullTrainingInfoIn>
    ) -> Result<(StatusCode, Json<Vec<RegistrationFullInfo>>), ApiError> {
    let days = state.training_service.get_trainings_by_date_with_registration(body.date).await?;
    Ok((StatusCode::OK, Json(days)))
}




