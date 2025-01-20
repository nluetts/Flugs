use crate::gui::DynRequestSender;

use super::{logic::parse_csv, FileHandler};

impl FileHandler {
    pub(crate) fn render_groups(
        &mut self,
        request_tx: &mut DynRequestSender,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        for (gid, grp) in self.groups.iter_mut() {
            ui.heading(&grp.name);
            ui.horizontal(|ui| {
                let lab = ui.label("rename:");
                ui.text_edit_singleline(&mut grp.name).labelled_by(lab.id);
                ui.checkbox(&mut grp.is_plotted, "plot?");
            });
            for f in grp.file_ids.iter() {
                if let Some(file) = self.registry.get(f) {
                    let file_label = ui.label(file.path.to_string_lossy());
                    if file_label.clicked() {
                        // TODO: The file cannot yet be loaded correctly,
                        // because we stripped the root search path
                        // which is not available here, this has to be
                        // changed.
                        parse_csv(&file.path, request_tx);
                    }
                } else {
                    continue;
                }
            }
            ui.separator();
        }
    }
}
