use std::{marker::PhantomData, sync::mpsc::Sender};

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
    pub fn new(backchannel: Sender<T>, description: String, action: F) -> Self {
        Self {
            backchannel,
            action,
            description,
            _marker: PhantomData,
        }
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
        let result = (self.action)(backend);
        self.backchannel
            .send(result)
            .expect("Trying to send message on closed channel.")
    }
    fn describe(&self) -> &str {
        &self.description
    }
}
