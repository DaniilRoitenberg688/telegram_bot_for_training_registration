mod commands;
mod config;
mod handlers;
mod keyboards;
mod models;
mod repo;
mod service;
mod states;
mod types;

use chrono::{Datelike, NaiveTime, Utc, Weekday};
use chrono_tz::Europe::Moscow;
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

use crate::handlers::admin::{callback_handler_admin_choose_day, callback_show_time_admin};
use crate::handlers::user::{
    callback_cancel_training, callback_confirm_cancel_training, callback_handler_choose_day, callback_handler_choose_time, callback_handler_choose_training_to_cancel, callback_handler_confirm_registration
};
use crate::repo::notification::{NotificationRepo};
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
    let training_repo = TrainingRepo::new(pool.clone());
    let training_service = Arc::new(TrainingService::new(training_repo, registration_repo));
    let notification_repo = NotificationRepo::new(pool.clone());

    let bot = Bot::new(config.token);
    let trainer_ids = Arc::new(config.trainer_ids);
    let notif_task = tokio::spawn(send_every_week_notification(
        bot.clone(),
        user_service.clone(),
        notification_repo,
    ));
    let _ = training_service.every_day_create().await;
    let creation_task = tokio::spawn(create_every_day_training_task(training_service.clone()));
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
    notif_task.abort();
    creation_task.abort();
    let _ = notif_task.await;
    println!("notif task stopped");
    let _ = creation_task.await;
    println!("creation task stopped");
    pool.close().await;
    println!("pool closed");
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
        .branch(dptree::entry().endpoint(handle_user_text));

    let callback_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<State>, State>()
        // .branch(dptree::entry().filter(|q: CallbackQuery| {
        //     q.data.as_deref().is_some_and(|d| d.starts_with("back:"))
        // }).endpoint(callback_handler_back))
        .branch(case![State::ChooseWeek].endpoint(callback_handler_choose_week))
        .branch(case![State::ChooseDay].endpoint(callback_handler_choose_day))
        .branch(case![State::ChooseTime].endpoint(callback_handler_choose_time))

        .branch(
            case![State::ConfirmRegistration { training }]
                .endpoint(callback_handler_confirm_registration),
        )
        .branch(case![State::ShowTrainings].endpoint(callback_handler_choose_training_to_cancel))
        .branch(case![State::ChooseCancelTraining].endpoint(callback_confirm_cancel_training))
        .branch(case![State::ConfirmCancelTraining { training }].endpoint(callback_cancel_training))
        .branch(case![State::AdminChooseWeek].endpoint(callback_handler_admin_choose_day))
        .branch(case![State::AdminChooseDay].endpoint(callback_show_time_admin));

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}

pub async fn send_every_week_notification(
    bot: Bot,
    user_service: Arc<UserService>,
    notif_repo: NotificationRepo,
) {
    let send_time_start = NaiveTime::from_hms_opt(18, 00, 00).unwrap();
    let send_time_end = NaiveTime::from_hms_opt(18, 20, 00).unwrap();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_mins(2)).await;
        let now = Utc::now().with_timezone(&Moscow);
        if now.weekday() == Weekday::Sun
            && now.time() >= send_time_start
            && now.time() <= send_time_end
        {
            match notif_repo.get_by_date(now.date_naive()).await {
                Ok(_) => {}
                Err(sqlx::Error::RowNotFound) => {
                    let users = user_service.get_simple_users().await;
                    for user in users.iter() {
                        if let Err(e) = bot
                            .send_message(user.id.clone(), "Добрый вечер! Запишитесь на тренировку!")
                            .await
                        {
                            eprintln!("cannot send message to user: {e}");
                        } else {
                            println!("daily message was send")
                        };
                    }
                    if let Err(e) = notif_repo.create(now.date_naive()).await {
                        eprintln!("cannot create notification: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("cannot get notification: {e}")
                }
            }
        }
    }
}

pub async fn create_every_day_training_task(training_service: Arc<TrainingService>) {
    let start_time = NaiveTime::from_hms_opt(00, 00, 00).unwrap();
    let end_time = NaiveTime::from_hms_opt(00, 10, 00).unwrap();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_mins(2)).await;
        let now = Utc::now().with_timezone(&Moscow);
        if start_time <= now.time() && end_time >= now.time() {
            match training_service.every_day_create().await {
                Ok(_) => println!("new trainings added succesfully"),
                Err(e) => eprintln!("cannot create new training {e}"),
            }
        }
    }
}
