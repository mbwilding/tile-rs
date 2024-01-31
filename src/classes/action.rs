use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize, Serialize)]
pub enum Action {
    ToggleFocusedWindowTiling,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = match self {
            Action::ToggleFocusedWindowTiling => "Toggle Focused Window Tiling",
        };
        write!(f, "{}", action)
    }
}
