use egui_plot::Legend;

use crate::app::components::{File, FileHandler, FileID};

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

        egui_plot::Plot::new("Plot")
            .legend(Legend::default())
            .show(ui, |plot_ui| {
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
                            self.plot(fid, file, plot_ui)
                        }
                    }
                }
            });
    }

    fn plot(&self, fid: &FileID, file: &File, plot_iu: &mut egui_plot::PlotUi) {
        if let Some(data) = file.get_cache() {
            let color = auto_color(Into::<i32>::into(*fid));
            plot_iu.line(
                egui_plot::Line::new(data.to_owned())
                    .color(color)
                    .name(file.file_name()),
            );
        } else {
            // This should never happen because if the filter applied in
            // `Plotter::render`.
            unreachable!(
                "unable to get cache for plotting for file '{}'",
                file.file_name()
            )
        }
    }
}

pub fn auto_color(color_idx: i32) -> egui::Color32 {
    // analog to egui_plot
    let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
    let h = color_idx as f32 * golden_ratio;
    egui::epaint::Hsva::new(h, 0.85, 0.5, 1.0).into()
}
