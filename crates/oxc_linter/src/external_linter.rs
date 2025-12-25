use std::{borrow::Cow, fmt::Debug};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use crate::{
    config::{LintConfig, OxlintEnv, OxlintGlobals},
    context::ContextHost,
};

pub type ExternalLinterLoadPluginCb = Box<
    dyn Fn(
            // File URL to load plugin from
            String,
            // Plugin name (either alias or package name).
            // If is package name, it is pre-normalized.
            Option<String>,
            // `true` if plugin name is an alias (takes priority over name that plugin defines itself)
            bool,
        ) -> Result<LoadPluginResult, String>
        + Send
        + Sync,
>;

/// Callback to load a custom parser from a file URL.
///
/// Arguments:
/// - File URL to load parser from
/// - Parser options JSON (optional configuration for the parser)
///
/// Returns:
/// - `Ok(LoadParserResult)` on success containing parser info
/// - `Err(String)` on failure with error message
pub type ExternalLinterLoadParserCb = Box<
    dyn Fn(
            // File URL to load parser from
            String,
            // Parser options as JSON string (may be "null" if no options)
            String,
        ) -> Result<LoadParserResult, String>
        + Send
        + Sync,
>;

/// Callback to parse a file using a custom parser.
///
/// This is called when linting a file that matches a custom parser's patterns.
/// The parser should use the ESLint parser API:
/// - `parse(code, options)` returning an ESTree-compatible AST, or
/// - `parseForESLint(code, options)` returning `{ ast, scopeManager?, visitorKeys?, services? }`
///
/// Arguments:
/// - Parser ID (from LoadParserResult)
/// - File path being parsed
/// - Source text content
/// - Parser options JSON
///
/// Returns:
/// - `Ok(ParseFileResult)` on success containing AST and optional scope info
/// - `Err(String)` on failure with error message
pub type ExternalLinterParseFileCb = Box<
    dyn Fn(
            // Parser ID to use
            u32,
            // Absolute path of file to parse
            String,
            // Source text content
            String,
            // Parser options as JSON string
            String,
        ) -> Result<ParseFileResult, String>
        + Send
        + Sync,
>;

pub type ExternalLinterSetupConfigsCb = Box<dyn Fn(String) -> Result<(), String> + Send + Sync>;

pub type ExternalLinterLintFileCb = Box<
    dyn Fn(
            String,
            Vec<u32>,
            Vec<u32>,
            String,
            String,
            &Allocator,
        ) -> Result<Vec<LintFileResult>, String>
        + Sync
        + Send,
>;

/// Callback to lint a file with a pre-parsed AST from a custom parser.
///
/// This is used when linting files that match custom parser patterns.
/// The AST is already parsed and provided as JSON, so no oxc parsing is needed.
///
/// Arguments:
/// - File path being linted
/// - Source text content
/// - AST as JSON string
/// - Rule IDs to apply
/// - Options IDs for the rules
/// - Settings JSON
/// - Globals JSON
/// - Parser services JSON (from parseForESLint)
///
/// Returns:
/// - `Ok(Vec<LintFileResult>)` with any diagnostics
/// - `Err(String)` on failure with error message
pub type ExternalLinterLintFileWithCustomAstCb = Box<
    dyn Fn(
            // Absolute path of file to lint
            String,
            // Source text content
            String,
            // AST as JSON string
            String,
            // Rule IDs
            Vec<u32>,
            // Options IDs
            Vec<u32>,
            // Settings JSON
            String,
            // Globals JSON
            String,
            // Parser services JSON
            String,
        ) -> Result<Vec<LintFileResult>, String>
        + Sync
        + Send,
>;

/// Callback to strip custom syntax from a file, producing valid JavaScript.
///
/// This is used in Phase 2 to enable Rust rules on files with custom syntax.
/// The custom parser strips out non-JS syntax (e.g., Marko template tags)
/// and provides span mappings so diagnostics can be remapped to original positions.
///
/// Arguments:
/// - Parser ID (from LoadParserResult)
/// - File path being stripped
/// - Source text content
/// - Parser options JSON
///
/// Returns:
/// - `Ok(Some(StripFileResult))` with stripped source and mappings
/// - `Ok(None)` if the parser doesn't support stripping (fall back to JS-only rules)
/// - `Err(String)` on failure with error message
pub type ExternalLinterStripFileCb = Box<
    dyn Fn(
            // Parser ID to use
            u32,
            // Absolute path of file to strip
            String,
            // Source text content
            String,
            // Parser options as JSON string
            String,
        ) -> Result<Option<StripFileResult>, String>
        + Send
        + Sync,
