use std::io::Write;

use egui::Vec2;
use egui_plot::PlotPoint;

use crate::{
    app::components::{file_handling::Annotation, File},
    EguiApp,
};

impl super::Plotter {
    pub(super) fn manipulate_file(
        &mut self,
        active_file: &mut File,
        modifiers: [bool; 3],
        drag: Vec2,
        yspan: f64,
        mouse_pos: Option<PlotPoint>,
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
            // Ctrl and Shift is pressed → insert annotation.
            [false, true, true] => {
                if let Some(mouse_pos) = mouse_pos {
                    active_file.properties.annotations.push(Annotation::new(
                        mouse_pos.x,
                        mouse_pos.y,
                        "foobar",
                    ));
                }
            }
            // If several modifiers are pressed at the same time,
            // we ignore the input.
            _ => (),
        }
    }

    pub fn apply_bounds(&mut self, bounds: [f64; 4]) {
        self.request_plot_bounds = Some(bounds);
    }

    pub fn get_current_plot_bounds(&self) -> [f64; 4] {
        self.current_plot_bounds
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

    let mut fig = Figure::empty(app.config.svg_width, app.config.svg_height);
    let mut ax = Axis::default()
        .with_xlim(xmin, xmax)
        .with_ylim(ymin, ymax)
        .with_xlabel(&app.config.x_label)
        .with_ylabel(&app.config.y_label)
        .with_legend(true)
        .draw_xaxis(app.config.draw_xaxis)
        .draw_yaxis(app.config.draw_yaxis)
        .with_x_minor_ticks(app.config.num_x_minorticks)
        .with_y_minor_ticks(app.config.num_y_minorticks);

    // Overwrite axis ticks with ticks from config, if available.
    if !app.config.x_ticks.pos.is_empty() {
        app.config.x_ticks.pos.clone_into(&mut ax.ticks.xpos);
    }
    if !app.config.y_ticks.pos.is_empty() {
        app.config.y_ticks.pos.clone_into(&mut ax.ticks.ypos);
    }

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
            if let Some((cached_data, plot_file)) = app
                .file_handler
                .registry
                .get(fid)
                .and_then(|file| file.get_cache().map(|cache| (cache, file)))
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

                // Downsample to a maximum of 1000 points.
                // TODO: Make this a number of points a global option.
                // let window_size = cached_data.len() / 1000;
                // let xs: Vec<_> = cached_data
                //     .chunks(window_size)
                //     .map(|vals| vals.iter().map(|[x, _]| x + x0).sum::<f64>() / vals.len() as f64)
                //     .collect();
                // let ys: Vec<_> = cached_data
                //     .chunks(window_size)
                //     .map(|vals| {
                //         vals.iter().map(|[_, y]| (y * scale) + y0).sum::<f64>() / window_size as f64
                //     })
                //     .collect();
                //

                let xs: Vec<_> = cached_data.iter().map(|[x, _]| x + x0).collect();
                let ymin = cached_data
                    .iter()
                    .map(|[_, y]| y)
                    .reduce(|current_min, yi| if yi < current_min { yi } else { current_min })
                    .unwrap_or(&0.0);
                let ys: Vec<_> = cached_data
                    .iter()
                    .map(|[_, y]| (y - ymin) * scale + y0 + ymin)
                    .collect();

                let line = LinePlot::new(&xs, &ys)
                    .with_color(&color)
                    .with_linewidth(app.config.plot_linewidth)
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
