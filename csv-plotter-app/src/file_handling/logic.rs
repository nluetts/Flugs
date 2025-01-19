use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use app_core::{
    backend::{BackendEventLoop, BackendLink},
    BACKEND_HUNG_UP_MSG,
};

use crate::{backend_state::CSVData, gui::DynRequestSender, BackendAppState};

use super::{File, Group, GroupID};

impl super::FileHandler {
    pub fn add_search_results(
        &mut self,
        search_results: HashSet<(PathBuf, GroupID)>,
        search_path: &Path,
    ) {
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

            self.registry.insert(
                fid,
                File {
                    path: search_path.join(fp),
                },
            );
        }
    }
}

pub fn parse_csv(path: &Path, request_tx: &mut DynRequestSender) {
    let path = path.to_owned();
    let (rx, linker) = BackendLink::new(
        &format!("load CSV data from file {:?}", path),
        move |b: &mut BackendEventLoop<BackendAppState>| {
            CSVData::from_path(&path);
        },
    );
    request_tx
        .send(Box::new(linker))
        .expect(BACKEND_HUNG_UP_MSG);
    rx.recv_timeout(std::time::Duration::from_secs(1))
        .expect("Just temporary for debugging purposes");
}
