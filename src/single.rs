use crate::APP_NAME;
use anyhow::Result;
use log::error;
use single_instance::SingleInstance;

pub fn check() {
    let instance = SingleInstance::new(APP_NAME).unwrap();

    if !instance.is_single() {
        error!("Another instance of tile-rs is already running");
        std::process::exit(69);
    }
}
