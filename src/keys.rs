use log::trace;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::{WM_KEYDOWN, WM_SYSKEYDOWN};

#[derive(Debug)]
pub struct Keys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
    pub key: VirtualKey,
}

impl Keys {
    pub unsafe fn new(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> Option<Self> {
        if n_code >= 0 {
            match w_param.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    let capture = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

                    let key = VirtualKey::from_vk(capture.vkCode);

                    trace!(
                        "keyboard | vk_code: 0x{:X} | key: {:?}",
                        capture.vkCode,
                        &key
                    );

                    let shift = GetAsyncKeyState(VK_SHIFT.0 as i32) & (1 << 15) != 0;
                    let ctrl = GetAsyncKeyState(VK_CONTROL.0 as i32) & (1 << 15) != 0;
                    let alt = GetAsyncKeyState(VK_MENU.0 as i32) & (1 << 15) != 0;
                    let win = GetAsyncKeyState(VK_LWIN.0 as i32) & (1 << 15) != 0
                        || GetAsyncKeyState(VK_RWIN.0 as i32) & (1 << 15) != 0;

                    Some(Keys {
                        shift,
                        ctrl,
                        alt,
                        win,
                        key,
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VirtualKey {
    Unknown = 0x00,
    Backspace = 0x08,
    Tab = 0x09,
    Clear = 0x0C,
    Enter = 0x0D,
    Shift = 0x10,
    Ctrl = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    Capslock = 0x14,
    Esc = 0x1B,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    Select = 0x29,
    Print = 0x2A,
    Execute = 0x2B,
    PrintScreen = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,
    Help = 0x2F,
    Key0 = 0x30,
    Key1 = 0x31,
    Key2 = 0x32,
    Key3 = 0x33,
    Key4 = 0x34,
    Key5 = 0x35,
    Key6 = 0x36,
    Key7 = 0x37,
    Key8 = 0x38,
    Key9 = 0x39,
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
    LeftWin = 0x5B,
    RightWin = 0x5C,
    Apps = 0x5D,
    Sleep = 0x5F,
    Num0 = 0x60,
    Num1 = 0x61,
    Num2 = 0x62,
    Num3 = 0x63,
    Num4 = 0x64,
    Num5 = 0x65,
    Num6 = 0x66,
    Num7 = 0x67,
    Num8 = 0x68,
    Num9 = 0x69,
    Multiply = 0x6A,
    Add = 0x6B,
    Separator = 0x6C,
    Subtract = 0x6D,
    Decimal = 0x6E,
    Divide = 0x6F,
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,
    NumLock = 0x90,
    ScrollLock = 0x91,
    LeftShift = 0xA0,
    RightShift = 0xA1,
    LeftControl = 0xA2,
    RightControl = 0xA3,
    LeftAlt = 0xA4,
    RightAlt = 0xA5,
    BrowserBack = 0xA6,
    BrowserForward = 0xA7,
    BrowserRefresh = 0xA8,
    BrowserStop = 0xA9,
    BrowserSearch = 0xAA,
    BrowserFavorites = 0xAB,
    BrowserHome = 0xAC,
    VolumeMute = 0xAD,
    VolumeDown = 0xAE,
    VolumeUp = 0xAF,
    MediaNextTrack = 0xB0,
    MediaPreviousTrack = 0xB1,
    MediaStop = 0xB2,
    MediaPlayPause = 0xB3,
    LaunchMail = 0xB4,
    LaunchMediaSelect = 0xB5,
    LaunchApp1 = 0xB6,
    LaunchApp2 = 0xB7,
    Semicolon = 0xBA,
    Equals = 0xBB,
    Comma = 0xBC,
    Minus = 0xBD,
    Period = 0xBE,
    Slash = 0xBF,
    BackTick = 0xC0,
    LeftBracket = 0xDB,
    Backslash = 0xDC,
    RightBracket = 0xDD,
    Quote = 0xDE,
}

impl VirtualKey {
    pub fn from_vk(vk: u32) -> Self {
        match vk {
            0x08 => VirtualKey::Backspace,
            0x09 => VirtualKey::Tab,
            0x0C => VirtualKey::Clear,
            0x0D => VirtualKey::Enter,
            0x10 => VirtualKey::Shift,
            0x11 => VirtualKey::Ctrl,
            0x12 => VirtualKey::Alt,
            0x13 => VirtualKey::Pause,
            0x14 => VirtualKey::Capslock,
            0x1B => VirtualKey::Esc,
            0x20 => VirtualKey::Space,
            0x21 => VirtualKey::PageUp,
            0x22 => VirtualKey::PageDown,
            0x23 => VirtualKey::End,
            0x24 => VirtualKey::Home,
            0x25 => VirtualKey::Left,
            0x26 => VirtualKey::Up,
            0x27 => VirtualKey::Right,
            0x28 => VirtualKey::Down,
            0x29 => VirtualKey::Select,
            0x2A => VirtualKey::Print,
            0x2B => VirtualKey::Execute,
            0x2C => VirtualKey::PrintScreen,
            0x2D => VirtualKey::Insert,
            0x2E => VirtualKey::Delete,
            0x2F => VirtualKey::Help,
            0x30 => VirtualKey::Key0,
            0x31 => VirtualKey::Key1,
            0x32 => VirtualKey::Key2,
            0x33 => VirtualKey::Key3,
            0x34 => VirtualKey::Key4,
            0x35 => VirtualKey::Key5,
            0x36 => VirtualKey::Key6,
            0x37 => VirtualKey::Key7,
            0x38 => VirtualKey::Key8,
            0x39 => VirtualKey::Key9,
            0x41 => VirtualKey::A,
            0x42 => VirtualKey::B,
            0x43 => VirtualKey::C,
            0x44 => VirtualKey::D,
            0x45 => VirtualKey::E,
            0x46 => VirtualKey::F,
            0x47 => VirtualKey::G,
            0x48 => VirtualKey::H,
            0x49 => VirtualKey::I,
            0x4A => VirtualKey::J,
            0x4B => VirtualKey::K,
            0x4C => VirtualKey::L,
            0x4D => VirtualKey::M,
            0x4E => VirtualKey::N,
            0x4F => VirtualKey::O,
            0x50 => VirtualKey::P,
            0x51 => VirtualKey::Q,
            0x52 => VirtualKey::R,
            0x53 => VirtualKey::S,
            0x54 => VirtualKey::T,
            0x55 => VirtualKey::U,
            0x56 => VirtualKey::V,
            0x57 => VirtualKey::W,
            0x58 => VirtualKey::X,
            0x59 => VirtualKey::Y,
            0x5A => VirtualKey::Z,
            0x5B => VirtualKey::LeftWin,
            0x5C => VirtualKey::RightWin,
            0x5D => VirtualKey::Apps,
            0x5F => VirtualKey::Sleep,
            0x60 => VirtualKey::Num0,
            0x61 => VirtualKey::Num1,
            0x62 => VirtualKey::Num2,
            0x63 => VirtualKey::Num3,
            0x64 => VirtualKey::Num4,
            0x65 => VirtualKey::Num5,
            0x66 => VirtualKey::Num6,
            0x67 => VirtualKey::Num7,
            0x68 => VirtualKey::Num8,
            0x69 => VirtualKey::Num9,
            0x6A => VirtualKey::Multiply,
            0x6B => VirtualKey::Add,
            0x6C => VirtualKey::Separator,
            0x6D => VirtualKey::Subtract,
            0x6E => VirtualKey::Decimal,
            0x6F => VirtualKey::Divide,
            0x70 => VirtualKey::F1,
            0x71 => VirtualKey::F2,
            0x72 => VirtualKey::F3,
            0x73 => VirtualKey::F4,
            0x74 => VirtualKey::F5,
            0x75 => VirtualKey::F6,
            0x76 => VirtualKey::F7,
            0x77 => VirtualKey::F8,
            0x78 => VirtualKey::F9,
            0x79 => VirtualKey::F10,
            0x7A => VirtualKey::F11,
            0x7B => VirtualKey::F12,
            0x7C => VirtualKey::F13,
            0x7D => VirtualKey::F14,
            0x7E => VirtualKey::F15,
            0x7F => VirtualKey::F16,
            0x80 => VirtualKey::F17,
            0x81 => VirtualKey::F18,
            0x82 => VirtualKey::F19,
            0x83 => VirtualKey::F20,
            0x84 => VirtualKey::F21,
            0x85 => VirtualKey::F22,
            0x86 => VirtualKey::F23,
            0x87 => VirtualKey::F24,
            0x90 => VirtualKey::NumLock,
            0x91 => VirtualKey::ScrollLock,
            0xA0 => VirtualKey::LeftShift,
            0xA1 => VirtualKey::RightShift,
            0xA2 => VirtualKey::LeftControl,
            0xA3 => VirtualKey::RightControl,
            0xA4 => VirtualKey::LeftAlt,
            0xA5 => VirtualKey::RightAlt,
            0xA6 => VirtualKey::BrowserBack,
            0xA7 => VirtualKey::BrowserForward,
            0xA8 => VirtualKey::BrowserRefresh,
            0xA9 => VirtualKey::BrowserStop,
            0xAA => VirtualKey::BrowserSearch,
            0xAB => VirtualKey::BrowserFavorites,
            0xAC => VirtualKey::BrowserHome,
            0xAD => VirtualKey::VolumeMute,
            0xAE => VirtualKey::VolumeDown,
            0xAF => VirtualKey::VolumeUp,
            0xB0 => VirtualKey::MediaNextTrack,
            0xB1 => VirtualKey::MediaPreviousTrack,
            0xB2 => VirtualKey::MediaStop,
            0xB3 => VirtualKey::MediaPlayPause,
            0xB4 => VirtualKey::LaunchMail,
            0xB5 => VirtualKey::LaunchMediaSelect,
            0xB6 => VirtualKey::LaunchApp1,
            0xB7 => VirtualKey::LaunchApp2,
            0xBA => VirtualKey::Semicolon,
            0xBB => VirtualKey::Equals,
            0xBC => VirtualKey::Comma,
            0xBD => VirtualKey::Minus,
            0xBE => VirtualKey::Period,
            0xBF => VirtualKey::Slash,
            0xC0 => VirtualKey::BackTick,
            0xDB => VirtualKey::LeftBracket,
            0xDC => VirtualKey::Backslash,
            0xDD => VirtualKey::RightBracket,
            0xDE => VirtualKey::Quote,
            _ => VirtualKey::Unknown,
        }
    }
}
