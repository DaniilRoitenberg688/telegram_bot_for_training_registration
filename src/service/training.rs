use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

use crate::{models::{Registration, Training}, repo::{registration::RegistrationRepo, training::TrainingRepo}, service::errors::ServiceError, types::MyResult};

pub struct TrainingService {
    repo: TrainingRepo,
    registration_repo: RegistrationRepo
}

impl TrainingService {
    pub fn new(repo: TrainingRepo, registration_repo: RegistrationRepo) -> Self {
        TrainingService { repo, registration_repo }
    }

    pub async fn get(&self, id: Uuid) -> Option<Training> {
        match self.repo.get_by_id(id).await {
            Ok(t) => Some(t),
            Err(err) => {eprintln!("{}", err); None}
        }
    }

    pub async fn every_day_create(&self) -> Result<(), ServiceError> {
        let trainings = self.repo.get_between_dates(Some(Utc::now().date_naive()), None, false).await?;
        println!("{:?}", trainings.len());
        let (start, days, last_day) = match trainings.last() {
            Some(t) => (1, 30 - trainings.len(), t.date),
            _ => (0, 30, Utc::now().date_naive())
        };
        for i in start..days {
            let date = last_day + Duration::days(i as i64);
            for j in 9..20 {
                let start_time = NaiveTime::from_hms_opt(j, 0, 0).unwrap_or_default();
                let end_time = NaiveTime::from_hms_opt(j + 1, 0, 0).unwrap_or_default();
                let training = Training {
                    id: Uuid::new_v4(),
                    date,
                    start_time,
                    end_time,
                    capacity: 1,
                    enabled: true
                };
                self.repo.create(training).await?;
            }
        }
        Ok(())
    }

    pub async fn get_week_traings(&self, from: NaiveDate, to: NaiveDate) -> Vec<Training> {
        match self.repo.get_between_dates(Some(from), Some(to), false).await {
            Ok(t) => t,
            Err(err) => {
                eprintln!("{err}");
                Vec::new()
            }
        }
    }

    pub async fn get_training_by_date(&self, date: NaiveDate) -> Vec<Training> {
        match self.repo.get_by_date(date).await {
            Ok(t) => t,
            Err(e) => {eprintln!("cannot get trainings by date: {}", e); Vec::new()}
        }
    }


    pub async fn register_to_training(&self, user_id: String, training_id: Uuid) -> Result<(), ServiceError> {
        let _training = self.repo.get_by_id(training_id).await?;
        let registration = Registration {
            id: Uuid::new_v4(),
            user_id, 
            training_id
        };
        self.registration_repo.create(registration).await?;
        Ok(())
    }

}
