use std::path::{Path, PathBuf};

use oxc_diagnostics::OxcDiagnostic;

/// A supported configuration file discovered on disk.
///
/// The variant identifies which config source matched, while the contained path
/// points to the concrete file. Consumers can use [`DiscoveredConfigFile::path`]
/// when they only need the filesystem location.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum DiscoveredConfigFile {
    Json(PathBuf),
    Jsonc(PathBuf),
    Js(PathBuf),
    Vite(PathBuf),
}

impl DiscoveredConfigFile {
    pub fn path(&self) -> &Path {
        match self {
            Self::Json(path) | Self::Jsonc(path) | Self::Js(path) | Self::Vite(path) => path,
        }
    }
}

/// File names accepted by [`ConfigDiscovery`].
///
/// Callers provide the names instead of hardcoding them in the discovery logic
/// so the same matcher can be reused for different config naming schemes.
pub struct ConfigFileNames {
    /// JSON config file name, such as `.oxlintrc.json`.
    pub json: &'static str,
    /// JSONC config file name, such as `.oxlintrc.jsonc`.
    pub jsonc: &'static str,
    /// JavaScript or TypeScript config file name, such as `oxlint.config.ts`.
    pub js: &'static str,
    /// Vite config file name used when Vite mode is enabled.
    pub vite: &'static str,
}

/// Finds supported config files using a caller-provided set of file names.
pub struct ConfigDiscovery {
    pub config_file_names: ConfigFileNames,
    pub vite_plus_mode: bool,
}

impl ConfigDiscovery {
    /// Create a config discovery helper for the provided file names and mode.
    pub fn new(config_file_names: ConfigFileNames, vite_plus_mode: bool) -> Self {
        Self { config_file_names, vite_plus_mode }
    }

    /// Find the only supported config file directly inside `dir`.
    ///
    /// Returns `Ok(None)` when no config file exists, and returns
    /// [`ConfigConflict`] when multiple configs are found.
    pub fn find_unique_config_in_directory(
        &self,
        dir: &Path,
    ) -> Result<Option<DiscoveredConfigFile>, ConfigConflict> {
        let configs = self.find_configs_in_directory(dir);

        match configs.len() {
            0 => Ok(None),
            1 => Ok(Some(configs.into_iter().next().unwrap())),
            _ => Err(ConfigConflict::new(dir.to_path_buf(), configs)),
        }
    }

    /// Find all supported config files directly inside `dir`.
    pub fn find_configs_in_directory(&self, dir: &Path) -> Vec<DiscoveredConfigFile> {
        if self.vite_plus_mode {
            let vite_path = dir.join(self.config_file_names.vite);
            if vite_path.is_file() {
                return vec![DiscoveredConfigFile::Vite(vite_path)];
            }
            return Vec::new();
        }

        let mut configs = Vec::new();

        let json_path = dir.join(self.config_file_names.json);
        if json_path.is_file() {
            configs.push(DiscoveredConfigFile::Json(json_path));
        }

        let jsonc_path = dir.join(self.config_file_names.jsonc);
        if jsonc_path.is_file() {
            configs.push(DiscoveredConfigFile::Jsonc(jsonc_path));
        }

        let js_path = dir.join(self.config_file_names.js);
        if js_path.is_file() {
            configs.push(DiscoveredConfigFile::Js(js_path));
        }

        configs
    }

    /// Convert `candidate` into a discovered config file when its file name is supported.
    ///
    /// The path does not need to exist on disk. This is intended for directory
    /// walkers that already know the candidate is a file.
    pub fn discover_config_file(&self, candidate: &Path) -> Option<DiscoveredConfigFile> {
        let file_name = candidate.file_name()?;

        if self.vite_plus_mode {
            if file_name == self.config_file_names.vite {
                return Some(DiscoveredConfigFile::Vite(candidate.to_path_buf()));
            }
            return None;
        }

        if file_name == self.config_file_names.json {
            return Some(DiscoveredConfigFile::Json(candidate.to_path_buf()));
        }
        if file_name == self.config_file_names.jsonc {
            return Some(DiscoveredConfigFile::Jsonc(candidate.to_path_buf()));
        }
        if file_name == self.config_file_names.js {
            return Some(DiscoveredConfigFile::Js(candidate.to_path_buf()));
        }
        None
    }
}

/// Multiple supported config files were found in the same directory.
///
/// Consumers should surface this as a user-facing configuration error. Use the
/// [`From<ConfigConflict>`] implementation to convert it into an [`OxcDiagnostic`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigConflict {
    /// Directory containing the conflicting config files.
    dir: PathBuf,
    /// Config files discovered in `dir`.
    configs: Vec<DiscoveredConfigFile>,
}

impl ConfigConflict {
    pub fn new(dir: PathBuf, configs: Vec<DiscoveredConfigFile>) -> Self {
        debug_assert!(
            configs.len() > 1,
            "ConfigConflict should only be created when multiple configs are found"
        );
        Self { dir, configs }
    }
}

impl ConfigConflict {
    fn message(&self) -> String {
        let config_names = self.config_names();

        if config_names.is_empty() {
            return String::new();
        }

        // if we are in cfg test, we need to sort config names to make sure the test is deterministic
        #[cfg(any(test, feature = "testing"))]
        let config_names = {
            let mut sorted = config_names;
            sorted.sort();
            sorted
        };

        let config_list = format_conflicting_config_names(&config_names);
        if config_names.len() == 2 {
            format!("Both {config_list} found in {}.", self.dir.display())
        } else {
            format!("Multiple config files found in {}: {config_list}.", self.dir.display())
        }
    }

    fn config_names(&self) -> Vec<String> {
        self.configs
            .iter()
            .filter_map(|config| {
                config.path().file_name().map(|name| name.to_string_lossy().into_owned())
            })
            .collect()
    }
}

impl From<ConfigConflict> for OxcDiagnostic {
    fn from(conflict: ConfigConflict) -> Self {
        OxcDiagnostic::error(conflict.message())
            .with_note("Only one of `.oxlintrc.json`, `.oxlintrc.jsonc`, or `oxlint.config.ts` is allowed per directory.")
            .with_help("Delete one of the configuration files.")
    }
}

fn format_conflicting_config_names(config_names: &[String]) -> String {
    debug_assert!(config_names.len() > 1);

    let mut quoted_names = config_names.iter().map(|name| format!("'{name}'")).collect::<Vec<_>>();
    if quoted_names.len() == 2 {
        return format!("{} and {}", quoted_names[0], quoted_names[1]);
    }

    let last = quoted_names.pop().unwrap();
    format!("{}, and {last}", quoted_names.join(", "))
}
