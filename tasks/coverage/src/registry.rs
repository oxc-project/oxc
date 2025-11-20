//! Suite Registry - Declarative test suite configuration
//!
//! This module provides a registry pattern for defining test suites,
//! reducing boilerplate in tool-specific run methods.

use std::path::{Path, PathBuf};

use crate::{
    babel::BabelFilter,
    misc::MiscFilter,
    suite::{MetadataParser, SkipFailingFilter, TestFilter},
    test262::Test262Filter,
    typescript::TypeScriptFilter,
};

/// Definition of a test suite (Test262, Babel, TypeScript, Misc)
pub struct SuiteDefinition {
    /// Suite identifier (e.g., "test262", "babel")
    pub id: &'static str,
    /// Root path for test files
    pub root_path: &'static str,
    /// Display path for snapshots
    pub display_path: &'static str,
    /// Whether this suite includes synthetic tests
    pub include_synthetic: bool,
}

/// Standard suite definitions
pub const TEST262: SuiteDefinition = SuiteDefinition {
    id: "test262",
    root_path: "test262/test",
    display_path: "test262/test",
    include_synthetic: false,
};

pub const BABEL: SuiteDefinition = SuiteDefinition {
    id: "babel",
    root_path: "babel/packages/babel-parser/test/fixtures",
    display_path: "babel/packages/babel-parser/test/fixtures",
    include_synthetic: false,
};

pub const TYPESCRIPT: SuiteDefinition = SuiteDefinition {
    id: "typescript",
    root_path: "typescript/tests/cases",
    display_path: "typescript/tests/cases",
    include_synthetic: false,
};

pub const MISC: SuiteDefinition = SuiteDefinition {
    id: "misc",
    root_path: "misc",
    display_path: "misc",
    include_synthetic: true,
};

/// TypeScript transpiler tests (subset of TypeScript suite)
pub const TYPESCRIPT_TRANSPILE: SuiteDefinition = SuiteDefinition {
    id: "transpile",
    root_path: "typescript/tests/cases/transpile",
    display_path: "typescript/tests/cases/transpile",
    include_synthetic: false,
};

/// Configuration for running a test suite with a specific tool
pub struct SuiteConfig<'a> {
    /// Full name (e.g., "parser_test262")
    pub name: String,
    /// Root path for the suite
    pub root_path: PathBuf,
    /// Display path for snapshots
    pub display_path: &'a Path,
    /// Metadata parser for this suite
    pub parser: &'a dyn MetadataParser,
    /// Filter for this suite
    pub filter: &'a dyn TestFilter,
    /// Include synthetic tests
    pub include_synthetic: bool,
}

/// Builder for creating suite configurations for a tool
pub struct ToolSuites<'a> {
    tool_name: &'a str,
    configs: Vec<SuiteConfig<'a>>,
}

impl<'a> ToolSuites<'a> {
    /// Create a new builder for the given tool
    pub fn new(tool_name: &'a str) -> Self {
        Self { tool_name, configs: Vec::with_capacity(4) }
    }

    /// Add a suite with its parser and filter
    pub fn add(
        mut self,
        suite: &SuiteDefinition,
        parser: &'a dyn MetadataParser,
        filter: &'a dyn TestFilter,
    ) -> Self {
        self.configs.push(SuiteConfig {
            name: format!("{}_{}", self.tool_name, suite.id),
            root_path: PathBuf::from(suite.root_path),
            display_path: Path::new(suite.display_path),
            parser,
            filter,
            include_synthetic: suite.include_synthetic,
        });
        self
    }

    /// Build the final list of suite configurations
    pub fn build(self) -> Vec<SuiteConfig<'a>> {
        self.configs
    }
}

/// Standard filter set for tools that skip failing tests
///
/// This provides the common pattern used by codegen, formatter, and transformer.
/// Each filter wraps the base suite filter with `SkipFailingFilter`.
pub struct StandardFilters {
    pub test262: SkipFailingFilter<Test262Filter>,
    pub babel: SkipFailingFilter<BabelFilter>,
    pub typescript: SkipFailingFilter<TypeScriptFilter>,
    pub misc: SkipFailingFilter<MiscFilter>,
}

impl StandardFilters {
    /// Create the standard filter set
    pub fn new() -> Self {
        Self {
            test262: SkipFailingFilter::new(Test262Filter::new()),
            babel: SkipFailingFilter::new(BabelFilter::new()),
            typescript: SkipFailingFilter::new(TypeScriptFilter::new()),
            misc: SkipFailingFilter::new(MiscFilter::new()),
        }
    }
}

impl Default for StandardFilters {
    fn default() -> Self {
        Self::new()
    }
}
