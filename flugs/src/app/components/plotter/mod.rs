mod logic;
mod ui;

pub use logic::save_svg;

use std::collections::HashMap;

use super::FileID;

pub struct Plotter {
    /// We use this as a buffer to store egui IDs to correlate them with file
    /// IDs. We need this to detect interactions with plotted files.
    files_plot_ids: HashMap<egui::Id, FileID>,
    selected_fid: Option<FileID>,
    current_plot_bounds: [f64; 4],
    current_integral: Option<(f64, f64)>,
    current_annotation_text: String,
    integrate_with_local_baseline: bool,
    auto_shift_after_scaling: bool,
    request_plot_bounds: Option<[f64; 4]>,
    pub mode: PlotterMode,
}

impl Plotter {
    pub fn new() -> Self {
        Self {
            files_plot_ids: HashMap::with_capacity(10),
            selected_fid: None,
            current_plot_bounds: [0.0, 0.0, 0.0, 0.0],
            current_integral: None,
            current_annotation_text: String::new(),
            mode: PlotterMode::Display,
            // TODO: make this a global option
            integrate_with_local_baseline: true,
            request_plot_bounds: None,
            auto_shift_after_scaling: false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PlotterMode {
    Display,
    Integrate,
    Annotage,
}

impl PlotterMode {
    pub fn next(&mut self) -> Self {
        match self {
            PlotterMode::Display => PlotterMode::Integrate,
            PlotterMode::Integrate => PlotterMode::Annotage,
            PlotterMode::Annotage => PlotterMode::Display,
        }
    }
}
