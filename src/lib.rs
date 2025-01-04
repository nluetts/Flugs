#![warn(clippy::all, rust_2018_idioms)]

mod backend;
mod counter_app;
mod frontend;

pub use backend::{
    backend_link::{BackendLink, BackendRequest},
    eventloop::BackendEventLoop,
    BackendState,
};
pub use counter_app::{backend_state::CounterAppState, App};
pub use frontend::ui_parameter::UIParameter;
