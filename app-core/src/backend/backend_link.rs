use log::{info, warn};
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        mpsc::{channel, Receiver, RecvTimeoutError, Sender, TryRecvError},
        Arc,
    },
};

use crate::backend::{BackendEventLoop, BackendState};

/// The linker is send to the backend thread and replies
/// once the action ran on the backend.
pub struct BackendLink<T, F, S>
where
    F: Fn(&mut BackendEventLoop<S>) -> T,
    S: BackendState,
{
    backchannel: Sender<T>,
    action: F,
    is_cancelled: Arc<AtomicBool>,
    description: String,
    _marker: PhantomData<S>,
}

impl<T, F, S> BackendLink<T, F, S>
where
    F: Fn(&mut BackendEventLoop<S>) -> T,
    S: BackendState,
{
    pub fn new(description: &str, action: F) -> (LinkReceiver<T>, Self) {
        let (tx, rx) = channel();
        let is_cancelled = Arc::new(AtomicBool::new(false));
        let rx = LinkReceiver {
            rx,
            is_cancelled: is_cancelled.clone(),
        };
        (
            rx,
            Self {
                backchannel: tx,
                action,
                description: description.to_owned(),
                is_cancelled: Arc::new(AtomicBool::new(false)),
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
        let result = if !self.is_cancelled.load(SeqCst) {
            (self.action)(backend)
        } else {
            return;
        };
        // we check for a cancelled request again, because
        // the request might have been cancelled while
        // running `self.action`
        if !self.is_cancelled.load(SeqCst) {
            let _ = self.backchannel.send(result).map_err(|_| {
                warn!(
                    "Trying to send message for request '{}' on closed channel.",
                    self.description
                )
            });
        }
    }
    fn describe(&self) -> &str {
        &self.description
    }
}

pub struct LinkReceiver<T> {
    rx: Receiver<T>,
    is_cancelled: Arc<AtomicBool>,
}

impl<T> LinkReceiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.rx.try_recv()
    }
    pub fn recv_timeout(&self, duration: std::time::Duration) -> Result<T, RecvTimeoutError> {
        self.rx.recv_timeout(duration)
    }
}

impl<T> Drop for LinkReceiver<T> {
    fn drop(&mut self) {
        info!("dropping link receiver");
        self.is_cancelled.store(true, SeqCst);
    }
}
