use oxc_span::CompactStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Controls how hoisting is handled when checking for shadowing.
#[derive(Debug, Clone, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HoistOption {
    /// Report shadowing even before the outer variable is declared (due to hoisting).
    All,
    /// Only report shadowing for function declarations that are hoisted.
    #[default]
    Functions,
    /// Never report shadowing before the outer variable is declared.
    Never,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoShadowConfig {
    /// Controls how hoisting is handled.
    #[serde(default)]
    pub hoist: HoistOption,

    /// List of variable names that are allowed to shadow.
    #[serde(default)]
    pub allow: Vec<CompactStr>,

    /// If `true`, ignore when a type and a value have the same name.
    /// This is common in TypeScript: `type Foo = ...; const Foo = ...;`
    #[serde(default = "default_true")]
    pub ignore_type_value_shadow: bool,

    /// If `true`, ignore when a function type parameter shadows a value.
    /// Example: `const T = 1; function foo<T>() {}`
    #[serde(default = "default_true")]
    pub ignore_function_type_parameter_name_value_shadow: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NoShadowConfig {
    fn default() -> Self {
        Self {
            hoist: HoistOption::default(),
            allow: Vec::new(),
            ignore_type_value_shadow: true,
            ignore_function_type_parameter_name_value_shadow: true,
        }
    }
}
