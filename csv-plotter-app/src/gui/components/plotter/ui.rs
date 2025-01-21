use crate::gui::components::FileHandler;

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
                        .and_then(|file| file.get_cache())
                        .map(|dat| self.plot(dat, plot_ui));
                }
            }
        });
    }

    fn plot(&self, data: &Vec<[f64; 2]>, plot_iu: &mut egui_plot::PlotUi) {
        // log::debug!("first data point: {:?}", data[0]);
        plot_iu.line(egui_plot::Line::new(data.to_owned()));
    }
}
