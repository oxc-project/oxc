use std::{
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use serde::Deserialize;

use crate::{path::PathUtil, ResolveError};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

// TODO: allocate everything into an arena or SoA
#[derive(Debug, Deserialize)]
pub struct PackageJson {
    #[serde(skip)]
    pub path: PathBuf,

    /// The "name" field defines your package's name.
    /// The "name" field can be used in addition to the "exports" field to self-reference a package using its name.
    ///
    /// <https://nodejs.org/api/packages.html#name>
    pub name: Option<String>,

    /// The "main" field defines the entry point of a package when imported by name via a node_modules lookup. Its value is a path.
    /// When a package has an "exports" field, this will take precedence over the "main" field when importing the package by name.
    ///
    /// <https://nodejs.org/api/packages.html#main>
    pub main: Option<String>,

    /// The "exports" field allows defining the entry points of a package when imported by name loaded either via a node_modules lookup or a self-reference to its own name.
    ///
    /// <https://nodejs.org/api/packages.html#exports>
    pub exports: Option<ExportsField>,

    /// The browser field is provided by a module author as a hint to javascript bundlers or component tools when packaging modules for client side use.
    ///
    /// <https://github.com/defunctzombie/package-browser-field-spec>
    pub browser: Option<BrowserField>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BrowserField {
    String(String),
    Map(FxIndexMap<PathBuf, serde_json::Value>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExportsField {
    String(String),
    Array(Vec<ExportsField>),
    Map(FxIndexMap<String, ExportsField>),
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

    /// [PACKAGE_EXPORTS_RESOLVE](https://nodejs.org/api/esm.html#resolution-algorithm-specification)
    #[allow(clippy::single_match)]
    pub fn package_exports_resolve(&self, request: &str) -> Option<PathBuf> {
        let Some(exports) = &self.exports else {
            return None;
        };
        match exports {
            ExportsField::Map(map) => match map.get(request) {
                Some(ExportsField::String(value)) => {
                    return Some(self.path.parent().unwrap().join(value));
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
}
