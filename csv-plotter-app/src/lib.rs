#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod backend_state;

pub use app::EguiApp;
pub use backend_state::BackendAppState;

pub const ROOT_PATH: &str = "/home/nluetts/ownCloud/Cookie-Measurement-Data/";
