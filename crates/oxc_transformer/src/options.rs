use std::path::{Path, PathBuf};

use oxc_diagnostics::{Error, OxcDiagnostic};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::{
    compiler_assumptions::CompilerAssumptions,
    es2015::{ArrowFunctionsOptions, ES2015Options},
    react::ReactOptions,
    typescript::TypeScriptOptions,
};

/// <https://babel.dev/docs/options>
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    //
    // Primary Options
    //
    /// The working directory that all paths in the programmatic options will be resolved relative to.
    pub cwd: PathBuf,

    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub react: ReactOptions,

    pub es2015: ES2015Options,
}

impl TransformOptions {
    /// # Errors
    ///
    pub fn from_babel_options(options: &BabelOptions) -> Result<Self, Vec<Error>> {
        fn get_options<T: Default + DeserializeOwned>(
            name: &str,
            babel_options: &BabelOptions,
            errors: &mut Vec<Error>,
            is_preset: bool,
        ) -> T {
            let target = if is_preset {
                babel_options.get_preset(name)
            } else {
                babel_options.get_plugin(name)
            };
            target
                .and_then(|plugin_options| {
                    plugin_options.and_then(|options| match serde_json::from_value::<T>(options) {
                        Ok(options) => Some(options),
                        Err(err) => {
                            let kind_msg =
                                if is_preset { format!("preset-{name}") } else { name.to_string() };
                            errors.push(OxcDiagnostic::error(format!("{kind_msg}: {err}")).into());
                            None
                        }
                    })
                })
                .unwrap_or_else(|| T::default())
        }

        let mut errors = Vec::<Error>::new();

        let react = if options.has_preset("react") {
            get_options::<ReactOptions>("react", options, &mut errors, true)
        } else {
            let has_jsx_plugin = options.has_plugin("transform-react-jsx");
            let has_jsx_development_plugin = options.has_plugin("transform-react-jsx-development");
            let mut react_options = if has_jsx_plugin {
                get_options::<ReactOptions>("transform-react-jsx", options, &mut errors, false)
            } else {
                get_options::<ReactOptions>(
                    "transform-react-jsx-development",
                    options,
                    &mut errors,
                    false,
                )
            };
            react_options.development = options.has_plugin("transform-react-jsx-development");
            react_options.jsx_plugin = has_jsx_plugin || has_jsx_development_plugin;
            react_options.display_name_plugin = options.has_plugin("transform-react-display-name");
            react_options.jsx_self_plugin = options.has_plugin("transform-react-jsx-self");
            react_options.jsx_source_plugin = options.has_plugin("transform-react-jsx-source");
            react_options
        };

        let es2015 = ES2015Options {
            arrow_function: options.has_plugin("transform-arrow-functions").then(|| {
                get_options::<ArrowFunctionsOptions>(
                    "transform-arrow-functions",
                    options,
                    &mut errors,
                    false,
                )
            }),
        };

        let typescript =
            get_options::<TypeScriptOptions>("transform-typescript", options, &mut errors, false);

        let assumptions = if options.assumptions.is_null() {
            CompilerAssumptions::default()
        } else {
            match serde_json::from_value::<CompilerAssumptions>(options.assumptions.clone()) {
                Ok(value) => value,
                Err(err) => {
                    errors.push(OxcDiagnostic::error(err.to_string()).into());
                    CompilerAssumptions::default()
                }
            }
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            cwd: options.cwd.clone().unwrap_or_default(),
            assumptions,
            typescript,
            react,
            es2015,
        })
    }
}

