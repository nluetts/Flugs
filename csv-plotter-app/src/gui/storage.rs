use std::path::PathBuf;

use app_core::storage::Storage;
use serde::{Deserialize, Serialize};

use crate::EguiApp;

#[derive(Clone, Serialize, Deserialize)]
struct BackendStorage {}

#[derive(Serialize, Deserialize)]
struct FrontendStorage {
    search_path: PathBuf,
}

pub fn save_json(app: &EguiApp) -> Result<(), String> {
    let backend_storage = BackendStorage {};

    let frontend_storage = FrontendStorage {
        search_path: app.search.get_search_path().to_path_buf(),
    };
    let storage = Storage::new(backend_storage, frontend_storage);
    storage.save_json()
}

pub fn load_json(app: &mut EguiApp) -> Result<(), String> {
    let Storage::<BackendStorage, FrontendStorage> {
        backend_storage,
        frontend_storage,
    } = Storage::from_json()?;
    app.search
        .set_search_path(&frontend_storage.search_path, &mut app.request_tx);
    Ok(())
}
