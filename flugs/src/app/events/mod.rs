use std::{path::PathBuf, thread::JoinHandle};

use derive_new::new;
use egui_plot::PlotPoint;

use crate::app::{
    components::{ParsedData, parse_csv},
    storage::{load_json, save_json},
};

use super::{
    EguiApp,
    components::{FileID, Group},
};
use app_core::{
    event::{AppEvent, EventState},
    frontend::UIParameter,
};

// ---------------------------------------------------------------------------
//
//
// EventQueue
//
//
// ---------------------------------------------------------------------------

// TODO: It would be nice if this could be part of app-core, but there are
// some borrowing rules that this would break, and I do not currently find
// a workaround.

/// The EventQueue stores events that are processed each iteration
/// of the application GUI event loop.
pub struct EventQueue<EguiApp> {
    /// Stores events for later processing.
    queue: Vec<Box<dyn AppEvent<App = EguiApp>>>,
    /// Temporarily stores events that have not yet finished running.
    tmp_backlog: Vec<Box<dyn AppEvent<App = EguiApp>>>,
}

impl<EguiApp> EventQueue<EguiApp> {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            tmp_backlog: Vec::new(),
        }
    }

    pub fn queue_event(&mut self, event: Box<dyn AppEvent<App = EguiApp>>) {
        self.queue.push(event);
    }

    pub fn discard_events(&mut self) {
        self.queue.drain(..);
        self.tmp_backlog.drain(..);
    }
}

impl EguiApp {
    pub fn run_events(&mut self) {
        // Fully drain all queued events.
        while let Some(mut event) = self.event_queue.queue.pop() {
            match event.apply(self) {
                Ok(EventState::Finished) => {
                    self.request_redraw();
                }
                Ok(EventState::Busy) => {
                    // Add busy event to the backlog.
                    self.event_queue.tmp_backlog.push(event);
                }
                Err(err) => {
                    log::error!("event failed: {:?}", err)
                }
            }
        }

        // Putting the backlog back in the queue by swapping the
        // vectors.
        std::mem::swap(
            &mut self.event_queue.queue,
            &mut self.event_queue.tmp_backlog,
        );
    }
}

// ---------------------------------------------------------------------------
//
//
// Events
//
//
// ---------------------------------------------------------------------------

#[derive(new)]
pub struct RemoveFile {
    fid: FileID,
    from_group: usize,
}

#[derive(new)]
pub struct MoveFile {
    fid: FileID,
    from_group: usize,
    to_group: usize,
}

#[derive(new)]
pub struct CopyFile {
    fid: FileID,
    to_group: usize,
}

#[derive(new)]
pub struct CloneFile {
    fid: FileID,
    to_group: usize,
}

#[derive(new)]
pub struct RemoveGroup {
    gid: usize,
}

/// Handles both, saving and loading the app state, depending on whether
/// `should_save` is true or false.
#[derive(new)]
pub struct SaveLoadRequested {
    should_save: bool,
    thread_handle: Option<JoinHandle<Option<PathBuf>>>,
}

/// Handles both, saving and loading the app state, depending on whether
/// `should_save` is true or false.
#[derive(new)]
pub struct SavePlotRequested {
    thread_handle: Option<JoinHandle<Option<PathBuf>>>,
}

/// Save all files in a single folder
#[derive(new)]
pub struct ConsolidateRequest {
    thread_handle: Option<JoinHandle<Option<PathBuf>>>,
}

/// Locate a missing file in the current search folder
#[derive(new)]
pub struct LocateFile {
    file_name: String,
    id: FileID,
    search_dispatched: bool,
}

/// Manipulate a file's scaling/offset interactively
#[derive(new)]
pub struct ManipulateFile {
    fid: FileID,
    drag: egui::Vec2,
    modifiers: egui::Modifiers,
}

/// Refresh cache for file
pub struct RefreshCache(pub FileID);

/// Reset scaling for file
pub struct ResetScaling(pub FileID);

// ---------------------------------------------------------------------------
//
//
// apply()
//
//
// ---------------------------------------------------------------------------

