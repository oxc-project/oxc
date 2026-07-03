//! Project discovery.
//!
//! Find the `tsconfig.json`, parse it (with `extends` chains resolved) via [`oxc_resolver`],
//! and enumerate the project's files. Ports `findConfigFile` and the `-p`/`--project`
//! resolution from typescript-go `internal/execute/tsc.go`.

use std::{fmt, path::Path, sync::Arc, sync::OnceLock};

use oxc_diagnostics::OxcDiagnostic;
use oxc_resolver::{ResolveError, ResolveOptions, Resolver, TsConfig};

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
    /// `-p <dir>` was given but `<dir>/tsconfig.json` does not exist. tsgo reuses the
    /// "current directory" diagnostic (TS5081) here, with the joined config path.
    CannotFindTsconfigAt(String),
    /// `-p <file>` was given but the file does not exist (TS5058).
    SpecifiedPathDoesNotExist(String),
    /// No `tsconfig.json` was found walking up from the search directory (TS5081).
    NoConfigFound(String),
    /// The config file could not be read (TS5083).
    Read(String),
    /// The config file could not be parsed.
    Parse(String),
}

impl fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotFindTsconfigAt(path) | Self::NoConfigFound(path) => {
                write!(f, "Cannot find a tsconfig.json file at the current directory: {path}.")
            }
            Self::SpecifiedPathDoesNotExist(path) => {
                write!(f, "The specified path does not exist: '{path}'.")
            }
            Self::Read(path) => write!(f, "Cannot read file '{path}'."),
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

    /// Parse the config (resolving `extends` chains) and enumerate the project's files.
    ///
    /// # Errors
    /// Fails if the config file cannot be read or its JSON cannot be parsed.
    pub fn load(&self) -> Result<DiscoverResult, DiscoverError> {
        let tsconfig = self.parse_tsconfig()?;
        let options =
            CompilerOptionsView::from_resolver(&tsconfig.compiler_options, &self.base_path);
        let (specs, mut diagnostics) =
            ConfigFileSpecs::from_tsconfig(&tsconfig, &options, &self.base_path);

        let host = StdFs { use_case_sensitive_file_names: use_case_sensitive_file_names() };
        let files = specs.file_names(&self.base_path, &options, &host);
        if specs.should_report_no_input_files(&files) {
            diagnostics.push(specs.no_inputs_diagnostic(&self.config_file));
        }

        Ok(DiscoverResult { config_file: self.config_file.clone(), files, diagnostics })
    }

    /// Parse the config, resolving `extends`. If the `extends` chain cannot be resolved
    /// (e.g. it names an uninstalled package), fall back to parsing this file alone —
    /// tsgo reports the unresolvable base as a diagnostic but still enumerates files.
    fn parse_tsconfig(&self) -> Result<Arc<TsConfig>, DiscoverError> {
        let resolver = Resolver::new(ResolveOptions::default());
        match resolver.resolve_tsconfig(Path::new(&self.config_file)) {
            Ok(tsconfig) => Ok(tsconfig),
            Err(ResolveError::Json(json_error)) => Err(DiscoverError::Parse(json_error.message)),
            Err(_) => {
                let text = std::fs::read_to_string(&self.config_file)
                    .map_err(|_| DiscoverError::Read(self.config_file.clone()))?;
                let path = Path::new(&self.config_file);
                TsConfig::parse(true, path, path, text)
                    .map(Arc::new)
                    .map_err(|err| DiscoverError::Parse(err.to_string()))
            }
        }
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

        // tsgo absolutizes the `-p` value against the current directory while parsing the
        // command line (`--project` is an `IsFilePath` option), so `-p .` works.
        let file_or_directory =
            TsPath::from(project.to_string_lossy().as_ref()).normalized_absolute(cwd).into_string();
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

/// Whether the file system is case-sensitive, probed once like tsgo's `osvfs`: stat the
/// running executable's path with its case swapped — if the swapped path does not exist,
/// the file system is case-sensitive.
fn use_case_sensitive_file_names() -> bool {
    static USE_CASE_SENSITIVE: OnceLock<bool> = OnceLock::new();
    *USE_CASE_SENSITIVE.get_or_init(probe_case_sensitivity)
}

fn probe_case_sensitivity() -> bool {
    // win32/win64 are case insensitive platforms (tsgo hardcodes this too).
    if cfg!(windows) {
        return false;
    }
    let fallback = || !cfg!(target_os = "macos");
    let Ok(exe) = std::env::current_exe() else {
        return fallback();
    };
    let exe = exe.to_string_lossy();
    let swapped = swap_case(&exe);
    if swapped == exe {
        // No letters to swap; tsgo would stat the identical path and conclude insensitive.
        return false;
    }
    match std::fs::metadata(&swapped) {
        Ok(_) => false,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => true,
        // tsgo panics here; fall back to a per-OS default instead.
        Err(_) => fallback(),
    }
}

/// tsgo `swapCase`: upper-case every lower-case char and vice-versa (simple mappings only,
/// so multi-char expansions like `ß` -> `SS` are left untouched, as in Go).
fn swap_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            let mut upper = c.to_uppercase();
            match (upper.next(), upper.next()) {
                (Some(u), None) if u != c => u,
                _ => {
                    let mut lower = c.to_lowercase();
                    match (lower.next(), lower.next()) {
                        (Some(l), None) => l,
                        _ => c,
                    }
                }
            }
        })
        .collect()
}

/// The current working directory. Like Go's `os.Getwd` (which tsgo uses), this prefers the
/// logical `$PWD` when it refers to the same directory as the physical one, preserving
/// symlinked paths.
fn current_directory() -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if let Ok(pwd) = std::env::var("PWD")
            && pwd.starts_with('/')
            && let (Ok(logical), Ok(physical)) = (std::fs::metadata(&pwd), std::fs::metadata("."))
            && (logical.dev(), logical.ino()) == (physical.dev(), physical.ino())
        {
            return TsPath::from_slashes(&pwd).into_string();
        }
    }
    std::env::current_dir()
        .map(|dir| TsPath::from_slashes(&dir.to_string_lossy()).into_string())
        .unwrap_or_default()
}
