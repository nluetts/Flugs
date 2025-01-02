#[derive(Debug)]
pub enum BackendResponse {
    _OK,
    _Error,
    CounterIncreased(i64),
    CounterDecreased(i64),
    CounterValue(i64),
}
