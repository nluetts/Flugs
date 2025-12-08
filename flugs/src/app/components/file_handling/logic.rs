use std::{collections::HashSet, path::Path};

use app_core::{
    backend::{BackendEventLoop, BackendLink, LinkReceiver},
    frontend::UIParameter,
    BACKEND_HUNG_UP_MSG,
};

use crate::{
    app::{
        components::{
            search::{Match, ParsedData},
            Search,
        },
        DynRequestSender,
    },
    backend_state::PlotData,
    BackendAppState,
};

use super::{File, FileHandler, FileID, Group};

impl File {
    pub fn get_cache(&self) -> Option<&Vec<[f64; 2]>> {
        self.data
            .value()
            .as_ref()
            .map(|dat| &dat.get_cache().data)
            .ok()
    }

    // Integrate data numerically using trapezoidal method.
    //
    // Returns NaN if something goes wrong.
    pub fn integrate(&mut self, left: f64, right: f64, local_baseline: bool) -> f64 {
        // Retrieve its data, if it was parsed correctly.
        let Ok(data) = self.data.value() else {
            log::error!(
                "File {} was not parsed correctly, cannot integrate",
                self.file_name()
            );
            return f64::NAN;
        };

        // Retrieve x and y data.
        let msg = format!(
            "File {} needs at least two columns for integration",
            self.file_name()
        );

        let Some(xs) = data.columns.get(self.properties.selected_x_column) else {
            log::error!("{msg}");
            return f64::NAN;
        };
        let Some(ys) = data.columns.get(self.properties.selected_y_column) else {
            log::error!("{msg}");
            return f64::NAN;
        };

        // Filter out data points where both x- and y-value are finite and not NaN.
        let (xs, ys) = xs
            .iter()
            .zip(ys)
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .fold(
                (Vec::with_capacity(xs.len()), Vec::with_capacity(xs.len())),
                |(mut xs, mut ys), (x, y)| {
                    xs.push(*x);
                    ys.push(*y);
                    (xs, ys)
                },
            );

        // Apply trapezoidal integration.

        match trapz(&xs, &ys, left, right, local_baseline) {
            Ok(area) => area,
            Err(err) => {
                log::error!("Failed to integrate file {}: {err}", self.file_name());
                f64::NAN
            }
        }
    }

    pub fn local_minimum(&mut self, left: f64, right: f64, after_scaling: bool) -> f64 {
        // Retrieve its data, if it was parsed correctly.
        let Ok(data) = self.data.value() else {
            log::error!(
                "File {} was not parsed correctly, cannot determine local minimum ",
                self.file_name()
            );
            return f64::NAN;
        };

        // Retrieve x and y data.
        let msg = format!(
            "File {} needs at least two columns to determine local minimum",
            self.file_name()
        );

        let Some(xs) = data.columns.get(self.properties.selected_x_column) else {
            log::error!("{msg}");
            return f64::NAN;
        };
        let Some(ys) = data.columns.get(self.properties.selected_y_column) else {
            log::error!("{msg}");
            return f64::NAN;
        };

        // Make sure left and right are sorted correctly.
        let (left, right) = (left.min(right), right.max(left));

        // Return minimum
        let mut minimum = xs
            .iter()
            .zip(ys)
            // Filter out y-values for which x is within left and right bound.
            .filter_map(|(x, y)| {
                if *x < left || *x > right {
                    None
                } else {
                    Some(y)
                }
            })
            // Reduce to minimum y-value.
            .reduce(|a, b| if a < b { a } else { b })
            .unwrap_or(&f64::NAN)
            .to_owned();

        if after_scaling {
            minimum *= self.properties.yscale
        }
        minimum
    }
}

