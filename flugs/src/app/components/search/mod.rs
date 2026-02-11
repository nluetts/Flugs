mod logic;
mod ui;

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    path::PathBuf,
    thread::JoinHandle,
};

use app_core::frontend::UIParameter;
use derive_new::new;

use crate::{app::DynRequestSender, backend_state::PlotData};

pub struct Search {
    pub matches: UIParameter<Vec<Match>>,
    search_path: UIParameter<PathBuf>,
    search_query: String,
    mode: SearchMode,
    selected_match: Option<usize>,
    awaiting_search_path_selection: Option<JoinHandle<Option<PathBuf>>>,
    request_tx: DynRequestSender,
}

#[derive(PartialEq, Eq)]
enum SearchMode {
    Disabled,
    KeyboardInput,
    KeyboardSelection,
    MouseSelection,
}

#[derive(Debug, Clone, new)]
pub struct Match {
    pub path: PathBuf,
    pub(super) matched_indices: HashSet<usize>,
    pub(super) assigned_group: Option<usize>,
    pub parsed_data: ParsedData,
}

#[derive(Debug, Clone)]
pub enum ParsedData {
    Ok(PlotData),
    Failed(String),
    None,
}

impl Search {
    pub fn new(request_tx: DynRequestSender) -> Self {
        Self {
            matches: Default::default(),
            search_path: Default::default(),
            search_query: Default::default(),
            mode: SearchMode::Disabled,
            selected_match: None,
            awaiting_search_path_selection: Default::default(),
            request_tx,
        }
    }
}

impl Hash for Match {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.path.clone(), self.assigned_group).hash(state)
    }
}

impl PartialEq for Match {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.matched_indices == other.matched_indices
            && self.assigned_group == other.assigned_group
    }
}

impl Eq for Match {}
