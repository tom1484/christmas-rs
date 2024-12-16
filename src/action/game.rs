use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

use crate::action::ActionState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum GameAction {
    Left,
    Right,
    Up,
    Down,
}