>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadPluginResult {
    pub name: String,
    pub offset: usize,
    pub rule_names: Vec<String>,
}

/// Result from loading an external parser.
///
/// The parser is loaded via the JS callback and registered for use with files
/// matching certain patterns.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadParserResult {
    /// Unique identifier for this parser instance.
    /// Used to reference the parser when parsing files.
    pub parser_id: u32,
    /// Whether the parser implements `parseForESLint` (true) or just `parse` (false).
    /// When true, the parser may provide additional data like scope manager.
    pub has_parse_for_eslint: bool,
}

/// Result from parsing a file with a custom parser.
///
/// Contains the ESTree-compatible AST as JSON, and optionally scope information
/// if the parser implements `parseForESLint` and provides a scope manager.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[expect(clippy::struct_field_names)]
pub struct ParseFileResult {
    /// The ESTree-compatible AST as a JSON string.
    /// This will be passed directly to JS rules for linting.
    pub ast_json: String,
    /// Optional scope manager information from `parseForESLint`.
    /// This is used in Phase 3 to support scope-dependent Rust rules.
    /// Format follows ESLint's ScopeManager structure.
    pub scope_manager_json: Option<String>,
    /// Optional visitor keys from `parseForESLint`.
    /// Used to properly traverse custom AST node types.
    pub visitor_keys_json: Option<String>,
    /// Optional parser services from `parseForESLint`.
    /// Passed to rules that need parser-specific functionality.
    pub services_json: Option<String>,
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

/// Result from stripping custom syntax from a file.
///
/// Contains valid JavaScript source (with custom syntax stripped)
/// and span mappings for remapping diagnostics to original positions.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StripFileResult {
    /// The stripped source code (valid JavaScript).
    pub source: String,
    /// The source type for parsing (e.g., module vs script, TypeScript vs JavaScript).
    /// If not provided, will be inferred from the file extension.
    pub source_type: Option<StripSourceType>,
    /// Span mappings from stripped positions to original positions.
    /// Used to remap diagnostic spans back to the original source.
    pub mappings: Vec<SpanMapping>,
}

/// Source type information from the custom parser.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StripSourceType {
    /// Whether this is a module (true) or script (false).
    #[serde(default)]
    pub module: bool,
    /// Whether to parse as TypeScript.
    #[serde(default)]
    pub typescript: bool,
    /// Whether to enable JSX parsing.
    #[serde(default)]
    pub jsx: bool,
}

impl StripSourceType {
    /// Convert to oxc's SourceType.
    #[must_use]
    pub fn to_source_type(&self) -> SourceType {
        let mut source_type = if self.module {
            SourceType::mjs()
        } else {
            SourceType::cjs()
        };

        if self.typescript {
            source_type = source_type.with_typescript(true);
        }
        if self.jsx {
            source_type = source_type.with_jsx(true);
        }

        source_type
    }
}

/// A mapping from a span in the stripped source to a span in the original source.
///
/// When the custom parser strips custom syntax (like template tags), it provides
/// these mappings so that diagnostics reported against the stripped source can
/// be remapped to the correct positions in the original file.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanMapping {
    /// Start position in the stripped source.
    pub stripped_start: u32,
    /// End position in the stripped source.
    pub stripped_end: u32,
    /// Start position in the original source.
    pub original_start: u32,
    /// End position in the original source.
    pub original_end: u32,
}

impl SpanMapping {
    /// Create a new span mapping.
    #[must_use]
    pub fn new(
        stripped_start: u32,
        stripped_end: u32,
        original_start: u32,
        original_end: u32,
    ) -> Self {
        Self { stripped_start, stripped_end, original_start, original_end }
    }
}

/// Stripped source ready for linting with Rust rules.
///
/// This is the processed result of calling the strip callback, containing
/// the valid JavaScript source and the mappings needed to remap diagnostics.
#[derive(Debug)]
pub struct StrippedSource<'a> {
    /// Valid JavaScript source after stripping custom syntax.
    pub source_text: Cow<'a, str>,
    /// Source type for parsing.
    pub source_type: SourceType,
    /// Span mappings for remapping diagnostics to original positions.
    pub span_mappings: Vec<SpanMapping>,
}

