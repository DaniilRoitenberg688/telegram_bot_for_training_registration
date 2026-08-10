mod commands;
mod config;
mod handlers;
mod keyboards;
mod models;
mod repo;
mod service;
mod states;
mod types;

use commands::Command;
use config::Config;
use handlers::base::{get_name, handle_commands};
use handlers::user::handle_user_text;
use states::State;
use std::{error::Error, sync::Arc};
use teloxide::{
    dispatching::{UpdateHandler, dialogue::InMemStorage},
    prelude::*,
    types::Update,
};

use crate::handlers::user::{callback_handler_choose_day, callback_handler_choose_time, callback_handler_confirm_registration};
use crate::repo::registration::RegistrationRepo;
use crate::{
    handlers::user::callback_handler_choose_week,
    repo::{training::TrainingRepo, user::UserRepo},
    service::{training::TrainingService, user::UserService},
    types::MyResult,
};

pub async fn run() -> MyResult<()> {
    let config = Config::build();
    let pool = sqlx::SqlitePool::connect(&config.database_url).await?;
    let registration_repo = RegistrationRepo::new(pool.clone());
    let user_repo = UserRepo::new(pool.clone());
    let user_service = Arc::new(UserService::new(user_repo));
    let training_repo = TrainingRepo::new(pool);
    let training_service = Arc::new(TrainingService::new(training_repo, registration_repo));
    training_service.every_day_create().await?;
    let bot = Bot::new(config.token);
    let trainer_ids = Arc::new(config.trainer_ids);
    Dispatcher::builder(bot, handler())
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            user_service,
            trainer_ids,
            training_service
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

pub fn handler() -> UpdateHandler<Box<dyn Error + Sync + Send>> {
    use dptree::case;
    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_commands),
        )
        .branch(case![State::Register].endpoint(get_name))
        .endpoint(handle_user_text);

    let callback_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<State>, State>()
        .branch(case![State::ChooseWeek].endpoint(callback_handler_choose_week))
        .branch(case![State::ChooseDay].endpoint(callback_handler_choose_day))
        .branch(case![State::ChooseTime].endpoint(callback_handler_choose_time))
        .branch(case![State::ConfirmRegistration {training}].endpoint(callback_handler_confirm_registration));

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}
