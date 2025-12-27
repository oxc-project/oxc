use std::{borrow::Cow, fmt::Debug, path::Path, sync::Arc};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use oxc_allocator::Allocator;
use oxc_ast::deserialize::{DeserError, FromESTree};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::module_record::ModuleRecord;

use crate::{
    Linter, Message,
    PossibleFixes,
    config::{LintConfig, OxlintEnv, OxlintGlobals},
    context::{ContextHost, ContextSubHost},
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

// ============================================================================
// Serialized ScopeManager types for Phase 3 (ESTree deserialization)
// ============================================================================

/// Serialized scope manager from custom parser's parseForESLint().
///
/// This is a flattened representation of ESLint's ScopeManager that can be
/// deserialized from JSON and used to inject scope information into oxc's Semantic.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedScopeManager {
    /// All scopes in the program.
    pub scopes: Vec<SerializedScope>,
    /// All variables declared in the program.
    pub variables: Vec<SerializedVariable>,
    /// All references in the program.
    pub references: Vec<SerializedReference>,
}

/// Serialized scope information.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedScope {
    /// Unique identifier for this scope (index in scopes array).
    pub id: u32,
    /// Scope type matching ESLint scope types.
    #[serde(rename = "type")]
    pub scope_type: String,
    /// Parent scope ID, or null for global scope.
    pub parent_id: Option<u32>,
    /// Whether this scope is in strict mode.
    pub is_strict: bool,
    /// IDs of variables declared in this scope.
    pub variable_ids: Vec<u32>,
    /// Span of the block node that created this scope.
    pub block_span: Option<SerializedSpan>,
}

/// Serialized variable information.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedVariable {
    /// Unique identifier for this variable (index in variables array).
    pub id: u32,
    /// Variable name.
    pub name: String,
    /// ID of the scope this variable is declared in.
    pub scope_id: u32,
    /// Definition type (e.g., "Variable", "Parameter", "ImportBinding").
    pub definition_type: Option<String>,
    /// Span of the variable declaration.
    pub span: Option<SerializedSpan>,
    /// IDs of references to this variable.
    pub reference_ids: Vec<u32>,
}

/// Serialized reference information.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedReference {
    /// Unique identifier for this reference (index in references array).
    pub id: u32,
    /// Name being referenced.
    pub name: String,
    /// ID of the variable this reference resolves to, or null if unresolved.
    pub variable_id: Option<u32>,
    /// Span of the reference identifier.
    pub span: SerializedSpan,
    /// Whether this is a read reference.
    pub is_read: bool,
    /// Whether this is a write reference.
    pub is_write: bool,
}

/// Serialized span (start/end positions).
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct SerializedSpan {
    /// Start byte offset.
    pub start: u32,
    /// End byte offset.
    pub end: u32,
}

impl SerializedSpan {
    /// Convert to oxc Span.
    #[must_use]
    pub fn to_span(self) -> oxc_span::Span {
        oxc_span::Span::new(self.start, self.end)
    }
}

impl SerializedScope {
    /// Convert ESLint scope type string to oxc ScopeFlags.
    #[must_use]
    pub fn to_scope_flags(&self) -> oxc_syntax::scope::ScopeFlags {
        use oxc_syntax::scope::ScopeFlags;

        let mut flags = ScopeFlags::empty();

        // Add strict mode flag if enabled
        if self.is_strict {
            flags |= ScopeFlags::StrictMode;
        }

        // Map ESLint scope types to oxc ScopeFlags
        match self.scope_type.as_str() {
            "global" => flags |= ScopeFlags::Top,
            "module" => flags |= ScopeFlags::Top | ScopeFlags::StrictMode,
            "function" => flags |= ScopeFlags::Function,
            "function-expression-name" => {
                // Function expression name scope is a special case
                // In oxc, this would be part of the function scope
                flags |= ScopeFlags::Function;
            }
            "class-static-block" => flags |= ScopeFlags::ClassStaticBlock,
            "catch" => flags |= ScopeFlags::CatchClause,
            "with" => flags |= ScopeFlags::With,
            "block" | "class" | "for" | "switch" | "class-field-initializer" => {
                // These are block scopes with no special flags
            }
            _ => {
                // Unknown scope type - treat as block scope
            }
        }

        flags
    }
}

impl SerializedReference {
    /// Convert to oxc ReferenceFlags.
    #[must_use]
    pub fn to_reference_flags(&self) -> oxc_syntax::reference::ReferenceFlags {
        use oxc_syntax::reference::ReferenceFlags;

        let mut flags = ReferenceFlags::None;

        if self.is_read {
            flags |= ReferenceFlags::Read;
        }
        if self.is_write {
            flags |= ReferenceFlags::Write;
        }

        flags
    }
}

