use crate::models::Training;

#[derive(Default, Debug, Clone)]
pub enum State {
    #[default]
    Default,
    Register,

    ChooseWeek,
    ChooseDay ,
    ChooseTime,
    ConfirmRegistration {training: Training},

    ShowTrainings,
    ChooseCancelTraining,
    ConfirmCancelTraining {training: Training},

    AdminChooseWeek,
    AdminChooseDay,
    AdminShowTrainings,
}


impl From<&str> for State {
    fn from(value: &str) -> Self {
       match value {
            "default" => Self::Default,
            "register" => Self::Register,
            "chooseweek" => Self::ChooseWeek,
            "chooseday" => Self::ChooseDay,
            "choosetime" => Self::ChooseTime,
            "showtrainings" => Self::ShowTrainings,
            "choosecanceltraining" => Self::ChooseCancelTraining,
            _ => Self::Default
        } 
    }
}
