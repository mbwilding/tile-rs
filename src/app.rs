use crate::layout_engines::LayoutEngineType;
use crate::native_monitor_container::NativeMonitorContainer;

use crate::windows_manager::WindowsManager;
use eframe::egui;
use eframe::emath::Align;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    pub settings: Settings,

    #[serde(skip)]
    pub windows_manager: WindowsManager,

    #[serde(skip)]
    monitor_container: NativeMonitorContainer,

    #[serde(skip)]
    window_state: WindowState,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub layout_engine_type: LayoutEngineType,
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
        ctx.request_repaint(); // Temp fix to keep loop going
        self.windows_manager.check();

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
                egui::containers::collapsing_header::CollapsingHeader::new("Parameters")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Layout");
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                let response = egui::ComboBox::new("layout_engine_type", "")
                                    .selected_text(self.settings.layout_engine_type.to_string())
                                    .show_ui(ui, |ui| {
                                        for option in LayoutEngineType::variants() {
                                            ui.selectable_value(
                                                &mut self.settings.layout_engine_type,
                                                option,
                                                option.to_string(),
                                            );
                                        }
                                    });

                                if response.response.changed() {
                                    self.windows_manager
                                        .change_layout(self.settings.layout_engine_type);
                                }
                            });
                        });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Windows");
                ui.label(self.windows_manager.windows.len().to_string());
            });

            ui.horizontal(|ui| {
                ui.heading("Floating");
                ui.label(self.windows_manager.floating.len().to_string());
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    egui::containers::collapsing_header::CollapsingHeader::new("Windows")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.windows_manager
                                .windows
                                .iter_mut()
                                .for_each(|(_, window)| {
                                    let title = window.title();

                                    egui::containers::collapsing_header::CollapsingHeader::new(
                                        &title,
                                    )
                                    .id_source(format!("window_{}", &title))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        let location = window.location();

                                        ui.horizontal(|ui| {
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
                                        });

                                        ui.horizontal(|ui| {
                                            ui.heading("State");
                                            ui.label(format!("{:?}", location.state));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.heading("Location");
                                            ui.label(format!("{} x {}", location.x, location.y));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.heading("Bounds");
                                            ui.label(format!(
                                                "{} x {}",
                                                location.width, location.height
                                            ));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.heading("Class");
                                            ui.label(window.class());
                                        });

                                        ui.horizontal(|ui| {
                                            ui.heading("Path");
                                            ui.label(window.process_name());
                                        });

                                        ui.horizontal(|ui| {
                                            ui.heading("Process");
                                            ui.label(window.process_file_name());
                                        });
                                    });
                                });
                        });
                });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let moving = self.windows_manager.windows.iter().find_map(|(_, window)| {
                if window.is_mouse_moving {
                    Some(window.title())
                } else {
                    None
                }
            });

            if let Some(moving) = moving {
                ui.horizontal(|ui| {
                    ui.monospace("Moving:");
                    ui.label(moving);
                });
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
