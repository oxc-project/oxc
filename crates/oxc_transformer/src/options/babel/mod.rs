mod env;

use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::{
    es2015::ArrowFunctionsOptions, es2018::ObjectRestSpreadOptions, es2022::ClassPropertiesOptions,
    jsx::JsxOptions, TypeScriptOptions,
};

pub use env::{BabelEnvOptions, Targets};

/// Babel options
///
/// <https://babel.dev/docs/options#plugin-and-preset-options>
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BabelOptions {
    // Primary options
    pub cwd: Option<PathBuf>,

    // Config Loading options

    // Plugin and Preset options
    #[serde(default)]
    pub plugins: BabelPlugins,

    #[serde(default)]
    pub presets: Vec<Value>, // Can be a string or an array

    // Misc options
    pub source_type: Option<String>,

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

    #[serde(default = "default_as_true")]
    pub external_helpers: bool,
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

fn default_as_true() -> bool {
    true
}

impl BabelOptions {
    /// Read options.json and merge them with options.json from ancestors directories.
    /// # Panics
    pub fn from_test_path(path: &Path) -> Self {
        let mut babel_options: Option<Self> = None;
        let mut plugins_json = None;

        for path in path.ancestors().take(3) {
            let file = path.join("options.json");
            if !file.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&file).unwrap();
            let mut new_value = serde_json::from_str::<serde_json::Value>(&content).unwrap();

            let new_plugins = new_value.as_object_mut().unwrap().remove("plugins");
            if plugins_json.is_none() {
                plugins_json = new_plugins;
            }

            let new_options: Self = serde_json::from_value::<BabelOptions>(new_value)
                .unwrap_or_else(|err| panic!("{err:?}\n{file:?}\n{content}"));

            if let Some(existing_options) = babel_options.as_mut() {
                if existing_options.source_type.is_none() {
                    if let Some(source_type) = new_options.source_type {
                        existing_options.source_type = Some(source_type);
                    }
                }
                if existing_options.throws.is_none() {
                    if let Some(throws) = new_options.throws {
                        existing_options.throws = Some(throws);
                    }
                }
                if existing_options.presets.is_empty() {
                    existing_options.presets = new_options.presets;
                }
            } else {
                babel_options = Some(new_options);
            }
        }

        let mut options = babel_options.unwrap_or_default();
        if let Some(plugins_json) = plugins_json {
            options.plugins = serde_json::from_value::<BabelPlugins>(plugins_json).unwrap();
        }
        options
    }

    pub fn is_jsx(&self) -> bool {
        self.plugins.syntax_jsx
    }

    pub fn is_typescript(&self) -> bool {
        self.plugins.syntax_typescript.is_some()
    }

    pub fn is_typescript_definition(&self) -> bool {
        self.plugins.syntax_typescript.is_some_and(|o| o.dts)
    }

    pub fn is_module(&self) -> bool {
        self.source_type.as_ref().map_or(false, |s| s.as_str() == "module")
    }

    pub fn is_unambiguous(&self) -> bool {
        self.source_type.as_ref().map_or(false, |s| s.as_str() == "unambiguous")
    }

    pub fn get_preset(&self, name: &str) -> Option<Option<Value>> {
        self.presets.iter().find_map(|v| Self::get_value(v, name))
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

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct SyntaxTypeScriptOptions {
    #[serde(default)]
    pub dts: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SyntaxDecoratorOptions {
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "PluginPresetEntries")]
pub struct BabelPlugins {
    pub errors: Vec<String>,
    pub unsupported: Vec<String>,
    // syntax
    pub syntax_typescript: Option<SyntaxTypeScriptOptions>,
    pub syntax_jsx: bool,
    // decorators
    pub syntax_decorators: Option<SyntaxDecoratorOptions>,
    pub proposal_decorators: Option<SyntaxDecoratorOptions>,
    // ts
    pub typescript: Option<TypeScriptOptions>,
    // jsx
    pub react_jsx: Option<JsxOptions>,
    pub react_jsx_dev: Option<JsxOptions>,
    pub react_jsx_self: bool,
    pub react_jsx_source: bool,
    pub react_display_name: bool,
    // regexp
    pub sticky_flag: bool,
    pub unicode_flag: bool,
    pub dot_all_flag: bool,
    pub look_behind_assertions: bool,
    pub named_capture_groups: bool,
    pub unicode_property_escapes: bool,
    pub match_indices: bool,
    /// Enables plugin to transform the RegExp literal has `v` flag
    pub set_notation: bool,
    // ES2015
    pub arrow_function: Option<ArrowFunctionsOptions>,
    // ES2016
    pub exponentiation_operator: bool,
    // ES2017
    pub async_to_generator: bool,
    // ES2018
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,
    pub async_generator_functions: bool,
    // ES2019
    pub optional_catch_binding: bool,
    // ES2020
    pub nullish_coalescing_operator: bool,
    // ES2021
    pub logical_assignment_operators: bool,
    // ES2022
    pub class_static_block: bool,
    pub class_properties: Option<ClassPropertiesOptions>,
}

/// <https://babeljs.io/docs/options#pluginpreset-entries>
#[derive(Debug, Deserialize)]
struct PluginPresetEntries(Vec<PluginPresetEntry>);

impl TryFrom<PluginPresetEntries> for BabelPlugins {
    type Error = String;

    fn try_from(entries: PluginPresetEntries) -> Result<Self, Self::Error> {
        let mut p = BabelPlugins::default();
        for entry in entries.0 {
            match entry.name() {
                "typescript" | "syntax-typescript" => {
                    p.syntax_typescript = Some(entry.value::<SyntaxTypeScriptOptions>()?);
                }
                "jsx" | "syntax-jsx" => p.syntax_jsx = true,
                "syntax-decorators" => {
                    p.syntax_decorators = Some(entry.value::<SyntaxDecoratorOptions>()?);
                }
                "proposal-decorators" => {
                    p.proposal_decorators = Some(entry.value::<SyntaxDecoratorOptions>()?);
                }
                "transform-typescript" => {
                    p.typescript =
                        entry.value::<TypeScriptOptions>().map_err(|err| p.errors.push(err)).ok();
                }
                "transform-react-jsx" => {
                    p.react_jsx =
                        entry.value::<JsxOptions>().map_err(|err| p.errors.push(err)).ok();
                }
                "transform-react-jsx-development" => {
                    p.react_jsx_dev =
                        entry.value::<JsxOptions>().map_err(|err| p.errors.push(err)).ok();
                }
                "transform-react-display-name" => p.react_display_name = true,
                "transform-react-jsx-self" => p.react_jsx_self = true,
                "transform-react-jsx-source" => p.react_jsx_source = true,
                "transform-sticky-regex" => p.sticky_flag = true,
                "transform-unicode-regex" => p.unicode_flag = true,
                "transform-dotall-regex" => p.dot_all_flag = true,
                "esbuild-regexp-lookbehind-assertions" => p.look_behind_assertions = true,
                "transform-named-capturing-groups-regex" => p.named_capture_groups = true,
                "transform-unicode-property-regex" => p.unicode_property_escapes = true,
                "esbuild-regexp-match-indices" => p.match_indices = true,
                "transform-unicode-sets-regex" => p.set_notation = true,
                "transform-arrow-functions" => {
                    p.arrow_function = entry
                        .value::<ArrowFunctionsOptions>()
                        .map_err(|err| p.errors.push(err))
                        .ok();
                }
                "transform-exponentiation-operator" => p.exponentiation_operator = true,
                "transform-async-to-generator" => p.async_to_generator = true,
                "transform-object-rest-spread" => {
                    p.object_rest_spread = entry
                        .value::<ObjectRestSpreadOptions>()
                        .inspect_err(|err| p.errors.push(err.to_string()))
                        .ok();
                }
                "transform-async-generator-functions" => p.async_generator_functions = true,
                "transform-optional-catch-binding" => p.optional_catch_binding = true,
                "transform-nullish-coalescing-operator" => p.nullish_coalescing_operator = true,
                "transform-logical-assignment-operators" => p.logical_assignment_operators = true,
                "transform-class-static-block" => p.class_static_block = true,
                "transform-class-properties" => {
                    p.class_properties = entry
                        .value::<ClassPropertiesOptions>()
                        .inspect_err(|err| p.errors.push(err.to_string()))
                        .ok();
                }
                s => p.unsupported.push(s.to_string()),
            }
        }
        Ok(p)
    }
}

/// <https://babeljs.io/docs/options#pluginpreset-entries>
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PluginPresetEntry {
    String(String),
    Vec1([String; 1]),
    Tuple(String, serde_json::Value),
}

impl PluginPresetEntry {
    fn name(&self) -> &str {
        match self {
            Self::String(s) | Self::Tuple(s, _) => s,
            Self::Vec1(s) => &s[0],
        }
    }

    fn value<T: DeserializeOwned + Default>(self) -> Result<T, String> {
        match self {
            Self::String(_) | Self::Vec1(_) => Ok(T::default()),
            Self::Tuple(name, v) => {
                serde_json::from_value::<T>(v).map_err(|err| format!("{name}: {err}"))
            }
        }
    }
}
