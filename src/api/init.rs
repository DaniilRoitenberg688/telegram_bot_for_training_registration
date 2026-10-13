use std::sync::Arc;

use axum::{Router, routing::get};

use crate::api::{
    state::ApiState,
    training::{get_trainigs_between_days, get_training_by_id, get_trainings_by_date},
    user::get_user,
};

pub fn init_router(state: ApiState) -> Router {
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/user/{id}", get(get_user))
        .route("/days", get(get_trainigs_between_days))
        .route("/trainings", get(get_trainings_by_date))
        .route("/training/{id}", get(get_training_by_id))
        .with_state(Arc::new(state));
    Router::new().nest("/api", app)
}

pub async fn ping() -> String {
    "Pong".to_string()
}
