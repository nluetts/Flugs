use std::fmt::Write;

use egui::{text::LayoutJob, Color32, FontId};

use crate::{
    app::{
        events::{CloneFile, CopyFile, EventQueue, MoveFile, RemoveFile, RemoveGroup},
        DynRequestSender,
    },
    EguiApp,
};

use super::{ActiveElement, File, FileHandler, FileID};

impl FileHandler {
    pub(crate) fn render(
        &mut self,
        _request_tx: &mut DynRequestSender,
        event_queue: &mut EventQueue<EguiApp>,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        for i in 0..10 {
            self.group_name_buffer[i].clear();
            if let Some(grp) = &self.groups[i] {
                self.group_name_buffer[i]
                    .write_str(&grp.name)
                    .expect("Unable to write to string buffer.");
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.heading("Groups and Files")
        });

        let scroll_area = egui::ScrollArea::both();

        let side_panel = egui::panel::SidePanel::left("file_and_group_tree").min_width(300.0);
        side_panel.show(ctx, |ui| {
            scroll_area.show(ui, |ui| {
                self.left_panel(_request_tx, event_queue, ui, ctx);
            })
        });

        let settings_panel = egui::panel::CentralPanel::default();
        settings_panel.show(ctx, |ui| match &self.active_element {
            super::ActiveElement::Group(gid) => {
                self.group_settings(*gid, _request_tx, event_queue, ui, ctx)
            }
            super::ActiveElement::File(fid, gid) => {
                self.file_settings(*fid, *gid, _request_tx, event_queue, ui, ctx)
            }
        });
    }

    pub(crate) fn left_panel(
        &mut self,
        _request_tx: &mut DynRequestSender,
        _event_queue: &mut EventQueue<EguiApp>,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        // Due to some ownership constraints, we have to do a little dance
        // here to get the group names into the `render_file_form` function.
        // (We cannot just pass `self`.)
        // To avoid endless allocations, we use an array of string buffers,
        // which are reused throughout.
        for i in 0..10 {
            self.group_name_buffer[i].clear();
            if let Some(grp) = &self.groups[i] {
                self.group_name_buffer[i]
                    .write_str(&grp.name)
                    .expect("Unable to write to string buffer.");
            }
        }

        ui.heading("Groups");

        for (gid, grp) in self
            .groups
            .iter_mut()
            .enumerate()
            .filter_map(|(id, x)| Some(id).zip(x.as_mut()))
        {
            let grp_label_txt = LayoutJob::simple_singleline(
                grp.name.to_string(),
                FontId::proportional(15.0),
                if ctx.theme() == egui::Theme::Light {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                },
            );
            let collapsing = egui::CollapsingHeader::new(grp_label_txt)
                .id_salt((&grp.name, gid))
                .default_open(false);
            let resp = collapsing.show(ui, |ui| {
                for fid in grp.file_ids.iter() {
                    let file = match self.registry.get_mut(fid) {
                        Some(file) => file,
                        None => {
                            log::error!(
                                "fid {fid:?} has no entry in registry (should never happen)"
                            );
                            continue;
                        }
                    };
                    let file_label_txt = match file_name_layout(file) {
                        Some(value) => value,
                        None => {
                            log::warn!("could not render file label for fid {fid:?}");
                            continue;
                        }
                    };
                    let label = egui::Label::new(file_label_txt).truncate();
                    if ui
                        .add(label)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.active_element = ActiveElement::File(*fid, gid);
                    }
                }
            });

            if resp
                .header_response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text_at_pointer("right-click to rename")
                .secondary_clicked()
            {
                self.active_element = ActiveElement::Group(gid);
            }
        }
    }

    pub(crate) fn group_settings(
        &mut self,
        gid: usize,
        _request_tx: &mut DynRequestSender,
        event_queue: &mut EventQueue<EguiApp>,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
    ) {
        let grp = if let Some(grp) = &mut self.groups[gid] {
            grp
        } else {
            return;
        };
        ui.heading(&grp.name);
        ui.horizontal(|ui| {
            let lab = ui.label("rename:");
            ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
        });
        ui.horizontal(|ui| {
            ui.label("delete group:");
            if ui.small_button("🗑").clicked() {
                event_queue.queue_event(Box::new(RemoveGroup::new(gid)));
            }
        });
    }
    pub fn file_settings(
        &mut self,
        fid: FileID,
        gid: usize,
        _request_tx: &mut DynRequestSender,
        event_queue: &mut EventQueue<EguiApp>,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
    ) {
        let file = match self.registry.get_mut(&fid) {
            Some(file) => file,
            None => {
                log::warn!("Requested file settings for a file ID that is missing in registry.");
                return;
            }
        };

        let id = file.file_name().to_owned();
        ui.push_id(id, |ui| {
            ui.horizontal(|ui| {
                let label = egui::Label::new(file.file_name()).truncate();
                ui.add(label).context_menu(|ui| {
                    if ui.button("copy to alias").clicked() {
                        if let Some(filename) = file.path.file_name() {
                            file.properties.alias = filename.to_string_lossy().into_owned()
                        }
                    }
                });
                // Identifier and delete button.
                ui.label(format!("(ID {})", fid.0));
                if ui.small_button("🗑").clicked() {
                    event_queue.queue_event(Box::new(RemoveFile::new(fid, gid)));
                }
            })
        });

        ui.separator();

        // Display error if csv could not be parsed.
        if let Err(error) = file.data.value() {
            ui.label(error).highlight();
        };

        file.render_property_settings(ui);

        ui.separator();

        egui::CollapsingHeader::new("Metadata Header").show(ui, |ui| {
            egui::ScrollArea::new([true, true])
                .max_height(300.0)
                .max_width(800.0)
                .show(ui, |ui| {
                    let comments = file
                        .data
                        .value()
                        .as_ref()
                        .map(|data| data.get_comments())
                        .unwrap_or_default();
                    if !comments.is_empty() {
                        for line in comments.lines() {
                            ui.add(egui::Label::new(line).wrap_mode(egui::TextWrapMode::Extend));
                        }
                    }
                })
        });

        egui::CollapsingHeader::new("Contents").show(ui, |ui| {
            egui::ScrollArea::new([true, true])
                .max_height(300.0)
                .max_width(800.0)
                .show(ui, |ui| {
                    let Ok(data) = file.data.value() else { return };
                    let Some(xs) = data.columns.first() else {
                        return;
                    };
                    let Some(ys) = data.columns.iter().nth(1) else {
                        return;
                    };
                    for (x, y) in xs.iter().zip(ys) {
                        ui.add(
                            egui::Label::new(format!("{x}, {y}"))
                                .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    }
                })
        });

        #[derive(PartialEq)]
        enum FileAction {
            Clone(usize),
            Copy(usize),
            Move(usize),
        }

        // Menu to move/copy file to other group.
        let mut action: Option<FileAction> = None;
        egui::ComboBox::new((fid, "move"), "Move to Group").show_ui(ui, |ui| {
            ui.selectable_value(&mut action, None, "");
            for (i, grp_name) in self.group_name_buffer.iter().enumerate().take(10) {
                let label = if grp_name.is_empty() {
                    format!("<insert new at {}>", i)
                } else {
                    format!("{} ({})", grp_name, i)
                };
                ui.selectable_value(&mut action, Some(FileAction::Move(i)), label);
            }
        });
        egui::ComboBox::new((fid, "copy"), "Copy to Group").show_ui(ui, |ui| {
            ui.selectable_value(&mut action, None, "");
            for (i, grp_name) in self.group_name_buffer.iter().enumerate().take(10) {
                let label = if grp_name.is_empty() {
                    format!("<insert new at {}>", i)
                } else {
                    format!("{} ({})", grp_name, i)
                };
                ui.selectable_value(&mut action, Some(FileAction::Copy(i)), label);
            }
        });
        egui::ComboBox::new((fid, "clone"), "Clone to Group").show_ui(ui, |ui| {
            ui.selectable_value(&mut action, None, "");
            for (i, grp_name) in self.group_name_buffer.iter().enumerate().take(10) {
                let label = if grp_name.is_empty() {
                    format!("<insert new at {}>", i)
                } else {
                    format!("{} ({})", grp_name, i)
                };
                ui.selectable_value(&mut action, Some(FileAction::Clone(i)), label);
            }
        });
        match action {
            Some(FileAction::Move(target_gid)) => {
                event_queue.queue_event(Box::new(MoveFile::new(fid, gid, target_gid)))
            }
            Some(FileAction::Copy(target_gid)) => {
                event_queue.queue_event(Box::new(CopyFile::new(fid, target_gid)))
            }
            Some(FileAction::Clone(target_gid)) => {
                event_queue.queue_event(Box::new(CloneFile::new(fid, target_gid)))
            }
            None => (),
        }
    }
}

