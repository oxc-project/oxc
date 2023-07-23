use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{path::PathUtil, ResolveError};

// TODO: allocate everything into an arena or SoA
#[derive(Debug, Deserialize)]
pub struct PackageJson {
    #[serde(skip)]
    pub path: PathBuf,
    pub main: Option<String>,
    pub browser: Option<BrowserField>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BrowserField {
    String(String),
    Map(HashMap<PathBuf, serde_json::Value>),
}

impl PackageJson {
    pub fn parse(path: PathBuf, json: &str) -> Result<Self, serde_json::Error> {
        let mut package_json: Self = serde_json::from_str(json)?;

        // Normalize all relative paths to make browser_field a constant value lookup
        // TODO: fix BrowserField::String
        if let Some(BrowserField::Map(map)) = &mut package_json.browser {
            let relative_paths =
                map.keys().filter(|path| path.starts_with(".")).cloned().collect::<Vec<_>>();
            let dir = path.parent().unwrap();
            for relative_path in relative_paths {
                if let Some(value) = map.remove(&relative_path) {
                    let normalized_path = dir.normalize_with(relative_path);
                    map.insert(normalized_path, value);
                }
            }
        }

        package_json.path = path;
        Ok(package_json)
    }

    /// Resolve the request string for this package.json by looking at the `browser` field.
    ///
    /// # Errors
    ///
    /// * Returns [ResolveError::Ignored] for `"path": false` in `browser` field.
    pub fn resolve(
        &self,
        path: &Path,
        request: Option<&str>,
    ) -> Result<Option<&str>, ResolveError> {
        // TODO: return ResolveError if the provided `alias_fields` is not `browser` for future proof
        match self.browser.as_ref() {
            Some(BrowserField::Map(field_data)) => {
                // look up by full path if request is empty
                request
                    .map_or_else(
                        || field_data.get(path),
                        |request| field_data.get(Path::new(request)),
                    )
                    .map_or_else(|| Ok(None), |value| Self::alias_value(path, value))
            }
            // TODO: implement <https://github.com/defunctzombie/package-browser-field-spec#alternate-main---basic>
            _ => Ok(None),
        }
    }

    fn alias_value<'a>(
        key: &Path,
        value: &'a serde_json::Value,
    ) -> Result<Option<&'a str>, ResolveError> {
        match value {
            serde_json::Value::String(value) => Ok(Some(value.as_str())),
            serde_json::Value::Bool(b) if !b => {
                Err(ResolveError::Ignored(key.to_path_buf().into_boxed_path()))
            }
            _ => Ok(None),
        }
    }
}
