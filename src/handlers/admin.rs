use std::str::FromStr;
use std::sync::Arc;

use crate::keyboards::*;
use chrono::{Duration, NaiveDate};
use teloxide::prelude::*;
use teloxide::sugar::bot::BotMessagesExt;
use teloxide::types::CallbackQuery;
use teloxide::types::InlineKeyboardMarkup;

use crate::states::State;
use crate::{
    service::training::TrainingService,
    types::{MyDialogue, MyResult},
};

pub async fn callback_handler_admin_choose_day(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let CallbackQuery { data, message, .. } = q;
    if let Some(m) = message {
        if let Some(d) = data {
            let start_week_string = d.rsplit("/").next().unwrap_or("");
            let start_week = NaiveDate::from_str(start_week_string)?;
            let end_week = start_week + Duration::days(6);
            let trainings = training_serivce
                .get_trainings_for_trainer_betweeen_dates(start_week, end_week)
                .await;
            bot.edit_message_text(m.chat().id, m.id(), "Выберите дату:")
                .await?;
            bot.edit_message_reply_markup(m.chat().id, m.id())
                .reply_markup(generate_days_inline_keyboard(trainings, d))
                .await?;
            dialogue.update(State::AdminChooseDay).await?;
        } else {
            bot.edit_message_text(m.chat().id, m.id(), "Извините, что-то пошло не так")
                .await?;
            dialogue.update(State::Default).await?;
        }
    }

    Ok(())
}

pub async fn callback_show_time_admin(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    training_serivce: Arc<TrainingService>,
) -> MyResult<()> {
    let CallbackQuery { data, message, .. } = q;
    if let Some(m) = message {
        if let Some(d) = data {
            let day_string = d.rsplit("/").next().unwrap_or("");
            let day = NaiveDate::from_str(day_string)?;
            let trainings = training_serivce
                .get_trainings_by_date_with_registration(day)
                .await;
            let mut my_message = format!("<b> Записи на {}:</b>\n\n", day.format("%d.%m"));
            for t in trainings {
                let l = format!(
                    "{} — {}, @{} \n",
                    t.start_time.format("%H:%M"),
                    t.full_name,
                    t.username
                );
                my_message += &l;
            }
            bot.edit_message_text(m.chat().id, m.id(), my_message)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            bot.edit_message_reply_markup(m.chat().id, m.id())
                .reply_markup(InlineKeyboardMarkup::new(vec![create_back_button(
                    "adminchooseday",
                    &d,
                )]))
                .await?;
            dialogue.update(State::Default).await?;
        } else {
            bot.edit_message_text(m.chat().id, m.id(), "Извините, что-то пошло не так")
                .await?;
            dialogue.update(State::Default).await?;
        }
    }
    Ok(())
}