impl<'a> StrippedSource<'a> {
    /// Create a new stripped source.
    #[must_use]
    pub fn new(
        source_text: Cow<'a, str>,
        source_type: SourceType,
        span_mappings: Vec<SpanMapping>,
    ) -> Self {
        Self { source_text, source_type, span_mappings }
    }

    /// Remap a span from the stripped source to the original source.
    ///
    /// Uses binary search to find the mapping that contains the stripped span,
    /// then applies the offset to get the original position.
    #[must_use]
    pub fn remap_span(&self, stripped_start: u32, stripped_end: u32) -> (u32, u32) {
        // If no mappings, return as-is (identity mapping)
        if self.span_mappings.is_empty() {
            return (stripped_start, stripped_end);
        }

        // Binary search to find the first mapping where stripped_start >= mapping.stripped_start
        let start_mapping = self
            .span_mappings
            .binary_search_by(|m| {
                if stripped_start < m.stripped_start {
                    std::cmp::Ordering::Greater
                } else if stripped_start > m.stripped_end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()
            .map(|idx| &self.span_mappings[idx]);

        let end_mapping = self
            .span_mappings
            .binary_search_by(|m| {
                if stripped_end < m.stripped_start {
                    std::cmp::Ordering::Greater
                } else if stripped_end > m.stripped_end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()
            .map(|idx| &self.span_mappings[idx]);

        // Calculate remapped positions
        let original_start = if let Some(m) = start_mapping {
            let offset = stripped_start.saturating_sub(m.stripped_start);
            m.original_start.saturating_add(offset)
        } else {
            // Fallback: find the closest preceding mapping and extrapolate
            let preceding = self
                .span_mappings
                .iter()
                .filter(|m| m.stripped_end <= stripped_start)
                .last();

            if let Some(m) = preceding {
                let offset = stripped_start.saturating_sub(m.stripped_end);
                m.original_end.saturating_add(offset)
            } else {
                // No preceding mapping, use first mapping's start as reference
                let first = &self.span_mappings[0];
                if stripped_start < first.stripped_start {
                    // Before first mapping
                    stripped_start
                } else {
                    first.original_start
                        .saturating_add(stripped_start.saturating_sub(first.stripped_start))
                }
            }
        };

        let original_end = if let Some(m) = end_mapping {
            let offset = stripped_end.saturating_sub(m.stripped_start);
            m.original_start.saturating_add(offset)
        } else {
            // Fallback: find the closest preceding mapping and extrapolate
            let preceding = self
                .span_mappings
                .iter()
                .filter(|m| m.stripped_end <= stripped_end)
                .last();

            if let Some(m) = preceding {
                let offset = stripped_end.saturating_sub(m.stripped_end);
                m.original_end.saturating_add(offset)
            } else {
                let first = &self.span_mappings[0];
                if stripped_end < first.stripped_start {
                    stripped_end
                } else {
                    first
                        .original_start
                        .saturating_add(stripped_end.saturating_sub(first.stripped_start))
                }
            }
        };

        (original_start, original_end)
    }
}

pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) load_parser: Option<ExternalLinterLoadParserCb>,
    pub(crate) parse_file: Option<ExternalLinterParseFileCb>,
    pub(crate) strip_file: Option<ExternalLinterStripFileCb>,
    pub(crate) lint_file_with_custom_ast: Option<ExternalLinterLintFileWithCustomAstCb>,
    pub(crate) setup_configs: ExternalLinterSetupConfigsCb,
    pub(crate) lint_file: ExternalLinterLintFileCb,
}

impl ExternalLinter {
    pub fn new(
        load_plugin: ExternalLinterLoadPluginCb,
        setup_configs: ExternalLinterSetupConfigsCb,
        lint_file: ExternalLinterLintFileCb,
    ) -> Self {
        Self {
            load_plugin,
            load_parser: None,
            parse_file: None,
            strip_file: None,
            lint_file_with_custom_ast: None,
            setup_configs,
            lint_file,
        }
    }

    /// Set the parser loading callback.
    ///
    /// This enables support for custom JS-based parsers configured via `jsParsers`.
    #[must_use]
    pub fn with_load_parser(mut self, load_parser: ExternalLinterLoadParserCb) -> Self {
        self.load_parser = Some(load_parser);
        self
    }

    /// Set the file parsing callback.
    ///
    /// This callback is called when parsing a file that matches a custom parser's patterns.
    /// It invokes the parser's `parse()` or `parseForESLint()` method and returns the result.
    #[must_use]
    pub fn with_parse_file(mut self, parse_file: ExternalLinterParseFileCb) -> Self {
        self.parse_file = Some(parse_file);
        self
    }

    /// Set the callback for stripping custom syntax from files.
    ///
    /// This callback is called in Phase 2 when a file matches a custom parser's patterns.
    /// The parser strips non-JS syntax and provides span mappings so diagnostics can be
    /// remapped to original positions after linting with Rust rules.
    #[must_use]
    pub fn with_strip_file(mut self, strip_file: ExternalLinterStripFileCb) -> Self {
        self.strip_file = Some(strip_file);
        self
    }

    /// Set the callback for linting files with pre-parsed AST from custom parsers.
    ///
    /// This callback is called instead of `lint_file` when linting a file that was
    /// parsed by a custom parser. It receives the AST as JSON instead of a binary buffer.
    #[must_use]
    pub fn with_lint_file_with_custom_ast(
        mut self,
        lint_file_with_custom_ast: ExternalLinterLintFileWithCustomAstCb,
    ) -> Self {
        self.lint_file_with_custom_ast = Some(lint_file_with_custom_ast);
        self
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}

/// Struct for serializing globals and envs to send to JS plugins.
///
/// Serializes as `{ "globals": { "React": "readonly" }, "envs": { "browser": true } }`.
/// `envs` only includes the environments that are enabled, so all properties are `true`.
#[derive(Serialize)]
pub struct GlobalsAndEnvs<'c> {
    globals: &'c OxlintGlobals,
    envs: EnabledEnvs<'c>,
}

impl<'c> GlobalsAndEnvs<'c> {
    pub fn new(ctx_host: &'c ContextHost<'_>) -> Self {
        Self { globals: ctx_host.globals(), envs: EnabledEnvs(ctx_host.env()) }
    }

    /// Create from config directly (for custom parser files that don't have ContextHost)
    pub fn from_config(config: &'c LintConfig) -> Self {
        Self { globals: &config.globals, envs: EnabledEnvs(&config.env) }
    }
}

struct EnabledEnvs<'c>(&'c OxlintEnv);

impl Serialize for EnabledEnvs<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        for env_name in self.0.iter() {
            map.serialize_entry(env_name, &true)?;
        }

