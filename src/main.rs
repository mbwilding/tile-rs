mod single;
mod structs;
mod win32_helpers;
mod window_location;
mod window_state;
mod windows_defer_pos_handle;
mod windows_manager;
mod windows_window;

use crate::windows_manager::WindowsManager;
use anyhow::Result;

pub const APP_NAME: &str = "Tile-RS";

fn main() -> Result<()> {
    env_logger::init();
    single::check()?;
    let windows_manager = WindowsManager::default();
    windows_manager.init();

    // TODO: Remove this.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
    }
}