/// Inject external scope information from a custom parser into oxc's semantic analysis.
///
/// This function takes the serialized scope manager from a custom parser's `parseForESLint()`
/// output and merges it with the existing semantic data. This enables Rust rules like
/// `no-unused-vars` to work correctly with custom syntax by providing scope/reference
/// information that oxc's parser couldn't extract from the stripped source.
///
/// # Arguments
///
/// * `scope_manager_json` - JSON string containing the serialized scope manager
///
/// # Returns
///
/// The deserialized scope manager, or an error if parsing fails.
///
/// # Example
///
/// ```ignore
/// let scope_json = r#"{"scopes": [], "variables": [], "references": []}"#;
/// let scope_manager = inject_external_scope(scope_json)?;
/// ```
pub fn inject_external_scope(scope_manager_json: &str) -> Result<SerializedScopeManager, String> {
    serde_json::from_str(scope_manager_json).map_err(|e| {
        format!("Failed to parse scope manager JSON: {e}")
    })
}

/// Result from deserializing ESTree JSON to oxc AST.
#[derive(Debug)]
pub enum DeserializeResult {
    /// Deserialization succeeded
    Success,
    /// Root node is not a JavaScript Program (e.g., JSON AST).
    /// Rust rules should be skipped; JS rules handle the file.
    NonJsAst(String),
    /// Unknown node type encountered during deserialization.
    /// Contains the node type name and optional context.
    UnknownNode(String),
    /// Deserialization failed with an error
    Error(String),
}

