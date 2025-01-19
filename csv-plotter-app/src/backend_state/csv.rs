#![allow(unused)]

use std::{collections::HashMap, path::Path};

pub struct CSVData {
    columns: Vec<Vec<f64>>,
    num_columns: usize,
    comments: String,
    delimiter: char,
}

impl CSVData {
    pub fn from_path(path: &Path) -> Result<CSVData, String> {
        log::debug!("reading raw lines");
        // Read file contents into vector of lines (raw strings).
        let raw_lines: Vec<String> = {
            let raw_text = std::fs::read_to_string(path)
                .map_err(|e| format!("Unable to open file {:?}: {}", path, e))?;
            raw_text.split('\n').map(|line| line.to_owned()).collect()
        };

        // Try to determine comment character.
        //
        // First we count the frequency of the first characters in the
        // raw lines, ignoring digits and whitespace.
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
        log::debug!("counting first characters");
        let first_character_counts = raw_lines
            .iter()
            .fold(HashMap::new(), count_frequency_first_char);

        log::info!(
            "character char counting for {:?}: {:?}",
            path,
            first_character_counts
        );
        Err(String::new())
    }
}
