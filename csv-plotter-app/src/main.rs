#![warn(clippy::all, rust_2018_idioms)]

use std::path::PathBuf;
use std::str::FromStr;

use app_core::backend::BackendEventLoop;
use csv_plotter::{BackendAppState, EguiApp};

const WINDOW_NAME: &str = "PlotMe CSV Plotter";
const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 300.0;

fn main() -> eframe::Result {
    env_logger::init();

    // start backend loop
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let backend_state = BackendAppState::new(
        PathBuf::from_str("/home/nluetts/ownCloud/Cookie-Measurement-Data/")
            .expect("unable to open demo file path"),
    );
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