/// Lint a file using an externally-provided ESTree AST and scope manager.
///
/// This is the Phase 3 entry point for custom parser support. Instead of re-parsing
/// stripped source, this function:
/// 1. Deserializes the ESTree JSON directly into an oxc AST
/// 2. Builds semantic analysis from that AST
/// 3. Injects external scope information if provided
/// 4. Runs Rust linting rules
///
/// # Arguments
///
/// * `linter` - The linter instance to run rules with
/// * `path` - Path to the file being linted
/// * `source_text` - Original source text (for diagnostics)
/// * `estree_json` - JSON string containing the ESTree AST from the custom parser
/// * `scope_manager_json` - Optional JSON string containing the serialized scope manager
///
/// # Returns
///
/// A tuple of (messages, result) where result indicates success or type of failure.
/// Even on success, messages may be empty if no rules triggered.
pub fn lint_with_external_ast(
    linter: &Linter,
    path: &Path,
    source_text: &str,
    estree_json: &str,
    scope_manager_json: Option<&str>,
) -> (Vec<Message>, DeserializeResult) {
    // Parse the ESTree JSON
    let estree_value: serde_json::Value = match serde_json::from_str(estree_json) {
        Ok(v) => v,
        Err(e) => {
            return (vec![], DeserializeResult::Error(format!("Failed to parse ESTree JSON: {e}")));
        }
    };

    // Verify the root is a Program node
    let root_type = match estree_value.get("type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return (
                vec![],
                DeserializeResult::Error("ESTree JSON missing 'type' field".to_string()),
            );
        }
    };

    if root_type != "Program" {
        // Non-JS AST (e.g., JSON AST from eslint-plugin-jsonc) - skip Rust rules
        // This is expected behavior, not an error. JS plugin rules will handle it.
        return (vec![], DeserializeResult::NonJsAst(root_type.to_string()));
    }

    // Parse scope manager if provided
    let external_scope = if let Some(json) = scope_manager_json {
        match inject_external_scope(json) {
            Ok(sm) => Some(sm),
            Err(e) => {
                // Log warning but continue without scope injection
                return (vec![], DeserializeResult::Error(format!("Failed to parse scope manager: {e}")));
            }
        }
    } else {
        None
    };

    // Create allocator for AST
    let allocator = Allocator::default();

    // Deserialize ESTree JSON to oxc Program
    let program: oxc_ast::ast::Program = match FromESTree::from_estree(&estree_value, &allocator) {
        Ok(p) => p,
        Err(DeserError::UnknownNodeType(node_type)) => {
            // Unknown node type - likely custom syntax. For now, skip Rust rules.
            // TODO: Implement placeholder handling as per plan
            return (vec![], DeserializeResult::UnknownNode(node_type));
        }
        Err(e) => {
            return (vec![], DeserializeResult::Error(format!("Failed to deserialize ESTree: {e}")));
        }
    };

    // Determine source type from the program
    let source_type = SourceType::from_path(path).unwrap_or_default();

    // Build semantic analysis
    let semantic_ret = SemanticBuilder::new()
        .with_cfg(true)
        .with_scope_tree_child_ids(true)
        .with_check_syntax_error(true)
        .build(allocator.alloc(program));

    if !semantic_ret.errors.is_empty() {
        // Semantic errors - return them as messages
        let messages = semantic_ret
            .errors
            .into_iter()
            .map(|err| Message::new(err, PossibleFixes::None))
            .collect();
        return (messages, DeserializeResult::Success);
    }

    let mut semantic = semantic_ret.semantic;

    // TODO: Inject external scope information
    // This is where we'd add references from custom syntax to fix no-unused-vars
    if let Some(_scope_manager) = external_scope {
        // Phase 3.3: Inject scope information
        // For now, just acknowledge we have scope info
        // inject_scope_references(&mut semantic, &scope_manager);
    }

    // Create module record (empty for external AST)
    let module_record = Arc::new(ModuleRecord::default());

    // Create context and run Rust rules
    let ctx_sub_host = ContextSubHost::new(semantic, module_record, 0);

    // Run only Rust rules
    let (messages, _disable_directives) = linter.run_rust_rules_only(
        path,
        vec![ctx_sub_host],
        &allocator,
    );

    (messages, DeserializeResult::Success)
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

    #[test]
    fn test_serialized_scope_manager_deserialize() {
        let json = r#"{
            "scopes": [
                {
                    "id": 0,
                    "type": "global",
                    "parentId": null,
                    "isStrict": false,
                    "variableIds": [0],
                    "blockSpan": {"start": 0, "end": 100}
                },
                {
                    "id": 1,
                    "type": "function",
                    "parentId": 0,
                    "isStrict": true,
                    "variableIds": [1, 2],
                    "blockSpan": {"start": 10, "end": 50}
                }
            ],
            "variables": [
                {
                    "id": 0,
                    "name": "x",
                    "scopeId": 0,
                    "definitionType": "Variable",
                    "span": {"start": 4, "end": 5},
                    "referenceIds": [0]
                },
                {
                    "id": 1,
                    "name": "y",
                    "scopeId": 1,
                    "definitionType": "Parameter",
                    "span": {"start": 20, "end": 21},
                    "referenceIds": []
                },
                {
                    "id": 2,
                    "name": "z",
                    "scopeId": 1,
                    "definitionType": null,
                    "span": null,
                    "referenceIds": [1]
                }
            ],
            "references": [
                {
                    "id": 0,
                    "name": "x",
                    "variableId": 0,
                    "span": {"start": 30, "end": 31},
                    "isRead": true,
                    "isWrite": false
                },
                {
                    "id": 1,
                    "name": "z",
                    "variableId": 2,
                    "span": {"start": 35, "end": 36},
                    "isRead": false,
                    "isWrite": true
                },
                {
                    "id": 2,
                    "name": "unknown",
                    "variableId": null,
                    "span": {"start": 40, "end": 47},
                    "isRead": true,
                    "isWrite": false
                }
            ]
        }"#;

        let scope_manager: SerializedScopeManager = serde_json::from_str(json).unwrap();

        // Check scopes
        assert_eq!(scope_manager.scopes.len(), 2);
        assert_eq!(scope_manager.scopes[0].scope_type, "global");
        assert!(scope_manager.scopes[0].parent_id.is_none());
        assert!(!scope_manager.scopes[0].is_strict);
        assert_eq!(scope_manager.scopes[1].scope_type, "function");
        assert_eq!(scope_manager.scopes[1].parent_id, Some(0));
        assert!(scope_manager.scopes[1].is_strict);

        // Check variables
        assert_eq!(scope_manager.variables.len(), 3);
        assert_eq!(scope_manager.variables[0].name, "x");
        assert_eq!(scope_manager.variables[0].definition_type, Some("Variable".to_string()));
        assert_eq!(scope_manager.variables[1].name, "y");
        assert_eq!(scope_manager.variables[1].definition_type, Some("Parameter".to_string()));
        assert!(scope_manager.variables[2].definition_type.is_none());

        // Check references
        assert_eq!(scope_manager.references.len(), 3);
        assert_eq!(scope_manager.references[0].variable_id, Some(0));
        assert!(scope_manager.references[0].is_read);
        assert!(!scope_manager.references[0].is_write);
        assert!(scope_manager.references[1].is_write);
        assert!(scope_manager.references[2].variable_id.is_none()); // unresolved
    }

    #[test]
    fn test_serialized_scope_to_scope_flags() {
        use oxc_syntax::scope::ScopeFlags;

        // Global scope
        let scope = SerializedScope {
            id: 0,
            scope_type: "global".to_string(),
            parent_id: None,
            is_strict: false,
            variable_ids: vec![],
            block_span: None,
        };
        assert!(scope.to_scope_flags().contains(ScopeFlags::Top));
        assert!(!scope.to_scope_flags().contains(ScopeFlags::StrictMode));

        // Module scope (always strict)
        let scope = SerializedScope {
            id: 0,
            scope_type: "module".to_string(),
            parent_id: None,
            is_strict: true,
            variable_ids: vec![],
            block_span: None,
        };
        let flags = scope.to_scope_flags();
        assert!(flags.contains(ScopeFlags::Top));
        assert!(flags.contains(ScopeFlags::StrictMode));

        // Function scope with strict mode
        let scope = SerializedScope {
            id: 1,
            scope_type: "function".to_string(),
            parent_id: Some(0),
            is_strict: true,
            variable_ids: vec![],
            block_span: None,
        };
        let flags = scope.to_scope_flags();
        assert!(flags.contains(ScopeFlags::Function));
        assert!(flags.contains(ScopeFlags::StrictMode));

        // Block scope (no special flags)
        let scope = SerializedScope {
            id: 2,
            scope_type: "block".to_string(),
            parent_id: Some(1),
            is_strict: true,
            variable_ids: vec![],
            block_span: None,
        };
        let flags = scope.to_scope_flags();
        assert!(!flags.contains(ScopeFlags::Function));
        assert!(!flags.contains(ScopeFlags::Top));
        assert!(flags.contains(ScopeFlags::StrictMode));
    }

    #[test]
    fn test_serialized_reference_to_reference_flags() {
        use oxc_syntax::reference::ReferenceFlags;

        // Read only
        let ref1 = SerializedReference {
            id: 0,
            name: "x".to_string(),
            variable_id: Some(0),
            span: SerializedSpan { start: 0, end: 1 },
            is_read: true,
            is_write: false,
        };
        let flags = ref1.to_reference_flags();
        assert!(flags.contains(ReferenceFlags::Read));
        assert!(!flags.contains(ReferenceFlags::Write));

        // Write only
        let ref2 = SerializedReference {
            id: 1,
            name: "x".to_string(),
            variable_id: Some(0),
            span: SerializedSpan { start: 5, end: 6 },
            is_read: false,
            is_write: true,
        };
        let flags = ref2.to_reference_flags();
        assert!(!flags.contains(ReferenceFlags::Read));
        assert!(flags.contains(ReferenceFlags::Write));

        // Read-write
        let ref3 = SerializedReference {
            id: 2,
            name: "x".to_string(),
            variable_id: Some(0),
            span: SerializedSpan { start: 10, end: 11 },
            is_read: true,
            is_write: true,
        };
        let flags = ref3.to_reference_flags();
        assert!(flags.contains(ReferenceFlags::Read));
        assert!(flags.contains(ReferenceFlags::Write));
    }

    #[test]
    fn test_serialized_span_to_span() {
        let span = SerializedSpan { start: 10, end: 25 };
        let oxc_span = span.to_span();
        assert_eq!(oxc_span.start, 10);
        assert_eq!(oxc_span.end, 25);
    }

    #[test]
    fn test_inject_external_scope_success() {
        let json = r#"{
            "scopes": [{"id": 0, "type": "global", "parentId": null, "isStrict": false, "variableIds": [], "blockSpan": null}],
            "variables": [],
            "references": []
        }"#;

        let result = inject_external_scope(json);
        assert!(result.is_ok());
        let scope_manager = result.unwrap();
        assert_eq!(scope_manager.scopes.len(), 1);
        assert_eq!(scope_manager.scopes[0].scope_type, "global");
    }

    #[test]
    fn test_inject_external_scope_invalid_json() {
        let json = r#"{ invalid json }"#;
        let result = inject_external_scope(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse scope manager JSON"));
    }

    #[test]
    fn test_deserialize_result_debug() {
        // Test that DeserializeResult can be debug-printed
        let result = DeserializeResult::Success;
        assert!(format!("{:?}", result).contains("Success"));

        let result = DeserializeResult::NonJsAst("JSONRoot".to_string());
        assert!(format!("{:?}", result).contains("NonJsAst"));
        assert!(format!("{:?}", result).contains("JSONRoot"));

        let result = DeserializeResult::UnknownNode("GlimmerTemplate".to_string());
        assert!(format!("{:?}", result).contains("UnknownNode"));
        assert!(format!("{:?}", result).contains("GlimmerTemplate"));

        let result = DeserializeResult::Error("test error".to_string());
        assert!(format!("{:?}", result).contains("Error"));
        assert!(format!("{:?}", result).contains("test error"));
    }
}
