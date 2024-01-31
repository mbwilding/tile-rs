use crate::window_location::WindowLocation;
use crate::window_state::WindowState;
use crate::windows_window::WindowsWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    DeferWindowPos, EndDeferWindowPos, ShowWindow, HDWP, SWP_FRAMECHANGED, SWP_NOACTIVATE,
    SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SW_MINIMIZE,
    SW_SHOWMAXIMIZED, SW_SHOWNOACTIVATE,
};

#[derive(Debug)]
pub struct WindowsDeferPosHandle<'a> {
    info: HDWP,
    to_minimize: Vec<&'a WindowsWindow>,
    to_maximize: Vec<&'a WindowsWindow>,
    to_normal: Vec<&'a WindowsWindow>,
}

impl<'a> WindowsDeferPosHandle<'a> {
    #[allow(dead_code)]
    pub fn new(info: HDWP) -> Self {
        Self {
            info,
            to_minimize: vec![],
            to_maximize: vec![],
            to_normal: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn defer_window_pos(&mut self, window: &'a WindowsWindow, location: WindowLocation) {
        let mut flags =
            SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOZORDER | SWP_NOOWNERZORDER;

        match location.state {
            WindowState::Maximized => {
                self.to_maximize.push(window);
                flags |= SWP_NOMOVE | SWP_NOSIZE;
            }
            WindowState::Minimized => {
                self.to_minimize.push(window);
                flags |= SWP_NOMOVE | SWP_NOSIZE;
            }
            WindowState::Normal => {
                self.to_normal.push(window);
            }
        }

        let offset = window.offset();
        let x = location.x + offset.x;
        let y = location.y + offset.y;
        let width = location.width + offset.width;
        let height = location.height + offset.height;

        let old_location = window.location();
        if old_location.x != x
            || old_location.y != y
            || old_location.width != width
            || old_location.height != height
        {
            unsafe {
                let _ = DeferWindowPos(self.info, window.hwnd(), None, x, y, width, height, flags);
            }
        }
    }
}

impl Drop for WindowsDeferPosHandle<'_> {
    fn drop(&mut self) {
        self.to_minimize.iter().for_each(|w| unsafe {
            if !w.is_minimized() {
                ShowWindow(w.hwnd(), SW_MINIMIZE);
            }
        });
        self.to_maximize.iter().for_each(|w| unsafe {
            if !w.is_maximized() {
                ShowWindow(w.hwnd(), SW_SHOWMAXIMIZED);
            }
        });
        self.to_normal.iter().for_each(|w| unsafe {
            ShowWindow(w.hwnd(), SW_SHOWNOACTIVATE);
        });

        unsafe {
            EndDeferWindowPos(self.info).unwrap_or_else(|e| {
                log::error!("Failed to EndDeferWindowPos ({:?}): {:?}", self.info, e);
            });
        };
    }
}
