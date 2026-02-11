use std::path::Path;

use app_core::backend::{BackendEventLoop, BackendLink};

use crate::{app::DynRequestSender, BackendAppState};

impl super::Search {
    pub fn try_update(&mut self) -> bool {
        // Receive new search path, if already available.
        if let Some(handle) = self
            .awaiting_search_path_selection
            .take_if(|handle| handle.is_finished())
        {
            log::debug!("receiving new search path");
            match handle.join() {
                Ok(Some(path)) => self.set_search_path(&path),
                Ok(None) => (),
                Err(err) => log::error!("Unable to set new search directory: {:?}", err),
            }
        }
        self.search_path.try_update() || self.matches.try_update()
    }

    pub fn set_search_path(&mut self, path: &Path) {
        let new_path = path.to_owned();
        BackendLink::request_parameter_update(
            &mut self.search_path,
            "request child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| {
                b.state.set_search_path(&new_path);
                b.state.get_search_path()
            },
            &mut self.request_tx,
        );
    }

    pub fn get_search_path(&self) -> &Path {
        self.search_path.value()
    }

    pub(super) fn query_current_path(&mut self, request_tx: &mut DynRequestSender) {
        let query = self.search_query.to_owned();
        BackendLink::request_parameter_update(
            &mut self.matches,
            "fuzzy match child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| {
                let search_results = b.state.search_filter(&query);
                search_results
                    .into_iter()
                    .map(|(path, indices)| {
                        super::Match::new(path, indices, None, super::ParsedData::None)
                    })
                    .collect()
            },
            request_tx,
        );
    }

    pub fn search_single(&mut self, phrase: &str, request_tx: &mut DynRequestSender) {
        self.search_query = phrase.to_owned();
        self.query_current_path(request_tx);
    }
}
