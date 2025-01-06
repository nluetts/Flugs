use crate::BackendAppState;
use log::{info, warn};

use std::{path::PathBuf, sync::mpsc::Sender, thread::JoinHandle};

use app_core::backend::{BackendEventLoop, BackendLink, BackendRequest};
use app_core::frontend::UIParameter;

pub struct EguiApp {
    read_current_child_paths: UIParameter<()>,
    matched_paths: UIParameter<Vec<PathBuf>>,
    search_query: String,
    request_tx: Sender<Box<dyn BackendRequest<BackendAppState>>>,
    backend_thread_handle: Option<JoinHandle<()>>,
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        request_tx: Sender<Box<dyn BackendRequest<BackendAppState>>>,
        backend_thread_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            read_current_child_paths: UIParameter::new(()),
            matched_paths: UIParameter::new(Vec::new()),
            search_query: String::new(),
            request_tx,
            backend_thread_handle: Some(backend_thread_handle),
        }
    }
    fn update_parameters(&mut self) {
        self.matched_paths.try_update();
        self.read_current_child_paths.try_update();
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after_secs(0.1);
        self.update_parameters();
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
            ui.heading("Overengineered `fuzzy` Command");

            let read_current_ui_enabled = self.read_current_child_paths.is_up_to_date();
            ui.add_enabled_ui(read_current_ui_enabled, |ui| {
                if ui.button("Read Child Paths").clicked() {
                    self.request_current_child_paths();
                }
            });
            if ui.text_edit_singleline(&mut self.search_query).changed() {
                self.query_current_path();
            };
            let paths_ui_enabled = self.matched_paths.is_up_to_date();
            ui.add_enabled_ui(paths_ui_enabled, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for fp in self.matched_paths.value() {
                            ui.label(fp.to_string_lossy());
                        }
                    });
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let (rx, signal_end_linker) = BackendLink::new(
            "try end event loop",
            |b: &mut BackendEventLoop<BackendAppState>| {
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
impl EguiApp {
    fn request_current_child_paths(&mut self) {
        let (rx, linker) = BackendLink::new(
            "request child paths",
            |b: &mut BackendEventLoop<BackendAppState>| {
                b.state.update_current_path_children();
            },
        );
        self.read_current_child_paths.set_recv(rx);
        self.request_tx
            .send(Box::new(linker))
            .expect("backend thread hung up");
    }

    fn query_current_path(&mut self) {
        let query = self.search_query.to_owned();
        let (rx, linker) = BackendLink::new(
            "fuzzy match child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| b.state.fuzzy_filter(&query),
        );
        self.matched_paths.set_recv(rx);
        self.request_tx
            .send(Box::new(linker))
            .expect("backend thread hung up");
    }
}
