use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use app_core::{
    backend::{BackendEventLoop, BackendLink, LinkReceiver},
    frontend::UIParameter,
    BACKEND_HUNG_UP_MSG,
};

use crate::{backend_state::CSVData, gui::DynRequestSender, BackendAppState};

use super::{File, FileHandler, Group, GroupID};

impl File {
    pub fn get_cache(&self) -> Option<&Vec<[f64; 2]>> {
        self.csv_data
            .value()
            .as_ref()
            .map(|dat| &dat.get_cache().data)
            .ok()
    }
}

impl FileHandler {
    pub fn add_search_results(
        &mut self,
        search_results: HashSet<(PathBuf, GroupID)>,
        search_path: &Path,
        request_tx: &mut DynRequestSender,
    ) {
        for (fp, gid) in search_results.into_iter() {
            // If file is already registered, we pull its ID from the registry,
            // otherwise we create a new ID and add the file to the registry.
            let fid = if let Some((fid, _)) = self
                .registry
                .iter()
                .filter(|(_, file)| file.path == search_path.join(&fp))
                .next()
            {
                fid.clone()
            } else {
                let fid = self.next_id.next();
                let mut csv_data = UIParameter::new(Ok(CSVData::default()));
                csv_data.set_recv(parse_csv(&search_path.join(&fp), request_tx));

                self.registry.insert(
                    fid.clone(),
                    File {
                        path: search_path.join(fp),
                        csv_data,
                    },
                );
                fid
            };

            // Add the ID to the group requested by user.
            if let Some(grp) = self.groups.get_mut(&gid) {
                grp.file_ids.insert(fid.clone());
            } else {
                let mut new_file_id_set = HashSet::new();
                new_file_id_set.insert(fid.clone());
                let name = format!("Group ({})", gid.0);
                self.groups.insert(
                    gid.clone(),
                    Group {
                        file_ids: new_file_id_set,
                        is_plotted: false,
                        _id: gid,
                        name,
                    },
                );
            };
        }
    }

    pub fn remove(&mut self, to_delete: Vec<(super::GroupID, super::FileID)>) {
        for (gid, fid) in to_delete {
            if let Some(grp) = self.groups.get_mut(&gid) {
                grp.file_ids.remove(&fid);
            } else {
                log::warn!("trying to remove file from group with ID {gid:?} which does not exist");
            }

            // If file with ID `fid` is not a member of any group, we remove it
            // from the registry as well.
            if !self.groups.values().any(|grp| grp.file_ids.contains(&fid)) {
                self.registry.remove(&fid);
            }
        }
    }

    pub fn try_update(&mut self) {
        for file in self.registry.values_mut() {
            file.csv_data.try_update();
        }
    }
}

pub fn parse_csv(
    path: &Path,
    request_tx: &mut DynRequestSender,
) -> LinkReceiver<Result<CSVData, String>> {
    let path = path.to_owned();
    let (rx, linker) = BackendLink::new(
        &format!("load CSV data from file {:?}", path),
        move |_b: &mut BackendEventLoop<BackendAppState>| CSVData::from_path(&path),
    );
    request_tx
        .send(Box::new(linker))
        .expect(BACKEND_HUNG_UP_MSG);
    rx
}
