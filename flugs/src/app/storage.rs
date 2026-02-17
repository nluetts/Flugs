use std::{
    collections::HashMap,
    path::{Path, PathBuf},
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
    plot_bounds: Option<[f64; 4]>,
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
        plot_bounds: Some({
            let bounds = app.plotter.get_current_plot_bounds();
            let ([xmin, ymin], [xmax, ymax]) = (bounds.min(), bounds.max());
            [xmin, xmax, ymin, ymax]
        }),
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
    if let Some(bounds) = frontend_storage.plot_bounds {
        app.plotter.apply_bounds(bounds);
    }
    app.file_handler = frontend_storage.into_file_handler(&mut app.request_tx);
    app.request_redraw();
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
