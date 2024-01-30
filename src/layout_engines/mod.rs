use crate::window_location::WindowLocation;
use crate::windows_window::WindowsWindow;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod dwindle_layout_engine;
pub mod focus_layout_engine;
pub mod full_layout_engine;
pub mod grid_layout_engine;
pub mod panel_layout_engine;
pub mod tall_layout_engine;

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutEngineType {
    Dwindle,
    Focus,
    #[default]
    Full,
    Grid,
    // Panel,
    // Tall,
}

impl LayoutEngineType {
    pub fn variants() -> [LayoutEngineType; 4] {
        [
            LayoutEngineType::Dwindle,
            LayoutEngineType::Focus,
            LayoutEngineType::Full,
            LayoutEngineType::Grid,
            // LayoutEngineType::Panel,
            // LayoutEngineType::Tall,
        ]
    }
}

impl Display for LayoutEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutEngineType::Dwindle => write!(f, "Dwindle"),
            LayoutEngineType::Focus => write!(f, "Focus"),
            LayoutEngineType::Full => write!(f, "Full"),
            LayoutEngineType::Grid => write!(f, "Grid"),
            // LayoutEngineType::Panel => write!(f, "Panel"),
            // LayoutEngineType::Tall => write!(f, "Tall"),
        }
    }
}

pub trait LayoutEngine {
    // the name of the layout engine
    fn name(&self) -> &str;

    // calculate the desired layout of the workspace
    fn calc_layout(
        &mut self,
        windows: &[&WindowsWindow],
        space_width: i32,
        space_height: i32,
    ) -> Vec<WindowLocation>;

    // shrink the primary area of the layout engine
    fn shrink_primary_area(&mut self);

    // expand the primary area of the layout engine
    fn expand_primary_area(&mut self);

    // reset the primary area of the layout engine
    fn reset_primary_area(&mut self);

    // increment the number of windows in the layout's primary area
    fn increment_num_in_primary(&mut self);

    // decrement the number of windows in the layout's primary area
    fn decrement_num_in_primary(&mut self);
}
