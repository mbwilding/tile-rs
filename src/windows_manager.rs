use crate::windows_defer_pos_handle::WindowsDeferPosHandle;
use crate::windows_window::WindowsWindow;
use anyhow::Result;
use log::{error, info, trace};
use std::collections::HashMap;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY,
    EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
    EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, MSG, WH_MOUSE_LL, WINEVENT_OUTOFCONTEXT,
    WM_LBUTTONUP,
};

#[derive(Debug, Default)]
pub struct WindowsManager {
    windows: HashMap<HWND, WindowsWindow>,
    floating: HashMap<WindowsWindow, bool>,
}

impl WindowsManager {
    pub fn init(&self) {
        std::thread::spawn(|| unsafe {
            info!("Initializing hook");

            let module_handle = GetModuleHandleW(PCWSTR::null()).unwrap_or_else(|e| {
                error!("Failed GetModuleHandleW: {:?}", e);
                std::process::exit(69);
            });

            let win_event_hooks = [
                Self::register_event_hook(EVENT_OBJECT_DESTROY, EVENT_OBJECT_SHOW, module_handle),
                Self::register_event_hook(
                    EVENT_OBJECT_CLOAKED,
                    EVENT_OBJECT_UNCLOAKED,
                    module_handle,
                ),
                Self::register_event_hook(
                    EVENT_SYSTEM_MINIMIZESTART,
                    EVENT_SYSTEM_MINIMIZEEND,
                    module_handle,
                ),
                Self::register_event_hook(
                    EVENT_SYSTEM_MOVESIZESTART,
                    EVENT_SYSTEM_MOVESIZEEND,
                    module_handle,
                ),
                Self::register_event_hook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    module_handle,
                ),
                Self::register_event_hook(
                    EVENT_OBJECT_LOCATIONCHANGE,
                    EVENT_OBJECT_LOCATIONCHANGE,
                    module_handle,
                ),
            ];

            let _event_mouse =
                SetWindowsHookExW(WH_MOUSE_LL, Some(Self::mouse_callback), module_handle, 0)
                    .unwrap_or_else(|e| {
                        error!("Failed SetWindowsHookExW: {:?}", e);
                        std::process::exit(69);
                    });

            info!("Initialized hook");

            let mut message = MSG::default();

            loop {
                if GetMessageW(&mut message, HWND(0), 0, 0).as_bool() {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                } else {
                    break;
                }
            }

            UnhookWindowsHookEx(_event_mouse).unwrap_or_else(|e| {
                error!("Failed UnhookWindowsHookEx: {:?}", e);
            });

            for hooks in win_event_hooks.into_iter() {
                if !UnhookWinEvent(hooks).as_bool() {
                    error!("Failed UnhookWinEvent");
                }
            }
        });
    }

    unsafe fn register_event_hook(
        event_min: u32,
        event_max: u32,
        module_handle: HMODULE,
    ) -> HWINEVENTHOOK {
        SetWinEventHook(
            event_min,
            event_max,
            module_handle,
            Some(Self::event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    }

    unsafe extern "system" fn mouse_callback(
        code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if w_param.0 == WM_LBUTTONUP as usize {
            trace!("mouse_callback | WM_LBUTTONUP");
            // TODO: HandleWindowMoveEnd();
        }

        CallNextHookEx(None, code, w_param, l_param)
    }

    unsafe extern "system" fn event_callback(
        _h_win_event_hook: HWINEVENTHOOK,
        event_type: u32,
        window_handle: HWND,
        _id_object: i32,
        _id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        if window_handle.0 == 0 {
            // trace!("event_callback | NULL WINDOW HANDLE");
            return;
        }

        match event_type {
            EVENT_OBJECT_DESTROY => trace!("event_callback | EVENT_OBJECT_DESTROY"),
            EVENT_OBJECT_SHOW => trace!("event_callback | EVENT_OBJECT_SHOW"),
            EVENT_OBJECT_CLOAKED => trace!("event_callback | EVENT_OBJECT_CLOAKED"),
            EVENT_OBJECT_UNCLOAKED => trace!("event_callback | EVENT_OBJECT_UNCLOAKED"),
            EVENT_SYSTEM_MINIMIZESTART => trace!("event_callback | EVENT_SYSTEM_MINIMIZESTART"),
            EVENT_SYSTEM_MINIMIZEEND => trace!("event_callback | EVENT_SYSTEM_MINIMIZEEND"),
            EVENT_SYSTEM_MOVESIZESTART => trace!("event_callback | EVENT_SYSTEM_MOVESIZESTART"),
            EVENT_SYSTEM_MOVESIZEEND => trace!("event_callback | EVENT_SYSTEM_MOVESIZEEND"),
            EVENT_SYSTEM_FOREGROUND => trace!("event_callback | EVENT_SYSTEM_FOREGROUND"),
            EVENT_OBJECT_LOCATIONCHANGE => trace!("event_callback | EVENT_OBJECT_LOCATIONCHANGE"),
            _ => trace!("event_callback | event_type: UNKNOWN({:?})", event_type),
        }
    }

    unsafe fn defer_windows_pos(count: i32) -> Result<WindowsDeferPosHandle> {
        let info = BeginDeferWindowPos(count)?;

        Ok(WindowsDeferPosHandle::new(info))
    }
}
