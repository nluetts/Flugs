use super::FileHandler;

impl FileHandler {
    pub(crate) fn render_groups(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        for (k, v) in self.groups.iter() {
            ui.heading(format!("{:?}", k));
            ui.horizontal(|ui| {
                for f in v.file_ids.iter() {
                    if let Some(file) = self.registry.get(f) {
                        ui.label(file.path.to_string_lossy());
                    } else {
                        continue;
                    }
                }
            });
        }
    }
}
