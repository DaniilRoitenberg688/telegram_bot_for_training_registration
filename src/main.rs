use std::process;
use tg_trainer::run;



#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("An error occured while running telegram bot: {:#}", err);
        process::exit(1)
    });
    println!("everything ended")

}
