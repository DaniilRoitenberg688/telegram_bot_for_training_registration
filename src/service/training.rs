use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use chrono_tz::Europe::Moscow;
use uuid::Uuid;

use crate::{
    api::schemes::UpdateTrainingIn,
    models::{CancelResponse, Registration, RegistrationFullInfo, Training, TrainerSlot},
    repo::{cancel::CancelRepo, registration::RegistrationRepo, training::TrainingRepo},
    service::errors::ServiceError,
};

pub struct TrainingService {
    repo: TrainingRepo,
    registration_repo: RegistrationRepo,
    cancel_repo: CancelRepo,
}

impl TrainingService {
    pub fn new(
        repo: TrainingRepo,
        registration_repo: RegistrationRepo,
        cancel_repo: CancelRepo,
    ) -> Self {
        TrainingService {
            repo,
            registration_repo,
            cancel_repo,
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<Training, ServiceError> {
        let training = match self.repo.get_by_id(id).await {
            Ok(t) => t,
            Err(sqlx::Error::RowNotFound) => return Err(ServiceError::NotFound),
            Err(e) => return Err(e.into()),
        };
        Ok(training)
    }

    pub async fn every_day_create(&self) -> Result<(), ServiceError> {
        let trainings = self
            .repo
            .get_between_dates(
                Some(Utc::now().with_timezone(&Moscow).date_naive()),
                None,
                false,
            )
            .await?;
        println!("{:?}", trainings.len());
        let (start, days, last_day) = match trainings.last() {
            Some(t) => (1, 30 - trainings.len(), t.date),
            _ => (0, 30, Utc::now().with_timezone(&Moscow).date_naive()),
        };
        for i in start..days {
            let date = last_day + Duration::days(i as i64);
            for j in 7..=22 {
                let start_time = NaiveTime::from_hms_opt(j, 0, 0).unwrap_or_default();
                let end_time = NaiveTime::from_hms_opt(j + 1, 0, 0).unwrap_or_default();
                let training = Training {
                    id: Uuid::new_v4(),
                    date,
                    start_time,
                    end_time,
                    capacity: 1,
                    enabled: true,
                };
                self.repo.create(training).await?;
            }
        }
        Ok(())
    }

    pub async fn get_week_traings(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        enabled: bool,
    ) -> Result<Vec<Training>, ServiceError> {
        // Берём все слоты (без GROUP BY): сгруппированный запрос возвращал по одной
        // случайной строке на день, и если она была отключена — пропадал весь день,
        // хотя другие слоты в нём ещё открыты. Сначала фильтруем, потом оставляем
        // по одному слоту на день.
        let mut trainings = self
            .repo
            .get_between_dates(Some(from), Some(to), true)
            .await?;
        let now = Utc::now().with_timezone(&Moscow);
        trainings.retain(|t| t.date >= now.date_naive() && t.enabled == enabled);
        trainings.sort_by_key(|t| (t.date, t.start_time));
        trainings.dedup_by_key(|t| t.date);
        Ok(trainings)
    }

    pub async fn get_training_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<Training>, ServiceError> {
        let mut trainings = self.repo.get_by_date_without_registration(date).await?;
        let now = Utc::now().with_timezone(&Moscow);
        trainings.retain(|t| {
            t.date > now.date_naive() || (t.start_time >= now.time() && t.date == now.date_naive())
        });
        Ok(trainings)
    }

    pub async fn register_to_training(
        &self,
        user_id: String,
        training_id: Uuid,
    ) -> Result<(), ServiceError> {
        let _training = match self.repo.get_by_id(training_id).await {
            Ok(t) => t,
            Err(sqlx::Error::RowNotFound) => return Err(ServiceError::NotFound),
            Err(e) => return Err(ServiceError::Sql(e)),
        };
        let Err(sqlx::Error::RowNotFound) =
            self.registration_repo.get_by_training_id(training_id).await
        else {
            return Err(ServiceError::Forbidden);
        };
        let registration = Registration {
            id: Uuid::new_v4(),
            user_id,
            training_id,
        };
        self.registration_repo.create(registration).await?;
        Ok(())
    }

    pub async fn get_registered_trainings_for_user(
        &self,
        user_id: String,
    ) -> Result<Vec<Training>, ServiceError> {
        let mut trainings = self.repo.get_registered_trainings_for_user(user_id).await?;
        let now = Utc::now().with_timezone(&Moscow);
        trainings.retain(|t| {
            t.date > now.date_naive() || (t.end_time > now.time() && t.date == now.date_naive())
        });
        trainings.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.start_time.cmp(&b.start_time))
        });
        Ok(trainings)
    }

    pub async fn cancel_training(
        &self,
        training_id: Uuid,
        user_id: String,
    ) -> Result<(), ServiceError> {
        self.registration_repo.delete(user_id, training_id).await?;
        Ok(())
    }

    pub async fn get_trainings_for_trainer_betweeen_dates(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Training>, ServiceError> {
        let mut trainings = self.repo.get_trainings_with_registration().await?;
        let now = Utc::now().with_timezone(&Moscow);
        trainings.retain(|t| t.date >= now.date_naive() && t.date >= from && t.date <= to);
        trainings.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(trainings)
    }

    pub async fn get_weeks_with_trainings(
        &self,
        weeks: Vec<Vec<NaiveDate>>,
    ) -> Result<Vec<Vec<NaiveDate>>, ServiceError> {
        let trainings = self.repo.get_trainings_with_registration().await?;
        let mut needed_weeks = vec![];
        for week in weeks {
            let trainings_in_week: Vec<_> = trainings
                .iter()
                .filter(|t| t.date >= week[0] && t.date <= week[1])
                .collect();
            if !trainings_in_week.is_empty() {
                needed_weeks.push(week);
            }
        }
        Ok(needed_weeks)
    }

    pub async fn get_trainings_by_date_with_registration(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<RegistrationFullInfo>, ServiceError> {
        let mut trainings = self
            .repo
            .get_trainings_with_registration_by_date(date)
            .await?;
        trainings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        Ok(trainings)
    }

    pub async fn get_all_slots_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainerSlot>, ServiceError> {
        Ok(self.repo.get_all_slots_by_date(date).await?)
    }

    pub async fn create_cancel_response(
        &self,
        message: String,
    ) -> Result<CancelResponse, ServiceError> {
        let now = Utc::now().with_timezone(&Moscow);
        let req = self
            .cancel_repo
            .construct_ai_request(message, now.date_naive(), now.time());
        let resp = self.cancel_repo.send_request_to_ai(req).await?;
        Ok(resp)
    }

    pub async fn cancel_trainings(
        &self,
        cancel_response: CancelResponse,
    ) -> Result<(), ServiceError> {
        self.repo.cancel_training(cancel_response).await?;
        Ok(())
    }

    pub async fn update_training(
        &self,
        id: Uuid,
        training_data: UpdateTrainingIn,
    ) -> Result<(), ServiceError> {
        let mut training = match self.repo.get_by_id(id).await {
            Ok(training) => training,
            Err(sqlx::Error::RowNotFound) => {
                return Err(ServiceError::NotFound);
            }
            Err(e) => {
                eprintln!("cannot get training by id: {e}");
                return Err(ServiceError::Sql(e));
            }
        };
        training.enabled = training_data.enabled;
        let _ = self.repo.update_training(training).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveTime, Utc};
    use chrono_tz::Europe::Moscow;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use super::TrainingService;
    use crate::{
        models::Training,
        repo::{cancel::CancelRepo, registration::RegistrationRepo, training::TrainingRepo},
    };

    #[tokio::test]
    async fn day_stays_visible_while_any_slot_is_enabled() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let service = TrainingService::new(
            TrainingRepo::new(pool.clone()),
            RegistrationRepo::new(pool.clone()),
            CancelRepo { promt: String::new(), url: String::new(), key: String::new() },
        );
        let today = Utc::now().with_timezone(&Moscow).date_naive();
        let partly_closed = today + Duration::days(2);
        let fully_closed = today + Duration::days(3);

        // Как в реальном расписании: слоты 7:00–22:00. В первом дне тренер закрыл всё до 19:00
        // (включая первый слот, который раньше и решал судьбу дня), во втором — весь день.
        for (date, open_from) in [(partly_closed, 19), (fully_closed, 24)] {
            for hour in 7..=22 {
                let repo = TrainingRepo::new(pool.clone());
                repo.create(Training {
                    id: Uuid::new_v4(),
                    date,
                    start_time: NaiveTime::from_hms_opt(hour, 0, 0).unwrap(),
                    end_time: NaiveTime::from_hms_opt(hour + 1, 0, 0).unwrap(),
                    capacity: 1,
                    enabled: hour >= open_from,
                })
                .await
                .unwrap();
            }
        }

        let days = service
            .get_week_traings(today, today + Duration::days(6), true)
            .await
            .unwrap();
        let dates: Vec<_> = days.iter().map(|t| t.date).collect();
        assert_eq!(dates, [partly_closed], "день с открытыми слотами виден, полностью закрытый — нет");
        assert_eq!(days[0].start_time, NaiveTime::from_hms_opt(19, 0, 0).unwrap());
    }
}
