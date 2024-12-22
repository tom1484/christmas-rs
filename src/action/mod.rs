mod card;
mod game;
mod home;

use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use strum::Display;

pub use crate::action::{card::CardAction, game::GameAction, home::HomeAction};

#[macro_export]
macro_rules! act {
    ( $command: expr ) => {
        Action { command: $command, state: ActionState::default() }
    };
    ( $command: expr, $state: expr ) => {
        Action { command: $command, state: $state }
    };
}

pub use act;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Action {
    pub command: Command,
    pub state: ActionState,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub enum ActionState {
    #[default]
    Start,
    Repeat,
    End,
}

//// ANCHOR: action_enum
#[derive(Debug, Clone, PartialEq, Eq, Display, Deserialize)]
pub enum Command {
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
    StartGame,
    // Page actions
    Home(HomeAction),
    Game(GameAction),
    Card(CardAction),
}

impl Command {
    pub fn string(&self) -> String {
        match self {
            Self::Home(command) => command.to_string(),
            Self::Game(command) => command.to_string(),
            Self::Card(command) => command.to_string(),
            _ => self.to_string(),
        }
    }
}
