use std::{collections::HashSet, path::PathBuf};

use egui::{text::LayoutJob, Color32, FontId, InputState, TextFormat};

use crate::{file_handling::GroupID, gui::DynRequestSender};

impl super::Search {
    pub fn render(
        &mut self,
        request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) -> Option<HashSet<PathBuf>> {
        ui.heading("Overengineered Fuzzy Finder");

        // sense if search shortcut was pressed
        if ctx.input(|i| i.modifiers.command && i.key_released(egui::Key::Space)) {
            self.popup_shown = !self.popup_shown;
        }

        if !self.popup_shown {
            return None;
        }

        // what should be loaded is returned from the function
        let mut files_to_load = HashSet::new();

        // modal response creates an "overlay" on top of the UI
        let modal = egui::Modal::new("search_popup".into());

        let modal_ui = |ui: &mut egui::Ui| {
            let read_current_ui_enabled = self.read_current_child_paths.is_up_to_date();

            ui.add_enabled_ui(read_current_ui_enabled, |ui| {
                if ui.button("Read Paths").clicked() {
                    self.request_current_child_paths(request_tx);
                }
            });

            let phrase_input = ui.text_edit_singleline(&mut self.search_query);
            if phrase_input.changed() {
                self.query_current_path(request_tx);
            };

            let paths_ui_enabled = self.matched_paths.is_up_to_date();

            ui.add_enabled_ui(paths_ui_enabled, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .max_width(800.0)
                    .show(ui, |ui: &mut egui::Ui| {
                        self.handle_matches(ui, phrase_input, &mut files_to_load, ctx);
                    })
            });
        };

        let modal_response = modal.show(ctx, modal_ui);

        if modal_response.should_close() || ctx.input(|i| i.key_released(egui::Key::Escape)) {
            self.popup_shown = false;
        };

        if ctx.input(|i| i.key_released(egui::Key::Enter)) {
            self.popup_shown = false;
            log::info!("accepted files to load");
        };

        Some(files_to_load)
    }

    fn handle_matches(
        &mut self,
        ui: &mut egui::Ui,
        phrase_input: egui::Response,
        files_to_load: &mut HashSet<PathBuf>,
        ctx: &egui::Context,
    ) {
        for (fp, indices, group_id) in self.matched_paths.value() {
            if indices.is_empty() {
                break;
            }

            ui.horizontal(|ui| {
                let path_label = ui.label(render_match_label(fp, indices));
                if path_label.hovered() {
                    // if we hover a file path, we loose focus on search phrase
                    // input so we do not put in the following keyboard events
                    // as search phrase
                    phrase_input.surrender_focus();
                    if let Some(released_num) = ctx.input(number_key_released) {
                        // TODO: I bet there is an easier way:
                        if let Some(gid) = group_id.take() {
                            if released_num != gid.id() {
                                group_id.replace(GroupID::new(released_num));
                                files_to_load.insert(fp.to_owned());
                            }
                        } else {
                            group_id.replace(GroupID::new(released_num));
                            files_to_load.insert(fp.to_owned());
                        }
                    }
                }
                if let Some(grp) = group_id {
                    let text = format!("{}", grp.id());
                    ui.label(&text);
                }
            });
        }
    }
}

fn render_match_label(fp: &mut PathBuf, indices: &mut HashSet<usize>) -> LayoutJob {
    let style_red = TextFormat::simple(FontId::default(), Color32::RED);
    let style_white = TextFormat::default();

    let fp_str = fp.to_string_lossy();
    let fp_len = fp_str.len();

    let mut label_text = LayoutJob::default();
    let (mut start, mut end) = (0, 0);
    let mut prev_ismatch = indices.contains(&0);

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
            label_text.append(&fp_str[start..=end], 2.0, format);
            (start, end) = (i, i);
            prev_ismatch = ismatch;
        }

        if i == fp_len - 1 {
            let format = if ismatch {
                style_red.to_owned()
            } else {
                style_white.to_owned()
            };
            label_text.append(&fp_str[start..=i], 2.0, format);
        }
    }
    label_text
}

fn number_key_released(i: &InputState) -> Option<usize> {
    if i.key_released(egui::Key::Num1) {
        return Some(1);
    }
    if i.key_released(egui::Key::Num2) {
        return Some(2);
    }
    if i.key_released(egui::Key::Num3) {
        return Some(3);
    }
    if i.key_released(egui::Key::Num4) {
        return Some(4);
    }
    if i.key_released(egui::Key::Num5) {
        return Some(5);
    }
    if i.key_released(egui::Key::Num6) {
        return Some(6);
    }
    if i.key_released(egui::Key::Num7) {
        return Some(7);
    }
    if i.key_released(egui::Key::Num8) {
        return Some(8);
    }
    if i.key_released(egui::Key::Num9) {
        return Some(9);
    }
    if i.key_released(egui::Key::Num0) {
        return Some(0);
    }
    None
}
