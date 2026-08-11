use anyhow::Context;
use chrono::NaiveDate;
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
    ) -> Result<Vec<Training>, sqlx::Error> {
        let mut sql = "select * from trainings where ($1 is NULL or date >= $1) and ($2 is NULL or date <= $2)";
        if !repeats {
            sql = "select id, date, start_time, end_time, capacity, enabled from trainings 
                    where ($1 is NULL or date >= $1) and ($2 is NULL or date <= $2)
                    group by date";
        }
        let trainings = sqlx::query_as::<_, Training>(sql)
            .bind(from)
            .bind(to)
            .fetch_all(&self.db)
            .await?;
        Ok(trainings)
    }

    pub async fn get_by_date(&self, date: NaiveDate) -> Result<Vec<Training>, sqlx::Error> {
        let training = sqlx::query_as::<_, Training>("select * from trainings where date = $1")
            .bind(date)
            .fetch_all(&self.db)
            .await?;
        Ok(training)
    }

    pub async fn get_by_date_without_registration(&self, date: NaiveDate) -> Result<Vec<Training>, sqlx::Error> {
        let training = sqlx::query_as::<_, Training>("select trainings.id as id, date, start_time, end_time, capacity, enabled from trainings
                                                        left join registrations on trainings.id = training_id
                                                        where registrations.id is NULL and date = $1")
            .bind(date)
            .fetch_all(&self.db)
            .await?;
        Ok(training)
    }

    pub async fn get_registered_trainings_for_user(&self, user_id: String) -> Result<Vec<Training>, sqlx::Error> {
        let trainings = sqlx::query_as::<_, Training>("select trainings.id as id, date, start_time, end_time, capacity, enabled from registrations 
                                                        join trainings on trainings.id = registrations.training_id 
                                                        where user_id = $1")
            .bind(user_id)
            .fetch_all(&self.db).await?;
        Ok(trainings)
    }
}
