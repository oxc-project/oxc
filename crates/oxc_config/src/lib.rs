//! Shared configuration utilities for Oxc command-line tools.

mod discovery;
mod glob_set;
mod ignore_patterns;
mod walk;

pub use discovery::{
    ConfigConflict, ConfigDiscovery, ConfigFileNames, DiscoveredConfigFile, is_js_config_path,
};
pub use glob_set::{GlobSet, validate_glob_pattern};
pub use ignore_patterns::validate_ignore_pattern;
pub use walk::{all_paths_have_vcs_boundary, configure_walk_builder};
