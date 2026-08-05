use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
use std::error::Error;

use crate::states::State;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type MyResult<T> = Result<T, Box<dyn Error + Sync + Send>>;
