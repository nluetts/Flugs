mod logic;
mod ui;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::app::DynRequestSender;
use crate::backend_state::PlotData;
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
    active_element: ActiveElement,
}

#[derive(Debug)]
pub struct File {
    pub data: UIParameter<Result<PlotData, String>>,
    pub path: PathBuf,
    pub properties: FileProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileProperties {
    pub alias: String,
    pub xoffset: f64,
    pub yoffset: f64,
    pub yscale: f64,
    pub comment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub file_ids: Vec<FileID>,
    pub is_plotted: bool,
    pub name: String,
}

#[derive(Debug)]
enum ActiveElement {
    Group(usize),
    File(FileID, usize),
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
            active_element: ActiveElement::Group(0),
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
            data: csv_data,
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
            comment: String::new(),
        }
    }
}

impl From<FileID> for i32 {
    fn from(val: FileID) -> Self {
        val.0 as i32
    }
}

impl Default for Group {
    fn default() -> Self {
        Self {
            file_ids: Default::default(),
            is_plotted: true,
            name: Default::default(),
        }
    }
}

impl Default for ActiveElement {
    fn default() -> Self {
        Self::Group(0)
    }
}
