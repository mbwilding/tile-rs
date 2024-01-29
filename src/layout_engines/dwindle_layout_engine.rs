use crate::layout_engines::LayoutEngine;
use crate::window_location::WindowLocation;
use crate::window_state::WindowState;
use crate::windows_window::WindowsWindow;

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct DwindleLayoutEngine {
    num_in_primary: i32,
    primary_percent: f64,
    primary_percent_increment: f64,
    num_in_primary_offset: i32,
    primary_percent_offset: f64,
    name: String,
}

impl DwindleLayoutEngine {
    pub fn new() -> DwindleLayoutEngine {
        DwindleLayoutEngine {
            num_in_primary: 1,
            primary_percent: 0.5,
            primary_percent_increment: 0.03,
            num_in_primary_offset: 0,
            primary_percent_offset: 0.0,
            name: "dwindle".to_string(),
        }
    }

    pub fn get_num_in_primary(&self) -> i32 {
        self.num_in_primary + self.num_in_primary_offset
    }
}

impl LayoutEngine for DwindleLayoutEngine {
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
        let num_windows = windows.len() as i32;

        if num_windows == 0 {
            return list;
        }

        let num_in_primary = std::cmp::min(self.get_num_in_primary(), num_windows);
        let primary_width =
            (space_width as f64 * (self.primary_percent + self.primary_percent_offset)) as i32;
        let primary_height = space_height / num_in_primary;
        let _height = space_height / std::cmp::max(num_windows - num_in_primary, 1);

        let primary_width = if num_in_primary >= num_windows {
            space_width
        } else {
            primary_width
        };

        let secondary_width = space_width - primary_width;

        let mut cur_orientation = Orientation::Vertical;
        let mut cur_width = secondary_width;
        let mut cur_top = 0;
        let mut cur_left = primary_width;
        let mut cur_height = if num_windows > 2 {
            space_height / 2
        } else {
            space_height
        };

        for i in 0..num_windows {
            if i < num_in_primary {
                list.push(WindowLocation::new(
                    0,
                    i * primary_height,
                    primary_width,
                    primary_height,
                    WindowState::Normal,
                ));
            } else {
                list.push(WindowLocation::new(
                    cur_left,
                    cur_top,
                    cur_width,
                    cur_height,
                    WindowState::Normal,
                ));
                match cur_orientation {
                    Orientation::Vertical => {
                        cur_top += cur_height;
                        if i < num_windows - 2 {
                            cur_width /= 2;
                        }
                        cur_orientation = Orientation::Horizontal;
                    }
                    Orientation::Horizontal => {
                        cur_left += cur_width;
                        if i < num_windows - 2 {
                            cur_height /= 2;
                        }
                        cur_orientation = Orientation::Vertical;
                    }
                }
            }
        }

        list
    }

    fn shrink_primary_area(&mut self) {
        self.primary_percent_offset -= self.primary_percent_increment;
    }

    fn expand_primary_area(&mut self) {
        self.primary_percent_offset += self.primary_percent_increment;
    }

    fn reset_primary_area(&mut self) {
        self.primary_percent_offset = 0.0;
    }

    fn increment_num_in_primary(&mut self) {
        self.num_in_primary_offset += 1;
    }

    fn decrement_num_in_primary(&mut self) {
        if self.get_num_in_primary() > 1 {
            self.num_in_primary_offset -= 1;
        }
    }
}
