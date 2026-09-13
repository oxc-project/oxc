use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Refs;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates correct usage of refs: `ref.current` may not be read or
    /// written during render, only in event handlers and effects.
    ///
    upstream = "refs",
    ///
    /// ### Why is this bad?
    ///
    /// React may not have attached the ref yet during render, and reading it
    /// does not subscribe the component to updates — the UI silently goes
    /// stale.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useRef } from 'react';
    /// function Component() {
    ///   const ref = useRef(null);
    ///   const value = ref.current; // read during render
    ///   return <div>{value}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useEffect, useRef } from 'react';
    /// function Component() {
    ///   const ref = useRef(null);
    ///   useEffect(() => {
    ///     ref.current.focus();
    ///   }, []);
    ///   return <input ref={ref} />;
    /// }
    /// ```
    Refs,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Validates correct usage of refs: not reading or writing `ref.current` during render.",
);

impl Rule for Refs {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Refs);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ---- ReactCompilerRuleTypescript-test.ts ----
        // Repro for hooks as normal values
        "
function Button(props) {
  const scrollview = React.useRef<ScrollView>(null);
  return <Button thing={scrollview} />;
}
",
    ];

    let fail = vec![
        // ---- NoRefAccessInRender-tests.ts ----
        // validate against simple ref access in render
        "
function Component(props) {
  const ref = useRef(null);
  const value = ref.current;
  return value;
}
",
        // Updating a ref should distinguish the ref object from its current value.
        "
function Component() {
  const ref = useRef(null);
  ref.current = 1;
  return null;
}
",
        // Nested writes should retain the base ref location through ref.current.
        "
function Component() {
  const ref = useRef({inner: null});
  ref.current.inner = 1;
  return null;
}
",
    ];

    Tester::new(Refs::NAME, Refs::PLUGIN, pass, fail).test_and_snapshot();
}
