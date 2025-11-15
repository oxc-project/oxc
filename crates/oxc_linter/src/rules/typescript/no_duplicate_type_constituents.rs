use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateTypeConstituents(Box<NoDuplicateTypeConstituentsConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NoDuplicateTypeConstituentsConfig {
    /// Whether to ignore duplicate types in intersection types.
    /// When true, allows `type T = A & A`.
    pub ignore_intersections: bool,
    /// Whether to ignore duplicate types in union types.
    /// When true, allows `type T = A | A`.
    pub ignore_unions: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows duplicate constituents of union or intersection types.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate constituents in union and intersection types serve no purpose and can make code harder to read. They are likely a mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type T1 = 'A' | 'A';
    ///
    /// type T2 = A | A | B;
    ///
    /// type T3 = { a: string } & { a: string };
    ///
    /// type T4 = [A, A];
    ///
    /// type T5 =
    ///   | 'foo'
    ///   | 'bar'
    ///   | 'foo';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type T1 = 'A' | 'B';
    ///
    /// type T2 = A | B | C;
    ///
    /// type T3 = { a: string } & { b: string };
    ///
    /// type T4 = [A, B];
    ///
    /// type T5 =
    ///   | 'foo'
    ///   | 'bar'
    ///   | 'baz';
    /// ```
    NoDuplicateTypeConstituents(tsgolint),
    typescript,
    correctness,
    pending,
    config = NoDuplicateTypeConstituentsConfig,
);

impl Rule for NoDuplicateTypeConstituents {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<NoDuplicateTypeConstituentsConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
