mod logic;
mod ui;

use std::{collections::HashSet, path::PathBuf};

use app_core::frontend::UIParameter;

use crate::file_handling::GroupID;

#[derive(Default)]
pub struct Search {
    matched_paths: UIParameter<Vec<(PathBuf, HashSet<usize>, Option<GroupID>)>>,
    popup_shown: bool,
    search_path: UIParameter<PathBuf>,
    search_query: String,
}
