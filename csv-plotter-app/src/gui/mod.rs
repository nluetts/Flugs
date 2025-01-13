mod components;

use self::components::Search;
use crate::file_handling::FileHandler;
use crate::BackendAppState;

use std::{sync::mpsc::Sender, thread::JoinHandle};

use app_core::backend::BackendRequest;

pub type DynRequestSender = Sender<Box<dyn BackendRequest<BackendAppState>>>;

pub struct EguiApp {
    backend_thread_handle: Option<JoinHandle<()>>,
    _file_handler: FileHandler,
    search: Search,
    request_tx: DynRequestSender,
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        request_tx: Sender<Box<dyn BackendRequest<BackendAppState>>>,
        backend_thread_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            backend_thread_handle: Some(backend_thread_handle),
            _file_handler: Default::default(),
            search: Default::default(),
            request_tx,
        }
    }
    fn update_state(&mut self) {
        // self.file_handler.try_update();
        self.search.try_update();
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ctx.request_repaint_after_secs(0.1);
        self.update_state();
        // ctx.show_viewport_deferred(1.into(), egui::ViewportBuilder::default(), |ui| ui.lab);
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
            self.search.render(&mut self.request_tx, ui, ctx);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(handle) = self.backend_thread_handle.take() {
            app_core::backend::request_stop(&self.request_tx, handle);
        }
    }
}
