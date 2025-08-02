//! Common CLI utilities for task binaries
//!
//! This module provides standardized patterns for CLI argument parsing
//! used across different task binaries in the oxc project.

use pico_args::Arguments;

/// Common CLI options pattern used across multiple tasks
#[derive(Debug, Clone)]
pub struct CommonCliOptions {
    /// Enable debug mode (usually single-threaded execution)
    pub debug: bool,
    /// Filter tests/operations by pattern
    pub filter: Option<String>,
    /// Override existing outputs
    pub r#override: bool,
    /// Show detailed output
    pub detail: bool,
    /// Show diff output
    pub diff: bool,
    /// Execute tests
    pub exec: bool,
}

impl Default for CommonCliOptions {
    fn default() -> Self {
        Self {
            debug: false,
            filter: None,
            r#override: false,
            detail: false,
            diff: false,
            exec: false,
        }
    }
}

impl CommonCliOptions {
    /// Parse common CLI options from command line arguments
    pub fn from_args(args: &mut Arguments) -> Self {
        Self {
            debug: args.contains("--debug"),
            filter: args.opt_value_from_str("--filter").unwrap(),
            r#override: args.contains("--override"),
            detail: args.contains("--detail"),
            diff: args.contains("--diff"),
            exec: args.contains("--exec"),
        }
    }

    /// Create a subset with only debug and filter (most common pattern)
    pub fn debug_filter_only(debug: bool, filter: Option<String>) -> Self {
        Self { debug, filter, ..Default::default() }
    }
}

/// Extract a subcommand from arguments (commonly used pattern)
pub fn get_subcommand(args: &mut Arguments) -> Option<String> {
    args.subcommand().unwrap_or(None)
}

/// Common pattern for handling CLI args in task binaries
pub fn parse_common_args() -> (Option<String>, CommonCliOptions) {
    let mut args = Arguments::from_env();
    let subcommand = get_subcommand(&mut args);
    let options = CommonCliOptions::from_args(&mut args);
    (subcommand, options)
}
