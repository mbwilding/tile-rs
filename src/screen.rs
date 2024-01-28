use crate::structs::Rectangle;
use crate::system_information;
use crate::system_information::multi_monitor_support;
use eframe::egui::mutex;
use std::ffi::OsStr;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, DeleteDC, EnumDisplayMonitors, GetDeviceCaps, GetMonitorInfoW, BITSPIXEL, HDC,
    HMONITOR, MONITORINFO, MONITORINFOEXW, PLANES,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;

const PRIMARY_MONITOR: isize = 0xBAADF00D;

static mut SCREENS: Option<Vec<Screen>> = None;

#[derive(Debug)]
pub struct Screen {
    bounds: Rectangle,
    primary: bool,
    device_name: String,
    hmonitor: HMONITOR,
    bit_depth: i32,
}

impl Screen {
    pub fn constructor(&mut self, monitor: isize, hdc: Option<HDC>) {
        let mut screen_dc = hdc;

        let mut bounds: RECT;
        let mut primary = false;
        let mut device_name = String::new();

        if multi_monitor_support() || monitor == PRIMARY_MONITOR {
            // Single monitor system
            bounds = system_information::virtual_screen();
            primary = true;
            device_name.push_str("DISPLAY");
        } else {
            // Multi monitor system
            let mut info = MONITORINFOEXW {
                monitorInfo: MONITORINFO {
                    cbSize: size_of::<MONITORINFOEXW>() as u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            // TODO: Call doesn't fill szDevice as in only takes mutable MonitorInfo
            unsafe { GetMonitorInfoW(HMONITOR(monitor), &mut info.monitorInfo) };

            bounds = info.monitorInfo.rcMonitor;
            primary = (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;

            device_name.push_str(&String::from_utf16_lossy(&info.szDevice));

            let pwsz_driver = OsStr::new(&device_name)
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();

            let pwsz_driver_ptr = PCWSTR(pwsz_driver.as_ptr() as *mut _);

            if hdc.is_none() {
                screen_dc = Some(unsafe { CreateDCW(pwsz_driver_ptr, None, None, None) });
            }
        }

        self.hmonitor = HMONITOR(monitor);

        let mut bit_depth = unsafe { GetDeviceCaps(hdc.unwrap(), BITSPIXEL) };
        bit_depth *= unsafe { GetDeviceCaps(hdc.unwrap(), PLANES) };

        self.bit_depth = bit_depth;

        if hdc != screen_dc {
            if let Some(screen_dc) = screen_dc {
                unsafe { DeleteDC(screen_dc) };
            }
        }
    }

    pub fn all_screens() -> Vec<Screen> {
        unsafe {
            if SCREENS.is_none() {
                if multi_monitor_support() {
                    let mut monitor_infos: Vec<MONITORINFO> = Vec::new();
                    let userdata = &mut monitor_infos as *mut _ as isize;

                    EnumDisplayMonitors(
                        None,
                        None,
                        Some(Screen::enumerate_monitors_callback),
                        LPARAM(userdata),
                    );

                    if !monitor_infos.is_empty() {
                        // self.screens = Some(screens);
                    } else {
                        // SCREENS = Some(vec![Screen {}]); // TODO: new Screen((IntPtr)PRIMARY_MONITOR)
                    }

                    println!("{:?}", monitor_infos);
                }
            }

            vec![] // TODO
        }
    }

    unsafe extern "system" fn enumerate_monitors_callback(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(userdata.0 as *mut Vec<MONITORINFO>);
        let mut monitor_info = MONITORINFO::default();
        monitor_info.cbSize = size_of::<MONITORINFO>() as u32;

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
