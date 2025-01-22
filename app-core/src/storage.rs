#![allow(unused)]

//! This module defines the `Storage` type which collects frontend and backend
//! state information and provides methods to store/load them to/from a JSON
//! file.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{from_reader, to_writer};
use std::{io::Read, os::fd::AsFd, str::FromStr};

use super::string_error::ErrorStringExt;

const STORAGE_FILE: &str = "./.app_storage.json";

#[derive(Serialize, Deserialize)]
pub struct Storage<B, F> {
    pub backend_storage: B,
    pub frontend_storage: F,
}

impl<F, B> Storage<B, F>
where
    for<'a> B: Serialize + Deserialize<'a>,
    for<'a> F: Serialize + Deserialize<'a>,
{
    pub fn new(backend_storage: B, frontend_storage: F) -> Self {
        Self {
            backend_storage,
            frontend_storage,
        }
    }

    pub fn save_json(&self) -> Result<(), String> {
        let file =
            std::fs::File::create(STORAGE_FILE).err_to_string("could not open storage file")?;
        to_writer(file, &self).err_to_string("could not save app state to json")?;
        let output_path = std::path::PathBuf::from_str(STORAGE_FILE).unwrap();
        log::debug!("saved app state to file {:?}", output_path.canonicalize());
        Ok(())
    }

    pub fn from_json() -> Result<Self, String> {
        let mut file =
            std::fs::File::open(STORAGE_FILE).err_to_string("could not open storage file")?;
        let storage =
            from_reader(file).err_to_string("could not load app state from storage file")?;
        Ok(storage)
    }
}
