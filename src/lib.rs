#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod backend;

pub use app::App;
pub use backend::eventloop::BackendEventLoop;
