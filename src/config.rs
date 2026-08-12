use std::{env, path::PathBuf};

pub struct Config {
    pub token: String,
    pub database_url: String,
    pub trainer_ids: Vec<String>
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
        let trainer_ids: Vec<String> = trainer_ids_string.split(",").map(|x| x.to_string()).collect();
        Config { token, database_url, trainer_ids}
    } 
}
