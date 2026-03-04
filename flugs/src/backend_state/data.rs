#![allow(unused)]

use std::{collections::HashMap, path::Path};

use app_core::string_error::ErrorStringExt;
use egui_plot::PlotPoint;

#[derive(Clone, Debug, Default)]
pub struct PlotData {
    pub columns: Vec<Vec<f64>>,
    num_columns: usize,
    comments: String,
    cache: Vec<PlotPoint>,
}

// Helper struct to counts frequencies of potential delimiter characters.
#[derive(Debug)]
struct DelimiterCounter {
    char: char,
    // How often a certain count per row is found.
    row_counter: HashMap<usize, usize>,
}

impl PlotData {
    pub fn from_path(path: &Path) -> Result<PlotData, String> {
        let (comments, columns) = if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.parse::<u32>().is_ok())
            .unwrap_or_default()
        {
            // If the file extension is an integer (.0, .1, etc.), we try to parse as a bruker file.
            let bruker_parser::OpusAbsorbanceData {
                wavenumber,
                absorbance,
            } = bruker_parser::OpusAbsorbanceData::from_path(path)?;
            (String::new(), vec![wavenumber, absorbance])
        } else {
            // Otherwise, we try to parse as CSV.
            let parser =
                turbo_csv::Parser::from_path(path).err_to_string("unable to initialize parser")?;
            parser.parse_as_floats()
        };

        let cache = if let Some(cache) = new_cache(&columns, Some(0), 1) {
            log::debug!("add first two columns to cache");
            cache
        } else {
            log::debug!("add first column to cache");
            new_cache(&columns, None, 0).ok_or(format!("unable to load cache for {:?}", path))?
        };

        let num_columns = columns.len();

        Ok(PlotData {
            columns,
            num_columns,
            comments,
            cache,
        })
    }

    pub fn get_cache(&self) -> &[PlotPoint] {
        &self.cache
    }

    pub fn regenerate_cache(&mut self, x_col: usize, y_col: usize) {
        if let Some(cache) = new_cache(&self.columns, Some(x_col), y_col) {
            self.cache = cache;
        }
    }

    pub fn rescale(&mut self, xoffset: f64, yoffset: f64, yscale: f64) -> Option<()> {
        // TODO: Selected columns are hard-coded, for now
        self.cache = scaled_cache(&self.columns, Some(0), 1, xoffset, yoffset, yscale)?;
        Some(())
    }

    pub fn ymin(&self, ycol: usize) -> Option<f64> {
        self.columns
            .get(ycol)
            .and_then(|ys| ys.iter().reduce(|a, b| if a < b { a } else { b }))
            .copied()
    }

    pub fn get_comments(&self) -> String {
        self.comments.clone()
    }
}

fn new_cache(columns: &[Vec<f64>], xcol: Option<usize>, ycol: usize) -> Option<Vec<PlotPoint>> {
    let ydata = columns.get(ycol)?;
    let data = if let Some(xdata) = xcol.map(|i| columns.get(i))? {
        xdata
            .iter()
            .zip(ydata)
            .map(|(&x, &y)| PlotPoint { x, y })
            .collect()
    } else {
        ydata
            .iter()
            .zip(0..)
            .map(|(&y, n)| PlotPoint { x: n as f64, y })
            .collect()
    };
    Some(data)
}

fn scaled_cache(
    columns: &[Vec<f64>],
    xcol: Option<usize>,
    ycol: usize,
    xoffset: f64,
    yoffset: f64,
    yscale: f64,
) -> Option<Vec<PlotPoint>> {
    let ydata = columns.get(ycol)?;
    let data = if let Some(xdata) = xcol.map(|i| columns.get(i))? {
        xdata
            .iter()
            .zip(ydata)
            .map(|(&x, &y)| PlotPoint {
                x: x + xoffset,
                y: (y * yscale) + yoffset,
            })
            .collect()
    } else {
        ydata
            .iter()
            .zip(0..)
            .map(|(&y, n)| PlotPoint {
                x: n as f64,
                y: (y * yscale) + yoffset,
            })
            .collect()
    };
    Some(data)
}
