use crate::gui::DynRequestSender;

impl super::Search {
    pub fn render(
        &mut self,
        request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
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

        self.fuzzy_search_ui(request_tx, ui, ctx);
    }

    fn fuzzy_search_ui(
        &mut self,
        request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        use egui::{Color32, FontId};

        let read_current_ui_enabled = self.read_current_child_paths.is_up_to_date();
        ui.add_enabled_ui(read_current_ui_enabled, |ui| {
            if ui.button("Read Child Paths").clicked() {
                self.request_current_child_paths(request_tx);
            }
        });
        let phrase_input = ui.text_edit_singleline(&mut self.search_query);
        if phrase_input.changed() {
            self.query_current_path(request_tx);
        };
        let paths_ui_enabled = self.matched_paths.is_up_to_date();

        let scroll_area = |ui: &mut egui::Ui| {
            let style_red = egui::TextFormat::simple(FontId::default(), Color32::RED);
            let style_white = egui::TextFormat::default();
            for (fp, indices) in self.matched_paths.value() {
                if indices.is_empty() {
                    break;
                }
                let fp_str = fp.to_string_lossy();
                let fp_len = fp_str.len();
                let mut prev_ismatch = indices.contains(&0);
                let (mut start, mut end) = (0, 0);
                let mut text = egui::text::LayoutJob::default();
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

                if ui.label(text).interact(egui::Sense::hover()).hovered() {
                    // if we hover a file path, we loose focus on search phrase
                    // input so we do not put in the following keyboard events
                    // as search phrase
                    phrase_input.surrender_focus();
                    if ctx.input(|i| i.key_released(egui::Key::Num1)) {
                        self.request_load_file(fp, request_tx);
                    }
                }
            }
        };

        ui.add_enabled_ui(paths_ui_enabled, |ui| {
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, scroll_area);
        });
    }
}
