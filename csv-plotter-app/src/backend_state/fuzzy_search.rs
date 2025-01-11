use std::collections::HashSet;

use crate::BackendAppState;

use std::path::PathBuf;

impl BackendAppState {
    /// Return the best file path matches for `query`, together with the
    /// corresponding matching indices in the file path.
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
