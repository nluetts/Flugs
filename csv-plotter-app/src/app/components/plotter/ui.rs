use crate::app::components::{File, FileHandler};

impl super::Plotter {
    pub fn render(
        &mut self,
        file_handler: &mut FileHandler,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
    ) {
        use egui_plot::Plot;
        Plot::new("Plot").show(ui, |plot_ui| {
            for (_, gid) in file_handler.groups.iter().filter(|(_, grp)| grp.is_plotted) {
                for fid in gid.file_ids.iter() {
                    file_handler
                        .registry
                        .get(fid)
                        .map(|file| self.plot(file, plot_ui));
                }
            }
        });
    }

    fn plot(&self, file: &File, plot_iu: &mut egui_plot::PlotUi) {
        if let Some(data) = file.get_cache() {
            plot_iu.line(egui_plot::Line::new(data.to_owned()));
        } else {
            log::warn!(
                "unable to get cache for plotting for file '{}'",
                file.file_name()
            )
        }
    }
}