/// Babel options
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BabelOptions {
    pub cwd: Option<PathBuf>,
    pub source_type: Option<String>,
    #[serde(default)]
    pub plugins: Vec<Value>, // Can be a string or an array
    #[serde(default)]
    pub presets: Vec<Value>, // Can be a string or an array
    #[serde(default)]
    pub assumptions: Value,
    // Test options
    pub throws: Option<String>,
    #[serde(rename = "BABEL_8_BREAKING")]
    pub babel_8_breaking: Option<bool>,
    /// Babel test helper for running tests on specific operating systems
    pub os: Option<Vec<TestOs>>,
    // Parser options for babel-parser
    #[serde(default)]
    pub allow_return_outside_function: bool,
    #[serde(default)]
    pub allow_await_outside_function: bool,
    #[serde(default)]
    pub allow_undeclared_exports: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestOs {
    Linux,
    Win32,
    Windows,
    Darwin,
}

impl TestOs {
    pub fn is_windows(&self) -> bool {
        matches!(self, Self::Win32 | Self::Windows)
    }
}

impl BabelOptions {
    /// Read options.json and merge them with options.json from ancestors directories.
    /// # Panics
    pub fn from_test_path(path: &Path) -> Self {
        let mut options_json: Option<Self> = None;
        for path in path.ancestors().take(3) {
            let file = path.join("options.json");
            if !file.exists() {
                continue;
            }
            let file = std::fs::read_to_string(&file).unwrap();
            let new_json: Self = serde_json::from_str(&file).unwrap();
            if let Some(existing_json) = options_json.as_mut() {
                if existing_json.source_type.is_none() {
                    if let Some(source_type) = new_json.source_type {
                        existing_json.source_type = Some(source_type);
                    }
                }
                if existing_json.throws.is_none() {
                    if let Some(throws) = new_json.throws {
                        existing_json.throws = Some(throws);
                    }
                }
                existing_json.plugins.extend(new_json.plugins);
            } else {
                options_json = Some(new_json);
            }
        }
        options_json.unwrap_or_default()
    }

    pub fn is_jsx(&self) -> bool {
        self.plugins.iter().any(|v| v.as_str().is_some_and(|v| v == "jsx"))
    }

    pub fn is_typescript(&self) -> bool {
        self.plugins.iter().any(|v| {
            let string_value = v.as_str().is_some_and(|v| v == "typescript");
            let array_value = v.get(0).and_then(Value::as_str).is_some_and(|s| s == "typescript");
            string_value || array_value
        })
    }

    pub fn is_typescript_definition(&self) -> bool {
        self.plugins.iter().filter_map(Value::as_array).any(|p| {
            let typescript = p.first().and_then(Value::as_str).is_some_and(|s| s == "typescript");
            let dts = p
                .get(1)
                .and_then(Value::as_object)
                .and_then(|v| v.get("dts"))
                .and_then(Value::as_bool)
                .is_some_and(|v| v);
            typescript && dts
        })
    }

    pub fn is_module(&self) -> bool {
        self.source_type.as_ref().map_or(false, |s| matches!(s.as_str(), "module" | "unambiguous"))
    }

    /// Returns
    /// * `Some<None>` if the plugin exists without a config
    /// * `Some<Some<Value>>` if the plugin exists with a config
    /// * `None` if the plugin does not exist
    pub fn get_plugin(&self, name: &str) -> Option<Option<Value>> {
        self.plugins.iter().find_map(|v| Self::get_value(v, name))
    }

    pub fn get_preset(&self, name: &str) -> Option<Option<Value>> {
        self.presets.iter().find_map(|v| Self::get_value(v, name))
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.get_plugin(name).is_some()
    }

    pub fn has_preset(&self, name: &str) -> bool {
        self.get_preset(name).is_some()
    }

    #[allow(clippy::option_option)]
    fn get_value(value: &Value, name: &str) -> Option<Option<Value>> {
        match value {
            Value::String(s) if s == name => Some(None),
            Value::Array(a) if a.first().and_then(Value::as_str).is_some_and(|s| s == name) => {
                Some(a.get(1).cloned())
            }
            _ => None,
        }
    }
}

#[test]
fn test_deny_unknown_fields() {
    let options = serde_json::json!({
      "plugins": [["transform-react-jsx", { "runtime": "automatic", "filter": 1 }]],
      "sourceType": "module"
    });
    let babel_options = serde_json::from_value::<BabelOptions>(options).unwrap();
    let result = TransformOptions::from_babel_options(&babel_options);
    assert!(result.is_err());
    let err_message =
        result.err().unwrap().iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    assert!(err_message.contains("transform-react-jsx: unknown field `filter`"));
}
