use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::PathBuf};

pub fn read_json<T: DeserializeOwned>(path: PathBuf) -> Result<T, String> {
    if !path.exists() {
        return Err("missing".to_string());
    }
    serde_json::from_str(&fs::read_to_string(path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

pub fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}
