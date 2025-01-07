use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use log::info;
use log::warn;

use crate::backend::BackendLink;
use crate::backend::BackendRequest;
use crate::backend::BackendState;

pub struct BackendEventLoop<S>
where
    S: BackendState,
{
    pub state: S,
    request_rx: Receiver<Box<dyn BackendRequest<S>>>,
    should_stop: bool,
}

impl<S: BackendState + Send + 'static> BackendEventLoop<S> {
    pub fn update(&mut self) -> bool {
        // handle the most important command
        while let Ok(request) = self.request_rx.try_recv() {
            info!("handeling request '{}'", request.describe());
            request.run_on_backend(self);
        }
        self.should_stop
    }
    pub fn run(mut self) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || loop {
            let stop_loop = self.update();
            if stop_loop {
                info!("stopping backend event loop");
                break;
            }
        })
    }
    pub fn new(command_rx: Receiver<Box<dyn BackendRequest<S>>>, state: S) -> Self {
        info!("creating new event loop");
        Self {
            state,
            request_rx: command_rx,
            should_stop: false,
        }
    }
    pub fn signal_stop(&mut self) -> bool {
        self.should_stop = true;
        true
    }
}

pub fn request_stop<S: BackendState + Send + 'static>(
    request_tx: &Sender<Box<dyn BackendRequest<S>>>,
    backend_thread_handle: JoinHandle<()>,
) {
    let (rx, signal_end_linker) =
        BackendLink::new("try end event loop", |b: &mut BackendEventLoop<S>| {
            b.signal_stop();
            true
        });
    info!("sending signal to end backend event loop");
    if request_tx.send(Box::new(signal_end_linker)).is_ok() {
        if let Err(e) = rx.recv_timeout(std::time::Duration::from_secs(10)) {
            warn!("did not receive a response after 10 seconds: {e}");
        };
    };
    match backend_thread_handle.join() {
        Ok(_) => info!("backend event loop ended"),
        Err(e) => warn!("failed to signal event loop to stop: {e:?}"),
    }
}
