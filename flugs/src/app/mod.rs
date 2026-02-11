pub mod common;
mod components;
pub mod config;
mod events;
pub mod storage;

use self::components::{Plotter, Search};
use crate::app::events::ConsolidateRequest;
use crate::app::events::EventQueue;
use crate::BackendAppState;
use app_core::backend::BackendRequest;
use config::Config;
use events::{SaveLoadRequested, SavePlotRequested};
use storage::{load_json, save_json};

pub use crate::app::components::FileHandler;
pub use crate::app::components::PlotterMode;

use std::{sync::mpsc::Sender, thread::JoinHandle};

pub type DynRequestSender = Sender<Box<dyn BackendRequest<BackendAppState>>>;

pub struct EguiApp {
    config: Config,
    backend_thread_handle: Option<JoinHandle<()>>,
    file_handler: FileHandler,
    plotter: Plotter,
    request_tx: DynRequestSender,
    search: Search,
    shortcuts_modal_open: bool,
    ui_selection: UISelection,
    event_queue: EventQueue<Self>,
    request_redraw: Option<()>,
}

#[derive(Debug, PartialEq, Eq)]
enum UISelection {
    Plot,
    FileSettings,
    Preferences,
}

impl UISelection {
    fn next(&self) -> Self {
        match self {
            UISelection::Plot => Self::FileSettings,
            UISelection::FileSettings => Self::Plot,
            UISelection::Preferences => Self::Plot,
        }
    }
}

