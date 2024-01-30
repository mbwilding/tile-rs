use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub enum Action {
    Stop,
    Start,
}