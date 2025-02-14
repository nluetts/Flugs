use app_core::string_error::ErrorStringExt;
use egui::Vec2;

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
    use plotters::prelude::*;

    log::debug!("requested to save svg at '{:?}'", path);

    let root = SVGBackend::new(&path, (1024, 768)).into_drawing_area();
    // let font = FontDesc::from(("sans-serif", 20.0));
    let [xmin, xmax, ymin, ymax] = app.plotter.current_plot_bounds;

    // Implement plotting as a closure, so we can short-circuit errors.
    let plot = || -> Result<(), String> {
        root.fill(&WHITE)
            .err_to_string("could not prepare canvas for plotting")?;

        // Initialize chart.
        let mut chart = ChartBuilder::on(&root)
            .margin(20u32)
            // .caption(format!("y=x^{}", 2), font)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .build_cartesian_2d(xmin..xmax, ymin..ymax)
            .err_to_string("could not prepare chart for plotting")?;

        // Add labels.
        chart
            .configure_mesh()
            .x_labels(3)
            .y_labels(3)
            .draw()
            .err_to_string("could not prepare chart for plotting")?;

        //
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
                if let Some((cached_data, file)) =
                    app.file_handler
                        .registry
                        .get(fid)
                        .and_then(|file| match file.get_cache() {
                            Some(cache) => Some((cache, file)),
                            None => None,
                        })
                {
                    // Color for current file.
                    let color_style = {
                        let color_id: i32 = (*fid).into();
                        let (r, g, b, a) = super::ui::auto_color(color_id).to_tuple();
                        RGBAColor(r, g, b, a as f64 / 255.).stroke_width(1)
                    };

                    chart
                        .draw_series(LineSeries::new(
                            cached_data.into_iter().map(|[x, y]| (*x, *y)),
                            color_style,
                        ))
                        .err_to_string("unable to draw data for SVG export")?
                        .label(file.file_name())
                        .legend(move |(x, y)| {
                            PathElement::new(vec![(x, y), (x + 20, y)], color_style)
                        });
                    chart
                        .configure_series_labels()
                        .background_style(WHITE.mix(0.8))
                        .border_style(BLACK)
                        .position(SeriesLabelPosition::UpperRight)
                        .draw()
                        .err_to_string("unable to configure labels for SVG export")?;

                    root.present().err_to_string("unable to write SVG output")?;
                }
            }
        }

        Ok(())
    };
    if let Err(err) = plot() {
        log::error!("unable to plot to svg: {:?}", err);
    };
}
