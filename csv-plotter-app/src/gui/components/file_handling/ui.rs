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
                if let Some(file) = self.registry.get_mut(f) {
                    let file_name =
                        if let Some(name) = file.path.file_name().and_then(|name| name.to_str()) {
                            name
                        } else {
                            log::warn!("could not render file name for {:?}, skipping", file.path);
                            continue;
                        };
                    let file_label = ui.label(file_name);
                    match file.csv_data.value() {
                        Ok(data) => {}
                        Err(error) => {
                            ui.label(error).highlight();
                        }
                    };
                    ui.horizontal(|ui| {
                        let xcol_sel = egui::ComboBox::from_label(file_name);
                        // if file.csv_data.value().cache()
                    });
                } else {
                    continue;
                }
            }
            ui.separator();
        }
    }
}
