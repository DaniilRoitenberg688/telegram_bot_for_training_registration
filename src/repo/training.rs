use chrono::NaiveDate;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    models::{CancelResponse, RegistrationFullInfo, Training, TrainerSlot},
    service::errors::ServiceError,
};

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
    ) -> Result<Vec<Training>, ServiceError> {
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

    pub async fn get_by_date_without_registration(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<Training>, sqlx::Error> {
        let training = sqlx::query_as::<_, Training>("select trainings.id as id, date, start_time, end_time, capacity, enabled from trainings
                                                        left join registrations on trainings.id = training_id
                                                        where enabled = true
                                                        and registrations.id is NULL and date = $1")
            .bind(date)
            .fetch_all(&self.db)
            .await?;
        Ok(training)
    }

    pub async fn get_registered_trainings_for_user(
        &self,
        user_id: String,
    ) -> Result<Vec<Training>, sqlx::Error> {
        let trainings = sqlx::query_as::<_, Training>("select trainings.id as id, date, start_time, end_time, capacity, enabled from registrations
                                                        join trainings on trainings.id = registrations.training_id
                                                        where user_id = $1 and enabled = true")
            .bind(user_id)
            .fetch_all(&self.db).await?;
        Ok(trainings)
    }

    pub async fn get_trainings_with_registration(&self) -> Result<Vec<Training>, sqlx::Error> {
        let training = sqlx::query_as::<_, Training>("select trainings.id as id, date, start_time, end_time, capacity, enabled from trainings
                                                        left join registrations on trainings.id = training_id
                                                        where registrations.id is not NULL and enabled = true
                                                        group by date")
            .fetch_all(&self.db)
            .await?;
        Ok(training)
    }

    pub async fn get_trainings_with_registration_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<RegistrationFullInfo>, sqlx::Error> {
        let trainings = sqlx::query_as::<_, RegistrationFullInfo>(
            "select date, start_time, end_time, full_name, username from trainings
            left join registrations on trainings.id = training_id
            join users on users.id = user_id
            where registrations.id is not NULL and date = $1 and enabled = true",
        )
        .bind(date)
        .fetch_all(&self.db)
        .await?;
        Ok(trainings)
    }

    pub async fn get_all_slots_by_date(&self, date: NaiveDate) -> Result<Vec<TrainerSlot>, sqlx::Error> {
        sqlx::query_as::<_, TrainerSlot>(
            "select trainings.id as id, date, start_time, end_time, enabled,
                    users.id as user_id, users.full_name as full_name, users.username as username
             from trainings
             left join registrations on registrations.training_id = trainings.id
             left join users on users.id = registrations.user_id
             where date = $1
             order by start_time",
        )
        .bind(date)
        .fetch_all(&self.db)
        .await
    }

    pub async fn cancel_training(
        &self,
        training_response: CancelResponse,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            UPDATE trainings
            SET enabled = false
            WHERE date >= ?1
              AND date <= ?2
              AND (?3 IS NULL OR start_time >= ?3)
              AND (?4 IS NULL OR start_time <= ?4)
          ",
        )
        .bind(training_response.date_from)
        .bind(training_response.date_to)
        .bind(training_response.time_from)
        .bind(training_response.time_to)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_training(&self, training: Training) -> Result<(), sqlx::Error> {
        // pub date: NaiveDate,
        //    pub start_time: NaiveTime,
        //    pub end_time: NaiveTime,
        //    pub capacity: u32,
        //    pub enabled: bool,
        sqlx::query("UPDATE trainings SET date = $1, start_time = $2, end_time = $3, enabled= $4 WHERE id = $5")
            .bind(training.date)
            .bind(training.start_time)
            .bind(training.end_time)
            .bind(training.enabled)
            .bind(training.id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use super::TrainingRepo;
    use crate::models::Training;

    async fn slot(repo: &TrainingRepo, date: NaiveDate, hour: u32, enabled: bool) -> Uuid {
        let id = Uuid::new_v4();
        repo.create(Training {
            id,
            date,
            start_time: NaiveTime::from_hms_opt(hour, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(hour + 1, 0, 0).unwrap(),
            capacity: 1,
            enabled,
        })
        .await
        .unwrap();
        id
    }

    #[tokio::test]
    async fn all_slots_by_date_include_disabled_and_booked() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let repo = TrainingRepo::new(pool.clone());
        let day = NaiveDate::from_ymd_opt(2030, 1, 10).unwrap();

        let booked = slot(&repo, day, 10, true).await;
        let disabled = slot(&repo, day, 8, false).await;
        let _free = slot(&repo, day, 9, true).await;
        let _other_day = slot(&repo, day.succ_opt().unwrap(), 9, true).await;

        sqlx::query("insert into users (id, full_name, username, first_name, last_name, is_trainer) values ('42', 'Иван', 'ivan', '', '', false)")
            .execute(&pool).await.unwrap();
        sqlx::query("insert into registrations (id, user_id, training_id) values ($1, '42', $2)")
            .bind(Uuid::new_v4())
            .bind(booked)
            .execute(&pool).await.unwrap();

        let slots = repo.get_all_slots_by_date(day).await.unwrap();
        let hours: Vec<_> = slots.iter().map(|s| s.start_time.format("%H").to_string()).collect();
        assert_eq!(hours, ["08", "09", "10"], "только слоты этого дня, по возрастанию времени");

        assert_eq!(slots[0].id, disabled);
        assert!(!slots[0].enabled);
        assert_eq!(slots[1].full_name, None);
        assert_eq!(slots[2].id, booked);
        assert_eq!(slots[2].user_id.as_deref(), Some("42"));
        assert_eq!(slots[2].full_name.as_deref(), Some("Иван"));
        assert_eq!(slots[2].username.as_deref(), Some("ivan"));
    }
}
