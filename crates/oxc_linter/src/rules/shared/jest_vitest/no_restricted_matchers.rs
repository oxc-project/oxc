use std::path::Path;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, KnownMemberExpressionProperty, PossibleJestNode, is_type_of_jest_fn_call,
        parse_expect_jest_fn_call,
    },
};

fn restricted_chain(chain_call: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use of `{chain_call}` is disallowed")).with_label(span)
}

fn restricted_chain_with_message(chain_call: &str, message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use of `{chain_call}` is disallowed"))
        .with_help(message.to_string())
        .with_label(span)
}

pub const DOCUMENTATION: &str = r#"### What it does

Ban specific matchers & modifiers from being used, and can suggest alternatives.

### Why is this bad?

Some matchers or modifiers might be discouraged in your codebase for various reasons:
they might be deprecated, cause confusion, have performance implications, or there
might be better alternatives available. This rule allows you to enforce consistent
testing patterns by restricting certain Jest matchers and providing guidance on
preferred alternatives.

### Examples

Bans are expressed in the form of a map, with the value being either a string message to be shown,
or null if only the default rule message should be used. Bans are checked against the start of
the expect chain - this means that to ban a specific matcher entirely you must specify all
six permutations, but allows you to ban modifiers as well. By default, this map is empty, meaning
no matchers or modifiers are banned.

Example configuration:
```json
{
  "jest/no-restricted-matchers": [
    "error",
    {
      "toBeFalsy": null,
      "resolves": "Use `expect(await promise)` instead.",
      "toHaveBeenCalledWith": null,
      "not.toHaveBeenCalledWith": null,
      "resolves.toHaveBeenCalledWith": null,
      "rejects.toHaveBeenCalledWith": null,
      "resolves.not.toHaveBeenCalledWith": null,
      "rejects.not.toHaveBeenCalledWith": null
    }
  ]
}
```

Examples of **incorrect** code for this rule with the above configuration:
```javascript
it('is false', () => {
  // if this has a modifier (i.e. `not.toBeFalsy`), it would be considered fine
  expect(a).toBeFalsy();
});

it('resolves', async () => {
  // all uses of this modifier are disallowed, regardless of matcher
  await expect(myPromise()).resolves.toBe(true);
});

describe('when an error happens', () => {
  it('does not upload the file', async () => {
    // all uses of this matcher are disallowed
    expect(uploadFileMock).not.toHaveBeenCalledWith('file.name');
  });
});
```
"#;

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoRestrictedMatchersConfig {
    /// A map of restricted matchers/modifiers to custom messages.
    /// The key is the matcher/modifier name (e.g., "toBeFalsy", "resolves", "not.toHaveBeenCalledWith").
    /// The value is an optional custom message to display when the matcher/modifier is used.
    pub restricted_matchers: FxHashMap<String, String>,
}

const MODIFIER_NAME: [&str; 3] = ["not", "rejects", "resolves"];

impl NoRestrictedMatchersConfig {
    #[expect(clippy::unnecessary_wraps)]
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let restricted_matchers = value
            .get(0)
            .and_then(serde_json::Value::as_object)
            .map(Self::compile_restricted_matchers)
            .unwrap_or_default();

        Ok(NoRestrictedMatchersConfig { restricted_matchers })
    }

    pub fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_type_of_jest_fn_call(call_expr, possible_jest_node, ctx, &[JestFnKind::Expect]) {
            return;
        }

        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let members = &jest_fn_call.members;

        if members.is_empty() {
            return;
        }

        let chain_call = members
            .iter()
            .filter_map(KnownMemberExpressionProperty::name)
            .collect::<Vec<_>>()
            .join(".");

        let span = Span::new(members.first().unwrap().span.start, members.last().unwrap().span.end);

        for (restriction, message) in &self.restricted_matchers {
            if Self::check_restriction(chain_call.as_str(), restriction.as_str()) {
                if message.is_empty() {
                    ctx.diagnostic(restricted_chain(&chain_call, span));
                } else {
                    ctx.diagnostic(restricted_chain_with_message(&chain_call, message, span));
                }
            }
        }
    }

    fn check_restriction(chain_call: &str, restriction: &str) -> bool {
        if MODIFIER_NAME.contains(&restriction)
            || Path::new(restriction).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("not"))
        {
            chain_call.starts_with(restriction)
        } else {
            chain_call == restriction
        }
    }

    pub fn compile_restricted_matchers(
        matchers: &serde_json::Map<String, serde_json::Value>,
    ) -> FxHashMap<String, String> {
        matchers
            .iter()
            .map(|(key, value)| {
                (String::from(key), String::from(value.as_str().unwrap_or_default()))
            })
            .collect()
    }
}
