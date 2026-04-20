use std::{
    io,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};

#[derive(Debug)]
pub enum PackageJsonError {
    Missing(PathBuf),
    Io(PathBuf, io::Error),
    Parse(PathBuf, serde_json::Error),
}

pub fn read_package_json(path: &Path) -> Result<Option<Map<String, Value>>, PackageJsonError> {
    let contents = match super::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(PackageJsonError::Io(path.to_path_buf(), err)),
    };

    let value: Value = serde_json::from_str(&contents)
        .map_err(|err| PackageJsonError::Parse(path.to_path_buf(), err))?;

    let object = value.as_object().cloned().ok_or_else(|| {
        PackageJsonError::Parse(
            path.to_path_buf(),
            serde_json::Error::io(io::Error::new(
                io::ErrorKind::InvalidData,
                "package.json must be an object",
            )),
        )
    })?;

    Ok(Some(object))
}

pub fn find_nearest_package_json(
    start_dir: &Path,
) -> Result<Option<(PathBuf, Map<String, Value>)>, PackageJsonError> {
    let mut current = Some(start_dir);
    while let Some(dir) = current {
        let path = dir.join("package.json");
        match read_package_json(&path)? {
            Some(package_json) => return Ok(Some((path, package_json))),
            None => current = dir.parent(),
        }
    }

    Ok(None)
}
