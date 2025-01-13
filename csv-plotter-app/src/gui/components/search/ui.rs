use egui::TextFormat;

use crate::{file_handling::GroupID, gui::DynRequestSender};

impl super::Search {
    pub fn render(
        &mut self,
        request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        // The central panel the region left after adding TopPanel's and SidePanel's
        let mut header = egui::text::LayoutJob::default();
        let red = TextFormat {
            color: egui::Color32::RED,
            ..Default::default()
        };
        let def = TextFormat {
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
            let style_red = TextFormat::simple(FontId::default(), Color32::RED);
            let style_white = TextFormat::default();
            // TODO: this vec must also hold the group ID?
            let mut to_load = Vec::new();
            for (fp, indices, group_id) in self.matched_paths.value() {
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

                let path_label = ui.label(text);
                if let Some(grp) = group_id {
                    let p = ui.painter_at(path_label.rect);
                    p.add(egui::Shape::rect_filled(
                        path_label.rect,
                        0.1,
                        Color32::DARK_RED.linear_multiply(0.1),
                    ));
                    let text = format!("{}", grp.id());
                    ui.put(path_label.rect, egui::Label::new(&text));
                }
                if path_label.hovered() {
                    // if we hover a file path, we loose focus on search phrase
                    // input so we do not put in the following keyboard events
                    // as search phrase
                    phrase_input.surrender_focus();
                    if ctx.input(|i| i.key_released(egui::Key::Num1)) {
                        if let Some(gid) = group_id {
                            if gid.id() == 1 {
                                // TODO: This should clear the selection
                                *group_id = None;
                            }
                        } else {
                            group_id.replace(GroupID::new(1));
                            to_load.push(fp.to_owned());
                        }
                    }
                }
            }
            to_load
        };

        let response = ui.add_enabled_ui(paths_ui_enabled, |ui| {
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, scroll_area)
        });
        let to_load = response.inner.inner;
        for fp in to_load.into_iter() {
            let rx = self.request_load_file(&fp, request_tx);
            self._requested_loading.push(rx);
        }
    }
}
