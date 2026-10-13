use crate::{models::User, repo::user::UserRepo, service::errors::ServiceError};

pub struct UserService {
    repo: UserRepo,
}

impl UserService {
    pub fn new(repo: UserRepo) -> Self {
        UserService { repo }
    }

    pub async fn get(&self, id: String) -> Result<User, ServiceError> {
        let user = match self.repo.get_by_id(id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => return Err(ServiceError::NotFound),
            Err(e) => return Err(e.into()),
        };
        Ok(user)
    }

    pub async fn register(&self, user: User) -> Result<(), ServiceError> {
        match self.repo.get_by_id(user.id.clone()).await {
            Ok(_) => Err(ServiceError::NotFound),
            Err(sqlx::Error::RowNotFound) => {
                self.repo.create(user).await?;
                Ok(())
            }
            Err(e) => Err(ServiceError::Sql(e)),
        }
    }

    pub async fn get_simple_users(&self) -> Vec<User> {
        match self.repo.get_all_users(Some(false)).await {
            Ok(u) => u,
            Err(e) => {
                eprintln!("cannot get users {}", e);
                Vec::new()
            }
        }
    }
}
