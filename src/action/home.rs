use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum HomeAction {
    Up,
    Down,
    Select,
}
