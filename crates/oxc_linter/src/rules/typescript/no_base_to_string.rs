use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoBaseToString(Box<NoBaseToStringConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoBaseToStringConfig {
    /// A list of type names to ignore when checking for unsafe toString usage.
    /// These types are considered safe to call toString on even if they don't
    /// provide a custom implementation.
    pub ignored_type_names: Vec<String>,
}

impl Default for NoBaseToStringConfig {
    fn default() -> Self {
        Self {
            ignored_type_names: vec![
                "Error".to_string(),
                "RegExp".to_string(),
                "URL".to_string(),
                "URLSearchParams".to_string(),
            ],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires toString() and toLocaleString() calls to only be called on objects which provide useful information when stringified.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript's toString() method returns '[object Object]' on plain objects, which is not useful information. This rule prevents toString() and toLocaleString() from being called on objects that return less useful strings.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // These will evaluate to '[object Object]'
    /// ({}).toString();
    /// ({foo: 'bar'}).toString();
    /// ({foo: 'bar'}).toLocaleString();
    ///
    /// // This will evaluate to 'Symbol()'
    /// Symbol('foo').toString();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const someString = 'Hello world';
    /// someString.toString();
    ///
    /// const someNumber = 42;
    /// someNumber.toString();
    ///
    /// const someBoolean = true;
    /// someBoolean.toString();
    ///
    /// class CustomToString {
    ///   toString() {
    ///     return 'CustomToString';
    ///   }
    /// }
    /// new CustomToString().toString();
    /// ```
    NoBaseToString(tsgolint),
    typescript,
    correctness,
    pending,
    config = NoBaseToStringConfig,
);

impl Rule for NoBaseToString {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoBaseToString>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
