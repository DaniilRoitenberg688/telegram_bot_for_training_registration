pub mod base;
pub mod user;
pub mod admin;


use teloxide::{Bot, requests::Requester, types::{CallbackQuery, MaybeInaccessibleMessage}};

use crate::{states::State, types::{MyDialogue, MyResult}};

pub trait GetMessage {
    fn get_message(&self) -> MyResult<&MaybeInaccessibleMessage>;
}

pub trait GetCallbackData {
    async fn get_callback_data(
        &self,
        message: &MaybeInaccessibleMessage,
        bot: &Bot,
        dialogue: &MyDialogue,
    ) -> MyResult<&String>;
}

impl GetMessage for CallbackQuery {
    fn get_message(&self) -> MyResult<&MaybeInaccessibleMessage> {
        let CallbackQuery { message, .. } = self;
        match message {
            Some(m) => Ok(m),
            None => {
                eprintln!("cannot get message");
                Err(Box::from("cannot get message"))
            }
        }
    }
}

impl GetCallbackData for CallbackQuery {
    async fn get_callback_data(
        &self,
        message: &MaybeInaccessibleMessage,
        bot: &Bot,
        dialogue: &MyDialogue,
    ) -> MyResult<&String> {
        let CallbackQuery { data, .. } = self;
        match data {
            Some(d) => Ok(d),
            None => {
                eprintln!("cannot get callback data");
                bot.edit_message_text(
                    message.chat().id,
                    message.id(),
                    "Извините, что-то пошло не так.",
                )
                .await?;
                dialogue.update(State::Default).await?;
                Err(Box::from("cannot get callback data"))
            }
        }
    }
}
