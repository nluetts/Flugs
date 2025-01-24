use derive_new::new;

use super::{
    components::{FileID, Group},
    EguiApp,
};
use app_core::event::AppEvent;

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

// ---------------------------------------------------------------------------
//
//
// apply()
//
//
// ---------------------------------------------------------------------------

impl AppEvent for RemoveFile {
    type App = EguiApp;

    fn apply(&self, app: &mut Self::App) -> Result<(), String> {
        app.file_handler
            .remove(vec![], vec![(self.from_group, self.fid)]);
        Ok(())
    }
}

impl AppEvent for CopyFile {
    type App = EguiApp;

    fn apply(&self, app: &mut Self::App) -> Result<(), String> {
        let Self { fid, to_group } = *self;

        if let Some(grp) = app
            .file_handler
            .groups
            .get_mut(to_group)
            .and_then(|grp| grp.as_mut())
        {
            grp.file_ids.insert(fid);
        } else {
            log::debug!("creating new group at slot {}", to_group);
            let mut grp = Group::default();
            grp.name = format!("Group ({})", to_group + 1);
            grp.file_ids.insert(fid);
            app.file_handler.groups[to_group] = Some(grp);
        }
        Ok(())
    }
}

impl AppEvent for MoveFile {
    type App = EguiApp;

    fn apply(&self, app: &mut Self::App) -> Result<(), String> {
        let Self {
            fid,
            from_group,
            to_group,
        } = *self;

        CopyFile::new(fid, to_group).apply(app)?;
        RemoveFile::new(fid, from_group).apply(app)?;
        Ok(())
    }
}

impl AppEvent for RemoveGroup {
    type App = EguiApp;

    fn apply(&self, app: &mut Self::App) -> Result<(), String> {
        app.file_handler.remove(vec![self.gid], vec![]);
        Ok(())
    }
}
