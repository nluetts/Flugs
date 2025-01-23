use crate::app::components::{File, FileHandler};

impl super::Plotter {
    pub fn render(
        &mut self,
        file_handler: &mut FileHandler,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
    ) {
        // Horizontal stripe of switch buttons enabeling/disabeling groups
        ui.horizontal(|ui| {
            for grp in file_handler.groups.iter_mut().filter_map(|x| x.as_mut()) {
                ui.toggle_value(&mut grp.is_plotted, &grp.name);
            }
        });

        use egui_plot::Plot;
        Plot::new("Plot").show(ui, |plot_ui| {
            for (_, grp) in file_handler
                .groups
                .iter_mut()
                .enumerate()
                .filter_map(|(id, x)| Some(id).zip(x.as_mut()))
            {
                if !grp.is_plotted {
                    continue;
                }
                for fid in grp.file_ids.iter() {
                    if let Some(file) = file_handler
                        .registry
                        .get(fid)
                        .filter(|file| file.get_cache().is_some())
                    {
                        self.plot(file, plot_ui)
                    }
                }
            }
        });
    }

    fn plot(&self, file: &File, plot_iu: &mut egui_plot::PlotUi) {
        if let Some(data) = file.get_cache() {
            plot_iu.line(egui_plot::Line::new(data.to_owned()));
        } else {
            // This should never happen because if the fileter applied in
            // `Plotter::render`.
            unreachable!(
                "unable to get cache for plotting for file '{}'",
                file.file_name()
            )
        }
    }
}
