pub trait AppEvent {
    type App;
    fn apply(&self, app: &mut Self::App) -> Result<(), String>;
}