impl FileHandler {
    pub fn add_search_results(&mut self, search: &mut Search, request_tx: &mut DynRequestSender) {
        let search_path = search.get_search_path().to_owned();
        for Match {
            path: fp,
            matched_indices: _,
            assigned_group: gid,
            // TODO: use pre-cached CSV data from search matches
            parsed_data,
        } in search
            .matches
            .value_mut()
            .drain(..)
            .filter(|mtch| mtch.assigned_group.is_some())
        {
            let gid =
                gid.expect("file handler was handed a search result not assigned to any group");
            if gid > 9 {
                log::warn!("Group ID > 9 invalid, only 10 slots available, ignoring");
            }
            // If file is already registered, we pull its ID from the registry,
            // otherwise we create a new ID and add the file to the registry.
            let fid = if let Some((fid, _)) = self
                .registry
                .iter()
                .find(|(_, file)| file.path == search_path.join(&fp))
            {
                *fid
            } else {
                let fid = self.next_id();
                let csv_data = match parsed_data {
                    ParsedData::Ok(data) => UIParameter::new(Ok(data)),
                    ParsedData::Failed(_) => {
                        UIParameter::new(Err("Failed to parse file.".to_string()))
                    }
                    ParsedData::None => {
                        let mut param = UIParameter::new(Err("Data no loaded".to_string()));
                        param.set_recv(parse_csv(&search_path.join(&fp), request_tx));
                        param
                    }
                };

                self.registry.insert(
                    fid,
                    File {
                        path: search_path.join(fp),
                        data: csv_data,
                        properties: super::FileProperties::default(),
                    },
                );
                fid
            };

            // Add the ID to the group requested by user, if it is not already a member.
            if let Some(grp) = &mut self.groups[gid] {
                if !grp.file_ids.contains(&fid) {
                    grp.file_ids.push(fid);
                }
            } else {
                let mut new_file_id_set = Vec::new();
                new_file_id_set.push(fid);
                let name = format!("G{}", gid);
                self.groups[gid] = Some(Group {
                    file_ids: new_file_id_set,
                    name,
                    ..Default::default()
                });
            };
        }
    }

    pub fn remove(
        &mut self,
        groups_to_delete: Vec<usize>,
        files_to_delete: Vec<(usize, super::FileID)>,
    ) {
        let mut item_was_removed = false;

        // Just in case, we filter `gid`s which would lead to a panic when used as index.
        for (gid, fid) in files_to_delete.into_iter().filter(|(gid, _)| *gid < 10) {
            let file_name = self.fid_to_filename_str(&fid).to_string();
            if let Some(grp) = &mut self.groups[gid] {
                // Find index of file ID in Vec of file IDs.
                let mut file_idx = None;
                for (i, cfid) in grp.file_ids.iter().enumerate() {
                    if *cfid == fid {
                        file_idx = Some(i)
                    }
                }
                // Remove if found, otherwise emit warning.
                match file_idx {
                    Some(idx) => {
                        grp.file_ids.remove(idx);
                        log::debug!(
                            "removed file '{file_name}' from group {} with ID {gid:?}",
                            grp.name
                        );
                        item_was_removed = true;
                    }
                    None => {
                        log::warn!(
                            "trying to remove file from group with ID {gid:?} which does not exist"
                        );
                    }
                }
            }
        }

        for gid in groups_to_delete.into_iter().filter(|gid| *gid < 10) {
            if let Some(Some(grp)) = self.groups.get(gid) {
                log::debug!("removed group '{}' with ID '{gid}'", grp.name);
            } else {
                log::warn!("trying to remove group with ID {gid} which does not exist");
            }
            self.groups[gid] = None;
        }

        // If for some reason nothing was remove (which is currently impossible,
        // but maybe can occur in the future) we omit checking the registry for
        // files to remove.
        if !item_was_removed {
            return;
        }
        // Remove files from registry which are not member of any group.
        let mut mark_delete = Vec::new();
        for fid in self.registry.keys() {
            if self
                .groups
                .iter()
                .filter_map(|x| x.as_ref())
                .all(|grp| !grp.file_ids.contains(fid))
            {
                mark_delete.push(*fid);
            }
        }
        for fid in mark_delete.into_iter() {
            log::debug!(
                "remove file '{}' from registry",
                self.fid_to_filename_str(&fid)
            );
            self.registry.remove(&fid);
        }
    }

    pub fn current_id(&self) -> FileID {
        self.next_id
    }

    fn next_id(&mut self) -> FileID {
        let fid = self.next_id;
        self.next_id.0 += 1;
        fid
    }

    pub fn add_new_file(&mut self, file: File) -> FileID {
        let fid = self.next_id();
        self.registry.insert(fid, file);
        fid
    }

    fn fid_to_filename_str(&self, fid: &FileID) -> &str {
        self.registry
            .get(fid)
            .map(|file| file.file_name())
            .unwrap_or("unreadable filename")
    }

    pub fn try_update(&mut self) -> bool {
        let mut was_updated = false;
        for file in self.registry.values_mut() {
            was_updated = was_updated || file.data.try_update();
        }
        was_updated
    }

    pub fn consolidate_files(&self, path: &Path) {
        let unique_paths: HashSet<_> = self
            .registry
            .values()
            .map(|file| file.path.to_owned())
            .collect();

        for fp in unique_paths {
            let Some(file_name) = fp.file_name() else {
                log::warn!("{fp:?} does not contain valid file name, skipping");
                continue;
            };
            if let Err(e) = std::fs::copy(&fp, path.join(file_name)) {
                log::error!("Error when trying to copy {:?}: {e}", fp);
            }
        }
    }
}

