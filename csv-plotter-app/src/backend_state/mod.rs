mod fuzzy_search;

use std::path::PathBuf;

use app_core::backend::BackendState;

pub use fuzzy_search::get_matched_unmatch_str_index_groups;

#[derive(Default)]
pub struct BackendAppState {
    current_path: PathBuf,
    child_paths_unfiltered: Vec<PathBuf>,
}

impl BackendState for BackendAppState {}

impl BackendAppState {
    pub fn new(current_path: PathBuf) -> Self {
        Self {
            current_path,
            child_paths_unfiltered: Vec::new(),
        }
    }
}

/// Implementations of backend actions
impl BackendAppState {
    pub fn update_child_paths_unfiltered(&mut self) {
        let mut file_paths = Vec::new();
        let mut dirs = vec![self.current_path.to_path_buf()];

        while let Some(current_path) = dirs.pop() {
            for path in std::fs::read_dir(&current_path)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
            {
                if path.is_dir() {
                    dirs.push(path);
                } else if path.is_file() {
                    file_paths.push(path);
                }
            }
        }

        self.child_paths_unfiltered = file_paths;
    }
}
