mod logic;
mod ui;

use std::path::PathBuf;

use app_core::frontend::UIParameter;

#[derive(Default)]
pub struct Search {
    matched_paths: UIParameter<Vec<(PathBuf, std::collections::HashSet<usize>)>>,
    read_current_child_paths: UIParameter<()>,
    search_query: String,
}
impl Search {}
