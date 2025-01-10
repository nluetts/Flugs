use crate::BackendAppState;

use std::{ops::RangeInclusive, path::PathBuf};

impl BackendAppState {
    /// Return the best file path matches for `query`, together with the
    /// corresponding matching indices in the file path.
    pub fn search_filter(&self, query: &str) -> Vec<(PathBuf, Vec<usize>)> {
        let contains_query = |filename: &&PathBuf| {
            let fp = filename.to_str();
            if fp.is_none() {
                return false;
            }
            let fp = fp.unwrap();
            query.split(" ").all(|q| fp.contains(q))
        };
        let query_indices = |filename: &PathBuf| {
            let mut indices = Vec::new();
            let fp = filename.to_str()?;
            for q in query.split(" ") {
                let idx = fp.find(q)?;
                indices.extend(idx..idx + q.len());
            }
            indices.sort_unstable();
            indices.dedup();
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

/// For a str, given the matched indices, return ranges
/// of matched/not matched indices (for formatting).
pub fn get_matched_unmatch_str_index_groups(
    s: &str,
    indices: &[usize],
) -> (Vec<RangeInclusive<usize>>, Vec<RangeInclusive<usize>>) {
    let not_matched: Vec<RangeInclusive<usize>> = {
        let ins: Vec<_> = (0..s.len()).filter(|n| !indices.contains(n)).collect();
        group_indices(&ins)
    };
    let matched = group_indices(indices);
    (matched, not_matched)
}

/// Group neighboring indices into inclusive ranges. Assumes the provided
/// indices are sorted.
fn group_indices(indices: &[usize]) -> Vec<RangeInclusive<usize>> {
    if indices.is_empty() {
        return Vec::new();
    }
    let init = vec![indices[0]..=indices[0]];
    let fold_fun = |mut output: Vec<RangeInclusive<usize>>, &index| {
        let range = output.last_mut().unwrap();
        if *range.end() + 1 == index {
            *range = *range.start()..=index;
            output
        } else {
            output.push(index..=index);
            output
        }
    };
    indices.iter().skip(1).fold(init, fold_fun)
}
#[cfg(test)]
mod test {
    use super::group_indices;

    #[test]
    fn test_group_indices() {
        let input = vec![0, 1, 2, 6, 11, 12, 13, 21];
        let res = group_indices(&input);
        assert_eq!(vec![0..=2, 6..=6, 11..=13, 21..=21], res);
        let input = vec![4, 6, 7, 8, 9, 10, 11, 12, 13, 21];
        let res = group_indices(&input);
        assert_eq!(vec![4..=4, 6..=13, 21..=21], res);
        let input = vec![4, 6, 7, 8, 9, 10, 11, 13, 21, 22];
        let res = group_indices(&input);
        assert_eq!(vec![4..=4, 6..=11, 13..=13, 21..=22], res);
    }
}
