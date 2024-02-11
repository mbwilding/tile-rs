use crate::window::Window;
use crate::windows_manager::WindowUpdateType;

pub type WindowDelegate = Window;
pub type WindowCreateDelegate = (Window, bool);
pub type WindowUpdateDelegate = (Window, WindowUpdateType);
