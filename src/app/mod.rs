pub mod linker;
mod ui_parameter;

use std::sync::mpsc::Sender;

use crate::{BackendLink, BackendRequest};
use ui_parameter::UIParameter;

pub struct App {
    counter: UIParameter<i64>,
    request_tx: Sender<Box<dyn BackendRequest>>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        request_tx: Sender<Box<dyn BackendRequest>>,
    ) -> Self {
        Self {
            counter: UIParameter::new(0),
            request_tx,
        }
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
            ui.heading("Overengineered, Slow Counter");

            self.counter.try_update();
            let ui_enabled = self.counter.is_up_to_date();
            let increment_button = egui::Button::new("Increment");
            let decrement_button = egui::Button::new("Decrement");
            if ui.add_enabled(ui_enabled, increment_button).clicked() {
                let (tx, rx) = std::sync::mpsc::channel();
                self.counter.set_recv(rx);
                let linker = BackendLink::new(
                    tx,
                    |b| {
                        b.increment_counter();
                        b.counter_value()
                    },
                    "increment counter".to_string(),
                );
                self.request_tx
                    .send(Box::new(linker))
                    .expect("Trying to send value via closed channel.");
            }
            ui.label(format!("counter {}", self.counter.value()));
            if ui.add_enabled(ui_enabled, decrement_button).clicked() {
                let (tx, rx) = std::sync::mpsc::channel();
                self.counter.set_recv(rx);
                let linker = BackendLink::new(
                    tx,
                    |b| {
                        b.decrement_counter();
                        b.counter_value()
                    },
                    "decrement counter".to_string(),
                );
                self.request_tx
                    .send(Box::new(linker))
                    .expect("Trying to send value via closed channel.");
            }
        });
    }
}
