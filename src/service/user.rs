
use crate::{models::User, repo::user::UserRepo, service::errors::ServiceError, types::MyResult};

pub struct UserService {
    repo: UserRepo,
}

impl UserService {
    pub fn new(repo: UserRepo) -> Self {
        UserService { repo }
    }

    pub async fn get(&self, id: String) -> Option<User> {
        self.repo.get_by_id(id).await.ok()
    }

    pub async fn register(&self, user: User) -> Result<(), ServiceError>{
        self.repo.get_by_id(user.id.clone()).await?;
        Ok(())
    }

}
