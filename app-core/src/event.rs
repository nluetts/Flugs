#![allow(unused)]

/// Defines what an event in the app must be able to do:
/// It must be able to apply the event which possibly changes
/// the app state.
pub trait AppEvent {
    type App;
    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String>;
}

/// Events can be finished or busy, after `apply`ing them. Busy events are those
/// which run in background threads because they would otherwise block the UI.
pub enum EventState {
    Busy,
    Finished,
}
