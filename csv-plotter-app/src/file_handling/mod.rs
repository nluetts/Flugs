#![allow(unused)]

mod logic;
mod ui;

use std::collections::HashSet;
use std::{collections::HashMap, path::PathBuf};

use crate::backend_state::CSVData;
use app_core::frontend::UIParameter;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FileID(usize);

#[derive(Default, Debug)]
pub struct FileHandler {
    pub groups: HashMap<GroupID, Group>,
    pub registry: HashMap<FileID, File>,
    next_id: FileID,
}

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    csv_data: UIParameter<Result<CSVData, String>>,
}

#[derive(Debug)]
pub struct Group {
    pub file_ids: HashSet<FileID>,
    pub is_plotted: bool,
    id: GroupID,
    pub name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct GroupID(usize);

impl GroupID {
    pub fn new(id: usize) -> Self {
        GroupID(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl FileID {
    fn next(&mut self) -> FileID {
        let id = self.clone();
        self.0 += 1;
        id
    }
}
