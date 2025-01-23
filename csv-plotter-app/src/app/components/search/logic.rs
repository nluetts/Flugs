use std::path::{Path, PathBuf};

use app_core::backend::{BackendEventLoop, BackendLink};

use crate::{app::DynRequestSender, BackendAppState};

impl super::Search {
    pub fn try_update(&mut self) {
        self.search_path.try_update();
        self.matched_paths.try_update();
    }

    pub fn set_search_path(&mut self, path: &PathBuf, request_tx: &mut DynRequestSender) {
        let new_path = path.to_owned();
        BackendLink::request_parameter_update(
            &mut self.search_path,
            "request child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| {
                b.state.set_search_path(&new_path);
                b.state.get_search_path()
            },
            request_tx,
        );
    }

    pub fn get_search_path(&self) -> &Path {
        self.search_path.value()
    }

    pub(super) fn query_current_path(&mut self, request_tx: &mut DynRequestSender) {
        let query = self.search_query.to_owned();
        BackendLink::request_parameter_update(
            &mut self.matched_paths,
            "fuzzy match child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| b.state.search_filter(&query),
            request_tx,
        );
    }
}
