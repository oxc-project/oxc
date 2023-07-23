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
    Map(HashMap<String, serde_json::Value>),
}

impl PackageJson {
    pub fn parse(path: PathBuf, json: &str) -> Result<Self, serde_json::Error> {
        let mut package_json: Self = serde_json::from_str(json)?;
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
        request: &str,
        extensions: &[String],
    ) -> Result<Option<&str>, ResolveError> {
        // TODO: return ResolveError if the provided `alias_fields` is not `browser` for future proof
        match self.browser.as_ref() {
            Some(BrowserField::Map(map)) => {
                for (key, value) in map {
                    if let Some(resolved_str) =
                        self.resolve_browser_field(path, key, value, request, extensions)?
                    {
                        return Ok(Some(resolved_str));
                    }
                }
                Ok(None)
            }
            // TODO: implement <https://github.com/defunctzombie/package-browser-field-spec#alternate-main---basic>
            _ => Ok(None),
        }
    }

    // TODO: refactor this mess
    fn resolve_browser_field<'a>(
        &'a self,
        start: &Path,
        key: &str,
        value: &'a serde_json::Value,
        request: &str,
        extensions: &[String],
    ) -> Result<Option<&str>, ResolveError> {
        let directory = self.path.parent().unwrap(); // `unwrap`: this is a path to package.json, parent is its containing directory
        let right = directory.join(key).normalize();
        let left = start.join(request).normalize();
        if key == request
            || extensions.iter().any(|ext| Path::new(request).with_extension(ext) == Path::new(key))
            || right == left
            || extensions.iter().any(|ext| left.with_extension(ext) == right)
        {
            if let serde_json::Value::String(value) = value {
                return Ok(Some(value.as_str()));
            }

            // key match without string value, i.e. `"path": false` for ignore
            let directory = self.path.parent().unwrap(); // `unwrap`: this is a path to package.json, parent is its containing directory
            let path_key = directory.join(key).normalize();
            return Err(ResolveError::Ignored(path_key.into_boxed_path()));
        }

        Ok(None)
    }
}
