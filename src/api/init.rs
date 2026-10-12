use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use reqwest::StatusCode;

use crate::{api::errors::ApiError, models::User, service::user::UserService};

pub fn init_router(state: ApiState) -> Router {
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/user/{id}", get(get_user))
        .with_state(Arc::new(state));
    app
}

pub async fn ping() -> String {
    "Pong".to_string()
}

pub async fn get_user(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    match state.user_service.get(id).await {
        None => Err(ApiError::NotFound),
        Some(user) => Ok((StatusCode::OK, Json(user))),
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub user_service: Arc<UserService>,
}
