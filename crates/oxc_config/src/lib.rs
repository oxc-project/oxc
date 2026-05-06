//! Shared configuration utilities for Oxc command-line tools.

mod discovery;
mod glob_set;

pub use discovery::{
    ConfigConflict, ConfigDiscovery, ConfigFileNames, DiscoveredConfigFile, is_js_config_path,
};
pub use glob_set::GlobSet;
