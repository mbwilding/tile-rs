use crate::native_monitor_container::NativeMonitorContainer;
use crate::windows_manager;
use eframe::egui;
use eframe::emath::Align;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    settings: Settings,

    #[serde(skip)]
    monitor_container: NativeMonitorContainer,

    #[serde(skip)]
    window_state: WindowState,
}

#[derive(Deserialize, Serialize)]
struct Settings {
    test: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { test: 0.0 }
    }
}

#[derive(Default)]
struct WindowState {
    settings: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // if !cfg!(target_arch = "wasm32") {
                //     ui.menu_button("File", |ui| {
                //         if ui.button("Quit").clicked() {
                //             ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                //         }
                //     });
                //     ui.add_space(16.0);
                // }

                egui::widgets::global_dark_light_mode_switch(ui);

                // ui.add_space(16.0);

                if ui.button("Settings").clicked() {
                    self.window_state.settings = !self.window_state.settings;
                }
            });
        });

        egui::Window::new("Settings")
            .open(&mut self.window_state.settings)
            .show(ctx, |ui| {
                let settings = &mut self.settings;

                egui::containers::collapsing_header::CollapsingHeader::new("Parameters")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Test");
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                ui.add(egui::Slider::new(&mut settings.test, 0.0..=600.0));
                            });
                        });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut windows_manager = windows_manager::INSTANCE.lock().unwrap();

            ui.horizontal(|ui| {
                ui.label("Windows");
                ui.label(windows_manager.windows.len().to_string());
            });
            ui.horizontal(|ui| {
                ui.label("Floating");
                ui.label(windows_manager.floating.len().to_string());
            });

            ui.separator();

            egui::containers::collapsing_header::CollapsingHeader::new("Windows")
                .default_open(true)
                .show(ui, |ui| {
                    windows_manager.windows.iter_mut().for_each(|(_, window)| {
                        ui.horizontal(|ui| {
                            let mut title = window.title();
                            if title.is_empty() {
                                title.push('_');
                            }

                            if ui.button("Normal").clicked() {
                                window.show_normal();
                            }

                            if ui.button("Minimize").clicked() {
                                window.show_minimized();
                            }

                            if ui.button("Maximize").clicked() {
                                window.show_maximized();
                            }

                            if ui.button("Close").clicked() {
                                window.close();
                            }

                            ui.label(format!(
                                "{} ({})    Class: {} | Path:{}",
                                title,
                                window.location(),
                                window.class(),
                                window.process_name(),
                            ));
                        });
                    });
                });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
