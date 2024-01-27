use crate::window_location::WindowLocation;
use crate::window_state::WindowState;
use crate::windows_window::WindowsWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    EndDeferWindowPos, ShowWindow, HDWP, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOCOPYBITS,
    SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SW_MINIMIZE, SW_SHOWMAXIMIZED,
    SW_SHOWNOACTIVATE,
};

#[derive(Debug)]
pub struct WindowsDeferPosHandle {
    info: HDWP,
    to_minimize: Vec<WindowsWindow>,
    to_maximize: Vec<WindowsWindow>,
    to_normal: Vec<WindowsWindow>,
}

impl WindowsDeferPosHandle {
    pub fn new(info: HDWP) -> Self {
        Self {
            info,
            to_minimize: vec![],
            to_maximize: vec![],
            to_normal: vec![],
        }
    }

    pub fn defer_window_pos(&mut self, window: WindowsWindow, location: WindowLocation) {
        let mut flags =
            SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOZORDER | SWP_NOOWNERZORDER;

        match location.state {
            WindowState::Maximized => {
                self.to_maximize.push(window);
                flags |= flags | SWP_NOMOVE | SWP_NOSIZE;
            }
            WindowState::Minimized => {
                self.to_minimize.push(window);
                flags |= flags | SWP_NOMOVE | SWP_NOSIZE;
            }
            WindowState::Normal => {
                self.to_normal.push(window);
            }
        }

        todo!()
        // let offset = window.offset;
    }
}

impl Drop for WindowsDeferPosHandle {
    fn drop(&mut self) {
        self.to_minimize.iter().for_each(|w| unsafe {
            ShowWindow(w.handle(), SW_MINIMIZE);
        });
        self.to_maximize.iter().for_each(|w| unsafe {
            ShowWindow(w.handle(), SW_SHOWMAXIMIZED);
        });
        self.to_normal.iter().for_each(|w| unsafe {
            ShowWindow(w.handle(), SW_SHOWNOACTIVATE);
        });

        unsafe {
            EndDeferWindowPos(self.info).unwrap_or_else(|e| {
                log::error!("Failed to EndDeferWindowPos ({:?}): {:?}", self.info, e);
            });
        };
    }
}
