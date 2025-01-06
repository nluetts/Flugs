mod backend_link;
mod eventloop;

pub use self::{
    backend_link::{BackendLink, BackendRequest, LinkReceiver},
    eventloop::BackendEventLoop,
};

pub trait BackendState {}
