use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use oxc_diagnostics::OxcDiagnostic;

/// Return `true` when `path` uses a JavaScript or TypeScript config extension.
pub fn is_js_config_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("js" | "mjs" | "cjs" | "ts" | "cts" | "mts")
    )
}

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
    /// Return the filesystem path for the discovered config file.
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub struct ConfigDiscovery {
    config_file_names: ConfigFileNames,
    vite_plus_mode: bool,
}

impl ConfigDiscovery {
    /// Create a config discovery helper for the provided file names and mode.
    pub fn new(config_file_names: ConfigFileNames, vite_plus_mode: bool) -> Self {
        Self { config_file_names, vite_plus_mode }
    }

    /// Return supported config file names in discovery order.
    ///
    /// In Vite+ mode, only the configured Vite file name is returned. In
    /// regular mode, JSON, JSONC, and JavaScript/TypeScript config names are
    /// returned in that order.
    pub fn config_file_names(&self) -> Vec<&'static str> {
        if self.vite_plus_mode {
            return vec![self.config_file_names.vite];
        }

        vec![self.config_file_names.json, self.config_file_names.jsonc, self.config_file_names.js]
    }

    /// Find the unique config file directly inside `dir` using a single `read_dir`.
    ///
    /// Issues one `read_dir()` and matches entry names in memory,
    /// avoiding the per-candidate `stat` syscalls that a name-by-name probe would incur.
    ///
    /// When `follow_symlinks` is `true`,
    /// symlink entries fall back to `Path::is_file()` so a symlinked config is still recognized.
    /// When `false`, only regular files are considered;
    /// symlinks, directories, and other entry types are skipped, matching walkers configured with `follow_links(false)`.
    ///
    /// Returns `Ok(None)` when `dir` is unreadable;
    /// the caller can decide whether that warrants a diagnostic.
    ///
    /// # Errors
    /// Returns [`ConfigConflict`] when more than one supported config file is found directly inside `dir`.
    pub fn find_unique_config_by_readdir(
        &self,
        dir: &Path,
        follow_symlinks: bool,
    ) -> Result<Option<DiscoveredConfigFile>, ConfigConflict> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(None);
        };

        let names = &self.config_file_names;
        let mut matches = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_supported = if self.vite_plus_mode {
                name == names.vite
            } else {
                name == names.json || name == names.jsonc || name == names.js
            };
            if !name_supported {
                continue;
            }

            // NOTE: `Path::is_file()` follows symlinks; `FileType::is_file()` does not.
            let is_match = if follow_symlinks {
                entry.path().is_file()
            } else {
                let Ok(file_type) = entry.file_type() else { continue };
                #[expect(clippy::filetype_is_file)]
                file_type.is_file()
            };
            if !is_match {
                continue;
            }

            if let Some(config) = self.discover_config_file(&entry.path()) {
                matches.push(config);
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(ConfigConflict::new(dir.to_path_buf(), matches)),
        }
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
/// Consumers should surface this as a user-facing configuration error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigConflict {
    /// Directory containing the conflicting config files.
    dir: PathBuf,
    /// Config files discovered in `dir`.
    configs: Vec<DiscoveredConfigFile>,
}

impl ConfigConflict {
    /// Create a config conflict from a directory and the files discovered inside it.
    pub fn new(dir: PathBuf, configs: Vec<DiscoveredConfigFile>) -> Self {
        debug_assert!(
            configs.len() > 1,
            "ConfigConflict should only be created when multiple configs are found"
        );
        Self { dir, configs }
    }

    fn message(&self) -> String {
        let mut config_names = self.config_names();

        if config_names.is_empty() {
            return String::new();
        }

        config_names.sort();

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
        let mut config_names = conflict.config_names();
        config_names.sort();

        let note = if config_names.is_empty() {
            "Only one config file is allowed per directory.".to_string()
        } else {
            let backticked_names =
                config_names.iter().map(|name| format!("`{name}`")).collect::<Vec<_>>();
            let config_list = if backticked_names.len() == 2 {
                format!("{} and {}", backticked_names[0], backticked_names[1])
            } else {
                let (last, backticked_names) = backticked_names.split_last().unwrap();
                format!("{}, and {last}", backticked_names.join(", "))
            };
            format!("Only one of {config_list} is allowed per directory.")
        };

        OxcDiagnostic::error(conflict.message())
            .with_note(note)
            .with_help("Delete one of the configuration files.")
    }
}

