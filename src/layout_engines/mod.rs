use crate::window_location::WindowLocation;
use crate::windows_window::WindowsWindow;

mod dwindle_layout_engine;
mod focus_layout_engine;
mod full_layout_engine;
mod grid_layout_engine;
mod panel_layout_engine;
mod tall_layout_engine;

pub trait LayoutEngine {
    // the name of the layout engine
    fn name(&self) -> &str;

    // calculate the desired layout of the workspace
    fn calc_layout(
        &mut self,
        windows: &[WindowsWindow],
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
