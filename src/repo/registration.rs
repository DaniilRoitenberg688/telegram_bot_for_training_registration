use sqlx::SqlitePool;

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
}
