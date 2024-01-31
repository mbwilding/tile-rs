use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum WindowOrder {
    #[default]
    NewWindowsLast,
    NewWindowsFirst,
}
