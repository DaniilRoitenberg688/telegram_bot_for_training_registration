mod api;
mod bot;
mod commands;
mod config;
mod handlers;
mod keyboards;
mod models;
mod repo;
mod service;
mod states;
mod types;
use std::sync::Arc;

use teloxide::dispatching::Dispatcher;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::{Bot, dptree};
use tokio;

use crate::api::init::init_router;
use crate::api::state::ApiState;
use crate::bot::{create_every_day_training_task, handler, send_every_week_notification};
use crate::config::Config;
use crate::repo::cancel::CancelRepo;
use crate::repo::notification::NotificationRepo;
use crate::repo::registration::RegistrationRepo;
use crate::states::State;
use crate::{
    repo::{training::TrainingRepo, user::UserRepo},
    service::{training::TrainingService, user::UserService},
    types::MyResult,
};

pub async fn run() -> MyResult<()> {
    // INIT
    let config = Config::build();
    let pool = sqlx::SqlitePool::connect(&config.database_url).await?;
    let registration_repo = RegistrationRepo::new(pool.clone());
    let user_repo = UserRepo::new(pool.clone());
    let user_service = Arc::new(UserService::new(user_repo));
    let training_repo = TrainingRepo::new(pool.clone());
    let cancel_repo = CancelRepo::new(config.promt_path, config.ai_api_url, config.ai_key)?;
    let training_service = Arc::new(TrainingService::new(
        training_repo,
        registration_repo,
        cancel_repo,
    ));
    let notification_repo = NotificationRepo::new(pool.clone());
    let bot = Bot::new(config.token);
    let trainer_ids = Arc::new(config.trainer_ids);
    //FINISH INIT

    // spawn task for sending notifications
    let notif_task = tokio::spawn(send_every_week_notification(
        bot.clone(),
        user_service.clone(),
        notification_repo,
    ));

    // spawn task for every day training creating
    let _ = training_service.every_day_create().await;
    let creation_task = tokio::spawn(create_every_day_training_task(training_service.clone()));

    // create state for api
    let api_state = ApiState {
        user_service: user_service.clone(),
        training_service: training_service.clone(),
    };

    // spawn api task
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    let api_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, init_router(api_state)).await {
            eprintln!("API server error: {e}");
        }
    });

    // starting telegram bot
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
    println!("dispatcher stopped");

    // aborting background tasks
    notif_task.abort();
    creation_task.abort();
    api_task.abort();

    // awaiting background tasks
    let _ = notif_task.await;
    println!("notif task stopped");
    let _ = creation_task.await;
    println!("creation task stopped");
    let _ = api_task.await;
    println!("api task stopped");

    // closing db pool
    pool.close().await;
    println!("pool closed");

    Ok(())
}
