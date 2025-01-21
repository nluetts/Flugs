#![allow(unused)]

use std::{collections::HashMap, path::Path};

use app_core::frontend::UIParameter;

#[derive(Debug, Default, Clone)]
pub struct CSVCache {
    pub data: Vec<[f64; 2]>,
    pub xcol: Option<usize>,
    pub ycol: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CSVData {
    columns: Vec<Vec<f64>>,
    num_columns: usize,
    comments: String,
    cache: CSVCache,
}

// Helper struct to counts frequencies of potential delimiter characters.
#[derive(Debug)]
struct DelimiterCounter {
    char: char,
    // How often a certain count per row is found.
    row_counter: HashMap<usize, usize>,
}

impl CSVData {
    pub fn from_path(path: &Path) -> Result<CSVData, String> {
        log::debug!("reading raw rows for {:?}", path);
        // Read file contents into vector of rows (raw strings).
        let raw_rows: Vec<String> = {
            let raw_text = std::fs::read_to_string(path)
                .map_err(|e| format!("Unable to open file {:?}: {}", path, e))?;
            raw_text
                .split('\n')
                .map(|row| {
                    let row = row.to_owned();
                    // Remove surrounding whitespace.
                    row.trim();
                    row
                })
                .collect()
        };
        // DEBUG OK
        dbg!(raw_rows.get(0));

        // Try to determine comment character.
        //
        // We count the frequency of the first characters in the raw rows,
        // ignoring digits and whitespace.
        let count_frequency_first_char = |mut counts: HashMap<char, usize>, row: &String| {
            if let Some(character) = row.chars().nth(0) {
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
        let (comment_char, counts) = raw_rows
            .iter()
            // This counts the frequencies of the first character in `raw_rows`
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
            raw_rows
                .iter()
                .filter_map(|row| {
                    if row.starts_with(comment_char) {
                        Some(row.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            String::new()
        };

        // Try to determine cell delimiter.
        //
        // In the rows holding data, count the frequencies of all characters
        // that could potentially be delimiter characters. Counting is done on
        // a per row basis, i.e. the result is a hash map mapping the number of
        // occurances per line to a line count.
        let init_delim_char = DelimiterCounter {
            char: '#',
            row_counter: HashMap::new(),
        };
        let init_counter: HashMap<char, DelimiterCounter> = HashMap::new();
        let delimiter = raw_rows
            .iter()
            .fold(init_counter, |mut counter, row| {
                // Count delimiter chars in curent row.
                let mut delimiter_counts_current_row: HashMap<char, usize> = HashMap::new();
                for (i, c) in row.chars().enumerate() {
                    // We assume that only rows starting with digits contain data.
                    if i == 0 && !c.is_ascii_digit() {
                        return counter;
                    }
                    // Characters `.`, `e`, `E`, `+`, `-` (all characters that
                    // can occur in scientific numbers, but which are never used
                    // as delimiters, plus the quote sign) are ignored.
                    if c.is_ascii_digit() || ['.', 'e', 'E', '+', '-', '"'].contains(&c) {
                        continue;
                    }
                    if let Some(counts) = delimiter_counts_current_row.get_mut(&c) {
                        *counts += 1;
                    } else {
                        delimiter_counts_current_row.insert(c, 1);
                    }
                }
                // Merge the result of the current row with `counter`.
                for (c, count) in delimiter_counts_current_row.into_iter() {
                    if let Some(ctr) = counter.get_mut(&c) {
                        // We note how often we have seen this count for this specific char `c`.
                        if let Some(row_count) = ctr.row_counter.get_mut(&count) {
                            *row_count += 1;
                        } else {
                            ctr.row_counter.insert(count, 1);
                        }
                    } else {
                        let mut row_counter = HashMap::new();
                        row_counter.insert(count, 1);
                        counter.insert(
                            c,
                            DelimiterCounter {
                                char: c,
                                row_counter,
                            },
                        );
                    }
                }
                counter
            })
            .into_values()
            // This reduces the hash map to the delimiter character with the
            // highest number of counts.
            .fold(init_delim_char, |delim_counter_a, delim_counter_b| {
                if delim_counter_b.row_counter.values().sum::<usize>()
                    > delim_counter_a.row_counter.values().sum()
                {
                    delim_counter_b
                } else {
                    delim_counter_a
                }
            });

        log::debug!(
            "delimiter character {} with {} counts found (per row counts: {:?})",
            &delimiter.char,
            &delimiter.row_counter.values().sum::<usize>(),
            &delimiter.row_counter
        );

        // We assume the most frequent number of delimiters per line
        // yields the correct number of columns (no. delimter + 1).
        let num_columns = delimiter
            .row_counter
            .iter()
            .max_by(|(&num_a, &freq_a), (&num_b, &freq_b)| freq_b.cmp(&freq_a))
            .ok_or(format!("Did not count any delimiters for file {:?}", path))?
            .0
            + 1;

        // Finally, parse the contents of the file.
        let mut columns: Vec<Vec<f64>> = (0..num_columns).map(|_| Vec::new()).collect();
        let mut row_buffer: Vec<f64> = vec![0.0; num_columns];
        let mut num_ignored = 0;
        'outer: for (i, row) in raw_rows.iter().enumerate() {
            log::debug!("parse row '{}'", &row);
            // Skip rows starting with comment char or non-digit, as well as
            // empty rows.
            if let Some(first_char) = row.chars().nth(0) {
                if first_char == comment_char || !first_char.is_ascii_digit() {
                    continue;
                }
            } else {
                continue;
            }

            for (j, entry) in row.split(delimiter.char).enumerate() {
                // Remove whitespace and `"` (if present) before parsing.
                match entry.trim().trim_matches('"').parse::<f64>() {
                    Ok(num) => row_buffer[j] = num,
                    Err(e) => {
                        log::debug!("failed to parse row {i} entry {j}: {e}");
                        break 'outer;
                        // If we cannot parse an entry, we ignore the entire row.
                        num_ignored += 1;
                        continue;
                    }
                }
            }
            for (i, num) in row_buffer.iter().enumerate() {
                columns[i].push(*num);
            }
        }

        for i in 0..num_columns {
            dbg!(&columns[i][0]);
        }

        log::debug!(
            "parsing {:?} succesful",
            path.file_name().and_then(|p| p.to_str()).unwrap_or("file")
        );

        let cache = if let Some(cache) = CSVCache::new(&columns, Some(0), 1) {
            log::debug!("add first two columns to cache");
            cache
        } else {
            log::debug!("add first column to cache");
            CSVCache::new(&columns, None, 0)
                .ok_or(format!("unable to load cache for {:?}", path))?
        };

        // DEBUG
        // std::thread::sleep_ms(10000);

        Ok(CSVData {
            columns,
            num_columns,
            comments,
            cache,
        })
    }

    pub fn get_cache(&self) -> &CSVCache {
        &self.cache
    }
}

impl CSVCache {
    fn new(columns: &Vec<Vec<f64>>, xcol: Option<usize>, ycol: usize) -> Option<Self> {
        let ydata = columns.get(ycol)?;
        let data = if let Some(xdata) = xcol.map(|i| columns.get(i))? {
            ydata
                .into_iter()
                .zip(xdata.into_iter())
                .map(|(y, x)| [*x, *y])
                .collect()
        } else {
            let n = ydata.len();
            ydata
                .into_iter()
                .zip(0..n)
                .map(|(y, n)| [n as f64, *y])
                .collect()
        };
        Some(Self { data, xcol, ycol })
    }
}
