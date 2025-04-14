use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use app_core::frontend::UIParameter;
use egui::{text::LayoutJob, Color32, FontId, InputState, Label, Pos2, TextFormat};

use crate::{app::DynRequestSender, backend_state::PlotData};

use super::SearchMode;

impl super::Search {
    pub fn render(
        &mut self,
        request_tx: &mut DynRequestSender,
        _ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) -> bool {
        // Sense if search shortcut was pressed.
        if ctx.input(|i| i.modifiers.command && i.key_released(egui::Key::Space)) {
            self.mode = match self.mode {
                SearchMode::Disabled => SearchMode::KeyboardInput,
                _ => SearchMode::Disabled,
            };
        }

        if self.mode == SearchMode::Disabled {
            return false;
        }

        let screen_width = ctx.screen_rect().width();
        let screen_height = ctx.screen_rect().height();
        let search_text_width = 800.0;

        // prepare search popup
        let draw_area = egui::Area::new("modal_area".into()).current_pos(Pos2::from((
            (screen_width - search_text_width) * 0.5,
            screen_height * 0.2,
        )));

        // Handle arrow keys for selecting entries.
        ctx.input(|i| {
            let n_matches = self.matches.value().len();
            if i.key_released(egui::Key::ArrowDown) {
                self.mode = SearchMode::KeyboardSelection;
                self.selected_match = match self.selected_match {
                    Some(n) if n < n_matches - 1 => Some(n + 1),
                    Some(n) if n == n_matches - 1 => Some(n),
                    None => Some(0),
                    _ => Some(n_matches - 1),
                }
            };
            if i.key_released(egui::Key::ArrowUp) {
                self.selected_match = match self.selected_match {
                    Some(0) => {
                        self.mode = SearchMode::KeyboardInput;
                        None
                    }
                    Some(n) if n <= n_matches => {
                        self.mode = SearchMode::KeyboardSelection;
                        Some(n - 1)
                    }
                    _ => None,
                }
            };
        });

        // Holds all of the search UI.
        let modal = egui::Modal::new("search_popup".into()).area(draw_area);

        // Declaration of search UI.
        let modal_ui = |ui: &mut egui::Ui| {
            // UI for search path loading and updating.
            ui.add_enabled_ui(self.search_path.is_up_to_date(), |ui| {
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

            if phrase_input.hovered() && ctx.input(|i| i.pointer.is_moving()) {
                self.mode = SearchMode::KeyboardInput;
            }

            match self.mode {
                SearchMode::KeyboardInput => phrase_input.request_focus(),
                _ => phrase_input.surrender_focus(),
            }

            if phrase_input.changed() {
                self.query_current_path(request_tx);
            };

            // Render the matched file list.
            ui.add_enabled_ui(self.matches.is_up_to_date(), |ui| {
                self.matches_ui(ui, phrase_input, ctx);
            });
        };

        // Show the search UI.
        let modal_response = modal.show(ctx, modal_ui);

        if modal_response.should_close() || ctx.input(|i| i.key_released(egui::Key::Escape)) {
            self.mode = SearchMode::Disabled;
        };

        if ctx.input(|i| i.key_released(egui::Key::Enter)) {
            self.mode = SearchMode::Disabled;
            // Count number of matches which were assigned to a group.
            let number_added: usize = self
                .matches
                .value_mut()
                .iter()
                .enumerate()
                .filter_map(|(i, mtch)| mtch.assigned_group.map(|_m| i))
                .sum();
            log::debug!("added {} paths to load", number_added);
            return true;
        };
        false
    }

    fn matches_ui(&mut self, ui: &mut egui::Ui, phrase_input: egui::Response, ctx: &egui::Context) {
        let width = 800.0;
        let height = 600.0;

        let scroll_area = |ui: &mut egui::Ui| {
            for (
                match_no,
                super::Match {
                    path: fp,
                    matched_indices: indices,
                    assigned_group: group_id,
                    parsed_data: csv_data,
                },
            ) in self.matches.value_mut().iter_mut().enumerate()
            {
                if indices.is_empty() {
                    break;
                }

                let path_label = Label::new(render_match_label(fp, indices, group_id)).wrap();
                // This cursor will be used when hovering a label.
                let mut cursor = egui::CursorIcon::Default;

                // Fancy hover ui for each match.
                let hover_ui = |ui: &mut egui::Ui| {
                    match_hover_ui(ui, csv_data, &mut cursor, fp, &self.search_path);
                };

                // Render the matched entry.
                let resp = ui.add(path_label);

                if resp.hovered() && ctx.input(|i| i.pointer.is_moving()) {
                    self.mode = SearchMode::MouseSelection;
                    self.selected_match = Some(match_no);
                }

                // Draw hover ui depending on mode (keyboard or mouse selection).
                if Some(match_no) == self.selected_match {
                    let resp = resp.highlight();
                    let resp = match self.mode {
                        SearchMode::KeyboardSelection => {
                            resp.show_tooltip_ui(hover_ui);
                            resp
                        }
                        SearchMode::MouseSelection => resp.on_hover_ui_at_pointer(hover_ui),
                        _ => resp,
                    };

                    // Adapt cursor to whether or not match entry was parsed.
                    resp.on_hover_cursor(cursor);

                    // Assign group based on number button presses.
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
                }
            }
        };

        egui::ScrollArea::vertical()
            .min_scrolled_height(height)
            .max_width(width)
            .max_height(height)
            .show(ui, scroll_area);
    }
}

fn match_hover_ui(
    ui: &mut egui::Ui,
    csv_data: &mut super::ParsedData,
    cursor: &mut egui::CursorIcon,
    fp: &mut std::path::PathBuf,
    search_path: &UIParameter<PathBuf>,
) {
    ui.set_min_width(300.0);
    match csv_data {
        super::ParsedData::Failed(msg) => {
            let msg = format!("could not parse this file:\n{}", msg);
            let txt = egui::RichText::new(msg).color(egui::Color32::RED);
            ui.label(txt);
            *cursor = egui::CursorIcon::NotAllowed;
        }
        super::ParsedData::None => match PlotData::from_path(&search_path.value().join(fp)) {
            Ok(data) => *csv_data = super::ParsedData::Ok(data),
            Err(err) => *csv_data = super::ParsedData::Failed(err.to_string()),
        },
        // If data was parsed, we show a mini plot.
        super::ParsedData::Ok(csv_data) => {
            ui.label("press '0' to '9' to add to group, <enter> to accept");
            ui.separator();
            ui.label("preview:");
            egui_plot::Plot::new("Plot")
                .view_aspect(4.0 / 3.0)
                .show_axes(false)
                .show(ui, |plot_ui| {
                    plot_ui.line(egui_plot::Line::new(csv_data.get_cache().data.to_owned()));
                });
            *cursor = egui::CursorIcon::PointingHand;
        }
    }
}

fn render_match_label(
    fp: &mut Path,
    indices: &mut HashSet<usize>,
    group_id: &Option<usize>,
) -> LayoutJob {
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

    // Add group label, if applicable.
    let fmt = TextFormat::simple(FontId::default(), Color32::RED);
    match group_id {
        Some(1) => label_text.append("１ ", 2.0, fmt),
        Some(2) => label_text.append("２ ", 2.0, fmt),
        Some(3) => label_text.append("３ ", 2.0, fmt),
        Some(4) => label_text.append("４ ", 2.0, fmt),
        Some(5) => label_text.append("５ ", 2.0, fmt),
        Some(6) => label_text.append("６ ", 2.0, fmt),
        Some(7) => label_text.append("７ ", 2.0, fmt),
        Some(8) => label_text.append("８ ", 2.0, fmt),
        Some(9) => label_text.append("９ ", 2.0, fmt),
        Some(0) => label_text.append("０ ", 2.0, fmt),
        Some(_) => {}
        None => {}
    };

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
