//! Project discovery.
//!
//! Find the `tsconfig.json`, parse it via [`oxc_resolver`], and enumerate the project's files.
//! Ports `findConfigFile` and the `-p`/`--project` resolution from typescript-go
//! `internal/execute/tsc.go`.

use std::{fmt, path::Path};

use oxc_diagnostics::OxcDiagnostic;
use oxc_resolver::TsConfig;

use crate::{
    tsconfig::{CompilerOptionsView, ConfigFileSpecs},
    tspath::TsPath,
    vfsmatch::StdFs,
};

/// The outcome of discovering + enumerating a project.
pub struct DiscoverResult {
    /// Absolute path of the `tsconfig.json` that was loaded.
    pub config_file: String,
    /// Absolute, normalized paths of the project's files.
    pub files: Vec<String>,
    /// Diagnostics produced while validating the config's file specs.
    pub diagnostics: Vec<OxcDiagnostic>,
}

/// Why discovery failed.
#[derive(Debug)]
pub enum DiscoverError {
    /// `-p <dir>` was given but `<dir>/tsconfig.json` does not exist.
    CannotFindTsconfigAt(String),
    /// `-p <file>` was given but the file does not exist.
    SpecifiedPathDoesNotExist(String),
    /// No `tsconfig.json` was found walking up from the search directory.
    NoConfigFound(String),
    /// The config file could not be read.
    Read(String),
    /// The config file could not be parsed.
    Parse(String),
}

impl fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotFindTsconfigAt(path) => {
                write!(f, "Cannot find a tsconfig.json file at the specified directory: '{path}'.")
            }
            Self::SpecifiedPathDoesNotExist(path) => {
                write!(f, "The specified path does not exist: '{path}'.")
            }
            Self::NoConfigFound(path) => {
                write!(f, "Cannot find a tsconfig.json file at the current directory: '{path}'.")
            }
            Self::Read(message) => write!(f, "{message}"),
            Self::Parse(message) => write!(f, "failed to parse tsconfig.json: {message}"),
        }
    }
}

impl std::error::Error for DiscoverError {}

/// A resolved TypeScript project: an absolute `tsconfig.json` and its directory.
pub struct Project {
    config_file: String,
    base_path: String,
}

impl Project {
    /// Resolve which project to load (ports `internal/execute/tsc.go`).
    ///
    /// With `-p`/`--project`, a directory uses `<dir>/tsconfig.json` and a file is used as-is.
    /// Otherwise `tsconfig.json` is discovered by walking up from `cwd`.
    ///
    /// # Errors
    /// Fails if the `-p` target (or its `tsconfig.json`) does not exist, or if no `tsconfig.json`
    /// is found walking up from `cwd`.
    pub fn resolve(project: Option<&Path>, cwd: &str) -> Result<Self, DiscoverError> {
        let config_file = Self::resolve_config_file_name(project, cwd)?;
        let config_file = TsPath::from(config_file.as_str()).normalized_absolute(cwd).into_string();
        let base_path = TsPath::from(config_file.as_str()).directory().into_string();
        Ok(Self { config_file, base_path })
    }

    /// Absolute path of the resolved `tsconfig.json`.
    pub fn config_file(&self) -> &str {
        &self.config_file
    }

    /// Parse the config and enumerate the project's files.
    ///
    /// # Errors
    /// Fails if the config file cannot be read or its JSON cannot be parsed.
    pub fn load(&self) -> Result<DiscoverResult, DiscoverError> {
        let text = std::fs::read_to_string(&self.config_file)
            .map_err(|err| DiscoverError::Read(err.to_string()))?;
        let path = Path::new(&self.config_file);
        let tsconfig = TsConfig::parse(true, path, path, text)
            .map_err(|err| DiscoverError::Parse(err.to_string()))?;
        let options = CompilerOptionsView::from_resolver(&tsconfig.compiler_options);
        let (specs, diagnostics) = ConfigFileSpecs::from_tsconfig(&tsconfig, &options);

        let host = StdFs { use_case_sensitive_file_names: default_use_case_sensitive_file_names() };
        let files = specs.file_names(&self.base_path, &options, &host);

        Ok(DiscoverResult { config_file: self.config_file.clone(), files, diagnostics })
    }

    fn resolve_config_file_name(
        project: Option<&Path>,
        cwd: &str,
    ) -> Result<String, DiscoverError> {
        let Some(project) = project else {
            let search_path = TsPath::from(cwd).normalized().into_string();
            return Self::find_config_file(&search_path)
                .ok_or(DiscoverError::NoConfigFound(search_path));
        };

        let file_or_directory =
            TsPath::from(project.to_string_lossy().as_ref()).normalized().into_string();
        if Path::new(&file_or_directory).is_dir() {
            let config_file_name =
                TsPath::from(file_or_directory.as_str()).combine(&["tsconfig.json"]).into_string();
            if Path::new(&config_file_name).is_file() {
                Ok(config_file_name)
            } else {
                Err(DiscoverError::CannotFindTsconfigAt(config_file_name))
            }
        } else if Path::new(&file_or_directory).is_file() {
            Ok(file_or_directory)
        } else {
            Err(DiscoverError::SpecifiedPathDoesNotExist(file_or_directory))
        }
    }

    /// Walk up from `search_path`, returning the first ancestor containing a `tsconfig.json`.
    fn find_config_file(search_path: &str) -> Option<String> {
        TsPath::from(search_path).for_each_ancestor(|ancestor| {
            let config = TsPath::from(ancestor).combine(&["tsconfig.json"]).into_string();
            Path::new(&config).is_file().then_some(config)
        })
    }
}

/// Discover and enumerate a project, using the current working directory.
///
/// # Errors
/// Fails if the config file cannot be resolved (see [`Project::resolve`]) or loaded (see
/// [`Project::load`]).
pub fn discover(project: Option<&Path>) -> Result<DiscoverResult, DiscoverError> {
    let cwd = current_directory();
    Project::resolve(project, &cwd)?.load()
}

/// Whether the running platform's file system is case-sensitive (used as tsgo's
/// `useCaseSensitiveFileNames`; a reasonable default, refined by an FS probe as a follow-up).
fn default_use_case_sensitive_file_names() -> bool {
    !cfg!(any(target_os = "macos", target_os = "windows"))
}

fn current_directory() -> String {
    std::env::current_dir()
        .map(|dir| TsPath::from_slashes(&dir.to_string_lossy()).into_string())
        .unwrap_or_default()
}
