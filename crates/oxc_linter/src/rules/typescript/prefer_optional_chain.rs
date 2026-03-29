use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferOptionalChain(Box<PreferOptionalChainConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferOptionalChainConfig {
    /// Allow autofixers that will change the return type of the expression.
    /// This option is considered unsafe as it may break the build.
    pub allow_potentially_unsafe_fixes_that_modify_the_return_type_i_know_what_im_doing: bool,

    /// Check operands that are typed as `any` when inspecting "loose boolean" operands.
    pub check_any: bool,

    /// Check operands that are typed as `bigint` when inspecting "loose boolean" operands.
    pub check_big_int: bool,

    /// Check operands that are typed as `boolean` when inspecting "loose boolean" operands.
    pub check_boolean: bool,

    /// Check operands that are typed as `number` when inspecting "loose boolean" operands.
    pub check_number: bool,

    /// Check operands that are typed as `string` when inspecting "loose boolean" operands.
    pub check_string: bool,

    /// Check operands that are typed as `unknown` when inspecting "loose boolean" operands.
    pub check_unknown: bool,

    /// Skip operands that are not typed with `null` and/or `undefined` when inspecting
    /// "loose boolean" operands.
    pub require_nullish: bool,
}

impl Default for PreferOptionalChainConfig {
    fn default() -> Self {
        Self {
            allow_potentially_unsafe_fixes_that_modify_the_return_type_i_know_what_im_doing: false,
            check_any: true,
            check_big_int: true,
            check_boolean: true,
            check_number: true,
            check_string: true,
            check_unknown: true,
            require_nullish: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using concise optional chain expressions instead of chained logical AND
    /// operators, negated logical OR operators, or empty objects.
    ///
    /// Note that this rule is in the nursery category while we ensure it is working
    /// correctly in as many edge-case scenarios as possible. The logic for this is
    /// complex and the autofix may cause logic changes in some edge-cases.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript 3.7 introduced optional chaining (`?.`) which provides a more concise
    /// and readable way to access properties on potentially nullish values. Using optional
    /// chaining instead of logical AND chains (`&&`) or other patterns improves code clarity.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// foo && foo.bar;
    /// foo && foo.bar && foo.bar.baz;
    /// foo && foo['bar'];
    /// foo && foo.bar && foo.bar.baz && foo.bar.baz.buzz;
    /// foo && foo.bar && foo.bar.baz.buzz;
    /// foo && foo.bar.baz && foo.bar.baz.buzz;
    /// (foo || {}).bar;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// foo?.bar;
    /// foo?.bar?.baz;
    /// foo?.['bar'];
    /// foo?.bar?.baz?.buzz;
    /// foo?.bar?.baz.buzz;
    /// foo?.bar.baz?.buzz;
    /// foo?.bar;
    /// ```
    PreferOptionalChain(tsgolint),
    typescript,
    nursery, // move to style after we've confirmed this works correctly on as many edge-cases as possible.
    dangerous_fix_suggestion,
    config = PreferOptionalChainConfig,
);

impl Rule for PreferOptionalChain {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
