use chrono::{Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Europe::Moscow;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::models::Training;

pub const USER_REPLY_KEYBOARD_TEXT: &str = "Записаться на тренировку";
pub const USER_GET_TRAININGS_REPLY_KEYBOARD_TEXT: &str = "Посмотреть мои записи";
pub const TRAINER_REPLY_KEYBOARD_SHOW_TEXT: &str = "Посмотреть записи";
pub const TRAINER_REPLY_KEYBOARD_EDIT_TEXT: &str = "Изменить расписание";

pub fn weekday_ru(date: NaiveDate) -> &'static str {
    match date.weekday() {
        chrono::Weekday::Mon => "Понедельник",
        chrono::Weekday::Tue => "Вторник",
        chrono::Weekday::Wed => "Среда",
        chrono::Weekday::Thu => "Четверг",
        chrono::Weekday::Fri => "Пятница",
        chrono::Weekday::Sat => "Суббота",
        chrono::Weekday::Sun => "Воскресенье",
    }
}


pub fn create_back_button(callback_data: &str, data: &str) -> Vec<InlineKeyboardButton> {
    vec![InlineKeyboardButton::callback("⬅️ Назад", format!("back:{callback_data},{data}"))]
}

pub fn user_reply_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(USER_REPLY_KEYBOARD_TEXT),
        KeyboardButton::new(USER_GET_TRAININGS_REPLY_KEYBOARD_TEXT),
    ]])
    .resize_keyboard()
}

pub fn trainer_reply_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(TRAINER_REPLY_KEYBOARD_SHOW_TEXT),
    ]])
    .resize_keyboard()
}

pub fn generate_confirm_registration_inline_keyboard(previous_data: String) -> InlineKeyboardMarkup {
    let b = InlineKeyboardButton::callback("✅ Да", "hi");
    let (new_previous_data, _) = previous_data.rsplit_once("/").unwrap_or(("", ""));
    let a = create_back_button("chooseday", &format!("{}/id", new_previous_data));
    println!("{}", previous_data);
    let data: Vec<Vec<InlineKeyboardButton>> = vec![vec![b], a];
    InlineKeyboardMarkup::new(data)
}

pub fn generate_time_inline_keyboard(trainings: Vec<Training>, previous_data: String) -> InlineKeyboardMarkup {
    let mut time: Vec<Vec<InlineKeyboardButton>> = vec![create_back_button("chooseweek", &previous_data)];
    for i in (0..=trainings.len()).step_by(2) {
        let f = trainings.get(i);
        let s = trainings.get(i + 1);
        let mut line: Vec<InlineKeyboardButton> = Vec::new();
        if let Some(t) = f {
            let k = InlineKeyboardButton::callback(
                t.start_time.format("%H:%M").to_string(),
                format!("{}/{}", previous_data, t.id)
            );
            line.push(k);
        }
        if let Some(t) = s {
            let k = InlineKeyboardButton::callback(
                t.start_time.format("%H:%M").to_string(),
                format!("{}/{}", previous_data, t.id)
            );
            line.push(k);
        }
        time.push(line);
    }

    InlineKeyboardMarkup::new(time)
}

pub fn generate_days_inline_keyboard(trainings: Vec<Training>, previous_data: String) -> InlineKeyboardMarkup {
    let mut days: Vec<Vec<InlineKeyboardButton>> = vec![create_back_button("default", "")];
    for t in trainings.iter() {
        let button =
            InlineKeyboardButton::callback(t.date.format("%d.%m").to_string(), format!("{}/{}", previous_data, t.date));
        days.push(vec![button]);
    }
    InlineKeyboardMarkup::new(days)
}


pub fn get_weeks() -> Vec<Vec<NaiveDate>> {
    let today = Utc::now().with_timezone(&Moscow).date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let mut weeks: Vec<Vec<NaiveDate>> = Vec::new();
    for i in 0..4 {
        let new_monday = monday + Duration::days(i * 7);
        let sunday = monday + Duration::days(i * 7 + 6);
        weeks.push(vec![new_monday, sunday]);
    }
    weeks
}

pub fn generate_week_inline_keyboard(weeks: Vec<Vec<NaiveDate>>) -> InlineKeyboardMarkup {
    let mut weeks_keyboard = vec![];
    for week in weeks {
        let monday = week[0];
        let sunday = week[1];
        let week = format!(
            "{} — {}",
            monday.format("%d.%m"),
            sunday.format("%d.%m")
        );
        weeks_keyboard.push(vec![InlineKeyboardButton::callback(
            &week,
            monday.to_string(),
        )]);
    }
    InlineKeyboardMarkup::new(weeks_keyboard)
}

pub fn generate_cancel_training_keyboard(trainings: Vec<Training>) -> InlineKeyboardMarkup {
    let mut keyboard = vec![];
    for t in trainings {
        let b = InlineKeyboardButton::callback(
            format!(
                "{}, {} — {}",
                weekday_ru(t.date),
                t.date.format("%d.%m"),
                t.start_time.format("%H:%M")
            ),
            t.id.to_string(),
        );
        keyboard.push(vec![b]);
    }
    InlineKeyboardMarkup::new(keyboard)
}
