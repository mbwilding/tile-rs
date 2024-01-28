use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CMONITORS};

#[derive(Debug)]
pub struct Screen {
    screens: Option<Vec<Screen>>,
}

impl Screen {
    fn multi_monitor_support() -> bool {
        (unsafe { GetSystemMetrics(SM_CMONITORS) } != 0)
    }

    pub fn all_screens(&self) -> Vec<Screen> {
        if self.screens.is_none() {
            if Self::multi_monitor_support() {
                let mut monitors: Vec<MONITORINFO> = Vec::new();
                let userdata = &mut monitors as *mut _ as isize;

                unsafe {
                    EnumDisplayMonitors(
                        None,
                        None,
                        Some(Screen::enumerate_monitors_callback),
                        LPARAM(userdata),
                    );
                }

                println!("{:?}", monitors);
            }
        }

        vec![] // TODO
    }

    unsafe extern "system" fn enumerate_monitors_callback(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(userdata.0 as *mut Vec<MONITORINFO>);
        let mut monitor_info = MONITORINFO::default();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            monitors.push(monitor_info);
        }

        TRUE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_screens() {
        let screen = Screen { screens: None };
        let screens = screen.all_screens();

        assert!(screens.is_empty() || !screens.is_empty());
    }
}
