use chrono::NaiveDate;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::Notification;

pub struct NotificationRepo {
    db: SqlitePool
}

impl NotificationRepo {
    pub fn new(db: SqlitePool) -> Self {
        Self {db}
    }

    pub async fn create(&self, date: NaiveDate) -> Result<(), sqlx::Error> {
        sqlx::query("insert into notifications (id, date) values ($1, $2)")
            .bind(Uuid::new_v4())
            .bind(date)
            .execute(&self.db).await?;
        Ok(())
    }

    pub async fn get_by_date(&self, date: NaiveDate) -> Result<Notification, sqlx::Error> {
        let notif = sqlx::query_as::<_, Notification>("select * from notifications where date = $1")
            .bind(date)
            .fetch_one(&self.db).await?;
        Ok(notif)
    }
}
