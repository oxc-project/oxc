//! package.json definitions
//!
//! Code related to export field are copied from [Parcel's resolver](https://github.com/parcel-bundler/parcel/blob/v2/packages/utils/node-resolver-rs/src/package_json.rs)
use std::{
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
    sync::Arc,
};

use indexmap::IndexMap;
use rustc_hash::FxHasher;
use serde::{Deserialize, Deserializer};

use crate::{path::PathUtil, ResolveError, ResolveOptions};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Deserialize)]
pub struct PackageJson {
    /// Path to `package.json`. Contains the `package.json` filename.
    #[serde(skip)]
    pub path: PathBuf,

    #[serde(skip)]
    #[serde(default)]
    pub raw_json: Arc<serde_json::Value>,

    /// The "name" field defines your package's name.
    /// The "name" field can be used in addition to the "exports" field to self-reference a package using its name.
    ///
    /// <https://nodejs.org/api/packages.html#name>
    pub name: Option<String>,

    /// The "main" field defines the entry point of a package when imported by name via a node_modules lookup. Its value is a path.
    /// When a package has an "exports" field, this will take precedence over the "main" field when importing the package by name.
    ///
    /// Values are dynamically added from [ResolveOptions::main_fields].
    ///
    /// <https://nodejs.org/api/packages.html#main>
    #[serde(skip)]
    pub main_fields: Vec<String>,

    /// The "exports" field allows defining the entry points of a package when imported by name loaded either via a node_modules lookup or a self-reference to its own name.
    ///
    /// <https://nodejs.org/api/packages.html#exports>
    #[serde(skip)]
    pub exports: Vec<ExportsField>,

    /// In addition to the "exports" field, there is a package "imports" field to create private mappings that only apply to import specifiers from within the package itself.
    ///
    /// <https://nodejs.org/api/packages.html#subpath-imports>
    #[serde(default)]
    pub imports: Box<MatchObject>,

    /// The "browser" field is provided by a module author as a hint to javascript bundlers or component tools when packaging modules for client side use.
    /// Multiple values are configured by [ResolveOptions::alias_fields].
    ///
    /// <https://github.com/defunctzombie/package-browser-field-spec>
    #[serde(skip)]
    pub browser_fields: Vec<BrowserField>,
}

/// `matchObj` defined in `PACKAGE_IMPORTS_EXPORTS_RESOLVE`
/// This is an IndexMap provided by the `preserve_order` feature.
pub type MatchObject = FxIndexMap<ExportsKey, ExportsField>;

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
pub enum ExportsField {
    #[default]
    None, // For `undefined` or `null` value.
    String(String),
    Array(Vec<ExportsField>),
    Map(MatchObject),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ExportsKey {
    Main,
    Pattern(String),
    CustomCondition(String),
}

impl From<&str> for ExportsKey {
    fn from(key: &str) -> Self {
        if key == "." {
            Self::Main
        } else if key.starts_with("./") {
            Self::Pattern(key.trim_start_matches('.').to_string())
        } else if key.starts_with('#') {
            Self::Pattern(key.to_string())
        } else {
            Self::CustomCondition(key.to_string())
        }
    }
}

impl<'de> Deserialize<'de> for ExportsKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &'de str = Deserialize::deserialize(deserializer)?;
        Ok(Self::from(s))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BrowserField {
    String(String),
    Map(FxIndexMap<PathBuf, serde_json::Value>),
}

impl PackageJson {
    /// # Panics
    /// # Errors
    pub fn parse(
        path: PathBuf,
        json: &str,
        options: &ResolveOptions,
    ) -> Result<Self, serde_json::Error> {
        let mut package_json_value: serde_json::Value = serde_json::from_str(json)?;
        let mut package_json: Self = Self::deserialize(&package_json_value)?;
        package_json.main_fields.reserve_exact(options.main_fields.len());
        package_json.browser_fields.reserve_exact(options.alias_fields.len());
        package_json.exports.reserve_exact(options.exports_fields.len());

        if let Some(package_json_value) = package_json_value.as_object_mut() {
            // Dynamically create `main_fields`.
            for main_field_key in &options.main_fields {
                // Using `get` + `clone` instead of remove here
                // because `main_fields` may contain `browser`, which is also used in `browser_fields.
                if let Some(serde_json::Value::String(value)) =
                    package_json_value.get(main_field_key)
                {
                    package_json.main_fields.push(value.clone());
                }
            }
            // Dynamically create `browser_fields`.
            let dir = path.parent().unwrap();
            for browser_field_key in &options.alias_fields {
                if let Some(value) = package_json_value.remove(browser_field_key) {
                    let mut browser_field: BrowserField = serde_json::from_value(value)?;
                    // Normalize all relative paths to make browser_field a constant value lookup
                    if let BrowserField::Map(map) = &mut browser_field {
                        let relative_paths = map
                            .keys()
                            .filter(|path| path.starts_with("."))
                            .cloned()
                            .collect::<Vec<_>>();
                        for relative_path in relative_paths {
                            if let Some(value) = map.remove(&relative_path) {
                                let normalized_path = dir.normalize_with(relative_path);
                                map.insert(normalized_path, value);
                            }
                        }
                    }
                    package_json.browser_fields.push(browser_field);
                }
            }
        }

        // Dynamically create `exports`.
        for object_path in &options.exports_fields {
            let exports = Self::get_value_by_path(&package_json_value, object_path);
            if let Some(exports) = exports {
                let exports = ExportsField::deserialize(exports)?;
                package_json.exports.push(exports);
            }
        }

        package_json.path = path;
        package_json.raw_json = Arc::new(package_json_value);
        Ok(package_json)
    }

    fn get_value_by_path<'a>(
        package_json: &'a serde_json::Value,
        path: &[String],
    ) -> Option<&'a serde_json::Value> {
        if path.is_empty() {
            return None;
        }
        let mut value = package_json;
        for key in path {
            if let Some(inner_value) = value.as_object().and_then(|o| o.get(key)) {
                value = inner_value;
            } else {
                return None;
            }
        }
        Some(value)
    }

    /// Directory to `package.json`
    pub fn raw_json(&self) -> &serde_json::Value {
        self.raw_json.as_ref()
    }

    /// Directory to `package.json`
    ///
    /// # Panics
    ///
    /// * When the package.json path is misconfigured.
    pub fn directory(&self) -> &Path {
        debug_assert!(self.path.file_name().is_some_and(|x| x == "package.json"));
        self.path.parent().unwrap()
    }

    /// Resolve the request string for this package.json by looking at the `browser` field.
    ///
    /// # Errors
    ///
    /// * Returns [ResolveError::Ignored] for `"path": false` in `browser` field.
    pub fn resolve_browser_field(
        &self,
        path: &Path,
        request: Option<&str>,
    ) -> Result<Option<&str>, ResolveError> {
        let request = request.map_or(path, |r| Path::new(r));
        for browser in &self.browser_fields {
            match browser {
                BrowserField::Map(field_data) => {
                    // look up by full path if request is empty
                    if let Some(value) = field_data.get(request) {
                        return Self::alias_value(path, value);
                    }
                }
                BrowserField::String(value) => {
                    return Ok(Some(value.as_str()));
                }
            }
        }
        Ok(None)
    }

    fn alias_value<'a>(
        key: &Path,
        value: &'a serde_json::Value,
    ) -> Result<Option<&'a str>, ResolveError> {
        match value {
            serde_json::Value::String(value) => Ok(Some(value.as_str())),
            serde_json::Value::Bool(b) if !b => Err(ResolveError::Ignored(key.to_path_buf())),
            _ => Ok(None),
        }
    }
}
