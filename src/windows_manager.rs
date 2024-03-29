use crate::classes::action::Action;
use crate::classes::keys::Keys;
use crate::delegates::{WindowCreateDelegate, WindowDelegate, WindowUpdateDelegate};
use crate::helpers::event::Event;
use crate::helpers::win32_helpers::is_app_window;
use crate::helpers::windows_defer_pos_handle::WindowsDeferPosHandle;
use crate::layout_engines;
use crate::layout_engines::{LayoutEngine, LayoutEngineType};
use crate::window::Window;
use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use windows::Win32::Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, TRUE, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, CallNextHookEx, DispatchMessageW, EnumWindows, GetMessageW,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, EVENT_OBJECT_CLOAKED,
    EVENT_OBJECT_DESTROY, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
    EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WINEVENT_OUTOFCONTEXT, WM_LBUTTONUP,
};

type EventType = (u32, isize);

lazy_static! {
    static ref EVENT: (Sender<EventType>, Receiver<EventType>) = crossbeam_channel::unbounded();
}

lazy_static! {
    static ref KEYS: (Sender<Keys>, Receiver<Keys>) = crossbeam_channel::unbounded();
}

lazy_static! {
    static ref MOUSE: (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();
}

pub struct WindowsManager {
    pub windows: BTreeMap<isize, Window>,
    pub floating: HashMap<isize, bool>,

    mouse_move_lock: Mutex<()>,
    mouse_move_window: Option<isize>,
    layout_engine_type: LayoutEngineType,

    pub event_window_created: Event<WindowCreateDelegate>,
    pub event_window_destroyed: Event<WindowDelegate>,
    pub event_window_updated: Event<WindowUpdateDelegate>,
    pub event_window_focused: Event<WindowDelegate>,
    pub event_external_window_update: Event<WindowDelegate>,
    pub event_external_window_closed: Event<WindowDelegate>,
}

impl Default for WindowsManager {
    fn default() -> Self {
        WindowsManager {
            windows: BTreeMap::new(),
            floating: HashMap::new(),

            mouse_move_lock: Mutex::new(()),
            mouse_move_window: None,
            layout_engine_type: LayoutEngineType::Dwindle,

            event_window_created: Event::<WindowCreateDelegate>::new(),
            event_window_destroyed: Event::<WindowDelegate>::new(),
            event_window_updated: Event::<WindowUpdateDelegate>::new(),
            event_window_focused: Event::<WindowDelegate>::new(),
            event_external_window_update: Event::<WindowDelegate>::new(),
            event_external_window_closed: Event::<WindowDelegate>::new(),
        }
    }
}

impl WindowsManager {
    #[allow(dead_code)]
    pub fn test_layout(&mut self, layout_engine_type: LayoutEngineType) {
        // TODO: Check if enabled

        // TODO: Don't construct every time
        let mut layout: Box<dyn LayoutEngine> = match layout_engine_type {
            LayoutEngineType::Dwindle => {
                Box::new(layout_engines::dwindle_layout_engine::DwindleLayoutEngine::new())
            }
            LayoutEngineType::Focus => {
                Box::new(layout_engines::focus_layout_engine::FocusLayoutEngine::new())
            }
            LayoutEngineType::Full => {
                Box::new(layout_engines::full_layout_engine::FullLayoutEngine::new())
            }
            LayoutEngineType::Grid => {
                Box::new(layout_engines::grid_layout_engine::GridLayoutEngine::new())
            }
        };

        self.windows
            .iter_mut()
            .for_each(|(_, window)| window.show_in_current_state());

        let window_data: Vec<_> = self
            .windows
            .iter()
            .map(|(&id, window)| (id, window.clone()))
            .collect();

        let calc = layout.calc_layout(
            &window_data.iter().map(|(_, w)| w).collect::<Vec<_>>(),
            3840,
            2160,
        );

        debug!("calc: {:?}", calc);

        let mut handle = self.defer_windows_pos(calc.len() as i32);

        for (i, loc) in calc.iter().enumerate() {
            let (_, window) = &window_data[i];

            if !window.is_mouse_moving && !window.is_fullscreen() {
                handle.defer_window_pos(window, loc);
            }
        }
    }

    pub fn init(&mut self, layout_engine_type: LayoutEngineType) {
        self.change_layout(layout_engine_type);

        info!("Initializing hooks");

        let module_handle = unsafe {
            GetModuleHandleW(None).unwrap_or_else(|e| {
                error!("Failed GetModuleHandleW: {:?}", e);
                std::process::exit(69);
            })
        };

        let event_windows = [
            Self::register_window_hook(EVENT_OBJECT_DESTROY, EVENT_OBJECT_SHOW, module_handle),
            Self::register_window_hook(EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED, module_handle),
            Self::register_window_hook(
                EVENT_SYSTEM_MINIMIZESTART,
                EVENT_SYSTEM_MINIMIZEEND,
                module_handle,
            ),
            Self::register_window_hook(
                EVENT_SYSTEM_MOVESIZESTART,
                EVENT_SYSTEM_MOVESIZEEND,
                module_handle,
            ),
            Self::register_window_hook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                module_handle,
            ),
            Self::register_window_hook(
                EVENT_OBJECT_LOCATIONCHANGE,
                EVENT_OBJECT_LOCATIONCHANGE,
                module_handle,
            ),
        ];

        let event_mouse = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(Self::mouse_callback), module_handle, 0)
                .unwrap_or_else(|e| {
                    error!("Failed SetWindowsHookExW[Mouse]: {:?}", e);
                    std::process::exit(69);
                })
        };

        let event_keyboard = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::keyboard_callback),
                module_handle,
                0,
            )
            .unwrap_or_else(|e| {
                error!("Failed SetWindowsHookExW[Keyboard]: {:?}", e);
                std::process::exit(69);
            })
        };

        info!("Initialized hooks");

        let mut windows: Vec<HWND> = vec![];

        unsafe {
            let _ = EnumWindows(
                Some(Self::enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<HWND> as isize),
            );
        };

        for hwnd in windows.into_iter() {
            if is_app_window(hwnd) {
                self.register_window(hwnd.0);
            }
        }

        let mut message = MSG::default();

        // Handle close
        std::thread::spawn(move || {
            loop {
                unsafe {
                    if GetMessageW(&mut message, HWND(0), 0, 0).as_bool() {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    } else {
                        break;
                    }
                }
            }

            unsafe {
                UnhookWindowsHookEx(event_mouse).unwrap_or_else(|e| {
                    error!("Failed UnhookWindowsHookEx (mouse): {:?}", e);
                })
            };

            unsafe {
                UnhookWindowsHookEx(event_keyboard).unwrap_or_else(|e| {
                    error!("Failed UnhookWindowsHookEx (keyboard): {:?}", e);
                })
            };

            for hooks in event_windows.into_iter() {
                unsafe {
                    if !UnhookWinEvent(hooks).as_bool() {
                        error!("Failed UnhookWinEvent");
                    }
                }
            }
        });
    }

    pub fn handle_window(&mut self) {
        if let Ok((event, hwnd)) = EVENT.1.try_recv() {
            match event {
                EVENT_OBJECT_SHOW => self.register_window(hwnd),
                EVENT_OBJECT_DESTROY => self.unregister_window(hwnd),
                EVENT_OBJECT_CLOAKED => self.update_window(hwnd, WindowUpdateType::Hide),
                EVENT_OBJECT_UNCLOAKED => self.update_window(hwnd, WindowUpdateType::Show),
                EVENT_SYSTEM_MINIMIZESTART => {
                    self.update_window(hwnd, WindowUpdateType::MinimizeStart)
                }
                EVENT_SYSTEM_MINIMIZEEND => self.update_window(hwnd, WindowUpdateType::MinimizeEnd),
                EVENT_SYSTEM_FOREGROUND => self.update_window(hwnd, WindowUpdateType::Foreground),
                EVENT_SYSTEM_MOVESIZESTART => self.start_move_window(hwnd),
                EVENT_SYSTEM_MOVESIZEEND => self.end_move_window(hwnd),
                EVENT_OBJECT_LOCATIONCHANGE => self.window_move(hwnd),
                _ => error!("handle_window | event_type: UNKNOWN({:?})", event),
            };
        }
    }

    pub fn handle_keys(&mut self, key_bindings: &HashMap<Action, Keys>) {
        if let Ok(keys) = KEYS.1.try_recv() {
            let matching = key_bindings.iter().find(|(_, key)| *key == &keys);

            if let Some((action, _)) = matching {
                match action {
                    Action::ToggleFocusedWindowTiling => {
                        info!("action: ToggleFocusedWindowTiling");
                        self.toggle_focused_window_tiling();
                    }
                }
            }
        }
    }

    pub fn handle_mouse(&mut self) {
        if let Ok(_mouse) = MOUSE.1.try_recv() {
            trace!("mouse_release")
        }
    }

    pub fn change_layout(&mut self, layout_engine_type: LayoutEngineType) {
        self.layout_engine_type = layout_engine_type;
        info!("Changed layout engine: {:?}", &layout_engine_type);
    }

    #[allow(dead_code)]
    fn defer_windows_pos(&mut self, count: i32) -> WindowsDeferPosHandle {
        let info = unsafe { BeginDeferWindowPos(count).unwrap() }; // TODO: Unwrap

        WindowsDeferPosHandle::new(info)
    }

    #[allow(dead_code)]
    pub fn toggle_focused_window_tiling(&mut self) {
        let hwnd_option = self
            .windows
            .values()
            .find(|w| w.is_focused())
            .map(|window| window.handle);

        if let Some(hwnd) = hwnd_option {
            if let std::collections::hash_map::Entry::Vacant(e) = self.floating.entry(hwnd) {
                e.insert(true);
                self.handle_window_remove(hwnd);

                if let Some(window) = self.windows.get_mut(&hwnd) {
                    window.bring_to_top();
                }
            } else {
                self.floating.remove(&hwnd);
                self.handle_window_add(hwnd, false);
            }

            if let Some(window) = self.windows.get_mut(&hwnd) {
                window.focus();
            }
        }
    }

    fn register_window_hook(event_min: u32, event_max: u32, hmodule: HMODULE) -> HWINEVENTHOOK {
        unsafe {
            SetWinEventHook(
                event_min,
                event_max,
                hmodule,
                Some(Self::window_callback),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        }
    }

    fn register_window(&mut self, hwnd: isize) {
        if self.windows.contains_key(&hwnd) {
            trace!("register_window | handle: 0x{:X} already registered", &hwnd);
            return;
        }

        trace!("register_window | handle: 0x{:X} not registered", &hwnd);

        match Window::new(hwnd) {
            Ok(window) => {
                debug!("register_window | handle: 0x{:X} registered", &hwnd);
                self.windows.insert(hwnd, window)
            }
            Err(_) => None,
        };
    }

    fn unregister_window(&mut self, hwnd: isize) {
        if !self.windows.contains_key(&hwnd) {
            trace!("unregister_window | handle: 0x{:X} not registered", &hwnd);
            return;
        }

        trace!("unregister_window | handle: 0x{:X} registered", &hwnd);

        self.windows.remove(&hwnd);
        self.handle_window_remove(hwnd);
    }

    fn update_window(&mut self, hwnd: isize, update_type: WindowUpdateType) {
        if update_type == WindowUpdateType::Show && self.windows.contains_key(&hwnd) {
            if let Some(window) = self.windows.get(&hwnd) {
                self.event_window_updated
                    .broadcast((window.clone(), update_type));
            };
        } else if update_type == WindowUpdateType::Show {
            self.register_window(hwnd);
        } else if update_type == WindowUpdateType::Hide && self.windows.contains_key(&hwnd) {
            if let Some(window) = self.windows.get(&hwnd) {
                if !window.did_manual_hide() {
                    self.unregister_window(hwnd);
                } else {
                    self.event_window_updated
                        .broadcast((window.clone(), update_type));
                }
            };
        } else if self.windows.contains_key(&hwnd) {
            if let Some(_window) = self.windows.get(&hwnd) {};
            self.event_window_updated
                .broadcast((self.windows[&hwnd].clone(), update_type));
        }
    }

    fn start_move_window(&mut self, hwnd: isize) {
        if self.windows.contains_key(&hwnd) {
            self.handle_window_move_start(hwnd);
            self.event_window_updated
                .broadcast((self.windows[&hwnd].clone(), WindowUpdateType::MoveStart));
            debug!("start_move_window | handle: 0x{:X}", &hwnd);
        }
    }

    fn end_move_window(&mut self, hwnd: isize) {
        if self.windows.contains_key(&hwnd) {
            self.handle_window_move_end();
            self.event_window_updated
                .broadcast((self.windows[&hwnd].clone(), WindowUpdateType::MoveEnd));
            debug!("end_move_window | handle: 0x{:X}", &hwnd);
        }
    }

    fn window_move(&mut self, hwnd: isize) {
        if let Some(window) = self.windows.get(&hwnd) {
            if window.can_layout() {
                self.event_window_updated
                    .broadcast((window.clone(), WindowUpdateType::Move));
            }
        }
    }

    fn handle_window_focused(&mut self, handle: isize) {
        self.event_window_focused
            .broadcast(self.windows[&handle].clone());
    }

    fn handle_window_updated(&mut self, handle: isize) {
        self.event_external_window_update
            .broadcast(self.windows[&handle].clone());
    }

    fn handle_window_closed(&mut self, handle: isize) {
        self.event_external_window_closed
            .broadcast(self.windows[&handle].clone());
    }

    fn handle_window_move_start(&mut self, handle: isize) {
        if let Some(current_handle) = self.mouse_move_window {
            if let Some(window) = self.windows.get_mut(&current_handle) {
                window.is_mouse_moving = false;
            }
        }

        self.mouse_move_window = Some(handle);

        if let Some(window) = self.windows.get_mut(&handle) {
            window.is_mouse_moving = true;
        }
    }

    fn handle_window_move_end(&mut self) {
        let _lock = self.mouse_move_lock.lock().unwrap();
        if let Some(ref mut handle) = self.mouse_move_window {
            if let Some(window) = self.windows.get_mut(handle) {
                window.is_mouse_moving = false;
            }
            self.mouse_move_window = None;
        }
    }

    fn handle_window_add(&mut self, handle: isize, first_create: bool) {
        self.event_window_created
            .broadcast((self.windows[&handle].clone(), first_create));
    }

    fn handle_window_remove(&mut self, handle: isize) {
        self.event_window_destroyed
            .broadcast(self.windows[&handle].clone());
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, userdata: LPARAM) -> BOOL {
        let windows = &mut *(userdata.0 as *mut Vec<HWND>);
        windows.push(hwnd);

        TRUE
    }

    unsafe extern "system" fn mouse_callback(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if w_param.0 == WM_LBUTTONUP as usize && MOUSE.0.send(()).is_err() {
            error!("mouse_callback | failed to send");
        }

        CallNextHookEx(None, n_code, w_param, l_param)
    }

    unsafe extern "system" fn keyboard_callback(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if let Some(keys) = Keys::new(n_code, w_param, l_param) {
            if KEYS.0.send(keys).is_err() {
                error!("keyboard_callback | failed to send");
            };
        };

        CallNextHookEx(None, n_code, w_param, l_param)
    }

    unsafe extern "system" fn window_callback(
        _h_win_event_hook: HWINEVENTHOOK,
        event_type: u32,
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        let hwnd = hwnd.0;

        if !(id_child == 0 && id_object == 0 && hwnd != 0) {
            return;
        }

        if EVENT.0.send((event_type, hwnd)).is_err() {
            error!(
                "event_callback | failed to send | event_type: {:?}, hwnd: 0x{:X}",
                event_type, hwnd
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowUpdateType {
    Show,
    Hide,
    MinimizeStart,
    MinimizeEnd,
    Foreground,
    #[allow(dead_code)]
    MoveStart,
    #[allow(dead_code)]
    MoveEnd,
    #[allow(dead_code)]
    Move,
}
