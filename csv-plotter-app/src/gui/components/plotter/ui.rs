use crate::file_handling::{FileHandler, FileID};

impl super::Plotter {
    pub fn render(
        &mut self,
        file_handler: &mut FileHandler,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
    ) {
        use egui_plot::Plot;
        Plot::new("Plot").show(ui, |_plot_ui| {
            for (_, gid) in file_handler.groups.iter().filter(|(_, grp)| grp.is_plotted) {
                for fid in gid.file_ids.iter() {
                    self.plot(fid, _plot_ui);
                }
            }
        });
    }

    fn plot(&self, fid: &FileID, _plot_iu: &mut egui_plot::PlotUi) {
        log::info!("should plot {:?}", fid)
    }
}
