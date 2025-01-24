mod csv;

use app_core::backend::BackendState;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub use csv::CSVData;

#[derive(Default)]
pub struct BackendAppState {
    search_path: PathBuf,
    child_paths_unfiltered: Vec<PathBuf>,
}

impl BackendState for BackendAppState {}

impl BackendAppState {
    pub fn new(search_path: PathBuf) -> Self {
        Self {
            search_path,
            child_paths_unfiltered: Vec::new(),
        }
    }
}

/// Implementations of backend actions
impl BackendAppState {
    /// Update the subpaths of the path which is currently selected
    /// (`current_path`)
    fn update_child_paths_unfiltered(&mut self) {
        let mut file_paths = Vec::new();
        let mut dirs = vec![self.search_path.to_path_buf()];

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
                    let p = path
                        .as_path()
                        .strip_prefix(&self.search_path)
                        .expect("failed to strip search path from sub directory");
                    file_paths.push(p.to_path_buf());
                }
            }
        }

        self.child_paths_unfiltered = file_paths;
    }

    pub fn get_search_path(&self) -> PathBuf {
        self.search_path.clone()
    }

    pub fn set_search_path(&mut self, new_path: &Path) {
        self.search_path = new_path.to_path_buf();
        // update index of files/paths
        self.update_child_paths_unfiltered();
    }

    /// Return the best file path matches for `query`, together with the
    /// corresponding matching indices in the file path.
    ///
    /// For a file path to match, the file path must contain all words
    /// (separated by white space).
    pub fn search_filter(&self, query: &str) -> Vec<(PathBuf, HashSet<usize>)> {
        let contains_query = |filename: &&PathBuf| {
            let fp = filename.to_str();
            if fp.is_none() {
                return false;
            }
            let fp = fp.unwrap();
            query.split(" ").all(|q| fp.contains(q))
        };
        let query_indices = |filename: &PathBuf| {
            let mut indices = HashSet::new();
            let fp = filename.to_str()?;
            for q in query.split(" ") {
                let idx = fp.find(q)?;
                indices.extend(idx..idx + q.len());
            }
            Some((filename.to_owned(), indices))
        };

        self.child_paths_unfiltered
            .iter()
            .filter(contains_query)
            .take(10)
            .filter_map(query_indices)
            .collect()
    }
}
