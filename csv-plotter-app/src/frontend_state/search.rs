use std::{collections::HashSet, path::PathBuf};

use app_core::{backend::LinkReceiver, frontend::UIParameter};

#[derive(Default)]
pub struct Search {
    matched_paths: UIParameter<Vec<(PathBuf, HashSet<usize>)>>,
    read_current_child_paths: UIParameter<()>,
    search_query: String,
}

impl Search {
    pub fn try_update(&mut self) {
        self.read_current_child_paths.try_update();
        self.matched_paths.try_update();
    }

    pub fn search_query_mut_ref(&mut self) -> &mut String {
        &mut self.search_query
    }
    pub fn matched_paths_is_up_to_date(&self) -> bool {
        self.matched_paths.is_up_to_date()
    }
    pub fn read_current_child_paths_is_up_to_date(&self) -> bool {
        self.read_current_child_paths.is_up_to_date()
    }
    pub fn matched_paths_value(&self) -> &Vec<(PathBuf, HashSet<usize>)> {
        self.matched_paths.value()
    }

    pub fn set_recv_read_current_child_paths(&mut self, rx: LinkReceiver<()>) {
        self.read_current_child_paths.set_recv(rx);
    }
    pub fn set_recv_matched_paths(&mut self, rx: LinkReceiver<Vec<(PathBuf, HashSet<usize>)>>) {
        self.matched_paths.set_recv(rx);
    }
}
