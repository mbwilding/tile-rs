use crate::delegates::{WindowCreateDelegate, WindowDelegate};
use crate::window::Window;
use crossbeam_channel::Receiver;

#[derive(Default)]
pub struct WorkspaceManager {
    window_manager: Option<Receiver<WindowDelegate>>,
}

impl WorkspaceManager {
    pub fn add_window_manager(&mut self, window_create_delegate: WindowCreateDelegate) {
        self.add_window(window_create_delegate.0, false, window_create_delegate.1);
    }

    pub fn add_window(&mut self, window: Window, switch_to_workspace: bool, first_create: bool) {}
}