impl AppEvent for RemoveFile {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        app.file_handler
            .remove(vec![], vec![(self.from_group, self.fid)]);
        Ok(EventState::Finished)
    }
}

impl AppEvent for CopyFile {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        let Self { fid, to_group } = *self;

        if let Some(grp) = app
            .file_handler
            .groups
            .get_mut(to_group)
            .and_then(|grp| grp.as_mut())
        {
            if !grp.file_ids.contains(&fid) {
                grp.file_ids.push(fid);
            }
        } else {
            log::debug!("creating new group at slot {}", to_group);
            let mut grp = Group {
                name: format!("G{}", to_group),
                ..Default::default()
            };
            grp.file_ids.push(fid);
            app.file_handler.groups[to_group] = Some(grp);
        }
        Ok(EventState::Finished)
    }
}

impl AppEvent for CloneFile {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        let Self { fid, to_group } = *self;

        // Make a copy of the file in the registy.
        let fid = if let Some(file) = app.file_handler.registry.get(&fid) {
            app.file_handler.add_new_file(file.clone())
        } else {
            return Err(format!("Requested file not present in registry: {:?}", fid));
        };

        if let Some(grp) = app
            .file_handler
            .groups
            .get_mut(to_group)
            .and_then(|grp| grp.as_mut())
        {
            if !grp.file_ids.contains(&fid) {
                grp.file_ids.push(fid);
            }
        } else {
            log::debug!("creating new group at slot {}", to_group);
            let mut grp = Group {
                name: format!("G{}", to_group),
                ..Default::default()
            };
            grp.file_ids.push(fid);
            app.file_handler.groups[to_group] = Some(grp);
        }
        Ok(EventState::Finished)
    }
}

impl AppEvent for MoveFile {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        let Self {
            fid,
            from_group,
            to_group,
        } = *self;

        CopyFile::new(fid, to_group).apply(app)?;
        RemoveFile::new(fid, from_group).apply(app)?;
        Ok(EventState::Finished)
    }
}

impl AppEvent for RemoveGroup {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        app.file_handler.remove(vec![self.gid], vec![]);
        Ok(EventState::Finished)
    }
}

impl AppEvent for SaveLoadRequested {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        if let Some(handle) = self.thread_handle.take_if(|handle| handle.is_finished()) {
            match handle.join() {
                Ok(Some(path)) => {
                    if self.should_save {
                        if let Err(err) = save_json(app, Some(path.as_ref())) {
                            log::error!("error while trying to save to {:?}: {:?}", &path, err)
                        };
                    } else if let Err(err) = load_json(app, Some(path.as_ref())) {
                        log::error!("error while trying to load to {:?}: {:?}", &path, err)
                    };
                }
                Ok(None) => (),
                Err(err) => {
                    let msg = if self.should_save { "save" } else { "load" };
                    log::error!("Unable to {} file: {:?}", msg, err)
                }
            };
            Ok(EventState::Finished)
        } else {
            Ok(EventState::Busy)
        }
    }
}

impl AppEvent for SavePlotRequested {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        if let Some(handle) = self.thread_handle.take_if(|handle| handle.is_finished()) {
            match handle.join() {
                Ok(Some(path)) => {
                    super::components::save_svg(app, &path);
                }
                Ok(None) => (),
                Err(err) => {
                    log::error!("unable to save plot: {:?}", err)
                }
            };
            Ok(EventState::Finished)
        } else {
            Ok(EventState::Busy)
        }
    }
}

impl AppEvent for ConsolidateRequest {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        if let Some(handle) = self.thread_handle.take_if(|handle| handle.is_finished()) {
            match handle.join() {
                Ok(Some(path)) => {
                    app.file_handler.consolidate_files(&path);
                }
                Ok(None) => (),
                Err(err) => {
                    log::error!("Unable to consolidate files: {:?}", err)
                }
            };
            Ok(EventState::Finished)
        } else {
            Ok(EventState::Busy)
        }
    }
}

impl AppEvent for LocateFile {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        if !self.search_dispatched {
            app.search
                .search_single(&self.file_name, &mut app.request_tx);
            self.search_dispatched = true;
        }

