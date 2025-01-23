mod logic;
mod ui;

use std::collections::HashSet;

use super::FileID;

pub struct Plotter {
    /// We use this as a buffer to store egui IDs to correlate them with file
    /// IDs. We need this to detect interactions with the plot and scale/shift
    /// the data accordingly.
    files_plot_ids: HashSet<(egui::Id, FileID)>,
}

impl Plotter {
    pub fn new() -> Self {
        Self {
            files_plot_ids: HashSet::with_capacity(10),
        }
    }
}
