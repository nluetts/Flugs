use egui::{text::LayoutJob, Color32, FontId};

use crate::app::DynRequestSender;

use super::{File, FileHandler, FileID};

impl FileHandler {
    pub(crate) fn render(
        &mut self,
        _request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.heading("Groups and Files")
        });

        let mut mark_delete_files = Vec::new();
        let mut mark_delete_groups = Vec::new();

        for (gid, grp) in self
            .groups
            .iter_mut()
            .enumerate()
            .filter_map(|(id, x)| Some(id).zip(x.as_mut()))
        {
            // Unwrapping is save here, because `group_ids` can only contain
            // valid keys.
            let grp_label_txt = LayoutJob::simple_singleline(
                format!("Group \"{}\"", grp.name),
                FontId::proportional(15.0),
                if ctx.theme() == egui::Theme::Light {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                },
            );
            let collapsing = egui::CollapsingHeader::new(grp_label_txt).id_salt((&grp.name, gid));
            collapsing.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let lab = ui.label("rename:");
                    ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
                    if ui.small_button("🗑").clicked() {
                        mark_delete_groups.push(gid);
                    }
                });
                for fid in grp.file_ids.iter() {
                    if let Some(file) = self.registry.get_mut(fid) {
                        render_file_form(ui, fid, file, gid, &mut mark_delete_files);
                    }
                }
            });

            ui.separator();
        }
        self.remove(mark_delete_groups, mark_delete_files);
    }
}

fn render_file_form(
    ui: &mut egui::Ui,
    fid: &FileID,
    file: &mut File,
    gid: usize,
    mark_delete_files: &mut Vec<(usize, FileID)>,
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
            if let Err(error) = file.csv_data.value() {
                ui.label(error).highlight();
            };
        });
        ui.label(format!("(ID {})", fid.0));
        if ui.small_button("🗑").clicked() {
            mark_delete_files.push((gid, *fid));
        }
    });
    ui.horizontal(|_ui| {
        // TODO move on developing UI
        // let xcol_sel = egui::ComboBox::from_label(file_name);
        // if file.csv_data.value().cache()
    });
}
