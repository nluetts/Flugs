#![warn(clippy::all, rust_2018_idioms)]

mod backend;
mod counter_app;
mod frontend;

pub use backend::{eventloop::BackendEventLoop, BackendState};
pub use counter_app::{backend_state::CounterAppState, App};
pub use frontend::{
    backend_link::{BackendLink, BackendRequest},
    ui_parameter::UIParameter,
};
