use crate::BackendState;

#[derive(Default)]
pub struct CounterAppState {
    counter: i64,
}

impl BackendState for CounterAppState {}

impl CounterAppState {
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
