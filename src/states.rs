use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone)]
pub enum State {
    #[default]
    Default,
    Register,

    ChooseWeek,
    ChooseDay
}
