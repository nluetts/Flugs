#![allow(unused)]

use std::{collections::HashMap, path::PathBuf};

pub type FileID = usize;

#[derive(Default)]
pub struct FileHandler {
    groups: Vec<Group>,
    registry: HashMap<FileID, File>,
}

struct File {
    path: PathBuf,
}

struct Group {
    file_ids: Vec<FileID>,
}
