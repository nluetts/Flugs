#![warn(clippy::all, rust_2018_idioms)]

use egui_app_template::App;
use egui_app_template::BackendEventLoop;
use egui_app_template::CounterAppState;

fn main() -> eframe::Result {
    // start backend loop
    let (command_tx, command_rx) = std::sync::mpsc::channel();

    let backend_state = CounterAppState::default();
    let _eventloop_handle = BackendEventLoop::new(command_rx, backend_state).run();

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, command_tx)))),
    )
}
