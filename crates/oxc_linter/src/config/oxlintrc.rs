use std::path::Path;

use oxc_diagnostics::OxcDiagnostic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{env::OxlintEnv, globals::OxlintGlobals, rules::OxlintRules, settings::OxlintSettings};

use crate::{options::LintPlugins, utils::read_to_string};

/// Oxlint Configuration File
///
/// This configuration is aligned with ESLint v8's configuration schema (`eslintrc.json`).
///
/// Usage: `oxlint -c oxlintrc.json --import-plugin`
///
/// ::: danger NOTE
///
/// Only the `.json` format is supported. You can use comments in configuration files.
///
/// :::
///
/// Example
///
/// `.oxlintrc.json`
///
/// ```json
/// {
///   "env": {
///     "browser": true
///   },
///   "globals": {
///     "foo": "readonly"
///   },
///   "settings": {
///   },
///   "rules": {
///     "eqeqeq": "warn",
///     "import/no-cycle": "error"
///   }
///  }
/// ```
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
#[non_exhaustive]
pub struct Oxlintrc {
    pub plugins: LintPlugins,
    /// See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html).
    pub rules: OxlintRules,
    pub settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub globals: OxlintGlobals,
}

impl Oxlintrc {
    /// # Errors
    ///
    /// * Parse Failure
    pub fn from_file(path: &Path) -> Result<Self, OxcDiagnostic> {
        let mut string = read_to_string(path).map_err(|e| {
            OxcDiagnostic::error(format!("Failed to parse config {path:?} with error {e:?}"))
        })?;

        // jsonc support
        json_strip_comments::strip(&mut string).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse jsonc file {path:?}: {err:?}"))
        })?;

        let json = serde_json::from_str::<serde_json::Value>(&string).map_err(|err| {
            let guess = mime_guess::from_path(path);
            let err = match guess.first() {
                // syntax error
                Some(mime) if mime.subtype() == "json" => err.to_string(),
                Some(_) => "Only json configuration is supported".to_string(),
                None => {
                    format!(
                        "{err}, if the configuration is not a json file, please use json instead."
                    )
                }
            };
            OxcDiagnostic::error(format!("Failed to parse eslint config {path:?}.\n{err}"))
        })?;

        let config = Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse config with error {err:?}"))
        })?;

        Ok(config)
    }
}
