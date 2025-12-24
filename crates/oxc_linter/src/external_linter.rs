use std::fmt::Debug;

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use oxc_allocator::Allocator;

use crate::{
    config::{OxlintEnv, OxlintGlobals},
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
    pub(crate) setup_configs: ExternalLinterSetupConfigsCb,
    pub(crate) lint_file: ExternalLinterLintFileCb,
}

impl ExternalLinter {
    pub fn new(
        load_plugin: ExternalLinterLoadPluginCb,
        setup_configs: ExternalLinterSetupConfigsCb,
        lint_file: ExternalLinterLintFileCb,
    ) -> Self {
        Self { load_plugin, load_parser: None, setup_configs, lint_file }
    }

    /// Set the parser loading callback.
    ///
    /// This enables support for custom JS-based parsers configured via `jsParsers`.
    #[must_use]
    pub fn with_load_parser(mut self, load_parser: ExternalLinterLoadParserCb) -> Self {
        self.load_parser = Some(load_parser);
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
}
