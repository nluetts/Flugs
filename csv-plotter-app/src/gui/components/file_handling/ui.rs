use crate::gui::DynRequestSender;

use super::FileHandler;

impl FileHandler {
    pub(crate) fn render_groups(
        &mut self,
        _request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
    ) {
        let mut mark_delete_files = Vec::new();
        let mut mark_delete_groups = Vec::new();

        let mut group_ids: Vec<_> = self.groups.keys().cloned().collect();
        group_ids.sort();
        for gid in group_ids {
            // Unwrapping is save here, because `group_ids` can only contain
            // valid keys.
            let grp = self.groups.get_mut(&gid).unwrap();
            ui.heading(&grp.name);
            ui.horizontal(|ui| {
                let lab = ui.label("rename:");
                ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
                ui.toggle_value(&mut grp.is_plotted, "plot?");
                if ui.small_button("🗑").clicked() {
                    mark_delete_groups.push(gid.clone());
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
                            mark_delete_files.push((gid.clone(), fid.clone()));
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
