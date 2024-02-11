use crate::app::App;
use std::thread;

pub fn context(mut app: &mut App) {
    app.windows_manager.init(app.settings.layout_engine_type);

    app.windows_manager.event_window_created.subscribe();
    app.workspace_manager.add_window_manager();

    let test = app.windows_manager.event_window_updated.subscribe();
    thread::spawn(move || {
        for message in test.iter() {
            println!("Received message: {:?}", message);
        }
    });
}
