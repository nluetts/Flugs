use std::sync::mpsc::Receiver;

use log::info;

use crate::BackendRequest;
use crate::BackendState;

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
            // info!("next backend event loop iteration");
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
