mod logic;
mod ui;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::app::DynRequestSender;
use crate::backend_state::CSVData;
use app_core::frontend::UIParameter;
use logic::parse_csv;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct FileID(usize);

#[derive(Default, Debug)]
pub struct FileHandler {
    pub groups: [Option<Group>; 10],
    pub registry: HashMap<FileID, File>,
    next_id: FileID,
    group_name_buffer: [String; 10],
}

#[derive(Debug)]
pub struct File {
    csv_data: UIParameter<Result<CSVData, String>>,
    pub path: PathBuf,
    pub properties: FileProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileProperties {
    pub alias: String,
    pub xoffset: f64,
    pub yoffset: f64,
    pub yscale: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Group {
    pub file_ids: HashSet<FileID>,
    pub is_plotted: bool,
    pub name: String,
}

impl FileHandler {
    pub fn new(
        groups: [Option<Group>; 10],
        registry: HashMap<FileID, File>,
        next_id: FileID,
    ) -> Self {
        Self {
            groups,
            registry,
            next_id,
            group_name_buffer: [const { String::new() }; 10],
        }
    }
}

impl File {
    pub fn new(
        path: PathBuf,
        properties: FileProperties,
        request_tx: &mut DynRequestSender,
    ) -> Self {
        let mut csv_data = UIParameter::new(Err("Data no loaded".to_string()));
        csv_data.set_recv(parse_csv(&path, request_tx));
        File {
            csv_data,
            path,
            properties,
        }
    }
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unreadable filename")
    }
}

impl Default for FileProperties {
    fn default() -> Self {
        Self {
            alias: String::new(),
            xoffset: 0.0,
            yoffset: 0.0,
            yscale: 1.0,
        }
    }
}

impl From<FileID> for i32 {
    fn from(val: FileID) -> Self {
        val.0 as i32
    }
}
