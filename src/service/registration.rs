use uuid::Uuid;

use crate::{repo::registration::RegistrationRepo, service::errors::ServiceError};

pub struct RegistrationService {
    repo: RegistrationRepo
}

impl RegistrationService {
    pub fn new(repo: RegistrationRepo) -> Self {
        Self { repo }
    }
}
