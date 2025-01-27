use std::fmt::Write;

use egui::{text::LayoutJob, Color32, FontId};

use crate::{
    app::{
        events::{CopyFile, EventQueue, MoveFile, RemoveFile, RemoveGroup},
        DynRequestSender,
    },
    EguiApp,
};

use super::{File, FileHandler, FileID};

impl FileHandler {
    pub(crate) fn render(
        &mut self,
        _request_tx: &mut DynRequestSender,
        event_queue: &mut EventQueue<EguiApp>,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.heading("Groups and Files")
        });

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

        for (gid, grp) in self
            .groups
            .iter_mut()
            .enumerate()
            .filter_map(|(id, x)| Some(id).zip(x.as_mut()))
        {
            ui.heading(&grp.name);
            ui.horizontal(|ui| {
                let lab = ui.label("rename:");
                ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
                if ui.small_button("🗑").clicked() {
                    event_queue.queue_event(Box::new(RemoveGroup::new(gid)));
                }
            });
            let grp_label_txt = LayoutJob::simple_singleline(
                format!("Files in Group {}", grp.name),
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
            collapsing.show(ui, |ui| {
                for fid in grp.file_ids.iter() {
                    if let Some(file) = self.registry.get_mut(fid) {
                        render_file_form(ui, fid, file, gid, &self.group_name_buffer, event_queue);
                    }
                }
            });

            ui.separator();
        }
    }
}

fn render_file_form(
    ui: &mut egui::Ui,
    fid: &FileID,
    file: &mut File,
    gid: usize,
    group_names: &[String],
    event_queue: &mut EventQueue<EguiApp>,
) {
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
            log::warn!("could not render file name for {:?}, skipping", file.path);
            return;
        };
    file_label_txt.wrap.max_rows = 1;
    ui.horizontal(|ui| {
        let fltxt = file_label_txt.text.clone();
        let collapsing = egui::CollapsingHeader::new(file_label_txt).id_salt((fltxt, gid));
        collapsing.show(ui, |ui| {
            // Display error if csv could not be parsed.
            if let Err(error) = file.csv_data.value() {
                ui.label(error).highlight();
            };
            // Text box to change alias.
            ui.horizontal(|ui| {
                ui.label("alias: ");
                ui.text_edit_singleline(&mut file.properties.alias)
            });

            // Menu to move/copy file to other group.
            let mut target: (Option<usize>, bool) = (None, false);
            ui.horizontal(|ui| {
                egui::ComboBox::new((gid, fid, "move"), "Move to Group").show_ui(ui, |ui| {
                    ui.selectable_value(&mut target, (None, false), "");
                    for (i, grp_name) in group_names.iter().enumerate().take(10) {
                        let label = if grp_name.is_empty() {
                            format!("<insert new at {}>", i + 1)
                        } else {
                            format!("{} ({})", grp_name, i + 1)
                        };
                        ui.selectable_value(&mut target, (Some(i), true), label);
                    }
                });
                egui::ComboBox::new((gid, fid, "copy"), "Copy to Group").show_ui(ui, |ui| {
                    ui.selectable_value(&mut target, (None, false), "");
                    for (i, grp_name) in group_names.iter().enumerate().take(10) {
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
                    event_queue.queue_event(Box::new(MoveFile::new(*fid, gid, target_gid)))
                }
                (Some(target_gid), false) => {
                    event_queue.queue_event(Box::new(CopyFile::new(*fid, target_gid)))
                }
                (None, _) => (),
            }
        });

        // Identifier and delete button.
        ui.label(format!("(ID {})", fid.0));
        if ui.small_button("🗑").clicked() {
            event_queue.queue_event(Box::new(RemoveFile::new(*fid, gid)));
        }
    });
}
