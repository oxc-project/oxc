use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::{LibFrom, LibSpecifier, NameSpecifier, TypeOrValueSpecifier},
};

fn default_restrict_template_allow() -> Vec<TypeOrValueSpecifier> {
    vec![TypeOrValueSpecifier::Lib(LibSpecifier {
        from: LibFrom::Lib,
        name: NameSpecifier::Multiple(vec![
            "Error".to_string(),
            "URL".to_string(),
            "URLSearchParams".to_string(),
        ]),
    })]
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RestrictTemplateExpressions(Box<RestrictTemplateExpressionsConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RestrictTemplateExpressionsConfig {
    /// Whether to allow `any` typed values in template expressions.
    pub allow_any: bool,
    /// Whether to allow array types in template expressions.
    pub allow_array: bool,
    /// Whether to allow boolean types in template expressions.
    pub allow_boolean: bool,
    /// Whether to allow nullish types (`null` or `undefined`) in template expressions.
    pub allow_nullish: bool,
    /// Whether to allow number and bigint types in template expressions.
    pub allow_number: bool,
    /// Whether to allow RegExp values in template expressions.
    pub allow_reg_exp: bool,
    /// Whether to allow `never` type in template expressions.
    pub allow_never: bool,
    /// An array of type or value specifiers for additional types that are allowed in template expressions.
    /// Defaults include Error, URL, and URLSearchParams from lib.
    #[serde(default = "default_restrict_template_allow")]
    pub allow: Vec<TypeOrValueSpecifier>,
}

impl Default for RestrictTemplateExpressionsConfig {
    fn default() -> Self {
        Self {
            allow_any: true,
            allow_array: false,
            allow_boolean: true,
            allow_nullish: true,
            allow_number: true,
            allow_reg_exp: true,
            allow_never: false,
            allow: default_restrict_template_allow(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule restricts the types allowed in template literal expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Template literals will call toString() on the interpolated values. Some types don't have meaningful string representations (like objects that become "[object Object]") or may not have a toString method at all. This rule helps ensure that only appropriate types are used in template expressions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const obj: object;
    /// declare const sym: symbol;
    /// declare const fn: () => void;
    /// declare const arr: unknown[];
    ///
    /// // Objects become "[object Object]"
    /// const str1 = `Value: ${obj}`;
    ///
    /// // Symbols might not be what you expect
    /// const str2 = `Symbol: ${sym}`;
    ///
    /// // Functions become their source code or "[Function]"
    /// const str3 = `Function: ${fn}`;
    ///
    /// // Arrays might not format as expected
    /// const str4 = `Array: ${arr}`;
    ///
    /// // undefined/null become "undefined"/"null" which might be confusing
    /// declare const maybeValue: string | undefined;
    /// const str5 = `Value: ${maybeValue}`; // Could be "Value: undefined"
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const str: string;
    /// declare const num: number;
    /// declare const bool: boolean;
    /// declare const obj: object;
    ///
    /// // Safe types
    /// const result1 = `String: ${str}`;
    /// const result2 = `Number: ${num}`;
    /// const result3 = `Boolean: ${bool}`;
    ///
    /// // Explicit conversions for complex types
    /// const result4 = `Object: ${JSON.stringify(obj)}`;
    /// const result5 = `Array: ${arr.join(', ')}`;
    ///
    /// // Handle undefined/null explicitly
    /// declare const maybeValue: string | undefined;
    /// const result6 = `Value: ${maybeValue ?? 'N/A'}`;
    /// const result7 = `Value: ${maybeValue || 'default'}`;
    ///
    /// // Type guards for unknown values
    /// declare const unknown: unknown;
    /// const result8 = typeof unknown === 'string' ? `Value: ${unknown}` : 'Invalid';
    /// ```
    RestrictTemplateExpressions(tsgolint),
    typescript,
    correctness,
    pending,
    config = RestrictTemplateExpressionsConfig,
);

impl Rule for RestrictTemplateExpressions {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<RestrictTemplateExpressions>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
