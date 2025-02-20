use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use app_core::storage::Storage;
use serde::{Deserialize, Serialize};

use crate::EguiApp;

use super::{
    components::{File, FileID, FileProperties, Group},
    DynRequestSender, FileHandler,
};

// Currently not used, since the only backend state to safe right now is the
// search path, which is also mirrored in the frontend (app.search).
#[derive(Clone, Serialize, Deserialize)]
struct BackendStorage {}

#[derive(Serialize, Deserialize)]
struct FrontendStorage {
    search_path: PathBuf,
    groups: [Option<Group>; 10],
    registry: HashMap<FileID, FileStorage>,
    next_id: FileID,
}

pub fn save_json(app: &EguiApp, path: Option<&Path>) -> Result<(), String> {
    let backend_storage = BackendStorage {};

    let frontend_storage = FrontendStorage {
        search_path: app.search.get_search_path().to_path_buf(),
        groups: app.file_handler.groups.clone(),
        registry: app
            .file_handler
            .registry
            .iter()
            .map(|(fid, file)| {
                (
                    *fid,
                    FileStorage {
                        path: file.path.clone(),
                        properties: file.properties.clone(),
                    },
                )
            })
            .collect(),
        next_id: app.file_handler.current_id(),
    };
    let storage = Storage::new(backend_storage, frontend_storage);
    storage.save_json(path)
}

pub fn load_json(app: &mut EguiApp, path: Option<&Path>) -> Result<(), String> {
    let Storage::<BackendStorage, FrontendStorage> {
        backend_storage: _,
        frontend_storage,
    } = Storage::load_json(path)?;

    app.search.set_search_path(&frontend_storage.search_path);
    app.file_handler = frontend_storage.into_file_handler(&mut app.request_tx);
    Ok(())
}

// Serializing the files is a special case, because we do not want to store the
// entire data contained in the csv files (plus it is non-trivial to do this).
#[derive(Serialize, Deserialize)]
struct FileStorage {
    path: PathBuf,
    properties: FileProperties,
}

impl FrontendStorage {
    fn into_file_handler(self, request_tx: &mut DynRequestSender) -> FileHandler {
        let groups = self.groups;
        let registry = self
            .registry
            .into_iter()
            .map(|(fid, file_storage)| {
                (
                    fid,
                    File::new(file_storage.path, file_storage.properties, request_tx),
                )
            })
            .collect();

        FileHandler::new(groups, registry, self.next_id)
    }
}

pub fn load_config() -> Option<PathBuf> {
    let mut search_path = None;
    #[allow(deprecated)]
    let home = std::env::home_dir();
    if home.is_none() {
        log::warn!("could not determine home directory to load config file");
    }
    let path = home.map(|home| home.join(PathBuf::from(".plotme_global_settings")))?;
    let mut file = std::fs::File::open(path)
        .map_err(|err| log::warn!("could not load config file: {:?}", err))
        .ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    for line in buf.lines() {
        let mut iter = line.split("=");
        let key = iter.next();
        let val = iter.next();
        let (key, val) = match (key, val) {
            (Some(key), Some(val)) => (key, val),
            _ => continue,
        };
        if key == "search_path" {
            let _ = PathBuf::from_str(val).map(|path| {
                search_path = Some(path);
            });
        }
    }
    search_path
}
