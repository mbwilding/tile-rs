use crate::classes::window_location::WindowLocation;
use crate::classes::window_state::WindowState;
use crate::layout_engines::LayoutEngine;
use crate::windows_window::WindowsWindow;

pub struct FullLayoutEngine {
    last_full: Option<isize>,
    name: String,
}

impl FullLayoutEngine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            last_full: None,
            name: "full".to_string(),
        }
    }

    fn get_desired_state(&self, window: &WindowsWindow, force_normal: bool) -> WindowState {
        if window.is_focused() || force_normal {
            WindowState::Normal
        } else {
            WindowState::Minimized
        }
    }
}

impl LayoutEngine for FullLayoutEngine {
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

        let no_focus = windows.iter().all(|w| !w.is_focused());

        for window in windows.iter() {
            let force_normal =
                no_focus && Some(window.handle()) == self.last_full || window.is_focused();

            if force_normal {
                self.last_full = Some(window.handle());
            }

            list.push(WindowLocation::new(
                0,
                0,
                space_width,
                space_height,
                self.get_desired_state(window, force_normal),
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
