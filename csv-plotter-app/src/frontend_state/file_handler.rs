use std::collections::HashMap;

#[derive(Default)]
pub struct FileHandler {
    groups: Vec<Group>,
    registry: HashMap<FileID, File>,
}

type FileID = usize;

struct File {}

struct Group {
    file_ids: Vec<FileID>,
}
