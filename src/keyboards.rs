use teloxide::types::{KeyboardButton, KeyboardMarkup};



pub const USER_REPLY_KEYBOARD_TEXT: &str = "Записаться на тренировку"; 
pub const TRAINER_REPLY_KEYBOARD_SHOW_TEXT: &str = "Посмотреть записи";
pub const TRAINER_REPLY_KEYBOARD_EDIT_TEXT: &str = "Изменить расписание";

pub fn user_reply_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(USER_REPLY_KEYBOARD_TEXT)]
    ]).resize_keyboard()
}

pub fn trainer_reply_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(TRAINER_REPLY_KEYBOARD_SHOW_TEXT), KeyboardButton::new(TRAINER_REPLY_KEYBOARD_EDIT_TEXT)]
    ]).resize_keyboard()
}