pub fn parse_csv(
    path: &Path,
    request_tx: &mut DynRequestSender,
) -> LinkReceiver<Result<PlotData, String>> {
    let path = path.to_owned();
    let (rx, linker) = BackendLink::new(
        &format!("load CSV data from file {:?}", path),
        move |_b: &mut BackendEventLoop<BackendAppState>| {
            PlotData::from_path(&path).map_err(|err| {
                log::error!("{}", err);
                err
            })
        },
    );
    request_tx
        .send(Box::new(linker))
        .expect(BACKEND_HUNG_UP_MSG);
    rx
}

// ----------------------------------------------------------------------------
//
//
// Integration Utilities
//
//
// ----------------------------------------------------------------------------

/// Trapezoidal integration.
///
/// Local baseline subtracts a linear baseline ranging from the start (left) to
/// the end (right) point of the integration window.
pub fn trapz(
    x: &[f64],
    y: &[f64],
    left: f64,
    right: f64,
    local_baseline: bool,
) -> Result<f64, String> {
    let (mut left, right) = (left.min(right), left.max(right));

    let n = x.len().min(y.len());
    if n <= 1 {
        return Err("Not enough values to integrate".into());
    }
    if x[0] >= right || x[n - 1] <= left {
        return Err("Integration window out of bounds".into());
    }

    let mut area: f64;
    // subtract local linear baseline, defined by start and end-point of integration window
    if local_baseline {
        let xs = vec![left, right];
        let ys = linear_resample_array(x, y, &xs);
        if ys.iter().any(|y| (*y).is_nan()) {
            return Err("Integration window out of bounds.".into());
        }
        area = -singletrapz(left, right, ys[0], ys[1])
    } else {
        area = 0.0_f64;
    }

    let mut inside_integration_window = false;
    let mut lastiter = false;
    let mut j = 2;

    while j <= n {
        let mut x0 = x[j - 1];
        let mut x1 = x[j];
        let mut y0 = y[j - 1];
        let mut y1 = y[j];

        if x1 <= left {
            j += 1;
            continue;
        } else if !inside_integration_window {
            // this will only run once, when we enter the integration window
            // test whether x0 should be replaced by left
            if x0 < left {
                y0 = lininterp(left, x0, x1, y0, y1);
                x0 = left;
            } else {
                // this case means that left <= x[0]
                left = x0;
            }
            inside_integration_window = true;
        }

        // test whether x1 should be replaced by right
        if x1 >= right {
            // we move out of the integration window

            if x1 != right {
                y1 = lininterp(right, x0, x1, y0, y1)
            };
            x1 = right;
            lastiter = true; // we shall break the loop after this iteration
        }

        area += singletrapz(x0, x1, y0, y1);

        if lastiter {
            break;
        }

        j += 1;
    }
    Ok(area)
}

pub fn linear_resample_array(xs: &[f64], ys: &[f64], grid: &[f64]) -> Vec<f64> {
    let segments = xs
        .iter()
        .zip(ys.iter())
        .zip(xs.iter().skip(1).zip(ys.iter().skip(1)))
        .map(|((x0, y0), (x1, y1))| (*x0, *y0, *x1, *y1))
        .collect::<Vec<_>>();

    let mut yp = Vec::with_capacity(grid.len());

    for xi in grid.iter() {
        if let Some((x0, y0, x1, y1)) = segments.iter().find(|(x0, _, x1, _)| xi >= x0 && xi < x1) {
            yp.push(lininterp(*xi, *x0, *x1, *y0, *y1));
            continue;
        }
        // only applies if xi happens to be == the last value in xs
        else if let Some((_, _, _, y1)) = segments.iter().last().filter(|(_, _, x1, _)| xi == x1)
        {
            yp.push(*y1);
            continue;
        }
        // applies if xi does not lie within the range of xs
        else {
            yp.push(f64::NAN)
        };
    }
    yp
}

/// Calculate area of single trapezoid.
fn singletrapz(x0: f64, x1: f64, y0: f64, y1: f64) -> f64 {
    0.5 * f64::abs(x1 - x0) * (y1 + y0)
}

/// Linearly interpolate y-value at position xp between two points (x0, y0) and (x1, y1).
pub fn lininterp(xp: f64, x0: f64, x1: f64, y0: f64, y1: f64) -> f64 {
    (y1 * (xp - x0) + y0 * (x1 - xp)) / (x1 - x0)
}
