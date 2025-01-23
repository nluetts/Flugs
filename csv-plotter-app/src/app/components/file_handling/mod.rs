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
}

#[derive(Debug)]
pub struct File {
    csv_data: UIParameter<Result<CSVData, String>>,
    pub path: PathBuf,
    pub properties: FileProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileProperties {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub file_ids: HashSet<FileID>,
    pub is_plotted: bool,
    // _id: GroupID,
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
