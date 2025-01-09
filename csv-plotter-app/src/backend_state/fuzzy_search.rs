use crate::BackendAppState;

use fuzzy_matcher::FuzzyMatcher;
use std::ops::RangeInclusive;

impl BackendAppState {
    /// Return the best file path matches for `query`, together with the
    /// corresponding matching indices in the file path.
    pub fn fuzzy_filter(&self, query: &str) -> Vec<(std::path::PathBuf, Vec<usize>)> {
        let mut res: Vec<_> = self
            .child_paths_unfiltered
            .iter()
            .filter_map(|fp| fp.to_str().map(|str| (fp, str)))
            .filter_map(|(fp, str)| {
                self.fzm
                    .fuzzy_indices(str, query)
                    .map(|(score, indices)| (fp, score, indices))
            })
            .collect();
        res.sort_unstable_by(|(_, score_a, _), (_, score_b, _)| score_b.cmp(score_a));
        res.into_iter()
            .map(|(fp, _score, indices)| (fp.to_owned(), indices))
            .take(10)
            .collect()
    }
}

/// For a str, given the matched indices, return ranges
/// of matched/not matched indices (for formatting).
pub fn get_matched_unmatch_str_index_groups(
    s: &str,
    indices: &Vec<usize>,
) -> (Vec<RangeInclusive<usize>>, Vec<RangeInclusive<usize>>) {
    let not_matched: Vec<RangeInclusive<usize>> = {
        let ins = (0..s.len()).filter(|n| !indices.contains(n)).collect();
        group_indices(&ins)
    };
    let matched = group_indices(indices);
    (matched, not_matched)
}

/// Group neighboring indices into inclusive ranges. Assumes the provided
/// indices are sorted.
fn group_indices(indices: &Vec<usize>) -> Vec<RangeInclusive<usize>> {
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
    indices.into_iter().skip(1).fold(init, fold_fun)
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