        map.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_parser_result_deserialize() {
        let json = r#"{"parserId": 42, "hasParseForEslint": true}"#;
        let result: LoadParserResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.parser_id, 42);
        assert!(result.has_parse_for_eslint);

        let json = r#"{"parserId": 0, "hasParseForEslint": false}"#;
        let result: LoadParserResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.parser_id, 0);
        assert!(!result.has_parse_for_eslint);
    }

    #[test]
    fn test_load_plugin_result_deserialize() {
        let json = r#"{"name": "test-plugin", "offset": 100, "ruleNames": ["rule1", "rule2"]}"#;
        let result: LoadPluginResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.name, "test-plugin");
        assert_eq!(result.offset, 100);
        assert_eq!(result.rule_names, vec!["rule1", "rule2"]);
    }

    #[test]
    fn test_parse_file_result_deserialize() {
        // Basic result with only AST
        let json = r#"{"astJson": "{\"type\":\"Program\"}", "scopeManagerJson": null, "visitorKeysJson": null, "servicesJson": null}"#;
        let result: ParseFileResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.ast_json, "{\"type\":\"Program\"}");
        assert!(result.scope_manager_json.is_none());
        assert!(result.visitor_keys_json.is_none());
        assert!(result.services_json.is_none());

        // Full result from parseForESLint
        let json = r#"{"astJson": "{\"type\":\"Program\"}", "scopeManagerJson": "{\"scopes\":[]}", "visitorKeysJson": "{\"Program\":[\"body\"]}", "servicesJson": "{\"custom\":true}"}"#;
        let result: ParseFileResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.ast_json, "{\"type\":\"Program\"}");
        assert_eq!(result.scope_manager_json.unwrap(), "{\"scopes\":[]}");
        assert_eq!(result.visitor_keys_json.unwrap(), "{\"Program\":[\"body\"]}");
        assert_eq!(result.services_json.unwrap(), "{\"custom\":true}");
    }

    #[test]
    fn test_strip_file_result_deserialize() {
        // Basic result with empty mappings
        let json = r#"{"source": "const x = 1;", "mappings": []}"#;
        let result: StripFileResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.source, "const x = 1;");
        assert!(result.source_type.is_none());
        assert!(result.mappings.is_empty());

        // Result with source type and mappings
        let json = r#"{
            "source": "const x = 1;",
            "sourceType": {"module": true, "typescript": false, "jsx": true},
            "mappings": [
                {"strippedStart": 0, "strippedEnd": 12, "originalStart": 10, "originalEnd": 22}
            ]
        }"#;
        let result: StripFileResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.source, "const x = 1;");
        let source_type = result.source_type.unwrap();
        assert!(source_type.module);
        assert!(!source_type.typescript);
        assert!(source_type.jsx);
        assert_eq!(result.mappings.len(), 1);
        assert_eq!(result.mappings[0].stripped_start, 0);
        assert_eq!(result.mappings[0].stripped_end, 12);
        assert_eq!(result.mappings[0].original_start, 10);
        assert_eq!(result.mappings[0].original_end, 22);
    }

    #[test]
    fn test_span_mapping_serialize_deserialize() {
        let mapping = SpanMapping::new(0, 10, 5, 15);
        let json = serde_json::to_string(&mapping).unwrap();
        let deserialized: SpanMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.stripped_start, 0);
        assert_eq!(deserialized.stripped_end, 10);
        assert_eq!(deserialized.original_start, 5);
        assert_eq!(deserialized.original_end, 15);
    }

    #[test]
    fn test_strip_source_type_to_source_type() {
        // Default (all false)
        let st = StripSourceType { module: false, typescript: false, jsx: false };
        let source_type = st.to_source_type();
        assert!(!source_type.is_module());
        assert!(!source_type.is_typescript());
        assert!(!source_type.is_jsx());

        // Module with TypeScript and JSX
        let st = StripSourceType { module: true, typescript: true, jsx: true };
        let source_type = st.to_source_type();
        assert!(source_type.is_module());
        assert!(source_type.is_typescript());
        assert!(source_type.is_jsx());
    }

    #[test]
    fn test_stripped_source_remap_span_empty_mappings() {
        let source = StrippedSource::new(Cow::Borrowed("const x = 1;"), SourceType::mjs(), vec![]);

        // With no mappings, spans are returned as-is
        assert_eq!(source.remap_span(0, 10), (0, 10));
        assert_eq!(source.remap_span(5, 15), (5, 15));
    }

    #[test]
    fn test_stripped_source_remap_span_single_mapping() {
        // Source had 10 chars of custom syntax prepended
        // Original: "<custom/>const x = 1;" (positions 0-21)
        // Stripped: "const x = 1;" (positions 0-11)
        // Mapping: stripped 0-11 -> original 10-21
        let mappings = vec![SpanMapping::new(0, 11, 10, 21)];
        let source = StrippedSource::new(Cow::Borrowed("const x = 1;"), SourceType::mjs(), mappings);

        // Position 0 in stripped -> position 10 in original
        assert_eq!(source.remap_span(0, 5), (10, 15));

        // Position 6 in stripped -> position 16 in original
        assert_eq!(source.remap_span(6, 11), (16, 21));
    }

    #[test]
    fn test_stripped_source_remap_span_multiple_mappings() {
        // Original: "<tag>code1</tag><tag>code2</tag>"
        //           0    5    10   15   20   25   30
        // Stripped: "code1code2"
        //           0    5    9
        // Mappings: 0-4 -> 5-9, 5-9 -> 21-25
        let mappings = vec![SpanMapping::new(0, 4, 5, 9), SpanMapping::new(5, 9, 21, 25)];
        let source = StrippedSource::new(Cow::Borrowed("code1code2"), SourceType::mjs(), mappings);

        // Position in first section
        assert_eq!(source.remap_span(0, 4), (5, 9));

        // Position in second section
        assert_eq!(source.remap_span(5, 9), (21, 25));
    }
}
