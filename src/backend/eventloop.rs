use std::sync::mpsc::Receiver;

use log::info;

use crate::BackendRequest;

pub struct BackendEventLoop {
    counter: i64,
    request_rx: Receiver<Box<dyn BackendRequest>>,
    should_stop: bool,
}

impl BackendEventLoop {
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
    pub fn new(command_rx: Receiver<Box<dyn BackendRequest>>) -> Self {
        info!("creating new event loop");
        Self {
            counter: 0,
            request_rx: command_rx,
            should_stop: false,
        }
    }
    pub fn increment_counter(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.counter += 1;
    }
    pub fn decrement_counter(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.counter -= 1;
    }
    pub fn counter_value(&self) -> i64 {
        self.counter
    }
}
