use std::sync::mpsc::TryRecvError;

use log::warn;

use crate::backend::LinkReceiver;

pub struct UIParameter<T> {
    pending_update_rx: Option<LinkReceiver<T>>,
    value: T,
}

impl<T: Default + Clone> Default for UIParameter<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> UIParameter<T> {
    pub fn new(val: T) -> Self {
        UIParameter {
            pending_update_rx: None,
            value: val,
        }
    }

    pub fn try_update(&mut self) {
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

    pub fn is_up_to_date(&self) -> bool {
        self.pending_update_rx.is_none()
    }

    pub fn set_recv(&mut self, rx: LinkReceiver<T>) {
        self.pending_update_rx = Some(rx);
    }

    pub fn value(&mut self) -> &mut T {
        &mut self.value
    }
}
