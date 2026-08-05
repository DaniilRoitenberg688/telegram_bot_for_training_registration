use std::sync::Arc;

use crate::commands::Command;
use crate::models::User;
use crate::service::user::UserService;
use crate::states::State;
use crate::types::{MyResult, MyDialogue};
use teloxide::prelude::*;
use crate::keyboards::{user_reply_keyboard, trainer_reply_keyboard};

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
                    bot.send_message(
                        msg.chat.id,
                        "Здравствуйте, босс! Чем могу помочь?"
                    ).reply_markup(trainer_reply_keyboard())
                    .await?;
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!("Здравствуйте, {}! Чем могу помочь?", user.full_name),
                    ).reply_markup(user_reply_keyboard())
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
                            bot.send_message(msg.chat.id, "Для вас успешно создан аккаунт админа!").reply_markup(trainer_reply_keyboard())
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
                        "Привет! Я бот для записи на тренировки. Введите свое ФИО:",
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
                bot.send_message(msg.chat.id, "Регистрация прошла успешно!").reply_markup(user_reply_keyboard())
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
    }
    dialogue.update(State::Default).await?;
    Ok(())
}
