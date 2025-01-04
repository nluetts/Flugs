#![warn(clippy::all, rust_2018_idioms)]

use egui_app_template::App;
use egui_app_template::BackendEventLoop;
use egui_app_template::CounterAppState;

const WINDOW_NAME: &str = "PlotMe CSV Plotter";
const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 300.0;

fn main() -> eframe::Result {
    env_logger::init();

    // start backend loop
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let backend_state = CounterAppState::default();
    let eventloop_handle = BackendEventLoop::new(command_rx, backend_state).run();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_HEIGHT, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
        ..Default::default()
    };
    eframe::run_native(
        WINDOW_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, command_tx, eventloop_handle)))),
    )
}
