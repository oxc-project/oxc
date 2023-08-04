use std::path::{Path, PathBuf};

use crate::request::RequestError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResolveError {
    /// Ignored path
    ///
    /// Derived from ignored path (false value) from browser field in package.json
    /// ```json
    /// {
    ///     "browser": {
    ///         "./module": false
    ///     }
    /// }
    /// ```
    /// See <https://github.com/defunctzombie/package-browser-field-spec#ignore-a-module>
    Ignored(Box<Path>),

    /// Path not found
    NotFound(Box<Path>),

    /// All of the aliased extension are not found
    ExtensionAlias,

    /// All of the aliases are not found
    Alias(String),

    /// The provided path request cannot be parsed
    Request(RequestError),

    /// JSON parse error
    JSON(JSONError),

    // TODO: TypeError [ERR_INVALID_MODULE_SPECIFIER]: Invalid module "./dist/../../../a.js" request is not a valid subpath for the "exports" resolution of /xxx/package.json
    InvalidModuleSpecifier(String),

    // TODO: Error [ERR_INVALID_PACKAGE_TARGET]: Invalid "exports" target "./../../a.js" defined for './dist/a.js' in the package config /xxx/package.json
    InvalidPackageTarget(String),

    // TODO: Error [ERR_PACKAGE_PATH_NOT_EXPORTED]: Package subpath './anything/else' is not defined by "exports" in /xxx/package.json
    PackagePathNotExported(String),

    // TODO: Invalid package config /xxx/package.json. "exports" cannot contain some keys starting with '.' and some not. The exports object must either be an object of package subpath keys or an object of main entry condition name keys only.
    InvalidPackageConfig(PathBuf),

    // TODO: Default condition should be last one
    InvalidPackageConfigDefault(PathBuf),

    // TODO:  Expecting folder to folder mapping. "./data/timezones" should end with "/"
    InvalidPackageConfigDirectory(PathBuf),

    PackageImportNotDefined(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JSONError {
    pub path: PathBuf,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ResolveError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_) | Self::ExtensionAlias | Self::Alias(_))
    }

    pub(crate) fn from_serde_json_error(path: PathBuf, error: &serde_json::Error) -> Self {
        Self::JSON(JSONError {
            path,
            message: error.to_string(),
            line: error.line(),
            column: error.column(),
        })
    }
}
