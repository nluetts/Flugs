use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use std::path::PathBuf;

use crate::BackendState;

#[derive(Default)]
pub struct BackendAppState {
    current_path: PathBuf,
    child_paths_unfiltered: Vec<PathBuf>,
    fzm: SkimMatcherV2,
}

impl BackendState for BackendAppState {}

impl BackendAppState {
    pub fn new(current_path: PathBuf) -> Self {
        let fzm = SkimMatcherV2::default().smart_case();
        Self {
            current_path,
            child_paths_unfiltered: Vec::new(),
            fzm,
        }
    }
}

/// Implementations of backend actions
impl BackendAppState {
    pub fn update_current_path_children(&mut self) {
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

    pub fn fuzzy_filter(&self, query: &str) -> Vec<PathBuf> {
        let mut res: Vec<_> = self
            .child_paths_unfiltered
            .iter()
            .filter_map(|fp| fp.to_str().map(|str| (fp, str)))
            .filter_map(|(fp, str)| self.fzm.fuzzy_match(str, query).map(|score| (fp, score)))
            .collect();
        res.sort_unstable_by(|(_, score_a), (_, score_b)| score_a.cmp(score_b));
        res.into_iter()
            .rev() // to make highest score come on top
            .filter_map(|(fp, score)| if score > 0 { Some(fp.to_owned()) } else { None })
            .take(10)
            .collect()
    }
}
