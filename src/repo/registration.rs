use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::Registration;

pub struct RegistrationRepo {
    db: SqlitePool
}

impl RegistrationRepo {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create(&self, registration: Registration) -> Result<(), sqlx::Error> {
        sqlx::query("insert into registrations (id, user_id, training_id) values ($1, $2, $3)")
            .bind(registration.id)
            .bind(registration.user_id)
            .bind(registration.training_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, user_id: String, training_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("delete from registrations where user_id = $1 and training_id = $2")
            .bind(user_id)
            .bind(training_id)
            .execute(&self.db).await?;
        Ok(())

    }

}
