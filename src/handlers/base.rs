use crate::commands::Command;
use crate::handlers::{GetCallbackData, GetMessage};
use crate::handlers::admin::callback_handler_admin_choose_day;
use crate::handlers::user::{
    callback_handler_choose_day, callback_handler_choose_training_to_cancel,
    callback_handler_choose_week,
};
use crate::keyboards::{
    generate_week_inline_keyboard, get_weeks, trainer_reply_keyboard, user_reply_keyboard,
};
use crate::models::User;
use crate::service::errors::ServiceError;
use crate::service::training::TrainingService;
use crate::service::user::UserService;
use crate::states::State;
use crate::types::{MyDialogue, MyResult};
use std::str::FromStr;
use std::sync::Arc;
use teloxide::prelude::*;

pub async fn handle_commands(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    cmd: Command,
    user_service: Arc<UserService>,
    trainer_ids: Arc<Vec<String>>,
) -> MyResult<()> {
    match cmd {
        Command::Start => match user_service.get(msg.chat.id.to_string()).await {
            Some(user) => {
                dialogue.update(State::Default).await?;
                if user.is_trainer {
                    bot.send_message(msg.chat.id, "Здравствуйте, босс! Чем могу помочь?")
                        .reply_markup(trainer_reply_keyboard())
                        .await?;
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!("Здравствуйте, {}! Чем могу помочь?", user.full_name),
                    )
                    .reply_markup(user_reply_keyboard())
                    .await?;
                }
            }
            None => {
                if trainer_ids.contains(&msg.chat.id.to_string()) {
                    let user = User {
                        id: msg.chat.id.to_string(),
                        full_name: "boss trainer".to_string(),
                        username: msg.chat.username().unwrap_or_default().to_string(),
                        first_name: msg.chat.first_name().unwrap_or_default().to_string(),
                        last_name: msg.chat.last_name().unwrap_or_default().to_string(),
                        is_trainer: true,
                    };

                    match user_service.register(user).await {
                        Ok(_) => {
                            bot.send_message(msg.chat.id, "Для вас успешно создан аккаунт админа!")
                                .reply_markup(trainer_reply_keyboard())
                                .await?;
                        }
                        _ => {
                            bot.send_message(
                                msg.chat.id,
                                "Произошла ошибка при регистрации. Повторите позже",
                            )
                            .await?;
                        }
                    }
                } else {
                    dialogue.update(State::Register).await?;
                    bot.send_message(
                        msg.chat.id,
                        "Привет! Я бот для записи на тренировки. Введите свое ФИО: (это нужно для опознавания тренером вашего аккаунта)",
                    )
                    .await?;
                }
            }
        },
        Command::Help => {
            bot.send_message(msg.chat.id, "За помощью пока некуда обращаться")
                .await?;
        }
    };
    Ok(())
}

pub async fn get_name(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    user_service: Arc<UserService>,
) -> MyResult<()> {
    println!("i am registering a chemp");
    if let Some(name) = msg.text() {
        let user = User {
            id: msg.chat.id.to_string(),
            full_name: name.to_string(),
            username: msg.chat.username().unwrap_or_default().to_string(),
            first_name: msg.chat.first_name().unwrap_or_default().to_string(),
            last_name: msg.chat.last_name().unwrap_or_default().to_string(),
            is_trainer: false,
        };
        match user_service.register(user).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, "Регистрация прошла успешно!")
                    .reply_markup(user_reply_keyboard())
                    .await?;
            }
            Err(ServiceError::NotFound) => {
                bot.send_message(
                    msg.chat.id,
                    "Произошла ошибка при регистрации. У вас уже есть аккаунт",
                )
                .await?;
            }
            Err(e) => {
                eprintln!("{:?}", e);
                bot.send_message(
                    msg.chat.id,
                    "Произошла ошибка при регистрации. Повторите позже",
                )
                .await?;
            }
        }
    }
    dialogue.update(State::Default).await?;
    Ok(())
}

pub async fn callback_handler_back(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    trainer_ids: Arc<Vec<String>>,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let msg = q.get_message()?;
    let d = q.get_callback_data(msg, &bot, &dialogue).await?;
    println!("{d}");
    let (st, back_data) = d.split_once(',').unwrap_or_else(|| {
        eprintln!("cannot split data from callback on going back: {d}");
        ("", "")
    });
    let (new_query_data, _) = back_data.rsplit_once("/").unwrap_or(("", ""));
    let mut new_query = q.clone();
    new_query.data = Some(String::from_str(new_query_data).unwrap());
    if let Some(state) = st.strip_prefix("back:") {
        let s: State = state.into();
        dialogue.update(state).await?;
        match s {
            State::Default => {
                if trainer_ids.contains(&msg.chat().id.to_string()) {
                    dialogue.update(State::AdminChooseWeek).await?;
                    bot.edit_message_text(msg.chat().id, msg.id(), "Выберите неделю:")
                        .await?;
                    bot.edit_message_reply_markup(msg.chat().id, msg.id())
                        .reply_markup(generate_week_inline_keyboard(
                            training_serivce.get_weeks_with_trainings(get_weeks()).await,
                        ))
                        .await?;
                } else {
                    dialogue.update(State::ChooseWeek).await?;
                    bot.edit_message_text(msg.chat().id, msg.id(), "Выберите неделю:")
                        .await?;
                    bot.edit_message_reply_markup(msg.chat().id, msg.id())
                        .reply_markup(generate_week_inline_keyboard(get_weeks()))
                        .await?;
                }
            }
            State::ChooseWeek => {
                callback_handler_choose_week(bot, new_query, dialogue, training_serivce)
                    .await?;
            }
            State::ChooseDay => {
                callback_handler_choose_day(bot, new_query, dialogue, training_serivce).await?;
            }
            State::ShowTrainings => {
                callback_handler_choose_training_to_cancel(bot, q, dialogue, training_serivce)
                    .await?;
            }
            State::AdminChooseDay => {
                callback_handler_admin_choose_day(bot, new_query, dialogue, training_serivce)
                    .await?;
            }
            _ => {}
        }
    }

    Ok(())
}
