use lazy_regex::Regex;
use oxc_react_compiler::ErrorCategory;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{
        ReactCompilerEnvOptions, deserialize_regex_option, run_react_compiler_rule,
        should_run_react_compiler,
    },
};

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CapitalizedCallsConfig {
    /// Exact names of capitalized functions that may be called directly.
    /// Forwarded to the React Compiler's `validateNoCapitalizedCalls`
    /// environment option.
    allow: Vec<String>,
    /// A regex pattern; capitalized functions whose name matches may be called
    /// directly, checked alongside `allow`. Useful when a codebase has a
    /// naming convention for capitalized non-component factories, such as
    /// generated event or schema builders (e.g. `"Event$"`).
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
        run_react_compiler_rule(ctx, ErrorCategory::CapitalizedCalls);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

impl CapitalizedCalls {
    /// The environment additions this rule's options contribute to the shared
    /// React Compiler run.
    pub(crate) fn react_compiler_env_options(&self) -> ReactCompilerEnvOptions {
        ReactCompilerEnvOptions {
            capitalized_calls_allow: self.0.allow.clone(),
            capitalized_calls_allow_pattern: self.0.allow_pattern.clone(),
        }
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
        // Exact name allowed
        (
            "
import Child from './Child';
function Component() {
  return <>
    {Child()}
  </>;
}
",
            Some(json!([{ "allow": ["Child"] }])),
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
        // Like the compiler's validateNoCapitalizedCalls, the allowlist only
        // applies to global loads, not to method calls
        (
            "
import myModule from './MyModule';
function Component() {
  return <>
    {myModule.Child()}
  </>;
}
",
            Some(json!([{ "allow": ["Child"], "allowPattern": "^Child$" }])),
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
            Some(json!([{ "allow": ["Child1"] }])),
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
