use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CMONITORS};

#[derive(Debug)]
pub struct Monitor {
    pub index: u32,
}

impl Monitor {
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}
