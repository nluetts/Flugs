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

use super::{File, FileHandler, FileID, FileProperties, Group, GroupID};

impl File {
    pub fn get_cache(&self) -> Option<&Vec<[f64; 2]>> {
        self.csv_data
            .value()
            .as_ref()
            .map(|dat| &dat.get_cache().data)
            .ok()
    }

    pub fn from_storage(
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
                *fid
            } else {
                let fid = self.next_id();
                let mut csv_data = UIParameter::new(Err("Data no loaded".to_string()));
                csv_data.set_recv(parse_csv(&search_path.join(&fp), request_tx));

                self.registry.insert(
                    fid,
                    File {
                        path: search_path.join(fp),
                        csv_data,
                        properties: super::FileProperties {},
                    },
                );
                fid
            };

            // Add the ID to the group requested by user.
            if let Some(grp) = self.groups.get_mut(&gid) {
                grp.file_ids.insert(fid);
            } else {
                let mut new_file_id_set = HashSet::new();
                new_file_id_set.insert(fid);
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

    pub fn remove(
        &mut self,
        groups_to_delete: Vec<super::GroupID>,
        files_to_delete: Vec<(super::GroupID, super::FileID)>,
    ) {
        let mut item_was_removed = false;

        for (gid, fid) in files_to_delete {
            if let Some(grp) = self.groups.get_mut(&gid) {
                grp.file_ids.remove(&fid);
                let file_name = self
                    .registry
                    .get(&fid)
                    .and_then(|file| file.path.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("unreadable filename");
                log::debug!(
                    "removed file '{file_name}' from group {} with ID {gid:?}",
                    grp.name
                );
                item_was_removed = true;
            } else {
                log::warn!("trying to remove file from group with ID {gid:?} which does not exist");
            }
        }

        for gid in groups_to_delete.into_iter() {
            if let Some(grp) = self.groups.remove(&gid) {
                log::debug!("removed group '{}' with ID '{gid:?}'", grp.name);
                item_was_removed = true;
            } else {
                log::warn!("trying to remove group with ID {gid:?} which does not exist");
            }
        }

        // If for some reason nothing was remove (which is currently impossible,
        // but maybe can occur in the future) we omit checking the registry for
        // files to remove.
        if !item_was_removed {
            return;
        }
        // Remove files from registry which are not member of any group.
        let mut mark_delete = Vec::new();
        for fid in self.registry.keys() {
            if !self.groups.values().any(|grp| grp.file_ids.contains(&fid)) {
                mark_delete.push(*fid);
            }
        }
        for fid in mark_delete.into_iter() {
            let file_name = self
                .registry
                .get(&fid)
                .and_then(|file| file.path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("unreadable filename");
            log::debug!("remove file '{file_name}' from registry");
            self.registry.remove(&fid);
        }
    }

    pub fn current_id(&self) -> FileID {
        self.next_id
    }

    fn next_id(&mut self) -> FileID {
        let fid = self.next_id;
        self.next_id.0 += 1;
        fid
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
        move |_b: &mut BackendEventLoop<BackendAppState>| {
            CSVData::from_path(&path).map_err(|err| {
                log::error!("{}", err);
                err
            })
        },
    );
    request_tx
        .send(Box::new(linker))
        .expect(BACKEND_HUNG_UP_MSG);
    rx
}