fn format_conflicting_config_names(config_names: &[String]) -> String {
    debug_assert!(config_names.len() > 1);

    let quoted_names = config_names.iter().map(|name| format!("'{name}'")).collect::<Vec<_>>();
    if quoted_names.len() == 2 {
        return format!("{} and {}", quoted_names[0], quoted_names[1]);
    }

    let (last, quoted_names) = quoted_names.split_last().unwrap();
    format!("{}, and {last}", quoted_names.join(", "))
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    use super::{ConfigDiscovery, ConfigFileNames, DiscoveredConfigFile, is_js_config_path};

    const NAMES: ConfigFileNames = ConfigFileNames {
        json: ".oxlintrc.json",
        jsonc: ".oxlintrc.jsonc",
        js: "oxlint.config.ts",
        vite: "vite.config.ts",
    };

    fn discovery() -> ConfigDiscovery {
        ConfigDiscovery::new(NAMES, false)
    }

    fn vite_discovery() -> ConfigDiscovery {
        ConfigDiscovery::new(NAMES, true)
    }

    #[test]
    fn test_is_js_config_path() {
        assert!(is_js_config_path(Path::new("my-config.js")));
        assert!(is_js_config_path(Path::new("my-config.cjs")));
        assert!(is_js_config_path(Path::new("my-config.mjs")));
        assert!(is_js_config_path(Path::new("my-config.ts")));
        assert!(is_js_config_path(Path::new("my-config.cts")));
        assert!(is_js_config_path(Path::new("my-config.mts")));
        assert!(!is_js_config_path(Path::new("oxlint.config.json")));
    }

    #[test]
    fn readdir_returns_none_for_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap().is_none()
        );
    }

    #[test]
    fn readdir_returns_none_for_unreadable_dir() {
        // Pointing at a path that doesn't exist mimics the "read_dir fails" case
        // without relying on platform-specific permission setups.
        let missing = std::path::PathBuf::from("/this/path/does/not/exist/__readdir_test__");
        assert!(discovery().find_unique_config_by_readdir(&missing, false).unwrap().is_none());
    }

    #[test]
    fn readdir_skips_unrelated_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("README.md"), "").unwrap();
        fs::write(temp_dir.path().join("package.json"), "").unwrap();

        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap().is_none()
        );
    }

    #[test]
    fn readdir_finds_unique_json_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cfg_path = temp_dir.path().join(NAMES.json);
        fs::write(&cfg_path, "{}").unwrap();

        let found = discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap();
        assert!(matches!(found, Some(DiscoveredConfigFile::Json(p)) if p == cfg_path));
    }

    #[test]
    fn readdir_returns_conflict_for_multiple_configs() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join(NAMES.json), "{}").unwrap();
        fs::write(temp_dir.path().join(NAMES.jsonc), "{}").unwrap();

        assert!(discovery().find_unique_config_by_readdir(temp_dir.path(), false).is_err());
    }

    #[test]
    fn readdir_skips_directory_named_like_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        // A directory whose name collides with a supported config must not be
        // treated as a config file.
        fs::create_dir(temp_dir.path().join(NAMES.json)).unwrap();

        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap().is_none()
        );
    }

    #[test]
    fn readdir_vite_mode_only_recognizes_vite_name() {
        let temp_dir = tempfile::tempdir().unwrap();
        // JSON config is ignored in Vite+ mode even though it exists.
        fs::write(temp_dir.path().join(NAMES.json), "{}").unwrap();
        fs::write(temp_dir.path().join(NAMES.vite), "").unwrap();

        let found = vite_discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap();
        assert!(
            matches!(found, Some(DiscoveredConfigFile::Vite(p)) if p.file_name().unwrap() == NAMES.vite)
        );
    }

    #[cfg(unix)]
    #[test]
    fn readdir_follow_symlinks_toggles_link_resolution() {
        use std::os::unix::fs::symlink;

        let temp_dir = tempfile::tempdir().unwrap();
        // Target lives outside the scanned dir so the symlink is the only entry
        // that could match.
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("real.json");
        fs::write(&target, "{}").unwrap();

        let link = temp_dir.path().join(NAMES.json);
        symlink(&target, &link).unwrap();

        // follow_symlinks=false: symlinked configs are ignored.
        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap().is_none()
        );

        // follow_symlinks=true: the symlink resolves to a file and matches.
        let found = discovery().find_unique_config_by_readdir(temp_dir.path(), true).unwrap();
        assert!(matches!(found, Some(DiscoveredConfigFile::Json(p)) if p == link));
    }

    #[cfg(unix)]
    #[test]
    fn readdir_follow_symlinks_skips_dangling_links() {
        use std::os::unix::fs::symlink;

        let temp_dir = tempfile::tempdir().unwrap();
        // Symlink target does not exist; even with follow_symlinks=true this
        // must not be reported as a config file.
        symlink("/nonexistent/target", temp_dir.path().join(NAMES.json)).unwrap();

        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), true).unwrap().is_none()
        );
    }
}
