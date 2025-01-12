use std::sync::mpsc::Sender;

use app_core::backend::{BackendEventLoop, BackendLink, BackendRequest};
use egui::{text::LayoutJob, Ui};

use crate::{frontend_state::Search, BackendAppState, EguiApp};

impl Search {
    pub fn render(
        &mut self,
        request_tx: &mut Sender<Box<dyn BackendRequest<BackendAppState>>>,
        ui: &mut Ui,
    ) {
        // The central panel the region left after adding TopPanel's and SidePanel's
        let mut header = egui::text::LayoutJob::default();
        let red = egui::TextFormat {
            color: egui::Color32::RED,
            ..Default::default()
        };
        let def = egui::TextFormat {
            color: egui::Color32::BLUE,
            ..Default::default()
        };
        header.append("Overengineered ", 0.0, def.clone());
        header.append("Fuzzy ", 0.0, red);
        header.append("Search", 0.0, def);
        ui.label(header);

        self.fuzzy_search_ui(request_tx, ui);
    }

    fn fuzzy_search_ui(
        &mut self,
        request_tx: &mut Sender<Box<dyn BackendRequest<BackendAppState>>>,
        ui: &mut egui::Ui,
    ) {
        use egui::{Color32, FontId};

        let read_current_ui_enabled = self.read_current_child_paths_is_up_to_date();
        ui.add_enabled_ui(read_current_ui_enabled, |ui| {
            if ui.button("Read Child Paths").clicked() {
                self.request_current_child_paths(request_tx);
            }
        });
        if ui
            .text_edit_singleline(self.search_query_mut_ref())
            .changed()
        {
            self.query_current_path(request_tx);
        };
        let paths_ui_enabled = self.matched_paths_is_up_to_date();

        let scroll_area = |ui: &mut egui::Ui| {
            let style_red = egui::TextFormat::simple(FontId::default(), Color32::RED);
            let style_white = egui::TextFormat::simple(FontId::default(), Color32::WHITE);
            for (fp, indices) in self.matched_paths_value() {
                if indices.is_empty() {
                    break;
                }
                let fp_str = fp.to_string_lossy();
                let fp_len = fp_str.len();
                let mut prev_ismatch = indices.contains(&0);
                let (mut start, mut end) = (0, 0);
                let mut text = LayoutJob::default();
                for i in 1..fp_len {
                    let ismatch = indices.contains(&i);
                    if prev_ismatch == ismatch {
                        end = i;
                    } else {
                        let format = if prev_ismatch {
                            style_red.to_owned()
                        } else {
                            style_white.to_owned()
                        };
                        text.append(&fp_str[start..=end], 2.0, format);
                        (start, end) = (i, i);
                        prev_ismatch = ismatch;
                    }

                    if i == fp_len - 1 {
                        let format = if ismatch {
                            style_red.to_owned()
                        } else {
                            style_white.to_owned()
                        };
                        text.append(&fp_str[start..=i], 2.0, format);
                    }
                }

                ui.label(text);
            }
        };

        ui.add_enabled_ui(paths_ui_enabled, |ui| {
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, scroll_area);
        });
    }

    fn request_current_child_paths(
        &mut self,
        request_tx: &mut Sender<Box<dyn BackendRequest<BackendAppState>>>,
    ) {
        let (rx, linker) = BackendLink::new(
            "request child paths",
            |b: &mut BackendEventLoop<BackendAppState>| {
                b.state.update_child_paths_unfiltered();
            },
        );
        self.set_recv_read_current_child_paths(rx);
        request_tx
            .send(Box::new(linker))
            .expect("backend thread hung up");
    }

    fn query_current_path(
        &mut self,
        request_tx: &mut Sender<Box<dyn BackendRequest<BackendAppState>>>,
    ) {
        let query = self.search_query_mut_ref().to_owned();
        let (rx, linker) = BackendLink::new(
            "fuzzy match child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| b.state.search_filter(&query),
        );
        self.set_recv_matched_paths(rx);
        request_tx
            .send(Box::new(linker))
            .expect("backend thread hung up unexpectedly");
    }
}
