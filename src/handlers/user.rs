use ::chrono::{Duration, NaiveDate};
use std::fmt::Write;
use std::{str::FromStr, sync::Arc};
use teloxide::types::{ParseMode};
use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use uuid::Uuid;

use crate::handlers::{GetCallbackData, GetMessage};
use crate::keyboards::*;
use crate::{
    keyboards::{
        TRAINER_REPLY_KEYBOARD_EDIT_TEXT, TRAINER_REPLY_KEYBOARD_SHOW_TEXT,
        USER_GET_TRAININGS_REPLY_KEYBOARD_TEXT, USER_REPLY_KEYBOARD_TEXT,
    },
    models::Training,
    service::{training::TrainingService, user::UserService},
    states::State,
    types::{MyDialogue, MyResult},
};

pub async fn handle_user_text(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    _user_service: Arc<UserService>,
    training_serivce: Arc<TrainingService>,
    trainer_ids: Arc<Vec<String>>,
) -> MyResult<()> {
    let user_id = msg.chat.id.to_string();
    if let Some(text) = msg.text() {
        match text {
            USER_REPLY_KEYBOARD_TEXT if !trainer_ids.contains(&user_id) => {
                dialogue.update(State::ChooseWeek).await?;
                bot.send_message(msg.chat.id, "Выберите неделю:")
                    .reply_markup(generate_week_inline_keyboard(get_weeks()))
                    .await?;
            }
            USER_GET_TRAININGS_REPLY_KEYBOARD_TEXT if !trainer_ids.contains(&user_id) => {
                println!("{}", msg.chat.id);
                let user_trainings = training_serivce
                    .get_registered_trainings_for_user(msg.chat.id.to_string())
                    .await;
                println!("{:?}", user_trainings);
                if user_trainings.is_empty() {
                    bot.send_message(msg.chat.id, "У вас пока нет записей")
                        .await?;
                } else {
                    let mut message = String::from("<b>Ваши записи:</b>\n\n");
                    for training in user_trainings {
                        let _ = write!(
                            message,
                            "🥊 {}, {} — {} \n\n",
                            weekday_ru(training.date),
                            training.date.format("%d.%m"),
                            training.start_time.format("%H:%M")
                        );
                    }
                    bot.send_message(msg.chat.id, message)
                        .reply_markup(InlineKeyboardMarkup::new(vec![vec![
                            InlineKeyboardButton::callback("❌ Отменить запись", "otm"),
                        ]]))
                        .parse_mode(ParseMode::Html)
                        .await?;
                    dialogue.update(State::ShowTrainings).await?;
                }
            }
            TRAINER_REPLY_KEYBOARD_SHOW_TEXT if trainer_ids.contains(&user_id) => {
                dialogue.update(State::AdminChooseWeek).await?;
                bot.send_message(msg.chat.id, "Выберите неделю:")
                    .reply_markup(generate_week_inline_keyboard(
                        training_serivce.get_weeks_with_trainings(get_weeks()).await,
                    ))
                    .await?;
            }
            TRAINER_REPLY_KEYBOARD_EDIT_TEXT if trainer_ids.contains(&user_id) => {
                bot.send_message(msg.chat.id, "Данная функция еще не работает")
                    .await?;
            }
            _ => {
                println!("cannot understand message");
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
    let m = q.get_message()?;
    let d = q.get_callback_data(m, &bot, &dialogue).await?;
    let start_week = NaiveDate::from_str(d)?;
    let end_week = start_week + Duration::days(6);
    let trainings = training_serivce
        .get_week_traings(start_week, end_week)
        .await;
    bot.edit_message_text(m.chat().id, m.id(), "Выберите дату:")
        .await?;
    bot.edit_message_reply_markup(m.chat().id, m.id())
        .reply_markup(generate_days_inline_keyboard(trainings, d.to_string()))
        .await?;
    dialogue.update(State::ChooseDay).await?;

    Ok(())
}

pub async fn callback_handler_choose_day(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let m = q.get_message()?;
    let d = q.get_callback_data(m, &bot, &dialogue).await?;
    let day_string = d.rsplit("/").next().unwrap_or("");
    let day = NaiveDate::from_str(day_string)?;
    let trainings = training_serivce.get_training_by_date(day).await;
    bot.edit_message_text(m.chat().id, m.id(), "Выберите время:")
        .await?;
    bot.edit_message_reply_markup(m.chat().id, m.id())
        .reply_markup(generate_time_inline_keyboard(trainings, d.to_string()))
        .await?;
    dialogue.update(State::ChooseTime).await?;
    Ok(())
}

pub async fn callback_handler_choose_time(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let m = q.get_message()?;
    let d = q.get_callback_data(m, &bot, &dialogue).await?;
    let id_string = d.rsplit("/").next().unwrap_or("");
    let id = Uuid::from_str(id_string)?;
    let training = training_serivce.get(id).await;
    match training {
        Some(t) => {
            bot.edit_message_text(
                m.chat().id,
                m.id(),
                format!(
                    "Записаться на {} в {}?",
                    t.date.format("%d.%m"),
                    t.start_time.format("%H:%M")
                ),
            )
            .await?;
            bot.edit_message_reply_markup(m.chat().id, m.id())
                .reply_markup(generate_confirm_registration_inline_keyboard(d.to_string()))
                .await?;

            dialogue
                .update(State::ConfirmRegistration { training: t })
                .await?;
        }
        None => {
            bot.edit_message_text(m.chat().id, m.id(), "Извините, что-то пошло не так")
                .await?;
        }
    }
    Ok(())
}

pub async fn callback_handler_confirm_registration(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
    training: Training,
) -> MyResult<()> {
    let m = q.get_message()?;
    let training_id = training.id;
    let user_id = m.chat().id.to_string();
    if let Err(e) = training_serivce
        .register_to_training(user_id, training_id)
        .await
    {
        eprintln!("{:?}", e)
    };
    bot.edit_message_text(m.chat().id, m.id(), "✅ Вы успешно записаны на тренировку!")
        .await?;
    dialogue.update(State::Default).await?;
    Ok(())
}

pub async fn callback_handler_choose_training_to_cancel(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let m = q.get_message()?;
    let user_trainings = training_serivce
        .get_registered_trainings_for_user(m.chat().id.to_string())
        .await;
    bot.edit_message_text(
        m.chat().id,
        m.id(),
        "❌ Выберите тренировку для отмены записи:",
    )
    .await?;
    bot.edit_message_reply_markup(m.chat().id, m.id())
        .reply_markup(generate_cancel_training_keyboard(user_trainings))
        .await?;
    dialogue.update(State::ChooseCancelTraining).await?;
    Ok(())
}

pub async fn callback_confirm_cancel_training(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_service: Arc<TrainingService>,
) -> MyResult<()> {
    let msg = q.get_message()?;
    let d = q.get_callback_data(msg, &bot, &dialogue).await?;
    let id = Uuid::from_str(d).unwrap();
    match training_service.get(id).await {
        Some(training) => {
            bot.edit_message_text(
                msg.chat().id,
                msg.id(),
                format!(
                    "Вы точно хотите отменить эту тренировку: {}, {} — {}",
                    weekday_ru(training.date),
                    training.date.format("%d.%m"),
                    training.start_time.format("%H:%M")
                ),
            )
            .await?;
            bot.edit_message_reply_markup(msg.chat().id, msg.id())
                .reply_markup(InlineKeyboardMarkup::new(vec![
                    vec![InlineKeyboardButton::callback("❌ Отменить", "cancel")],
                    create_back_button("showtrainings", "sdf"),
                ]))
                .await?;
            dialogue
                .update(State::ConfirmCancelTraining { training })
                .await?;
        }
        None => {
            bot.edit_message_text(msg.chat().id, msg.id(), "Не могу найти такую тренировку")
                .await?;
            dialogue.update(State::Default).await?;
        }
    }
    Ok(())
}

pub async fn callback_cancel_training(
    bot: Bot,
    q: CallbackQuery,
    training: Training,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let m = q.get_message()?;
    match training_serivce
        .cancel_training(training.id, m.chat().id.to_string())
        .await
    {
        Ok(_) => {
            bot.edit_message_text(m.chat().id, m.id(), "Запись успешно отменена!")
                .await?;
        }
        Err(e) => {
            bot.edit_message_text(
                m.chat().id,
                m.id(),
                "Извините, что-то пошло не так, запись отменить не удалось",
            )
            .await?;
            eprint!("cannot cancel training: {e}");
        }
    }
    dialogue.update(State::Default).await?;
    Ok(())
}
