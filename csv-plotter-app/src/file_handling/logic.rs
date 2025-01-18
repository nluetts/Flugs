use std::{collections::HashSet, path::PathBuf};

use super::{File, Group, GroupID};

impl super::FileHandler {
    pub fn handle_search_results(&mut self, search_results: HashSet<(PathBuf, GroupID)>) {
        for (fp, gid) in search_results.into_iter() {
            let fid = self.next_id.next();

            if let Some(mut grp) = self.groups.get_mut(&gid) {
                grp.file_ids.insert(fid.clone());
            } else {
                let mut new_file_id_set = HashSet::new();
                new_file_id_set.insert(fid.clone());
                let name = format!("Group ({})", gid.0 + 1);
                self.groups.insert(
                    gid.clone(),
                    Group {
                        file_ids: new_file_id_set,
                        is_plotted: false,
                        id: gid,
                        name,
                    },
                );
            };

            self.registry.insert(fid, File { path: fp });
        }
    }
}
