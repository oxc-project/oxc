use std::fmt::Debug;

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use oxc_allocator::Allocator;

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

pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) load_parser: Option<ExternalLinterLoadParserCb>,
    pub(crate) parse_file: Option<ExternalLinterParseFileCb>,
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
}
