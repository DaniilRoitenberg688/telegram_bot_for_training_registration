use std::{str::FromStr, sync::Arc};

use chrono::Utc;
use ::chrono::{Datelike, Duration, NaiveDate};
use teloxide::{
    Bot, dispatching::dialogue::GetChatId, payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters, SendMessageSetters}, requests::Requester, sugar::bot::BotMessagesExt, types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message}
};

use crate::{
    keyboards::{
        TRAINER_REPLY_KEYBOARD_EDIT_TEXT, TRAINER_REPLY_KEYBOARD_SHOW_TEXT,
        USER_REPLY_KEYBOARD_TEXT,
    }, models::Training, service::{training::TrainingService, user::UserService}, states::State, types::{MyDialogue, MyResult}
};

pub async fn handle_user_text(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    _user_service: Arc<UserService>,
    trainer_ids: Arc<Vec<String>>,
) -> MyResult<()> {
    let user_id = msg.chat.id.to_string();
    if let Some(text) = msg.text() {
        match text {
            USER_REPLY_KEYBOARD_TEXT => {
                dialogue.update(State::ChooseWeek).await?;
                bot.send_message(msg.chat.id, "Выберете неделю:")
                    .reply_markup(generate_week_inline_keyboard())
                    .await?;
            }
            TRAINER_REPLY_KEYBOARD_SHOW_TEXT if trainer_ids.contains(&user_id) => {
                bot.send_message(msg.chat.id, "Данная функция еще не работает")
                    .await?;
            }
            TRAINER_REPLY_KEYBOARD_EDIT_TEXT if trainer_ids.contains(&user_id) => {
                bot.send_message(msg.chat.id, "Данная функция еще не работает")
                    .await?;
            }
            _ => {
                bot.send_message(msg.chat.id, "Извините но я не понимаю вашей команды")
                    .await?;
            }
        }
    }
    Ok(())
}

pub async fn callback_handler_choose_week(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let CallbackQuery {
    data,
    message,
    ..
    } = q;
    if let Some(m) = message {
        if let Some(d) = data {
            let start_week = NaiveDate::from_str(&d)?;
            let end_week = start_week + Duration::days(6);
            let trainings = training_serivce.get_week_traings(start_week, end_week).await;
            bot.edit_message_text(m.chat().id, m.id(), "Выберите дату:").await?;
            bot.edit_message_reply_markup(m.chat().id, m.id()).reply_markup(generate_days_inline_keyboard(trainings)).await?;
            dialogue.update(State::ChooseDay).await?;
        } else {
            bot.edit_message_text(m.chat().id, m.id(), "Извините, что-то пошло не так").await?;
        }
    }

    Ok(())
}


pub fn generate_days_inline_keyboard(trainings: Vec<Training>) -> InlineKeyboardMarkup {
    let mut days: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for t in trainings.iter() {
        let button = InlineKeyboardButton::callback(t.date.format("%d.%m").to_string(), t.date.to_string());
        days.push(vec![button]);
    };
    InlineKeyboardMarkup::new(days)
}

pub fn generate_week_inline_keyboard() -> InlineKeyboardMarkup {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let mut weeks: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for i in 0..4 {
        let new_monday = monday + Duration::days(i * 7);
        let sunday = monday + Duration::days(i * 7 + 6);
        let week = format!(
            "{} — {}",
            new_monday.format("%d.%m"),
            sunday.format("%d.%m")
        );
        weeks.push(vec![InlineKeyboardButton::callback(
            &week,
            new_monday.to_string(),
        )]);
    }
    InlineKeyboardMarkup::new(weeks)
}
