use std::ops::RangeInclusive;

/// Group neighboring indices into inclusive ranges. Assumes the provided
/// indices are sorted.
pub fn group_indices(indices: Vec<usize>) -> Vec<RangeInclusive<usize>> {
    if indices.is_empty() {
        return Vec::new();
    }
    let init = vec![indices[0]..=indices[0]];
    let fold_fun = |mut output: Vec<RangeInclusive<usize>>, index| {
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
        let res = group_indices(input);
        assert_eq!(vec![0..=2, 6..=6, 11..=13, 21..=21], res);
        let input = vec![4, 6, 7, 8, 9, 10, 11, 12, 13, 21];
        let res = group_indices(input);
        assert_eq!(vec![4..=4, 6..=13, 21..=21], res);
        let input = vec![4, 6, 7, 8, 9, 10, 11, 13, 21, 22];
        let res = group_indices(input);
        assert_eq!(vec![4..=4, 6..=11, 13..=13, 21..=22], res);
    }
}
