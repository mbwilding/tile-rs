use crate::layout_engines::LayoutEngine;
use crate::window_location::WindowLocation;
use crate::window_state::WindowState;
use crate::windows_window::WindowsWindow;

pub struct GridLayoutEngine {
    name: String,
}

impl GridLayoutEngine {
    pub fn new() -> Self {
        Self {
            name: "full".to_string(),
        }
    }
}

impl LayoutEngine for GridLayoutEngine {
    fn name(&self) -> &str {
        &self.name
    }

    fn calc_layout(
        &mut self,
        windows: &[&WindowsWindow],
        space_width: i32,
        space_height: i32,
    ) -> Vec<WindowLocation> {
        let mut list = Vec::new();
        let num_windows = windows.len();

        if num_windows == 0 {
            return list;
        }

        let grid_width = (num_windows as f64).sqrt().ceil() as i32;
        let grid_height = (num_windows as f64 / grid_width as f64).ceil() as i32;

        let width = space_width / grid_width;
        let height = space_height / grid_height;

        for i in 0..windows.len() {
            let i = i as i32;

            let x = i / grid_width * width;
            let y = i % grid_height * height;

            list.push(WindowLocation::new(
                x,
                y,
                width,
                height,
                WindowState::Normal,
            ));
        }

        list
    }

    fn shrink_primary_area(&mut self) {}
    fn expand_primary_area(&mut self) {}
    fn reset_primary_area(&mut self) {}
    fn increment_num_in_primary(&mut self) {}
    fn decrement_num_in_primary(&mut self) {}
}
