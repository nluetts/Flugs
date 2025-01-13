mod logic;
mod ui;

use std::{collections::HashSet, path::PathBuf};

use app_core::{backend::LinkReceiver, frontend::UIParameter};

use crate::file_handling::GroupID;

#[derive(Default)]
pub struct Search {
    matched_paths: UIParameter<Vec<(PathBuf, HashSet<usize>, Option<GroupID>)>>,
    read_current_child_paths: UIParameter<()>,
    search_query: String,
    _requested_loading: Vec<LinkReceiver<PathBuf>>,
}
