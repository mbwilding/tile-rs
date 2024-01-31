// hide console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod classes;
mod csharp;
mod helpers;
mod layout_engines;
mod manager;
mod window;
mod workspace;

use crate::helpers::single;
use eframe::egui;
use log::info;

pub const APP_NAME: &str = "Tile-RS";

fn main() -> eframe::Result<()> {
    env_logger::init();
    info!("Starting Tile-RS");
    single::check();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |cc| {
            let app = app::App::new(cc);
            app.windows_manager
                .lock()
                .unwrap()
                .init(app.settings.layout_engine_type);
            Box::new(app)
        }),
    )
}
