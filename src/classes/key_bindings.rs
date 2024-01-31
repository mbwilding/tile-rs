use crate::classes::action::Action;
use crate::classes::keys::{Keys, VirtualKey};
use std::collections::HashMap;

pub fn default_key_bindings() -> HashMap<Action, Keys> {
    HashMap::from([(
        Action::ToggleFocusedWindowTiling,
        Keys {
            alt: true,
            key: VirtualKey::T,
            ..Default::default()
        },
    )])
}