impl File {
    pub fn render_property_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = ui.label("Alias: ");
            if ui.small_button("shorten").clicked() {
                self.shorten_alias('_');
            }
            ui.text_edit_singleline(&mut self.properties.alias)
                .labelled_by(label.id);
        });

        ui.label("X-Offset: ");
        let dragv = egui::DragValue::new(&mut self.properties.xoffset);
        ui.add(dragv);
        ui.label("Y-Offset: ");
        let dragv = egui::DragValue::new(&mut self.properties.yoffset);
        ui.add(dragv);
        ui.label("Y-Scale: ");
        let dragv = egui::DragValue::new(&mut self.properties.yscale);
        ui.add(dragv);
        ui.horizontal(|ui| {
            ui.label("Custom Color: ");
            if let Some(color) = self.properties.color.as_mut() {
                ui.color_edit_button_srgba(color);
            } else {
                let mut color = egui::Color32::RED;
                if ui.color_edit_button_srgba(&mut color).clicked() {
                    self.properties.color = Some(color);
                };
            }
        });

        ui.label("Comment:");
        egui::TextEdit::multiline(&mut self.properties.comment)
            .hint_text("Type a comment.")
            .show(ui);
    }

    fn shorten_alias(&mut self, delim: char) {
        if let Some(shortened) = self
            .properties
            .alias
            .rsplit_once(delim)
            .iter()
            .next()
            .map(|(head, _rest)| *head)
        {
            self.properties.alias = shortened.to_owned();
        }
    }
}

fn file_name_layout(file: &mut File) -> Option<LayoutJob> {
    let file_label_txt = if let Some(name) = file.path.file_name().and_then(|name| name.to_str()) {
        if file.data.value().is_ok() {
            egui::text::LayoutJob::single_section(name.to_owned(), egui::TextFormat::default())
        } else {
            // Make file label red if parsin CSV data failed.
            egui::text::LayoutJob::simple_singleline(
                name.to_owned(),
                FontId::default(),
                egui::Color32::RED,
            )
        }
    } else {
        return None;
    };
    Some(file_label_txt)
}