        if !app.search.matches.is_up_to_date() {
            return Ok(EventState::Busy);
        }

        let file = app
            .file_handler
            .registry
            .get_mut(&self.id)
            .expect("Tried to located data for a file that does not exist!");
        let fdata = file.data.value_mut();
        match app.search.matches.value().first().take() {
            Some(m) => {
                file.path = app.search.get_search_path().join(&m.path);
                match &m.parsed_data {
                    ParsedData::Ok(plot_data) => *fdata = Ok(plot_data.to_owned()),
                    ParsedData::Failed(msg) => *fdata = Err(msg.to_owned()),
                    ParsedData::None => {
                        let mut param = UIParameter::new(Err("Data no loaded".to_string()));
                        param.set_recv(parse_csv(&file.path, &mut app.request_tx));
                        file.data = param
                    }
                };
            }
            None => *fdata = Err(format!("File not found in current search path!")),
        }
        Ok(EventState::Finished)
    }
}

impl AppEvent for ManipulateFile {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        // How much did the mouse move?
        let egui::Vec2 { x: dx, y: dy } = self.drag;
        let bounds = app.plotter.get_current_plot_bounds();
        let yspan = bounds.height();

        let Some(active_file) = app.file_handler.registry.get_mut(&self.fid) else {
            return Err(format!(
                "Cannot manipulate file with ID {:?}: not found!",
                self.fid
            ));
        };

        if self.modifiers.alt {
            // Alt key is pressed → change xoffset.
            active_file.properties.xoffset += dx as f64;
        } else if self.modifiers.ctrl {
            // Ctrl key is pressed → change yoffset.
            active_file.properties.yoffset += dy as f64;
        } else if self.modifiers.shift {
            let yscale_old = active_file.properties.yscale;
            let yoffset_old = active_file.properties.yoffset;
            // Find index of point with minimum y-value that falls within
            // plot bounds
            let mut ymin = None;
            for PlotPoint { x, y } in active_file
                .data
                .value()
                .as_ref()
                .ok()
                .map(|data| data.get_cache())
                .into_iter()
                .flatten()
            {
                if bounds.range_x().contains(x) && bounds.range_y().contains(y) && y.is_finite() {
                    match ymin {
                        Some(ymin_inner) if ymin_inner > *y => ymin = Some(*y),
                        None if y.is_finite() => ymin = Some(*y),
                        _ => (),
                    }
                }
            }

            let Some(ymin) = ymin else {
                return Err(format!(
                    "Manipulate file with ID {:?} failed: unable to find minimum y-value",
                    self.fid
                ));
            };

            let yscale_new = yscale_old * (1.0 + 3.0 / yspan * (dy as f64));
            // Minimal y before any manipulations
            let ymin_0 = (ymin - yoffset_old) / yscale_old;
            let ymin_new = ymin_0 * yscale_new + yoffset_old;
            active_file.properties.yscale = yscale_new;
            active_file.properties.yoffset -= ymin_new - ymin;
        }
        active_file.refresh_cache();
        Ok(EventState::Finished)
    }
}

impl AppEvent for RefreshCache {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        match app.file_handler.registry.get_mut(&self.0) {
            Some(file) => {
                file.refresh_cache();
                return Ok(EventState::Finished);
            }
            None => {
                return Err(format!(
                    "Could not refresh cache of file with ID {:?}, not found.",
                    self.0
                ));
            }
        }
    }
}

impl AppEvent for ResetScaling {
    type App = EguiApp;

    fn apply(&mut self, app: &mut Self::App) -> Result<EventState, String> {
        match app.file_handler.registry.get_mut(&self.0) {
            Some(file) => {
                file.properties.xoffset = 0.0;
                file.properties.yoffset = 0.0;
                file.properties.yscale = 1.0;
                file.refresh_cache();
                return Ok(EventState::Finished);
            }
            None => {
                return Err(format!(
                    "Could not reset scaling for file with ID {:?}, not found.",
                    self.0
                ));
            }
        }
    }
}
