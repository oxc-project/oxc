use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};

use oxc_allocator::Allocator;

pub type ExternalLinterLoadPluginCb = Arc<
    dyn Fn(
            String,
            Option<String>,
        ) -> Result<PluginLoadResult, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

pub type ExternalLinterLintFileCb = Arc<
    dyn Fn(
            String,
            Vec<u32>,
            String,
            String,
            String,
            &Allocator,
        ) -> Result<Vec<LintFileResult>, String>
        + Sync
        + Send,
>;

pub type ExternalLinterLoadParserCb = Arc<
    dyn Fn(
            String,
            Option<String>,
        ) -> Result<ParserLoadResult, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

pub type ExternalLinterParseWithCustomParserCb = Arc<
    dyn Fn(
            String,
            String,
            Option<String>,
        ) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

#[derive(Clone, Debug, Deserialize)]
pub enum PluginLoadResult {
    #[serde(rename_all = "camelCase")]
    Success {
        name: String,
        offset: usize,
        rule_names: Vec<String>,
    },
    Failure(String),
}

#[derive(Clone, Debug, Deserialize)]
pub enum ParserLoadResult {
    #[serde(rename_all = "camelCase")]
    Success {
        name: String,
        path: String,
    },
    Failure(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFileResult {
    pub rule_index: u32,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub fixes: Option<Vec<JsFix>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsFix {
    pub range: [u32; 2],
    pub text: String,
}

/// Result from parsing with a custom parser.
/// Contains the ESTree AST buffer and parseForESLint return values.
#[derive(Clone, Debug)]
pub struct ParseResult {
    /// Buffer containing serialized ESTree AST (with length prefix)
    pub buffer: Vec<u8>,
    /// Offset into buffer where ESTree JSON starts (after length prefix)
    pub estree_offset: u32,
    /// Parser services (parser-specific, e.g., TypeScript type checker)
    pub services: Option<serde_json::Value>,
    /// Custom scope manager (optional, oxc rebuilds scopes)
    pub scope_manager: Option<serde_json::Value>,
    /// Custom visitor keys for AST traversal
    pub visitor_keys: Option<serde_json::Value>,
}

/// JSON representation of ParseResult for transfer from JS to Rust.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParseResultJson {
    /// Base64-encoded buffer containing serialized ESTree AST
    buffer: String,
    /// Offset into buffer where ESTree JSON starts (after length prefix)
    estree_offset: u32,
    /// Parser services (parser-specific, e.g., TypeScript type checker)
    services: Option<serde_json::Value>,
    /// Custom scope manager (optional, oxc rebuilds scopes)
    scope_manager: Option<serde_json::Value>,
    /// Custom visitor keys for AST traversal
    visitor_keys: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) lint_file: ExternalLinterLintFileCb,
    pub(crate) load_parser: ExternalLinterLoadParserCb,
    pub(crate) parse_with_custom_parser: ExternalLinterParseWithCustomParserCb,
}

impl ExternalLinter {
    pub fn new(
        load_plugin: ExternalLinterLoadPluginCb,
        lint_file: ExternalLinterLintFileCb,
        load_parser: ExternalLinterLoadParserCb,
        parse_with_custom_parser: ExternalLinterParseWithCustomParserCb,
    ) -> Self {
        Self { load_plugin, lint_file, load_parser, parse_with_custom_parser }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}
