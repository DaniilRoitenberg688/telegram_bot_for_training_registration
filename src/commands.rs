use teloxide::utils::command::BotCommands;


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "начать регистрацию")]
    Start,
    #[command(description = "помощь")]
    Help,
}






