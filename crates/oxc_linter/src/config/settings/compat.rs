use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configure browser compatibility checking for the `compat` plugin.
///
/// This mirrors the settings of
/// [eslint-plugin-compat](https://github.com/amilajack/eslint-plugin-compat),
/// namespaced under `compat`.
///
/// Example:
///
/// ```json
/// {
///   "settings": {
///     "compat": {
///       "browsers": ["defaults", "not ie < 11"],
///       "polyfills": ["Promise", "fetch"]
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(default)]
pub struct CompatPluginSettings {
    /// The browserslist targets to lint against, e.g. `"defaults, not ie < 9"`
    /// or `["chrome 70", "firefox 60"]`.
    pub browsers: Option<BrowserslistTargetsConfig>,

    /// Alias for `browsers`. `browsers` takes precedence when both are set.
    pub targets: Option<BrowserslistTargetsConfig>,

    /// APIs that are polyfilled and should not be reported, e.g.
    /// `["Promise", "WebAssembly.compile", "fetch"]`.
    /// The special entry `"es:all"` disables linting of all ECMAScript APIs.
    pub polyfills: Vec<String>,

    /// Lint all ECMAScript APIs, regardless of the `es:all` polyfill entry.
    #[serde(rename = "lintAllEsApis")]
    pub lint_all_es_apis: bool,

    /// Report incompatible API usage even when it is wrapped in a conditional
    /// (feature-detection) check such as `if (window.fetch) { ... }`.
    #[serde(rename = "ignoreConditionalChecks")]
    pub ignore_conditional_checks: bool,
}

/// Browserslist targets: either a single query string, a list of queries, or
/// an object with `production`/`development` query lists.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum BrowserslistTargetsConfig {
    /// A single browserslist query, e.g. `"defaults, not ie < 9"`.
    Query(String),
    /// A list of browserslist queries, e.g. `["chrome 70", "firefox 60"]`.
    Queries(Vec<String>),
    /// Environment-specific browserslist queries; the resolved targets are the
    /// union of all environments.
    Env(BrowserslistEnvTargets),
}

/// Environment-specific browserslist queries.
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(default)]
pub struct BrowserslistEnvTargets {
    /// Browserslist queries for the production environment.
    pub production: Vec<String>,
    /// Browserslist queries for the development environment.
    pub development: Vec<String>,
}

impl BrowserslistTargetsConfig {
    /// Flatten the configuration into a list of browserslist queries,
    /// mirroring `determineTargetsFromConfig` in eslint-plugin-compat.
    pub fn to_queries(&self) -> Vec<String> {
        match self {
            Self::Query(query) => vec![query.clone()],
            Self::Queries(queries) => queries.clone(),
            Self::Env(env) => {
                let mut queries = env.production.clone();
                queries.extend(env.development.iter().cloned());
                queries
            }
        }
    }
}
