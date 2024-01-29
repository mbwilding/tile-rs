use crate::window_location::WindowLocation;
use crate::window_state::WindowState;
use crate::windows_window::WindowsWindow;

pub struct FocusLayoutEngine {
    num_in_primary: i32,
    primary_percent: f64,
    primary_percent_increment: f64,
    num_in_primary_offset: i32,
    primary_percent_offset: f64,
    name: String, // Assuming name needs to be mutable
}

impl FocusLayoutEngine {
    pub fn new() -> FocusLayoutEngine {
        FocusLayoutEngine {
            num_in_primary: 1,
            primary_percent: 0.7,
            primary_percent_increment: 0.03,
            num_in_primary_offset: 0,
            primary_percent_offset: 0.0,
            name: "focus".to_string(),
        }
    }

    pub fn with_params(
        num_in_primary: i32,
        primary_percent: f64,
        primary_percent_increment: f64,
    ) -> FocusLayoutEngine {
        FocusLayoutEngine {
            num_in_primary,
            primary_percent,
            primary_percent_increment,
            num_in_primary_offset: 0,
            primary_percent_offset: 0.0,
            name: "focus".to_string(),
        }
    }

    pub fn calc_layout(
        &self,
        windows: &Vec<&WindowsWindow>,
        space_width: i32,
        space_height: i32,
    ) -> Vec<WindowLocation> {
        let mut list = Vec::new();
        let num_windows = windows.len() as i32;

        if num_windows == 0 {
            return list;
        }

        let num_in_primary = std::cmp::min(self.get_num_in_primary(), num_windows);
        let nb_left_windows = self.get_nb_left_windows(num_windows, num_in_primary);
        let nb_right_windows = self.get_nb_right_windows(num_windows, num_in_primary);

        let mut primary_width =
            (space_width as f64 * (self.primary_percent + self.primary_percent_offset)) as i32;
        let mut secondary_width = (space_width - primary_width) / 2;

        let primary_height = space_height / num_in_primary;
        let left_height = self.get_secondary_height(space_height, nb_left_windows);
        let right_height = self.get_secondary_height(space_height, nb_right_windows);

        if num_in_primary >= num_windows {
            primary_width = space_width;
            secondary_width = 0;
        } else if nb_right_windows == 0 {
            primary_width += secondary_width;
        }

        for i in 0..num_windows {
            if i < num_in_primary {
                list.push(WindowLocation::new(
                    secondary_width,
                    i * primary_height,
                    primary_width,
                    primary_height,
                    WindowState::Normal,
                ));
            } else if i < nb_left_windows + num_in_primary {
                // left side
                list.push(WindowLocation::new(
                    0,
                    (i - num_in_primary) * left_height,
                    secondary_width,
                    left_height,
                    WindowState::Normal,
                ));
            } else {
                // right side
                list.push(WindowLocation::new(
                    secondary_width + primary_width,
                    (i - num_in_primary - nb_left_windows) * right_height,
                    secondary_width,
                    right_height,
                    WindowState::Normal,
                ));
            }
        }
        list
    }

    fn get_nb_left_windows(&self, num_windows: i32, num_in_primary: i32) -> i32 {
        (num_windows - num_in_primary + 1) / 2
    }

    fn get_nb_right_windows(&self, num_windows: i32, num_in_primary: i32) -> i32 {
        (num_windows - num_in_primary) / 2
    }

    fn get_secondary_height(&self, space_height: i32, nb_windows: i32) -> i32 {
        space_height / std::cmp::max(nb_windows, 1)
    }

    pub fn shrink_primary_area(&mut self) {
        self.primary_percent_offset -= self.primary_percent_increment;
    }

    pub fn expand_primary_area(&mut self) {
        self.primary_percent_offset += self.primary_percent_increment;
    }

    pub fn reset_primary_area(&mut self) {
        self.primary_percent_offset = 0.0;
    }

    pub fn increment_num_in_primary(&mut self) {
        self.num_in_primary_offset += 1;
    }

    pub fn decrement_num_in_primary(&mut self) {
        if self.get_num_in_primary() > 1 {
            self.num_in_primary_offset -= 1;
        }
    }

    fn get_num_in_primary(&self) -> i32 {
        self.num_in_primary + self.num_in_primary_offset
    }
}
