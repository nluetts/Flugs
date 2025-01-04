use log::warn;
use std::{
    marker::PhantomData,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{BackendEventLoop, BackendState};

/// The linker is send to the backend thread and replies
/// once the action ran on the backend.
pub struct BackendLink<T, F, S>
where
    F: Fn(&mut BackendEventLoop<S>) -> T,
    S: BackendState,
{
    backchannel: Sender<T>,
    action: F,
    description: String,
    _marker: PhantomData<S>,
}

impl<T, F, S> BackendLink<T, F, S>
where
    F: Fn(&mut BackendEventLoop<S>) -> T,
    S: BackendState,
{
    pub fn new(description: &str, action: F) -> (Receiver<T>, Self) {
        let (tx, rx) = channel();
        (
            rx,
            Self {
                backchannel: tx,
                action,
                description: description.to_owned(),
                _marker: PhantomData,
            },
        )
    }
}

pub trait BackendRequest<S>: Send
where
    S: BackendState,
{
    fn run_on_backend(&self, backend: &mut BackendEventLoop<S>);
    fn describe(&self) -> &str;
}

impl<T, F, S> BackendRequest<S> for BackendLink<T, F, S>
where
    F: Fn(&mut BackendEventLoop<S>) -> T + Send,
    S: BackendState + Send,
    T: Send,
{
    fn run_on_backend(&self, backend: &mut BackendEventLoop<S>) {
        // TODO: the action should only run if the listening side is still
        // alive; consider implementing this with an atomic bool
        let result = (self.action)(backend);
        let _ = self.backchannel.send(result).map_err(|_| {
            warn!(
                "Trying to send message for request '{}' on closed channel.",
                self.description
            )
        });
    }
    fn describe(&self) -> &str {
        &self.description
    }
}
