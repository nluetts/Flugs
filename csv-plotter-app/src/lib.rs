#![warn(clippy::all, rust_2018_idioms)]

mod backend_state;
mod file_handling;
mod gui;

pub use backend_state::BackendAppState;
pub use gui::EguiApp;
