use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use oxc_diagnostics::OxcDiagnostic;

/// Returns the value of the `VP_VERSION` environment variable, if set.
/// Vite+ sets it when launching oxlint / oxfmt, which switches config discovery to Vite+ mode.
pub fn vp_version() -> Option<std::ffi::OsString> {
    std::env::var_os("VP_VERSION")
}

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

/// A supported file name and the [`DiscoveredConfigFile`] variant it maps to.
type ConfigFileEntry = (&'static str, fn(PathBuf) -> DiscoveredConfigFile);

const OXLINT_CONFIG_FILE_NAMES: &[ConfigFileEntry] = &[
    (".oxlintrc.json", DiscoveredConfigFile::Json),
    (".oxlintrc.jsonc", DiscoveredConfigFile::Jsonc),
    ("oxlint.config.ts", DiscoveredConfigFile::Js),
    ("oxlint.config.mts", DiscoveredConfigFile::Js),
];

const OXFMT_CONFIG_FILE_NAMES: &[ConfigFileEntry] = &[
    (".oxfmtrc.json", DiscoveredConfigFile::Json),
    (".oxfmtrc.jsonc", DiscoveredConfigFile::Jsonc),
    ("oxfmt.config.ts", DiscoveredConfigFile::Js),
    ("oxfmt.config.mts", DiscoveredConfigFile::Js),
];

/// Config file names used in Vite+ mode, in Vite's own resolution order.
const VITE_PLUS_CONFIG_FILE_NAMES: &[ConfigFileEntry] = &[
    ("vite.config.js", DiscoveredConfigFile::Vite),
    ("vite.config.mjs", DiscoveredConfigFile::Vite),
    ("vite.config.ts", DiscoveredConfigFile::Vite),
    ("vite.config.cjs", DiscoveredConfigFile::Vite),
    ("vite.config.mts", DiscoveredConfigFile::Vite),
    ("vite.config.cts", DiscoveredConfigFile::Vite),
];

/// Finds supported config files for one tool, or for Vite+ mode.
#[derive(Debug, Clone, Copy)]
pub struct ConfigDiscovery {
    /// Supported file names in priority order.
    entries: &'static [ConfigFileEntry],
    /// When several entries exist in one directory,
    /// pick the first one in `entries` order instead of reporting a [`ConfigConflict`].
    first_wins: bool,
}

impl ConfigDiscovery {
    /// Config discovery for oxlint: `.oxlintrc.json(c)` and `oxlint.config.(m)ts`.
    pub fn oxlint() -> Self {
        Self { entries: OXLINT_CONFIG_FILE_NAMES, first_wins: false }
    }

    /// Config discovery for oxfmt: `.oxfmtrc.json(c)` and `oxfmt.config.(m)ts`.
    pub fn oxfmt() -> Self {
        Self { entries: OXFMT_CONFIG_FILE_NAMES, first_wins: false }
    }

    /// Config discovery for Vite+ mode, which only looks for `vite.config.*`.
    ///
    /// Vite+ reads `lint` / `fmt` from whichever of these it finds first,
    /// so multiple files are resolved the same way instead of being a conflict.
    pub fn vite_plus() -> Self {
        Self { entries: VITE_PLUS_CONFIG_FILE_NAMES, first_wins: true }
    }

    /// Return supported config file names in discovery order.
    pub fn config_file_names(&self) -> Vec<&'static str> {
        self.entries.iter().map(|(name, _)| *name).collect()
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
    /// In Vite+ mode (see [`ConfigDiscovery::vite_plus`]), the first one in Vite's order wins instead.
    pub fn find_unique_config_by_readdir(
        &self,
        dir: &Path,
        follow_symlinks: bool,
    ) -> Result<Option<DiscoveredConfigFile>, ConfigConflict> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(None);
        };

        // `(priority, config)` so Vite+ mode can pick the first entry in `entries` order.
        let mut matches = Vec::new();
        for entry in entries.flatten() {
            let Some(priority) = self.priority(&entry.file_name()) else { continue };

            // NOTE: `Path::is_file()` follows symlinks; `FileType::is_file()` does not.
            let is_match = if follow_symlinks {
                entry.path().is_file()
            } else {
                let Ok(file_type) = entry.file_type() else { continue };
                #[expect(clippy::filetype_is_file)]
                file_type.is_file()
            };
            if is_match {
                matches.push((priority, self.entries[priority].1(entry.path())));
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.pop().map(|(_, config)| config)),
            _ if self.first_wins => Ok(matches
                .into_iter()
                .min_by_key(|(priority, _)| *priority)
                .map(|(_, config)| config)),
            _ => Err(ConfigConflict::new(
                dir.to_path_buf(),
                matches.into_iter().map(|(_, config)| config).collect(),
            )),
        }
    }

    /// Convert `candidate` into a discovered config file when its file name is supported.
    ///
    /// The path does not need to exist on disk. This is intended for directory
    /// walkers that already know the candidate is a file.
    pub fn discover_config_file(&self, candidate: &Path) -> Option<DiscoveredConfigFile> {
        let file_name = candidate.file_name()?;
        let (_, discovered) = self.entries.iter().find(|(name, _)| file_name == *name)?;
        Some(discovered(candidate.to_path_buf()))
    }

    /// Position of `file_name` in the configured entries, or `None` when it is not a config name.
    fn priority(&self, file_name: &OsStr) -> Option<usize> {
        self.entries.iter().position(|(name, _)| file_name == *name)
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

    use super::{ConfigDiscovery, DiscoveredConfigFile, is_js_config_path};

    const JSON: &str = ".oxlintrc.json";

    fn discovery() -> ConfigDiscovery {
        ConfigDiscovery::oxlint()
    }

    fn vite_discovery() -> ConfigDiscovery {
        ConfigDiscovery::vite_plus()
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
        let cfg_path = temp_dir.path().join(JSON);
        fs::write(&cfg_path, "{}").unwrap();

        let found = discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap();
        assert!(matches!(found, Some(DiscoveredConfigFile::Json(p)) if p == cfg_path));
    }

    #[test]
    fn readdir_finds_unique_mts_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cfg_path = temp_dir.path().join("oxlint.config.mts");
        fs::write(&cfg_path, "export default {};").unwrap();

        let found = discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap();
        assert!(matches!(found, Some(DiscoveredConfigFile::Js(p)) if p == cfg_path));
    }

    #[test]
    fn readdir_returns_conflict_for_multiple_js_configs() {
        let temp_dir = tempfile::tempdir().unwrap();
        for name in ["oxlint.config.ts", "oxlint.config.mts"] {
            fs::write(temp_dir.path().join(name), "export default {};").unwrap();
        }

        assert!(discovery().find_unique_config_by_readdir(temp_dir.path(), false).is_err());
    }

    #[test]
    fn readdir_returns_conflict_for_multiple_configs() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join(JSON), "{}").unwrap();
        fs::write(temp_dir.path().join(".oxlintrc.jsonc"), "{}").unwrap();

        assert!(discovery().find_unique_config_by_readdir(temp_dir.path(), false).is_err());
    }

    #[test]
    fn readdir_skips_directory_named_like_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        // A directory whose name collides with a supported config must not be
        // treated as a config file.
        fs::create_dir(temp_dir.path().join(JSON)).unwrap();

        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap().is_none()
        );
    }

    #[test]
    fn readdir_vite_mode_only_recognizes_vite_names() {
        let temp_dir = tempfile::tempdir().unwrap();
        // JSON config is ignored in Vite+ mode even though it exists.
        fs::write(temp_dir.path().join(JSON), "{}").unwrap();
        fs::write(temp_dir.path().join("vite.config.ts"), "").unwrap();

        let found = vite_discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap();
        assert!(
            matches!(found, Some(DiscoveredConfigFile::Vite(p)) if p.file_name().unwrap() == "vite.config.ts")
        );
    }

    #[test]
    fn readdir_vite_mode_picks_first_by_vite_order_instead_of_conflict() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("vite.config.ts"), "").unwrap();
        fs::write(temp_dir.path().join("vite.config.mjs"), "").unwrap();

        let found = vite_discovery().find_unique_config_by_readdir(temp_dir.path(), false).unwrap();
        assert!(
            matches!(found, Some(DiscoveredConfigFile::Vite(p)) if p.file_name().unwrap() == "vite.config.mjs")
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

        let link = temp_dir.path().join(JSON);
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
        symlink("/nonexistent/target", temp_dir.path().join(JSON)).unwrap();

        assert!(
            discovery().find_unique_config_by_readdir(temp_dir.path(), true).unwrap().is_none()
        );
    }
}
