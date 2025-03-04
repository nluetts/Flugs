#![warn(clippy::all, rust_2018_idioms)]

use app_core::backend::BackendEventLoop;
use flugs::{storage::load_config, BackendAppState, EguiApp};

const WINDOW_NAME: &str = "Flugs >>";
const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 300.0;

fn main() -> eframe::Result {
    env_logger::init();

    // start backend loop
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    #[allow(deprecated)]
    let search_path =
        load_config().unwrap_or(std::env::home_dir().expect("Could not set root search path!"));
    let backend_state = BackendAppState::new(search_path);
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
        Box::new(|cc| Ok(Box::new(EguiApp::new(cc, command_tx, eventloop_handle)))),
    )
}
