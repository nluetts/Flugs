#![allow(unused)]

use std::{collections::HashMap, path::PathBuf};

use std::collections::HashSet;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FileID(usize);

#[derive(Default, Debug)]
pub struct FileHandler {
    groups: HashMap<GroupID, Group>,
    registry: HashMap<FileID, File>,
    next_id: FileID,
}

#[derive(Debug)]
struct File {
    path: PathBuf,
}

#[derive(Debug)]
struct Group {
    file_ids: HashSet<FileID>,
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

impl FileHandler {
    pub fn handle_search_results(&mut self, search_results: HashSet<(PathBuf, GroupID)>) {
        for (fp, gid) in search_results.into_iter() {
            let fid = self.next_id.next();

            if let Some(mut grp) = self.groups.get_mut(&gid) {
                grp.file_ids.insert(fid.clone());
            } else {
                let mut new_file_id_set = HashSet::new();
                new_file_id_set.insert(fid.clone());
                let new_grp = Group {
                    file_ids: new_file_id_set,
                };
                self.groups.insert(gid, new_grp);
            };

            self.registry.insert(fid, File { path: fp });
        }
    }
}
