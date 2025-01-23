mod components;
pub mod storage;

use self::components::{Plotter, Search};
use crate::BackendAppState;
use crate::ROOT_PATH;
use app_core::backend::BackendRequest;
use storage::load_json;
use storage::save_json;

pub use crate::app::components::{FileHandler, GroupID};

use std::path::PathBuf;
use std::time::Duration;
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub type DynRequestSender = Sender<Box<dyn BackendRequest<BackendAppState>>>;

pub struct EguiApp {
    backend_thread_handle: Option<JoinHandle<()>>,
    file_handler: FileHandler,
    plotter: Plotter,
    request_tx: DynRequestSender,
    search: Search,
    shortcuts_modal_open: bool,
    ui_selection: UISelection,
}

#[derive(Debug, PartialEq, Eq)]
enum UISelection {
    Plot,
    FileSettings,
}

impl UISelection {
    fn next(&self) -> Self {
        match self {
            UISelection::Plot => Self::FileSettings,
            UISelection::FileSettings => Self::Plot,
        }
    }
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        mut request_tx: Sender<Box<dyn BackendRequest<BackendAppState>>>,
        backend_thread_handle: JoinHandle<()>,
    ) -> Self {
        // initialize search component with root path and index
        // subpaths
        let mut search = Search::default();
        let search_path = PathBuf::from(ROOT_PATH);
        search.set_search_path(&search_path, &mut request_tx);

        Self {
            backend_thread_handle: Some(backend_thread_handle),
            file_handler: Default::default(),
            plotter: Plotter::new(),
            request_tx,
            search,
            shortcuts_modal_open: false,
            ui_selection: UISelection::Plot,
        }
    }

    fn update_state(&mut self) {
        self.file_handler.try_update();
        self.search.try_update();
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(50));
        self.update_state();

        let mut should_quit = false;

        // Handle keyboard input.
        ctx.input(|i| {
            // Help window.
            if i.key_pressed(egui::Key::F1) {
                self.shortcuts_modal_open = !self.shortcuts_modal_open;
            }
            // Circle main window.
            if i.key_pressed(egui::Key::F3) {
                self.ui_selection = self.ui_selection.next();
            }
            // Quick save app state.
            if i.key_pressed(egui::Key::F5) {
                if let Err(error) = save_json(self) {
                    log::error!("{}", error)
                };
            }
            // Quick load app state.
            if i.key_pressed(egui::Key::F6) {
                if let Err(error) = load_json(self) {
                    log::error!("{}", error)
                };
            }
            // Close app.
            if i.key_pressed(egui::Key::F10) {
                // Quitting cannot be requested from within here, the UI stops,
                // but not the backend thread.
                should_quit = true;
            }
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_shortcut_modal(ctx);
            self.menu(ui, ctx);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui, ctx);
        });

        if should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(handle) = self.backend_thread_handle.take() {
            app_core::backend::request_stop(&self.request_tx, handle);
        }
    }
}

impl EguiApp {
    fn central_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let search_results = self.search.render(&mut self.request_tx, ui, ctx);
        let num_results = search_results.len();
        if !search_results.is_empty() {
            self.file_handler.add_search_results(
                search_results,
                self.search.get_search_path(),
                &mut self.request_tx,
            );
            log::debug!("file handler updated with {} new entries", num_results);
        }

        use UISelection as U;
        match self.ui_selection {
            U::Plot => self.plotter.render(&mut self.file_handler, ui, ctx),
            U::FileSettings => self.file_handler.render(&mut self.request_tx, ui, ctx),
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::menu::bar(ui, |ui| {
            {
                ui.menu_button("File", |ui| {
                    if ui.button("Quick Save (F5)").clicked() {
                        if let Err(error) = save_json(self) {
                            log::error!("{}", error)
                        };
                    }
                    if ui.button("Quick Load (F6)").clicked() {
                        // We can do the loading on the main thread, because
                        // files (the only thing that takes time) are loaded on
                        // the backend anyway.
                        if let Err(error) = load_json(self) {
                            log::error!("{}", error)
                        };
                    }
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

                ui.toggle_value(&mut self.shortcuts_modal_open, "Help");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            };
        });
    }

    fn render_shortcut_modal(&mut self, ctx: &egui::Context) {
        if self.shortcuts_modal_open
            && egui::Modal::new("shortcut_modal".into())
                .show(ctx, |ui| {
                    ui.heading("Keyboard Shortcuts");
                    ui.separator();
                    ui.label("CTRL + Space = Open Search Menu");
                    ui.separator();
                    ui.label("F1 = Show Keyboard Shortcuts");
                    ui.separator();
                    ui.label("F3 = Cycle View");
                    ui.separator();
                    ui.label("F5 = Save App State");
                    ui.separator();
                    ui.label("F6 = Load App State");
                    ui.separator();
                    ui.label("F10 = Quit App");
                    ui.separator();
                })
                .should_close()
        {
            self.shortcuts_modal_open = false;
        };
    }
}
