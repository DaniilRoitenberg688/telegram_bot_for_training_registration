use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter};

use crate::api::{
    state::ApiState, training::{
        get_trainigs_between_days, get_training_by_id, get_trainings_by_date, update_training,
    }, user::{delete_user_to_training, get_fullinfo_about_trainigs_for_trainer, get_trainigs_for_trainner_between_dates, get_user, get_user_trainings, get_weeks_for_trainer, register_user_to_training},
};

pub fn init_router(state: ApiState) -> Router {
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/user/{id}", get(get_user))
        .route("/days", get(get_trainigs_between_days))
        .route("/trainings", get(get_trainings_by_date))
        .route(
            "/training/{id}",
            get(get_training_by_id).put(update_training),
        )
        .route("/registrations/{id}", get(get_user_trainings))
        .route(
            "/registrations",
            post(register_user_to_training).delete(delete_user_to_training),
        )
        .route("/trainer/weeks", get(get_weeks_for_trainer))
        .route("/trainer/days", get(get_trainigs_for_trainner_between_dates))
        .route("/trainer/trainings", get(get_fullinfo_about_trainigs_for_trainer))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));
    Router::new().nest("/api", app)
}

pub async fn ping() -> String {
    "Pong".to_string()
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();
}
