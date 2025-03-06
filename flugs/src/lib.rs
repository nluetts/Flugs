#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod backend_state;

pub use app::config::Config;
pub use app::storage;
pub use app::EguiApp;
pub use backend_state::BackendAppState;
