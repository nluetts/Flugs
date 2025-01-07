mod backend_link;
mod eventloop;

pub use self::{
    backend_link::{BackendLink, BackendRequest, LinkReceiver},
    eventloop::{request_stop, BackendEventLoop},
};

pub trait BackendState {}
