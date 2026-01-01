use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::TypeOrValueSpecifier,
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoDeprecated(Box<NoDeprecatedConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NoDeprecatedConfig {
    /// An array of type or value specifiers that are allowed to be used even if deprecated.
    /// Use this to allow specific deprecated APIs that you intentionally want to continue using.
    pub allow: Vec<TypeOrValueSpecifier>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using code marked as `@deprecated`.
    ///
    /// ### Why is this bad?
    ///
    /// The JSDoc `@deprecated` tag can be used to document some piece of code
    /// being deprecated. It's best to avoid using code marked as deprecated.
    /// This rule reports on any references to code marked as `@deprecated`.
    ///
    /// TypeScript recognizes the `@deprecated` tag, allowing editors to visually
    /// indicate deprecated code — usually with a strikethrough. However, TypeScript
    /// doesn't report type errors for deprecated code on its own.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// /** @deprecated Use apiV2 instead. */
    /// declare function apiV1(): Promise<string>;
    /// declare function apiV2(): Promise<string>;
    ///
    /// await apiV1(); // Using deprecated function
    ///
    /// import { parse } from 'node:url';
    /// // 'parse' is deprecated. Use the WHATWG URL API instead.
    /// const url = parse('/foo');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// /** @deprecated Use apiV2 instead. */
    /// declare function apiV1(): Promise<string>;
    /// declare function apiV2(): Promise<string>;
    ///
    /// await apiV2(); // Using non-deprecated function
    ///
    /// // Modern Node.js API, uses `new URL()`
    /// const url2 = new URL('/foo', 'http://www.example.com');
    /// ```
    NoDeprecated(tsgolint),
    typescript,
    pedantic,
    config = NoDeprecatedConfig,
);

impl Rule for NoDeprecated {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
