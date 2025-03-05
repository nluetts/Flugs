use std::io::Write;

use egui::Vec2;

use crate::{app::components::File, EguiApp};

impl super::Plotter {
    pub(super) fn manipulate_file(
        &mut self,
        active_file: &mut File,
        modifiers: [bool; 3],
        drag: Vec2,
        yspan: f64,
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
                active_file.properties.yscale += yscale * 3.0 / yspan * (dy as f64);
            }
            // If several modifiers are pressed at the same time,
            // we ignore the input.
            _ => (),
        }
    }
}

pub fn save_svg(app: &EguiApp, path: &std::path::Path) {
    use svg_export::{self, Axis, Figure, LinePlot};

    log::debug!("requested to save svg at '{:?}'", path);

    let mut file = match std::fs::File::create(path) {
        Ok(file) => file,
        Err(err) => {
            log::error!("unable to create file for saving svg: {:?}", err);
            return;
        }
    };

    let [xmin, xmax, ymin, ymax] = app.plotter.current_plot_bounds;

    let mut fig = Figure::empty(800, 600);
    let mut ax = Axis::default()
        .with_xlim(xmin, xmax)
        .with_ylim(ymin, ymax)
        .with_xlabel("x-label")
        .with_ylabel("ylabel")
        .with_legend(true);

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
            // I'll need `file` later for labels.
            if let Some((cached_data, plot_file)) =
                app.file_handler
                    .registry
                    .get(fid)
                    .and_then(|file| match file.get_cache() {
                        Some(cache) => Some((cache, file)),
                        None => None,
                    })
            {
                // Color for current file.
                let color: String = {
                    let color_id: i32 = (*fid).into();
                    super::ui::auto_color(color_id)
                        .to_hex()
                        .chars()
                        .take(7)
                        .collect()
                };

                let (scale, x0, y0) = (
                    plot_file.properties.yscale,
                    plot_file.properties.xoffset,
                    plot_file.properties.yoffset,
                );
                let label = if !plot_file.properties.alias.is_empty() {
                    format!("{} ({})", &plot_file.properties.alias, grp.name)
                } else {
                    format!("{} ({})", plot_file.file_name(), grp.name)
                };
                log::debug!("plotting line with label {}", label);
                let line = LinePlot::new(
                    &cached_data.iter().map(|[x, _]| *x + x0).collect::<Vec<_>>(),
                    &cached_data
                        .iter()
                        .map(|[_, y]| (*y * scale) + y0)
                        .collect::<Vec<_>>(),
                )
                .with_color(&color)
                .with_linewidth(1.0)
                .with_name(&label);

                ax.add_line(line);
            }
        }
    }
    ax.insert_into(&mut fig);

    if let Err(err) = file.write_all(&fig.render().into_bytes()) {
        log::error!("could not write svg file {:?}: {:?}", path, err)
    }
}
