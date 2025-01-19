#![allow(unused)]

use std::{ascii::AsciiExt, collections::HashMap, path::Path};

pub struct CSVData {
    columns: Vec<Vec<f64>>,
    num_columns: usize,
    comments: String,
}

impl CSVData {
    pub fn from_path(path: &Path) -> Result<CSVData, String> {
        log::debug!("reading raw lines for {:?}", path);
        // Read file contents into vector of lines (raw strings).
        let raw_lines: Vec<String> = {
            let raw_text = std::fs::read_to_string(path)
                .map_err(|e| format!("Unable to open file {:?}: {}", path, e))?;
            raw_text
                .split('\n')
                .map(|line| {
                    let line = line.to_owned();
                    // Remove surrounding whitespace.
                    line.trim();
                    line
                })
                .collect()
        };

        // Try to determine comment character.
        //
        // We count the frequency of the first characters in the raw lines,
        // ignoring digits and whitespace.
        let count_frequency_first_char = |mut counts: HashMap<char, usize>, line: &String| {
            if let Some(character) = line.chars().nth(0) {
                if character.is_ascii_digit() || character.is_whitespace() {
                    return counts;
                }
                if let Some(count) = counts.get_mut(&character) {
                    *count += 1;
                } else {
                    counts.insert(character, 1);
                }
            }
            counts
        };
        let (comment_char, counts) = raw_lines
            .iter()
            // This counts the frequencies of the first character in `raw_lines`
            // and returns a hash map of character => counts.
            .fold(HashMap::new(), count_frequency_first_char)
            .into_iter()
            // This reduces the hash map to the character with the highest
            // number of counts
            .fold(('#', 0), |(char_a, counts_a), (char_b, counts_b)| {
                if counts_b > counts_a {
                    (char_b, counts_b)
                } else {
                    (char_a, counts_a)
                }
            });
        log::debug!(
            "comment character {} with {} counts found",
            comment_char,
            counts
        );

        // Read out the comments in the CSV file, if any.
        let comments = if counts > 0 {
            raw_lines
                .iter()
                .filter_map(|line| {
                    if line.starts_with(comment_char) {
                        Some(line.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
        } else {
            String::new();
        };

        // Try to determine cell delimiter.
        //
        // We do something very similar to the above, we counts all character
        // in the lines which are not comments and select the character with the
        // highest occurance.
        let count_frequencies_delimiter = |mut counts: HashMap<char, usize>, line: &String| {
            for char in line.chars() {
                if char.is_ascii_digit() || ['.', 'e', 'E', '+', '-'].contains(&char) {
                    continue;
                }
                if let Some(count) = counts.get_mut(&char) {
                    *count += 1;
                } else {
                    counts.insert(char, 1);
                }
            }
            counts
        };

        let (delimiter_char, counts) = raw_lines
            .iter()
            // We only consider lines which start with digits.
            .filter(|line| {
                line.chars()
                    .nth(0)
                    .map(|char| char.is_ascii_digit())
                    .unwrap_or_default()
            })
            // This counts the frequencies of all non-digit characters, except
            // `.`, `e`, `E`, `+`, `-` (all characters that can occur in
            // scientific numbers, but which are never used as delimiters) and
            // returns a hash map of character => counts.
            .fold(HashMap::new(), count_frequencies_delimiter)
            .into_iter()
            // This reduces the hash map to the character with the highest
            // number of counts
            .fold((',', 0), |(char_a, counts_a), (char_b, counts_b)| {
                if counts_b > counts_a {
                    (char_b, counts_b)
                } else {
                    (char_a, counts_a)
                }
            });
        log::debug!(
            "delimiter character {} with {} counts found",
            delimiter_char,
            counts
        );

        Err(String::new())
    }
}
