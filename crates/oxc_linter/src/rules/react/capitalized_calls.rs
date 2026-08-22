use lazy_regex::Regex;
use oxc_react_compiler::{ErrorCategory, LintDiagnostic};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{
        deserialize_regex_option, run_react_compiler_rule_filtered, should_run_react_compiler,
    },
};

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CapitalizedCallsConfig {
    /// A regex pattern; capitalized functions and methods whose name matches
    /// may be called directly. Anchor the pattern to allow exact names, e.g.
    /// `"^(StyleSheet|Schema)$"`.
    #[serde(deserialize_with = "deserialize_regex_option")]
    allow_pattern: Option<Regex>,
}

#[derive(Debug, Default, Clone)]
pub struct CapitalizedCalls(Box<CapitalizedCallsConfig>);

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows calling capitalized functions or methods directly during
    /// render instead of rendering them with JSX, since capitalized names are
    /// reserved for components.
    ///
    unlinked_upstream = "capitalized-calls",
    ///
    /// ### Why is this bad?
    ///
    /// Calling a component as a plain function hides it from React: it gets
    /// no state isolation and no hooks context of its own, and it breaks
    /// memoization.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import Child from './Child';
    /// function Component() {
    ///   return <div>{Child()}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import Child from './Child';
    /// function Component() {
    ///   return <div><Child /></div>;
    /// }
    /// ```
    CapitalizedCalls,
    react,
    suspicious,
    config = CapitalizedCallsConfig,
    version = "1.79.0",
    short_description = "Disallow calling capitalized functions and methods instead of using JSX.",
);

impl Rule for CapitalizedCalls {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<CapitalizedCallsConfig>::from_value(value)
            .map(|config| Self(Box::new(config.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule_filtered(ctx, ErrorCategory::CapitalizedCalls, |finding| {
            !self.is_allowed(finding, ctx)
        });
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

impl CapitalizedCalls {
    fn is_allowed(&self, finding: &LintDiagnostic, ctx: &LintContext) -> bool {
        let Some(pattern) = self.0.allow_pattern.as_ref() else {
            return false;
        };
        // The finding's label covers exactly the offending function or method
        // name (see `diagnostics::capitalized_call`).
        let Some(label) = finding.diagnostic.labels.first() else {
            return false;
        };
        pattern.is_match(ctx.source_range(label.span()))
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
function Component(props) {
  return <div>{props.text}</div>;
}
",
            None,
        ),
        // Anchored pattern allows an exact name
        (
            "
import Child from './Child';
function Component() {
  return <>
    {Child()}
  </>;
}
",
            Some(json!([{ "allowPattern": "^Child$" }])),
        ),
        // The pattern applies to method calls too
        (
            "
import myModule from './MyModule';
function Component() {
  return <>
    {myModule.Child()}
  </>;
}
",
            Some(json!([{ "allowPattern": "^Child$" }])),
        ),
        // Pattern allowed
        (
            "
import { ButtonClickEvent } from './events';
function Component() {
  const event = ButtonClickEvent();
  return <button onClick={() => log(event)} />;
}
",
            Some(json!([{ "allowPattern": "Event$" }])),
        ),
    ];

    let fail = vec![
        // ---- NoCapitalizedCallsRule-test.ts ----
        // Simple violation
        (
            "
import Child from './Child';
function Component() {
  return <>
    {Child()}
  </>;
}
",
            None,
        ),
        // Method call violation
        (
            "
import myModule from './MyModule';
function Component() {
  return <>
    {myModule.Child()}
  </>;
}
",
            None,
        ),
        // Multiple diagnostics within the same function are surfaced
        (
            "
import Child1 from './Child1';
import MyModule from './MyModule';
function Component() {
  return <>
    {Child1()}
    {MyModule.Child2()}
  </>;
}",
            None,
        ),
        // The pattern-allowed pass case above is a violation without config
        (
            "
import { ButtonClickEvent } from './events';
function Component() {
  const event = ButtonClickEvent();
  return <button onClick={() => log(event)} />;
}
",
            None,
        ),
        // Allowing one name does not allow others
        (
            "
import Child1 from './Child1';
import Child2 from './Child2';
function Component() {
  return <>
    {Child1()}
    {Child2()}
  </>;
}",
            Some(json!([{ "allowPattern": "^Child1$" }])),
        ),
        // Pattern matches the whole name, not the suffix convention
        (
            "
import { EventChild } from './EventChild';
function Component() {
  return <>
    {EventChild()}
  </>;
}",
            Some(json!([{ "allowPattern": "Event$" }])),
        ),
    ];

    Tester::new(CapitalizedCalls::NAME, CapitalizedCalls::PLUGIN, pass, fail).test_and_snapshot();
}
