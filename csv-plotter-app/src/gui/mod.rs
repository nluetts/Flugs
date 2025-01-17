mod components;

use self::components::Search;
use crate::file_handling::FileHandler;
use crate::BackendAppState;

use std::{sync::mpsc::Sender, thread::JoinHandle};

use app_core::backend::BackendRequest;

pub type DynRequestSender = Sender<Box<dyn BackendRequest<BackendAppState>>>;

pub struct EguiApp {
    backend_thread_handle: Option<JoinHandle<()>>,
    file_handler: FileHandler,
    search: Search,
    request_tx: DynRequestSender,
    ui_selection: UISelection,
}

#[derive(Debug, PartialEq, Eq)]
enum UISelection {
    Plot,
    FileSettings,
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        request_tx: Sender<Box<dyn BackendRequest<BackendAppState>>>,
        backend_thread_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            backend_thread_handle: Some(backend_thread_handle),
            file_handler: Default::default(),
            search: Default::default(),
            request_tx,
            ui_selection: UISelection::Plot,
        }
    }

    fn update_state(&mut self) {
        // self.file_handler.try_update();
        self.search.try_update();
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_state();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.menu(ui, ctx);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui, ctx);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(handle) = self.backend_thread_handle.take() {
            app_core::backend::request_stop(&self.request_tx, handle);
        }
    }
}

impl EguiApp {
    fn central_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.heading("PlotMe CSV Plotter")
        });

        let search_results = self.search.render(&mut self.request_tx, ui, ctx);
        if !search_results.is_empty() {
            self.file_handler.handle_search_results(search_results);
            log::info!("file handler updated: {:?}", self.file_handler);
        }

        use UISelection as U;
        match self.ui_selection {
            U::Plot => self.plot(ui, ctx),
            U::FileSettings => self.file_handler.render_groups(ui, ctx),
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::menu::bar(ui, |ui| {
            {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // selection of ui view
                ui.menu_button("View", |ui| {
                    ui.selectable_value(&mut self.ui_selection, UISelection::Plot, "Plot");
                    ui.selectable_value(
                        &mut self.ui_selection,
                        UISelection::FileSettings,
                        "File Settings",
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            };
        });
    }

    fn plot(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        use egui_plot::Plot;
        Plot::new("Plot").show(ui, |_plot_ui| {});
    }
}
