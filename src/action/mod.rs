mod home;
mod game;
mod card;

use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use strum::Display;

pub use crate::action::home::HomeAction;
pub use crate::action::game::GameAction;
pub use crate::action::card::CardAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize, Default)]
pub enum ActionState {
    #[default]
    Start,
    Repeat,
    End,
}

//// ANCHOR: action_enum
#[derive(Debug, Clone, PartialEq, Eq, Display, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,
    ToggleShowHelp,
    ScheduleIncrement,
    ScheduleDecrement,
    Increment(usize),
    Decrement(usize),
    CompleteInput(String),
    EnterNormal,
    EnterInsert,
    EnterProcessing,
    ExitProcessing,
    Update,
    SwitchPage,
    // Page actions
    Home(HomeAction),
    Game(GameAction),
    Card(CardAction),
}
