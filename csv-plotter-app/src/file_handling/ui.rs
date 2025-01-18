use super::FileHandler;

impl FileHandler {
    pub(crate) fn render_groups(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        for (gid, grp) in self.groups.iter_mut() {
            ui.heading(&grp.name);
            ui.horizontal(|ui| {
                let lab = ui.label("rename:");
                ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
                ui.checkbox(&mut grp.is_plotted, "plot: ");
            });
            ui.horizontal(|ui| {
                for f in grp.file_ids.iter() {
                    if let Some(file) = self.registry.get(f) {
                        ui.label(file.path.to_string_lossy());
                    } else {
                        continue;
                    }
                }
            });
            ui.separator();
        }
    }
}
