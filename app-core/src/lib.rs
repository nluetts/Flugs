#![warn(clippy::all, rust_2018_idioms)]

pub mod backend;
pub mod frontend;

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use env_logger;
    use log::trace;

    use crate::backend::{request_stop, BackendEventLoop, BackendLink, BackendState};

    struct TestState {}
    impl BackendState for TestState {}

    #[test]
    fn test_cancel_request_working() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (request_tx, request_rx) = std::sync::mpsc::channel();
        let backend_state = TestState {};
        let eventloop_handle = BackendEventLoop::new(request_rx, backend_state).run();

        let tic = Instant::now();

        let (rx, linker) = BackendLink::new("test", |_| {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        });

        // dropping rx should make the request invalid, such that the backend
        // action (waiting for 1 s) is not executed ...
        drop(rx);
        trace!("drop of receiver done");
        assert!(linker.is_cancelled());
        request_tx.send(Box::new(linker)).unwrap();
        // (this joins the thread handle of the event loop, making it block
        // for as long as the backend action takes, i.e. at least 50 ms)
        request_stop(&request_tx, eventloop_handle);
        let delta_time = (Instant::now() - tic).as_millis();
        // ... thus this whole process here should take much less than 50 ms
        assert!(delta_time < 50);
    }
}
