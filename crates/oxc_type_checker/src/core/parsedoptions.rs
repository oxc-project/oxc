//! Port of typescript-go's `internal/core/parsedoptions.go`.

use std::path::PathBuf;

use super::{CompilerOptions, ProjectReference};

/// tsgo `core.ParsedOptions`: the result of parsing a project configuration.
///
/// Carries the merged compiler options plus the project's root files, expanded from the
/// config's `files`/`include`/`exclude` specs at parse time. tsgo's `WatchOptions` and
/// `TypeAcquisition` are not ported.
#[derive(Debug)]
pub struct ParsedOptions {
    /// The merged `compilerOptions`.
    pub compiler_options: CompilerOptions,
    /// The project's root files (tsgo `FileNames`), absolute, in spec order (literal `files`
    /// first, then `include` matches).
    pub file_names: Vec<PathBuf>,
    /// The root config's `references` (tsgo `ProjectReferences`). Not consumed yet —
    /// project references are a later program-loading step.
    pub project_references: Vec<ProjectReference>,
}
