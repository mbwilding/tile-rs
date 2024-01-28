use windows::Win32::Foundation::RECT;
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

pub fn virtual_screen() -> RECT {
    if multi_monitor_support() {
        RECT {
            left: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
            right: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
            top: unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) },
            bottom: unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) },
        }
    } else {
        let size = primary_monitor_size();

        RECT {
            left: 0,
            right: 0,
            top: size.width,
            bottom: size.height,
        }
    }
}
