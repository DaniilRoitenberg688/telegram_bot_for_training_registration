use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

use crate::{models::Training, repo::training::TrainingRepo, types::MyResult};

pub struct TrainingService {
    repo: TrainingRepo,
}

impl TrainingService {
    pub fn new(repo: TrainingRepo) -> Self {
        TrainingService { repo }
    }

    pub async fn get(&self, id: Uuid) -> Option<Training> {
        match self.repo.get_by_id(id).await {
            Ok(t) => Some(t),
            Err(err) => {eprintln!("{}", err); None}
        }
    }

    pub async fn every_day_create(&self) -> MyResult<()> {
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

}
