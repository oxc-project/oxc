//! Shared rule infrastructure, independent of the built-in rule registry and linter services.

#![expect(clippy::missing_errors_doc)]

pub mod config;
pub mod context;
pub mod disable_directives;
pub mod fixer;
pub mod frameworks;
pub mod module_record;
pub mod options;
pub mod react_compiler;
pub mod rule;

pub use config::{LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings};
pub use context::{ContextHost, ContextSubHost, ContextSubHostOptions, LintContext};
pub use fixer::{Fix, FixKind, Message, PossibleFixes, oxc_code_short_canonical_name};
pub use frameworks::FrameworkFlags;
pub use module_record::ModuleRecord;
pub use options::{AllowWarnDeny, LintOptions};
pub use oxc_semantic::AstNode;

pub const WEBSITE_BASE_RULES_URL: &str = "https://oxc.rs/docs/guide/usage/linter/rules";

/// File extensions whose JavaScript is extracted from a containing framework file.
pub const LINT_PARTIAL_LOADER_EXTENSIONS: &[&str] = &["vue", "astro", "svelte"];
