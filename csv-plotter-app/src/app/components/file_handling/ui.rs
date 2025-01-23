use crate::app::DynRequestSender;

use super::FileHandler;

impl FileHandler {
    pub(crate) fn render(
        &mut self,
        _request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
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
            ui.heading(&grp.name);
            ui.horizontal(|ui| {
                let lab = ui.label("rename:");
                ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
                ui.toggle_value(&mut grp.is_plotted, "plot?");
                if ui.small_button("🗑").clicked() {
                    mark_delete_groups.push(gid);
                }
            });
            for fid in grp.file_ids.iter() {
                if let Some(file) = self.registry.get_mut(fid) {
                    let file_name =
                        if let Some(name) = file.path.file_name().and_then(|name| name.to_str()) {
                            name
                        } else {
                            log::warn!("could not render file name for {:?}, skipping", file.path);
                            continue;
                        };
                    ui.horizontal(|ui| {
                        ui.label(file_name);
                        ui.label(format!("(ID {})", fid.0));
                        if ui.small_button("🗑").clicked() {
                            mark_delete_files.push((gid, *fid));
                        }
                    });
                    if let Err(error) = file.csv_data.value() {
                        ui.label(error).highlight();
                    };
                    ui.horizontal(|_ui| {
                        // TODO move on developing UI
                        // let xcol_sel = egui::ComboBox::from_label(file_name);
                        // if file.csv_data.value().cache()
                    });
                } else {
                    continue;
                }
            }
            ui.separator();
        }
        self.remove(mark_delete_groups, mark_delete_files);
    }
}
