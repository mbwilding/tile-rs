use crate::classes::window_state::WindowState;
use std::fmt::Display;

#[derive(Debug)]
pub struct WindowLocation {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub state: WindowState,
}

impl WindowLocation {
    pub fn new(x: i32, y: i32, width: i32, height: i32, state: WindowState) -> Self {
        Self {
            x,
            y,
            width,
            height,
            state,
        }
    }

    #[allow(dead_code)]
    pub fn is_point_inside(&self, x: i32, y: i32) -> bool {
        self.x <= x && x <= self.x + self.width && self.y <= y && y <= self.y + self.height
    }
}

impl Display for WindowLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}x{} | {}x{} [{:?}]",
            self.x, self.y, self.width, self.height, self.state,
        )
    }
}
