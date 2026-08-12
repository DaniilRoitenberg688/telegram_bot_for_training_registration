use chrono::{Duration, Local, NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::{
    models::{Registration, RegistrationFullInfo, Training},
    repo::{registration::RegistrationRepo, training::TrainingRepo},
    service::errors::ServiceError,
};

pub struct TrainingService {
    repo: TrainingRepo,
    registration_repo: RegistrationRepo,
}

impl TrainingService {
    pub fn new(repo: TrainingRepo, registration_repo: RegistrationRepo) -> Self {
        TrainingService {
            repo,
            registration_repo,
        }
    }

    pub async fn get(&self, id: Uuid) -> Option<Training> {
        match self.repo.get_by_id(id).await {
            Ok(t) => Some(t),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }

    pub async fn every_day_create(&self) -> Result<(), ServiceError> {
        let trainings = self
            .repo
            .get_between_dates(Some(Local::now().date_naive()), None, false)
            .await?;
        println!("{:?}", trainings.len());
        let (start, days, last_day) = match trainings.last() {
            Some(t) => (1, 30 - trainings.len(), t.date),
            _ => (0, 30, Local::now().date_naive()),
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
                    enabled: true,
                };
                self.repo.create(training).await?;
            }
        }
        Ok(())
    }

    pub async fn get_week_traings(&self, from: NaiveDate, to: NaiveDate) -> Vec<Training> {
        let mut trainings = self.repo.get_between_dates(Some(from), Some(to), false)
            .await.unwrap_or_else(|e| {
                eprintln!("cannot get trainings for user between dates: {e}");
                Vec::new()
            });
        let now = Local::now();
        trainings.retain(|t| t.date >= now.date_naive());
        trainings
    }

    pub async fn get_training_by_date(&self, date: NaiveDate) -> Vec<Training> {
        let mut trainings = self.repo.get_by_date_without_registration(date).await.unwrap_or_else(
            |e| {
                eprintln!("cannot get trainings by date: {}", e);
                Vec::new()
            }
        );
        let now = Local::now();
        trainings.retain(|t| t.date > now.date_naive() || (t.start_time >= now.time() && t.date == now.date_naive()));
        trainings
    }

    pub async fn register_to_training(
        &self,
        user_id: String,
        training_id: Uuid,
    ) -> Result<(), ServiceError> {
        let _training = self.repo.get_by_id(training_id).await?;
        let registration = Registration {
            id: Uuid::new_v4(),
            user_id,
            training_id,
        };
        self.registration_repo.create(registration).await?;
        Ok(())
    }

    pub async fn get_registered_trainings_for_user(&self, user_id: String) -> Vec<Training> {
        let mut trainings = self
            .repo
            .get_registered_trainings_for_user(user_id)
            .await
            .unwrap_or_else(|e| {
                eprintln!("cannot get trainings for user: {e}");
                Vec::new()
            });
        let now = Local::now();
        trainings.retain(|t| {
            t.date > now.date_naive() || (t.end_time > now.time() && t.date == now.date_naive())
        });
        trainings.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.start_time.cmp(&b.start_time))
        });
        trainings
    }

    pub async fn cancel_training(&self, training_id: Uuid, user_id: String) -> Result<(), ServiceError> {
        self.registration_repo.delete(user_id, training_id).await?;
        Ok(())
    }


    pub async fn get_trainings_for_trainer_betweeen_dates(&self, from: NaiveDate, to: NaiveDate) -> Vec<Training> {
        let mut trainings = self.repo.get_trainings_with_registration().await.unwrap_or_else(|e| {
            eprintln!("cannot get trainings with registrations: {e}");
            Vec::new()
        });
        let now = Local::now();
        trainings.retain(|t| {
            t.date >= now.date_naive() && t.date >= from && t.date <= to
        });
        trainings.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
        });
        trainings
    }

    pub async fn get_weeks_with_trainings(&self, weeks: Vec<Vec<NaiveDate>>) -> Vec<Vec<NaiveDate>> {
        let trainings = self.repo.get_trainings_with_registration().await.unwrap_or_else(|e| {
            eprintln!("cannot get trainings with registrations: {e}");
            Vec::new()
        });
        let mut needed_weeks = vec![];
        for week in weeks {
            let trainings_in_week: Vec<_> = trainings.iter().filter(|t| t.date >= week[0] && t.date <= week[1]).collect();
            if !trainings_in_week.is_empty() {
                needed_weeks.push(week);
            }

        };
        needed_weeks
    }

    pub async fn get_trainings_by_date_with_registration(&self, date: NaiveDate) -> Vec<RegistrationFullInfo> {
        let mut trainings = self.repo.get_trainings_with_registration_by_date(date).await.unwrap_or_else(|e| {
            eprintln!("cannot get trainings with registration by date: {e}");
            Vec::new()
        });
        trainings.sort_by(|a, b| {
            a.start_time
                .cmp(&b.start_time)
        });
        trainings
    }

}
