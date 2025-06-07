use std::sync::mpsc::TryRecvError;

use log::warn;

use crate::backend::LinkReceiver;

#[derive(Debug)]
pub struct UIParameter<T> {
    pending_update_rx: Option<LinkReceiver<T>>,
    value: T,
}

impl<T: Clone> Clone for UIParameter<T> {
    fn clone(&self) -> Self {
        Self {
            pending_update_rx: None,
            value: self.value.clone(),
        }
    }
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

    pub fn try_update(&mut self) -> bool {
        if let Some(rx) = &self.pending_update_rx {
            match rx.try_recv() {
                Ok(val) => {
                    self.value = val;
                    self.pending_update_rx = None;
                    true
                }
                Err(err) => match err {
                    TryRecvError::Empty => false,
                    TryRecvError::Disconnected => {
                        warn!("Tried to receive message from closed channel.");
                        self.pending_update_rx = None;
                        true
                    }
                },
            }
        } else {
            false
        }
    }

    pub fn is_up_to_date(&self) -> bool {
        self.pending_update_rx.is_none()
    }

    pub fn set_recv(&mut self, rx: LinkReceiver<T>) {
        self.pending_update_rx = Some(rx);
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}
