use sqlx::SqlitePool;
use crate::models::User;




pub struct UserRepo {
    db: SqlitePool
}


impl UserRepo {
    pub fn new(db: SqlitePool) -> Self {
        UserRepo { db }
    }

    pub async fn create(&self, user: User) -> Result<(), sqlx::Error> {
        sqlx::query("insert into users (id, full_name, username, first_name, last_name, is_trainer) values ($1, $2, $3, $4, $5, $6)")
            .bind(user.id)
            .bind(user.full_name)
            .bind(user.username)
            .bind(user.first_name)
            .bind(user.last_name)
            .bind(user.is_trainer)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("select * from users where id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(user)

    }

    pub async fn get_all_users(&self, is_trainer: Option<bool>) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>("select * from users where ($1 is NULL or is_trainer = $1)")
            .bind(is_trainer)
            .fetch_all(&self.db).await?;
        Ok(users)
    }
}
