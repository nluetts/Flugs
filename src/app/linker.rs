use std::sync::mpsc::Sender;

use crate::BackendEventLoop;

/// The linker is send to the backend thread and replies
/// once the action ran on the backend.
pub struct BackendLink<T, F>
where
    F: Fn(&mut BackendEventLoop) -> T,
{
    backchannel: Sender<T>,
    action: F,
    description: String,
}

impl<T, F> BackendLink<T, F>
where
    F: Fn(&mut BackendEventLoop) -> T,
{
    pub fn new(backchannel: Sender<T>, action: F, description: String) -> Self {
        Self {
            backchannel,
            action,
            description,
        }
    }
}

pub trait BackendRequest: Send {
    fn run_on_backend(&self, backend: &mut BackendEventLoop);
    fn describe(&self) -> &str;
}

impl<T, F> BackendRequest for BackendLink<T, F>
where
    F: Fn(&mut BackendEventLoop) -> T + Send,
    T: Send,
{
    fn run_on_backend(&self, backend: &mut BackendEventLoop) {
        let result = (self.action)(backend);
        self.backchannel
            .send(result)
            .expect("Trying to send message on closed channel.")
    }
    fn describe(&self) -> &str {
        &self.description
    }
}
