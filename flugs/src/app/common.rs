use egui_plot::PlotPoint;

pub fn global_ymin(data: &[PlotPoint]) -> f64 {
    let ymin = data
        .iter()
        .map(|PlotPoint { x: _x, y }| *y)
        .reduce(|current_min, yi| if yi < current_min { yi } else { current_min })
        .unwrap_or(0.0);
    ymin
}
