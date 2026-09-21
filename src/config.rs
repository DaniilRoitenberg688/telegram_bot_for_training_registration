use std::{env, path::PathBuf};

pub struct Config {
    pub token: String,
    pub database_url: String,
    pub trainer_ids: Vec<String>,
    pub ai_key: String,
    pub ai_api_url: String,
    pub promt_path: String,
}

impl Config {
    pub fn build() -> Config {
        dotenvy::dotenv().unwrap_or_else(|e| {
            eprintln!("cannot load .env file: {e}");
            PathBuf::new()
        });
        let token = env::var("TOKEN").expect("TOKEN variable must be set");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL variable must be set");
        let trainer_ids_string = env::var("TRAINER_IDS").expect("TRAINER_IDS variable must be set");
        let ai_key = env::var("AI_KEY").expect("AI_KEY variable must be set");
        let ai_api_url = env::var("AI_API_URL").expect("AI_API_URL variable must be set");
        let promt_path = env::var("PROMPT_PATH").expect("PROMPT_PATH variable must be set");

        let trainer_ids: Vec<String> = trainer_ids_string
            .split(",")
            .map(|x| x.to_string())
            .collect();
        Config {
            token,
            database_url,
            trainer_ids,
            ai_key,
            ai_api_url,
            promt_path,
        }
    }
}
