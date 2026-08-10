use crate::models::Training;

#[derive(Default, Debug, Clone)]
pub enum State {
    #[default]
    Default,
    Register,

    ChooseWeek,
    ChooseDay ,
    ChooseTime,
    ConfirmRegistration {training: Training}
}
