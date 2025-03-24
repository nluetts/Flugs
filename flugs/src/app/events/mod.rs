use std::{path::PathBuf, thread::JoinHandle};

use derive_new::new;

use crate::app::storage::{load_json, save_json};

use super::{
    components::{FileID, Group},
    EguiApp,
};
use app_core::event::{AppEvent, EventState};

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
                    // Great, nothing to do.
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
                name: format!("Group {}", to_group),
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
                    super::components::save_svg(&app, &path);
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
