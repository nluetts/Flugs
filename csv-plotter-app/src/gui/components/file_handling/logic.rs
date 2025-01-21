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
            let fid = self.next_id.next();

            if let Some(mut grp) = self.groups.get_mut(&gid) {
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
                        id: gid,
                        name,
                    },
                );
            };

            let mut csv_data = UIParameter::new(Ok(CSVData::default()));
            csv_data.set_recv(parse_csv(&search_path.join(&fp), request_tx));

            self.registry.insert(
                fid,
                File {
                    path: search_path.join(fp),
                    csv_data,
                },
            );
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
        move |b: &mut BackendEventLoop<BackendAppState>| CSVData::from_path(&path),
    );
    request_tx
        .send(Box::new(linker))
        .expect(BACKEND_HUNG_UP_MSG);
    rx
}
