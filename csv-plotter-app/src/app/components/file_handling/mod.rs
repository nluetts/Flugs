mod logic;
mod ui;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::backend_state::CSVData;
use app_core::frontend::UIParameter;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct FileID(usize);

#[derive(Default, Debug)]
pub struct FileHandler {
    pub groups: HashMap<GroupID, Group>,
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
    _id: GroupID,
    pub name: String,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct GroupID(usize);

impl File {
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unreadable filename")
    }
}

impl GroupID {
    pub fn new(id: usize) -> Self {
        GroupID(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}
