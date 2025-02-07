#![allow(unused)]

use std::{collections::HashMap, path::Path};

use app_core::string_error::ErrorStringExt;

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
        let parser =
            turbo_csv::Parser::from_path(path).err_to_string("unable to initialize parser")?;
        let (comments, columns) = parser.parse_float();

        let cache = if let Some(cache) = CSVCache::new(&columns, Some(0), 1) {
            log::debug!("add first two columns to cache");
            cache
        } else {
            log::debug!("add first column to cache");
            CSVCache::new(&columns, None, 0)
                .ok_or(format!("unable to load cache for {:?}", path))?
        };

        let num_columns = columns.len();

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
    fn new(columns: &[Vec<f64>], xcol: Option<usize>, ycol: usize) -> Option<Self> {
        let ydata = columns.get(ycol)?;
        let data = if let Some(xdata) = xcol.map(|i| columns.get(i))? {
            ydata.iter().zip(xdata).map(|(y, x)| [*x, *y]).collect()
        } else {
            let n = ydata.len();
            ydata
                .iter()
                .zip(0..n)
                .map(|(y, n)| [n as f64, *y])
                .collect()
        };
        Some(Self { data, xcol, ycol })
    }
}
