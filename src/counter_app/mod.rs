pub mod backend_state;
use backend_state::CounterAppState;
use log::{info, warn};

use std::{
    sync::mpsc::{channel, Sender},
    thread::JoinHandle,
};

use crate::{BackendEventLoop, BackendLink, BackendRequest, UIParameter};

pub struct App {
    counter: UIParameter<i64>,
    request_tx: Sender<Box<dyn BackendRequest<CounterAppState>>>,
    backend_thread_handle: Option<JoinHandle<()>>,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        request_tx: Sender<Box<dyn BackendRequest<CounterAppState>>>,
        backend_thread_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            counter: UIParameter::new(0),
            request_tx,
            backend_thread_handle: Some(backend_thread_handle),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after_secs(0.1);
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
            let counter_ui_enabled = self.counter.is_up_to_date();
            ui.add_enabled_ui(counter_ui_enabled, |ui| {
                if ui.button("Increment").clicked() {
                    self.request_increment();
                }
                ui.label(format!("counter {}", self.counter.value()));
                if ui.button("Decrement").clicked() {
                    self.request_decrement();
                }
            })
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let (tx, rx) = channel();
        let signal_end_linker = BackendLink::new(
            tx,
            "try end event loop".to_string(),
            |b: &mut BackendEventLoop<CounterAppState>| {
                b.signal_stop();
                true
            },
        );
        info!("sending signal to end backend event loop");
        if self.request_tx.send(Box::new(signal_end_linker)).is_ok() {
            if let Err(e) = rx.recv_timeout(std::time::Duration::from_secs(10)) {
                warn!("did not receive a response after 10 seconds: {e}");
            };
        };
        if let Some(handle) = self.backend_thread_handle.take() {
            match handle.join() {
                Ok(_) => info!("backend event loop ended"),
                Err(e) => warn!("failed to signal event loop to stop: {e:?}"),
            }
        };
    }
}

/// Define UI events here
impl App {
    fn request_increment(&mut self) {
        let (tx, rx) = channel();
        self.counter.set_recv(rx);
        let linker = BackendLink::new(
            tx,
            "increment counter".to_string(),
            |b: &mut BackendEventLoop<CounterAppState>| {
                b.state.increment_counter();
                b.state.counter_value()
            },
        );
        self.request_tx
            .send(Box::new(linker))
            .expect("Trying to send value via closed channel.");
    }

    fn request_decrement(&mut self) {
        let (tx, rx) = channel();
        self.counter.set_recv(rx);
        let linker = BackendLink::new(
            tx,
            "decrement counter".to_string(),
            |b: &mut BackendEventLoop<CounterAppState>| {
                b.state.decrement_counter();
                b.state.counter_value()
            },
        );
        self.request_tx
            .send(Box::new(linker))
            .expect("Trying to send value via closed channel.");
    }
}
