#![warn(clippy::all, rust_2018_idioms)]

mod backend;
mod csv_plotter_app;
mod frontend;

pub use backend::{
    backend_link::{BackendLink, BackendRequest},
    eventloop::BackendEventLoop,
    BackendState,
};
pub use csv_plotter_app::{backend_state::BackendAppState, EguiApp};
pub use frontend::ui_parameter::UIParameter;