impl EguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        config: Config,
        request_tx: Sender<Box<dyn BackendRequest<BackendAppState>>>,
        backend_thread_handle: JoinHandle<()>,
    ) -> Self {
        let mut search = Search::new(request_tx.clone());
        search.set_search_path(&config.search_path);

        Self {
            config,
            backend_thread_handle: Some(backend_thread_handle),
            file_handler: Default::default(),
            plotter: Plotter::new(),
            request_tx,
            search,
            shortcuts_modal_open: false,
            ui_selection: UISelection::Plot,
            event_queue: EventQueue::<Self>::new(),
            request_redraw: None,
        }
    }

    fn reset_state(&mut self) {
        self.file_handler = Default::default();
        self.event_queue.discard_events();
    }

    fn update_state(&mut self) {
        self.run_events();
        if self.file_handler.try_update() || self.search.try_update() {
            self.request_redraw();
        }
    }

    pub fn request_redraw(&mut self) {
        self.request_redraw = Some(());
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(_) = self.request_redraw.take() {
            ctx.request_repaint();
        }
        // ctx.request_repaint_after(std::time::Duration::from_millis(300));

        self.update_state();

        let mut should_quit = false;

        // Handle keyboard input.
        ctx.input(|i| {
            // Help window.
            if i.key_pressed(egui::Key::F1) {
                self.shortcuts_modal_open = !self.shortcuts_modal_open;
            }
            // Circle main window view.
            if i.key_pressed(egui::Key::F3) {
                self.ui_selection = self.ui_selection.next();
            }
            // Circle mode.
            if i.key_pressed(egui::Key::F4) {
                self.plotter.mode = self.plotter.mode.next();
            }
            // Quick save app state.
            if i.key_pressed(egui::Key::F6) {
                if let Err(error) = save_json(self, None) {
                    log::error!("{}", error)
                };
            }
            // Quick load app state.
            if i.key_pressed(egui::Key::F5) {
                if let Err(error) = load_json(self, None) {
                    log::error!("{}", error)
                };
            }
            // Close app.
            if i.key_pressed(egui::Key::F10) {
                // Quitting cannot be requested from within here, the UI stops,
                // but not the backend thread.
                should_quit = true;
            }
            // Open preferences.
            if i.key_pressed(egui::Key::F12) {
                self.ui_selection = UISelection::Preferences;
            }
            if i.key_pressed(egui::Key::S) && i.modifiers.ctrl {
                log::debug!("open dialog to select save path");
                let handle = std::thread::spawn(|| rfd::FileDialog::new().save_file());
                let event = SaveLoadRequested::new(true, Some(handle));
                self.event_queue.queue_event(Box::new(event));
            }
            if i.key_pressed(egui::Key::L) && i.modifiers.ctrl {
                log::debug!("open dialog to select load path");
                let handle = std::thread::spawn(|| rfd::FileDialog::new().pick_file());
                let event = SaveLoadRequested::new(false, Some(handle));
                self.event_queue.queue_event(Box::new(event));
            }
            if i.key_pressed(egui::Key::P) && i.modifiers.ctrl {
                log::debug!("open dialog to select svg plot path");
                let handle = std::thread::spawn(|| rfd::FileDialog::new().save_file());
                let event = SavePlotRequested::new(Some(handle));
                self.event_queue.queue_event(Box::new(event));
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
        let should_add_files = self.search.render(&mut self.request_tx, ui, ctx);
        if should_add_files {
            self.file_handler
                .add_search_results(&mut self.search, &mut self.request_tx);
        }

        use UISelection as U;
        match self.ui_selection {
            U::Plot => self.plotter.render(&mut self.file_handler, ui, ctx),
            U::FileSettings => {
                self.file_handler
                    .render(&mut self.request_tx, &mut self.event_queue, ui, ctx)
            }
            U::Preferences => {
                self.config.render(ctx, ui);
            }
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::menu::bar(ui, |ui| {
            {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        log::debug!("open dialog to select save path");
                        let handle = std::thread::spawn(|| rfd::FileDialog::new().save_file());
                        let event = SaveLoadRequested::new(true, Some(handle));
                        self.event_queue.queue_event(Box::new(event));
                    }
                    if ui.button("Load").clicked() {
                        log::debug!("open dialog to select load path");
                        let handle = std::thread::spawn(|| rfd::FileDialog::new().pick_file());
                        let event = SaveLoadRequested::new(false, Some(handle));
                        self.event_queue.queue_event(Box::new(event));
                    }
                    if ui.button("Quick Save").clicked() {
                        if let Err(error) = save_json(self, None) {
                            log::error!("{}", error)
                        };
                    }
                    if ui.button("Quick Load").clicked() {
                        // We can do the loading on the main thread, because
                        // files (the only thing that takes time) are loaded on
                        // the backend anyway.
                        if let Err(error) = load_json(self, None) {
                            log::error!("{}", error)
                        };
                    }
                    if ui.button("Preferences").clicked() {
                        self.ui_selection = UISelection::Preferences
                    };
                    if ui.button("Reset Session").clicked() {
                        self.reset_state();
                    };
                    if ui.button("Consolidate Files").clicked() {
                        log::debug!("open dialog to select consolidation path");
                        let handle = std::thread::spawn(|| rfd::FileDialog::new().pick_folder());
                        let event = ConsolidateRequest::new(Some(handle));
                        self.event_queue.queue_event(Box::new(event));
                    };
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Selection of ui view.
                ui.menu_button("View", |ui| {
                    ui.selectable_value(&mut self.ui_selection, UISelection::Plot, "Plot");
                    ui.selectable_value(
                        &mut self.ui_selection,
                        UISelection::FileSettings,
                        "File Settings",
                    );
                });

                let mode_button_label = format!(
                    "Mode ({})",
                    match self.plotter.mode {
                        PlotterMode::Display => "D",
                        PlotterMode::Integrate => "I",
                        PlotterMode::Annotage => "A",
                    },
                );
                ui.menu_button(mode_button_label, |ui| {
                    ui.selectable_value(
                        &mut self.plotter.mode,
                        crate::app::PlotterMode::Display,
                        "Display Plots",
                    );
                    ui.selectable_value(
                        &mut self.plotter.mode,
                        crate::app::PlotterMode::Integrate,
                        "Integrate",
                    );
                    ui.selectable_value(
                        &mut self.plotter.mode,
                        crate::app::PlotterMode::Annotage,
                        "Annotate",
                    );
                });

                if ui.button("Export").clicked() {
                    log::debug!("open dialog to select svg plot path");
                    let handle = std::thread::spawn(|| {
                        rfd::FileDialog::new().set_file_name("plot.svg").save_file()
                    });
                    let event = SavePlotRequested::new(Some(handle));
                    self.event_queue.queue_event(Box::new(event));
                };

                ui.toggle_value(&mut self.shortcuts_modal_open, "Help (F1)");

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
                    ui.label("CTRL + S = Open Save Dialog");
                    ui.separator();
                    ui.label("CTRL + L = Open Load Dialog");
                    ui.separator();
                    ui.label("F1 = Show Keyboard Shortcuts");
                    ui.separator();
                    ui.label("F3 = Cycle View");
                    ui.separator();
                    ui.label("F4 = Cycle Mode");
                    ui.separator();
                    ui.label("F6 = Save App State");
                    ui.separator();
                    ui.label("F5 = Load App State");
                    ui.separator();
                    ui.label("F10 = Quit App");
                    ui.separator();
                    ui.label("F12 = Open Preferences");
                    ui.separator();
                })
                .should_close()
        {
            self.shortcuts_modal_open = false;
        };
    }
}
