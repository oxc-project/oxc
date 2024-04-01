use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

/// Babel options.json for tests
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BabelOptions {
    #[serde(rename = "BABEL_8_BREAKING")]
    pub babel_8_breaking: Option<bool>,
    pub source_type: Option<String>,
    pub throws: Option<String>,
    #[serde(default)]
    pub plugins: Vec<Value>, // Can be a string or an array
    #[serde(default)]
    pub allow_return_outside_function: bool,
    #[serde(default)]
    pub allow_await_outside_function: bool,
    #[serde(default)]
    pub allow_undeclared_exports: bool,
    #[serde(default)]
    pub assumptions: Value,
}

impl BabelOptions {
    /// Read options.json and merge them with options.json from ancestors directories.
    /// # Panics
    pub fn from_path(path: &Path) -> Self {
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
        self.plugins.iter().find_map(|v| match v {
            Value::String(s) if s == name => Some(None),
            Value::Array(a) if a.first().and_then(Value::as_str).is_some_and(|s| s == name) => {
                Some(a.get(1).cloned())
            }
            _ => None,
        })
    }
}
