use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::PathBuf};

pub fn read_json<T: DeserializeOwned>(path: PathBuf) -> Result<T, String> {
    if !path.exists() {
        return Err("missing".to_string());
    }
    serde_json::from_str(&fs::read_to_string(path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

/// Write JSON atomically: serialize to a sibling temp file, then rename over the target so a
/// crash mid-write can never leave a truncated or partially written config behind.
pub fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let contents = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &contents).map_err(|err| err.to_string())?;

    match fs::rename(&tmp, &path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            Err(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_round_trips_and_leaves_no_temp() {
        let dir = std::env::temp_dir().join(format!("spf-store-{}", uuid::Uuid::new_v4()));
        let path = dir.join("data.json");
        let value = vec!["a".to_string(), "b".to_string()];

        write_json(path.clone(), &value).unwrap();
        let read: Vec<String> = read_json(path.clone()).unwrap();

        assert_eq!(read, value);
        // The atomic temp file must not linger after a successful write.
        assert!(!path.with_extension("tmp").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_missing_file_is_err() {
        let path = std::env::temp_dir().join(format!("spf-missing-{}.json", uuid::Uuid::new_v4()));
        let result: Result<Vec<String>, String> = read_json(path);
        assert!(result.is_err());
    }
}
