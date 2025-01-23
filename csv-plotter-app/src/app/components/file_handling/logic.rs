use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use app_core::{
    backend::{BackendEventLoop, BackendLink, LinkReceiver},
    frontend::UIParameter,
    BACKEND_HUNG_UP_MSG,
};

use crate::{app::DynRequestSender, backend_state::CSVData, BackendAppState};

use super::{File, FileHandler, FileID, Group};

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
        search_results: HashSet<(PathBuf, usize)>,
        search_path: &Path,
        request_tx: &mut DynRequestSender,
    ) {
        for (fp, gid) in search_results.into_iter() {
            if gid > 9 {
                log::warn!("Group ID > 9 invalid, only 10 slots available, ignoring");
            }
            // If file is already registered, we pull its ID from the registry,
            // otherwise we create a new ID and add the file to the registry.
            let fid = if let Some((fid, _)) = self
                .registry
                .iter()
                .find(|(_, file)| file.path == search_path.join(&fp))
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
            if let Some(grp) = &mut self.groups[gid] {
                grp.file_ids.insert(fid);
            } else {
                let mut new_file_id_set = HashSet::new();
                new_file_id_set.insert(fid);
                let name = format!("Group ({})", gid);
                self.groups[gid] = Some(Group {
                    file_ids: new_file_id_set,
                    is_plotted: false,
                    name,
                });
            };
        }
    }

    pub fn remove(
        &mut self,
        groups_to_delete: Vec<usize>,
        files_to_delete: Vec<(usize, super::FileID)>,
    ) {
        let mut item_was_removed = false;

        // Just in case, we filter gid which would lead to a panic when used as index.
        for (gid, fid) in files_to_delete.into_iter().filter(|(gid, _)| *gid < 10) {
            let file_name = self.fid_to_filename_str(&fid).to_string();
            if let Some(grp) = &mut self.groups[gid] {
                grp.file_ids.remove(&fid);
                log::debug!(
                    "removed file '{file_name}' from group {} with ID {gid:?}",
                    grp.name
                );
                item_was_removed = true;
            } else {
                log::warn!("trying to remove file from group with ID {gid:?} which does not exist");
            }
        }

        for gid in groups_to_delete.into_iter().filter(|gid| *gid < 10) {
            if let Some(Some(grp)) = self.groups.get(gid) {
                log::debug!("removed group '{}' with ID '{gid}'", grp.name);
            } else {
                log::warn!("trying to remove group with ID {gid} which does not exist");
            }
            self.groups[gid] = None;
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
            if self
                .groups
                .iter()
                .filter_map(|x| x.as_ref())
                .any(|grp| grp.file_ids.contains(fid))
            {
                mark_delete.push(*fid);
            }
        }
        for fid in mark_delete.into_iter() {
            log::debug!(
                "remove file '{}' from registry",
                self.fid_to_filename_str(&fid)
            );
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

    fn fid_to_filename_str(&self, fid: &FileID) -> &str {
        self.registry
            .get(fid)
            .map(|file| file.file_name())
            .unwrap_or("unreadable filename")
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
