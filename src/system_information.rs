use crate::structs::Rectangle;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CMONITORS, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

#[derive(Debug)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

pub fn multi_monitor_support() -> bool {
    (unsafe { GetSystemMetrics(SM_CMONITORS) } != 0)
}

pub fn primary_monitor_size() -> Size {
    Size {
        width: unsafe { GetSystemMetrics(SM_CXSCREEN) },
        height: unsafe { GetSystemMetrics(SM_CYSCREEN) },
    }
}

pub fn virtual_screen() -> Rectangle {
    if multi_monitor_support() {
        Rectangle {
            x: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
            y: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
            width: unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) },
            height: unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) },
        }
    } else {
        let size = primary_monitor_size();

        Rectangle {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        }
    }
}
