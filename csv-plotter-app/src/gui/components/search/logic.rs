use app_core::{
    backend::{BackendEventLoop, BackendLink},
    BACKEND_HUNG_UP_MSG,
};

use crate::{gui::DynRequestSender, BackendAppState};

impl super::Search {
    pub fn try_update(&mut self) {
        self.read_current_child_paths.try_update();
        self.matched_paths.try_update();
    }

    pub(super) fn request_current_child_paths(&mut self, request_tx: &mut DynRequestSender) {
        let (rx, linker) = BackendLink::new(
            "request child paths",
            |b: &mut BackendEventLoop<BackendAppState>| {
                b.state.update_child_paths_unfiltered();
            },
        );
        self.read_current_child_paths.set_recv(rx);
        request_tx
            .send(Box::new(linker))
            .expect(BACKEND_HUNG_UP_MSG);
    }

    pub(super) fn query_current_path(&mut self, request_tx: &mut DynRequestSender) {
        let query = self.search_query.to_owned();
        let (rx, linker) = BackendLink::new(
            "fuzzy match child paths",
            move |b: &mut BackendEventLoop<BackendAppState>| b.state.search_filter(&query),
        );
        self.matched_paths.set_recv(rx);
        request_tx
            .send(Box::new(linker))
            .expect(BACKEND_HUNG_UP_MSG);
    }
}
