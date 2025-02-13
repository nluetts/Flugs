use egui::Vec2;
use plotlib::view::View;

use crate::{app::components::File, EguiApp};

impl super::Plotter {
    pub(super) fn manipulate_file(
        &mut self,
        active_file: &mut File,
        modifiers: [bool; 3],
        drag: Vec2,
    ) {
        // How much did the mouse move?
        let Vec2 { x: dx, y: dy } = drag;
        match modifiers {
            // Alt key is pressed → change xoffset.
            [true, false, false] => {
                active_file.properties.xoffset += dx as f64;
            }
            // Ctrl key is pressed → change yoffset.
            [false, true, false] => {
                active_file.properties.yoffset += dy as f64;
            }
            // Shift is pressed → change yscale.
            [false, false, true] => {
                let yscale = active_file.properties.yscale;
                active_file.properties.yscale += yscale * 0.03 * (dy as f64);
            }
            // If several modifiers are pressed at the same time,
            // we ignore the input.
            _ => (),
        }
    }
}

pub fn save_svg(app: &EguiApp, path: &std::path::Path) {
    use plotlib::grid::Grid;
    use plotlib::page::Page;
    use plotlib::repr::Plot;
    use plotlib::style::*;
    use plotlib::view::ContinuousView;

    log::debug!("requested to save svg at '{:?}'", path);

    let [xmin, xmax, ymin, ymax] = app.plotter.current_plot_bounds;

    // Instantiate and customize plot.
    let mut view = ContinuousView::new()
        .x_label("x-label / x-unit")
        .y_label("y-label / y-unit")
        .x_range(xmin, xmax)
        .y_range(ymin, ymax)
        .x_max_ticks(5)
        .y_max_ticks(5);
    view.add_grid(Grid::new(4, 4));

    for (_, grp) in app
        .file_handler
        .groups
        .iter()
        .enumerate()
        .filter_map(|(id, x)| Some(id).zip(x.as_ref()))
    {
        if !grp.is_plotted {
            continue;
        }
        for fid in grp.file_ids.iter() {
            if let Some(cached_data) = app
                .file_handler
                .registry
                .get(fid)
                .and_then(|file| file.get_cache())
            {
                // Color for current file.
                let color = {
                    let color_id: i32 = (*fid).into();
                    let color = super::ui::auto_color(color_id);
                    color.to_hex()
                };

                // Add plot of current file to view.
                let plot = Plot::new(
                    cached_data
                        .into_iter()
                        .filter_map(|[x, y]| {
                            // Filter NaNs that may be present due to problems
                            // during CSV parsing.
                            if !(x.is_nan() || y.is_nan()) {
                                Some((*x, *y))
                            } else {
                                None
                            }
                        })
                        .collect(),
                )
                .line_style(
                    LineStyle::new()
                        .width(1.0)
                        .colour(color)
                        .linejoin(LineJoin::Round),
                );
                view = view.add(plot);
            }
        }
    }

    if let Err(err) = Page::single(&view).save(path) {
        log::error!("unable to save {:?}: {:?}", path, err);
    };
}
