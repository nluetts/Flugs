use std::fmt::Write;

use egui::{text::LayoutJob, Color32, FontId};

use crate::{
    app::{
        events::{CopyFile, EventQueue, MoveFile, RemoveFile, RemoveGroup},
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

        let scroll_area = egui::ScrollArea::both().max_width(400.0);

        egui::panel::SidePanel::left("file_and_group_tree").show(ctx, |ui| {
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
                    let file_label_txt = match render_file_label(file) {
                        Some(value) => value,
                        None => {
                            log::warn!("could not render file label for fid {fid:?}");
                            continue;
                        }
                    };
                    if ui.label(file_label_txt).clicked() {
                        self.active_element = ActiveElement::File(*fid, gid);
                    }
                }
            });

            if resp.header_response.clicked() {
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
            if ui.small_button("🗑").clicked() {
                event_queue.queue_event(Box::new(RemoveGroup::new(gid)));
            }
        });
    }
    fn file_settings(
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
            None => return,
        };

        // Display error if csv could not be parsed.
        if let Err(error) = file.csv_data.value() {
            ui.label(error).highlight();
        };
        // Text box to change alias.
        // ui.horizontal(|ui| {
        // TODO: This gives a runtime error in the ui
        // ui.label("alias: ");
        ui.text_edit_singleline(&mut file.properties.alias);
        // });

        // Menu to move/copy file to other group.
        let mut target: (Option<usize>, bool) = (None, false);
        ui.horizontal(|ui| {
            egui::ComboBox::new((fid, "move"), "Move to Group").show_ui(ui, |ui| {
                ui.selectable_value(&mut target, (None, false), "");
                for (i, grp_name) in self.group_name_buffer.iter().enumerate().take(10) {
                    let label = if grp_name.is_empty() {
                        format!("<insert new at {}>", i + 1)
                    } else {
                        format!("{} ({})", grp_name, i + 1)
                    };
                    ui.selectable_value(&mut target, (Some(i), true), label);
                }
            });
            egui::ComboBox::new((fid, "copy"), "Copy to Group").show_ui(ui, |ui| {
                ui.selectable_value(&mut target, (None, false), "");
                for (i, grp_name) in self.group_name_buffer.iter().enumerate().take(10) {
                    let label = if grp_name.is_empty() {
                        format!("<insert new at {}>", i + 1)
                    } else {
                        format!("{} ({})", grp_name, i + 1)
                    };
                    ui.selectable_value(&mut target, (Some(i), false), label);
                }
            });
        });
        match target {
            (Some(target_gid), true) => {
                event_queue.queue_event(Box::new(MoveFile::new(fid, gid, target_gid)))
            }
            (Some(target_gid), false) => {
                event_queue.queue_event(Box::new(CopyFile::new(fid, target_gid)))
            }
            (None, _) => (),
        }

        // Identifier and delete button.
        ui.label(format!("(ID {})", fid.0));
        if ui.small_button("🗑").clicked() {
            event_queue.queue_event(Box::new(RemoveFile::new(fid, gid)));
        }
    }
}

fn render_file_label(file: &mut File) -> Option<LayoutJob> {
    let mut file_label_txt =
        if let Some(name) = file.path.file_name().and_then(|name| name.to_str()) {
            if file.csv_data.value().is_ok() {
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
    // TODO: The label is not yet correctly shortened
    file_label_txt.wrap.max_width = 800.0;
    file_label_txt.wrap.max_rows = 1;
    Some(file_label_txt)
}
