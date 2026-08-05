use anyhow::Context;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::Training;

pub struct TrainingRepo {
    db: SqlitePool,
}

impl TrainingRepo {
    pub fn new(db: SqlitePool) -> Self {
        TrainingRepo { db }
    }

    pub async fn create(&self, training: Training) -> Result<(), sqlx::Error> {
        sqlx::query("insert into trainings (id, date, start_time, end_time, capacity, enabled) values ($1, $2, $3, $4, $5, $6)")
            .bind(training.id)
            .bind(training.date)
            .bind(training.start_time)
            .bind(training.end_time)
            .bind(training.capacity)
            .bind(training.enabled)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Training, sqlx::Error> {
        let training = sqlx::query_as::<_, Training>("select * from trainings where id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(training)
    }

    pub async fn get_between_dates(
        &self,
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
        repeats: bool,
    ) -> anyhow::Result<Vec<Training>> {
        let mut sql = "select * from trainings where ($1 is NULL or date >= $1) and ($2 is NULL or date <= $2)";
        if !repeats {
            sql = "select id, date, start_time, end_time, capacity, enabled from trainings 
                    where ($1 is NULL or date >= $1) and ($2 is NULL or date <= $2)
                    group by date";
        }
        let trainings = sqlx::query_as::<_, Training>(sql)
            .bind(from)
            .bind(to)
            .fetch_all(&self.db).await?;
        Ok(trainings)



    }
}
