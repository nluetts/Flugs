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
}

impl Plotter {
    pub fn new() -> Self {
        Self {
            files_plot_ids: HashMap::with_capacity(10),
            selected_fid: None,
            current_plot_bounds: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
