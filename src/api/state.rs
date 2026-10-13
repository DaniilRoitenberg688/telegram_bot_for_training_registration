use std::sync::Arc;

use crate::service::{training::TrainingService, user::UserService};

#[derive(Clone)]
pub struct ApiState {
    pub user_service: Arc<UserService>,
    pub training_service: Arc<TrainingService>,
}
