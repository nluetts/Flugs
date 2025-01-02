use std::sync::mpsc::Receiver;

use log::info;

use super::super::app::BackendRequest;

pub struct BackendEventLoop {
    counter: i64,
    request_rx: Receiver<Box<dyn BackendRequest>>,
    stop: bool,
}

impl BackendEventLoop {
    pub fn update(&mut self) -> bool {
        // handle the most important command
        while let Some(request) = self.request_rx.try_recv().ok() {
            info!("handeling request");
            request.run_on_backend(self);
        }
        return self.stop;
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
            stop: false,
        }
    }
    pub fn increase_counter(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.counter += 1;
    }
    pub fn decrease_counter(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.counter -= 1;
    }
    pub fn counter_value(&self) -> i64 {
        self.counter
    }
}
