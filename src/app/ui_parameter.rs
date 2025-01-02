use std::sync::mpsc::{Receiver, TryRecvError};

use log::warn;

pub(super) struct UIParameter<T> {
    pending_update_rx: Option<Receiver<T>>,
    value: T,
}

impl<T: Clone> UIParameter<T> {
    pub(super) fn new(val: T) -> Self {
        UIParameter {
            pending_update_rx: None,
            value: val,
        }
    }

    pub(super) fn try_update(&mut self) {
        if let Some(rx) = &self.pending_update_rx {
            match rx.try_recv() {
                Ok(val) => {
                    self.value = val;
                    self.pending_update_rx = None;
                }
                Err(err) => match err {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected => {
                        warn!("Tried to receive message from closed channel.");
                        self.pending_update_rx = None;
                    }
                },
            }
        }
    }

    pub(super) fn is_up_to_date(&self) -> bool {
        self.pending_update_rx.is_none()
    }

    pub(super) fn set_recv(&mut self, rx: Receiver<T>) {
        self.pending_update_rx = Some(rx);
    }

    pub(super) fn value(&self) -> T {
        self.value.clone()
    }
}
