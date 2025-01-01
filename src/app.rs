use log::warn;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct App {
    counter: UIParameter<i64>,
    request_tx: Sender<Box<dyn BackendRequest>>,
}

struct UIParameter<T> {
    pending_update_rx: Option<Receiver<T>>,
    value: T,
}

/// The linker is send to the backend thread and replies
/// once the action ran on the backend.
struct Linker<T, F>
where
    F: Fn(&mut Backend) -> T,
{
    backchannel: Sender<T>,
    action: F,
}

impl<T, F> Linker<T, F>
where
    F: Fn(&mut Backend) -> T,
{
    fn new(backchannel: Sender<T>, action: F) -> Self {
        Self {
            backchannel,
            action,
        }
    }
}

trait BackendRequest {
    fn run_on_backend(self, backend: &mut Backend);
}

impl<T, F> BackendRequest for Linker<T, F>
where
    F: Fn(&mut Backend) -> T,
{
    fn run_on_backend(self, backend: &mut Backend) {
        let result = (self.action)(backend);
        self.backchannel
            .send(result)
            .expect("The Backchannel is open to receive answers.")
    }
}

pub struct Backend {
    value: i64,
}

impl Backend {
    fn increase_counter(&mut self) {
        self.value += 1;
    }
    fn decrease_counter(&mut self) {
        self.value -= 1;
    }
}

impl<T> UIParameter<T> {
    fn new(val: T) -> Self {
        UIParameter {
            pending_update_rx: None,
            value: val,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        request_tx: Sender<Box<dyn BackendRequest>>,
    ) -> Self {
        Self {
            counter: UIParameter::new(0),
            request_tx,
        }
    }

    fn try_update_counter(&mut self) -> bool {
        let ui_enabled;
        match &self.counter.pending_update_rx {
            Some(rx) => match rx.try_recv() {
                Ok(val) => {
                    self.counter.value = val;
                    self.counter.pending_update_rx = None;
                    ui_enabled = true
                }
                Err(err) => match err {
                    TryRecvError::Empty => ui_enabled = false,
                    TryRecvError::Disconnected => {
                        warn!("Tried to receive message from closed channel.");
                        self.counter.pending_update_rx = None;
                        ui_enabled = true;
                    }
                },
            },
            None => ui_enabled = true,
        }
        ui_enabled
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("overengineered counter");

            let ui_enabled = self.try_update_counter();
            let increment_button = egui::Button::new("Increment");
            let decrement_button = egui::Button::new("Decrement");
            if ui.button("Increment").clicked() {
                let (tx, rx) = std::sync::mpsc::channel();
                self.counter.pending_update_rx = Some(rx);
                let linker = Linker::new(tx, |b| {
                    b.increase_counter();
                    b.value
                });
                self.request_tx
                    .send(Box::new(linker))
                    .expect("Trying to send value via closed channel.");
            }
            if ui.button("Decrement").clicked() {
                let (tx, rx) = std::sync::mpsc::channel();
                self.counter.pending_update_rx = Some(rx);
                let linker = Linker::new(tx, |b| {
                    b.decrease_counter();
                    b.value
                });
                self.request_tx
                    .send(Box::new(linker))
                    .expect("Trying to send value via closed channel.");
            }
            ui.add_enabled(ui_enabled, increment_button);
            ui.label(format!("counter {}", self.counter.value));
            ui.add_enabled(ui_enabled, decrement_button);
        });
    }
}
