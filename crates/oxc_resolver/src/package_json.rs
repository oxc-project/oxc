use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::path::PathUtil;

#[derive(Debug, Deserialize)]
pub struct PackageJson<'a> {
    #[serde(skip)]
    pub path: PathBuf,
    pub main: Option<&'a str>,
    pub browser: Option<BrowserField<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BrowserField<'a> {
    String(&'a str),
    Map(HashMap<&'a str, &'a str>),
}

impl<'a> PackageJson<'a> {
    pub fn parse(path: PathBuf, json: &'a str) -> Result<PackageJson<'a>, serde_json::Error> {
        let mut package_json: PackageJson = serde_json::from_str(json)?;
        package_json.path = path;
        Ok(package_json)
    }

    pub fn resolve(&self, path: &Path) -> Option<PathBuf> {
        // TODO: return ResolveError if the provided `alias_fields` is not `browser` for future
        // proof
        let browser_field = self.browser.as_ref()?;
        match browser_field {
            BrowserField::Map(map) => {
                for (key, value) in map {
                    let resolved_path = self.resolve_browser_field(key, value, path);
                    if resolved_path.is_some() {
                        return resolved_path;
                    }
                }
                None
            }
            // TODO: implement <https://github.com/defunctzombie/package-browser-field-spec#alternate-main---basic>
            BrowserField::String(_) => None,
        }
    }

    fn resolve_browser_field(&self, key: &str, value: &str, path: &Path) -> Option<PathBuf> {
        let directory = self.path.parent().unwrap(); // `unwrap`: this is a path to package.json, parent is its containing directory
        // TODO: cache this join
        (directory.join(key).normalize() == path).then(|| directory.join(value).normalize())
    }
}
