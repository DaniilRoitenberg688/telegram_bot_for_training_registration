
use crate::repo::registration::RegistrationRepo;

pub struct RegistrationService {
    repo: RegistrationRepo
}

impl RegistrationService {
    pub fn new(repo: RegistrationRepo) -> Self {
        Self { repo }
    }
}
