use std::{collections::HashSet, path::Path};

use egui::{text::LayoutJob, Color32, FontId, InputState, Label, Pos2, TextFormat};

use crate::{app::DynRequestSender, backend_state::CSVData};

impl super::Search {
    pub fn render(
        &mut self,
        request_tx: &mut DynRequestSender,
        _ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) -> HashSet<super::Match> {
        let mut popup_opened_this_frame = false;
        // sense if search shortcut was pressed
        if ctx.input(|i| i.modifiers.command && i.key_released(egui::Key::Space)) {
            self.popup_shown = !self.popup_shown;
            if self.popup_shown {
                popup_opened_this_frame = true;
            }
        }

        if !self.popup_shown {
            return HashSet::new();
        }

        let screen_width = ctx.screen_rect().width();
        let screen_height = ctx.screen_rect().height();
        let search_text_width = 800.0;

        // prepare search popup
        let draw_area = egui::Area::new("modal_area".into()).current_pos(Pos2::from((
            (screen_width - search_text_width) * 0.5,
            screen_height * 0.2,
        )));

        let modal = egui::Modal::new("search_popup".into()).area(draw_area);

        let modal_ui = |ui: &mut egui::Ui| {
            let read_current_ui_enabled = self.search_path.is_up_to_date();

            // UI for search path loading and updating
            ui.add_enabled_ui(read_current_ui_enabled, |ui| {
                ui.label("current search root path:");
                ui.horizontal(|ui| {
                    ui.label(self.search_path.value_mut().to_string_lossy());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("↺")
                            .on_hover_text("re-index search path")
                            .clicked()
                        {
                            let new_path = self.search_path.value_mut().clone();
                            self.set_search_path(&new_path);
                        }
                        if ui
                            .small_button("...")
                            .on_hover_text("change search path")
                            .clicked()
                        {
                            log::debug!("open dialog to select new search path");
                            self.awaiting_search_path_selection =
                                Some(std::thread::spawn(|| rfd::FileDialog::new().pick_folder()));
                        }
                    })
                });
            });

            ui.separator();

            let phrase_input = ui.add(
                egui::TextEdit::singleline(&mut self.search_query).desired_width(search_text_width),
            );
            if popup_opened_this_frame || phrase_input.hovered() {
                phrase_input.request_focus()
            };
            if phrase_input.changed() {
                self.query_current_path(request_tx);
            };

            let paths_ui_enabled = self.matches.is_up_to_date();

            ui.add_enabled_ui(paths_ui_enabled, |ui| {
                self.matches_ui(ui, phrase_input, ctx);
            });
        };

        let modal_response = modal.show(ctx, modal_ui);

        if modal_response.should_close() || ctx.input(|i| i.key_released(egui::Key::Escape)) {
            self.popup_shown = false;
        };

        if ctx.input(|i| i.key_released(egui::Key::Enter)) {
            self.popup_shown = false;
            let to_load: HashSet<super::Match> = self
                .matches
                .value_mut()
                .drain(..)
                .filter(|mtch| mtch.assigned_group.is_some())
                .collect();
            self.search_query.clear();
            log::debug!("returning {} paths to load", to_load.len());
            return to_load;
        };
        HashSet::new()
    }

    fn matches_ui(&mut self, ui: &mut egui::Ui, phrase_input: egui::Response, ctx: &egui::Context) {
        let width = 800.0;
        let height = 600.0;

        let scroll_area = |ui: &mut egui::Ui| {
            for super::Match {
                path: fp,
                matched_indices: indices,
                assigned_group: group_id,
                parsed_data: csv_data,
            } in self.matches.value_mut()
            {
                if indices.is_empty() {
                    break;
                }

                ui.horizontal(|ui| {
                    let path_label = Label::new(render_match_label(fp, indices)).wrap();

                    ui.add(path_label).on_hover_ui_at_pointer(|ui| {
                        ui.label("press '0' to '9' to add to group, <enter> to accept");
                        // If we hover a file path, we loose focus on search phrase
                        // input so we do not put in the following keyboard events
                        // as search phrase; however, if we just opened the search
                        // popup and do not move the mouse we keep the focus.
                        if ctx.input(|i| i.pointer.is_moving()) {
                            phrase_input.surrender_focus()
                        // If we do not move the mouse and did not try yet, we
                        // try parsing the current file.
                        } else if let None = csv_data {
                            if let Ok(data) = CSVData::from_path(&self.search_path.value().join(fp))
                            {
                                *csv_data = Some(data);
                            };
                        } else if let Some(csv_data) = csv_data {
                            ui.separator();
                            ui.label("preview:");
                            egui_plot::Plot::new("Plot")
                                .view_aspect(4.0 / 3.0)
                                .show_axes(false)
                                .show(ui, |plot_ui| {
                                    plot_ui.line(egui_plot::Line::new(
                                        csv_data.get_cache().data.to_owned(),
                                    ));
                                });
                        };

                        let (input_active, numkey_released) =
                            (phrase_input.has_focus(), ctx.input(number_key_released));
                        match (input_active, numkey_released) {
                            (_, None) => (),
                            (true, Some(_)) => (),
                            (false, Some(released_num)) => {
                                // TODO: I bet there is an easier way:
                                if let Some(gid) = group_id.take() {
                                    if released_num != gid {
                                        group_id.replace(released_num);
                                    }
                                } else {
                                    group_id.replace(released_num);
                                }
                            }
                        }
                    });
                    if let Some(grp) = group_id {
                        let text = format!("({})", grp);
                        let text = LayoutJob::simple(text, FontId::default(), Color32::RED, 5.0);
                        ui.add(Label::new(text));
                    }
                });
            }
        };
        egui::ScrollArea::vertical()
            .min_scrolled_height(height)
            .max_width(width)
            .max_height(height)
            .show(ui, scroll_area);
    }
}

fn render_match_label(fp: &mut Path, indices: &mut HashSet<usize>) -> LayoutJob {
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
