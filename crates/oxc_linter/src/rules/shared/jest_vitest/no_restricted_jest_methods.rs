use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        object_with_nullable_string_schema,
    },
};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::rule::DefaultRuleConfig;

fn restricted_jest_method(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use of `{method_name}` is not allowed")).with_label(span)
}

fn restricted_jest_method_with_message(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string()).with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Restrict the use of specific `jest` and `vi` methods.

### Why is this bad?

Certain Jest or Vitest methods may be deprecated, discouraged in specific
contexts, or incompatible with your testing environment. Restricting
them helps maintain consistent and reliable test practices.

By default, no methods are restricted by this rule.
You must configure the rule for it to disable anything.

### Examples

Examples of **incorrect** code for this rule:
```javascript
jest.useFakeTimers();
it('calls the callback after 1 second via advanceTimersByTime', () => {
  // ...

  jest.advanceTimersByTime(1000);

  // ...
});

test('plays video', () => {
  const spy = jest.spyOn(video, 'play');

  // ...
});
```
";

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoRestrictedTestMethodsConfig {
    /// A mapping of restricted Jest method names to custom messages - or
    /// `null`, for a generic message.
    #[schemars(schema_with = "object_with_nullable_string_schema")]
    #[serde(flatten)]
    pub restricted_methods: FxHashMap<String, Option<CompactStr>>,
}

impl NoRestrictedTestMethodsConfig {
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value.clone())
            .map(DefaultRuleConfig::into_inner)
    }

    pub fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[
                JestFnKind::General(JestGeneralFnKind::Jest),
                JestFnKind::General(JestGeneralFnKind::Vitest),
            ],
        ) {
            return;
        }

        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        let Some(property_name) = mem_expr.static_property_name() else {
            return;
        };
        let Some((span, _)) = mem_expr.static_property_info() else {
            return;
        };

        if self.contains(property_name) {
            self.get_message(property_name).map_or_else(
                || {
                    ctx.diagnostic(restricted_jest_method(property_name, span));
                },
                |message| {
                    ctx.diagnostic(restricted_jest_method_with_message(&message, span));
                },
            );
        }
    }

    fn contains(&self, key: &str) -> bool {
        self.restricted_methods.contains_key(key)
    }

    fn get_message(&self, name: &str) -> Option<CompactStr> {
        self.restricted_methods.get(name).cloned().flatten()
    }
}
