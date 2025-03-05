use egui::Vec2;
use egui_plot::Legend;

use crate::app::components::{File, FileHandler, FileID};

impl super::Plotter {
    pub fn render(
        &mut self,
        file_handler: &mut FileHandler,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        // Horizontal stripe of switch buttons enabeling/disabeling groups
        ui.horizontal(|ui| {
            for grp in file_handler.groups.iter_mut().filter_map(|x| x.as_mut()) {
                ui.toggle_value(&mut grp.is_plotted, &grp.name);
            }
        });

        // These are needed to apply modifications to the selected file.
        let mut spans = (0.0, 0.0);
        let mut drag = Vec2::default();

        self.files_plot_ids.drain();
        let response = egui_plot::Plot::new("Plot")
            .allow_drag(self.selected_fid.is_none())
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
                            let egui_id = self.plot(fid, file, &grp.name, plot_ui);
                            self.files_plot_ids.insert(egui_id, *fid);
                        }
                    }
                }
                drag = plot_ui.pointer_coordinate_drag_delta();
                spans = {
                    let bounds = plot_ui.plot_bounds();
                    let xspan = (bounds.max()[0] - bounds.min()[0]).abs();
                    let yspan = (bounds.max()[1] - bounds.min()[1]).abs();
                    (xspan, yspan)
                };
                self.current_plot_bounds = {
                    let [xmin, ymin] = plot_ui.plot_bounds().min();
                    let [xmax, ymax] = plot_ui.plot_bounds().max();
                    [xmin, xmax, ymin, ymax]
                };
                plot_ui.plot_bounds()
            });

        // Get modifier input (we need this here already, to disallow the plot
        // to be panned).
        let modifiers = ctx.input(|i| [i.modifiers.alt, i.modifiers.ctrl, i.modifiers.shift]);
        let modifier_down = modifiers.iter().any(|x| *x);
        let mouse_pressed = ctx.input(|i| i.pointer.primary_pressed());

        if let Some(hovered_fid) = response
            .hovered_plot_item
            .and_then(|id| self.files_plot_ids.get(&id))
        {
            // Select file, if its plot was clicked this frame.
            if mouse_pressed {
                self.selected_fid = Some(*hovered_fid);
            }
        } else {
            // If we clicked somewhere and no modifier was pressed, we deselect
            // the currently selected file.
            if mouse_pressed && !modifier_down {
                self.selected_fid = None;
            }
        }
        if let Some(selected_file) = self
            .selected_fid
            .and_then(|fid| file_handler.registry.get_mut(&fid))
        {
            let yspan = response.inner.height();
            let should_modify = modifier_down && drag.length() > 0.0;
            if should_modify {
                self.manipulate_file(selected_file, modifiers, drag, yspan);
            }
        }
    }

    fn plot(
        &self,
        fid: &FileID,
        file: &File,
        group_name: &str,
        plot_iu: &mut egui_plot::PlotUi,
    ) -> egui::Id {
        if let Some(data) = file.get_cache() {
            // Apply custom shifting/scaling to data.
            let data: Vec<[f64; 2]> = data
                .iter()
                .map(|[x, y]| {
                    [
                        x + file.properties.xoffset,
                        y * file.properties.yscale + file.properties.yoffset,
                    ]
                })
                .collect();

            // Plot the data.
            let color = auto_color(Into::<i32>::into(*fid));
            let width = if self.selected_fid.is_some_and(|sfid| sfid == *fid) {
                2.5
            } else {
                1.0
            };
            let name = if file.properties.alias.is_empty() {
                format!("{} ({})", file.file_name(), group_name)
            } else {
                format!("{} ({})", file.properties.alias, group_name)
            };
            let egui_id = name.clone().into();
            plot_iu.line(
                egui_plot::Line::new(data.to_owned())
                    .color(color)
                    .width(width)
                    .name(name)
                    .id(egui_id),
            );

            egui_id
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
