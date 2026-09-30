use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct VoidUseMemo;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates that `useMemo()` callbacks return a value and that the
    /// memoized result is actually used by the component or hook.
    ///
    unlinked_upstream = "void-use-memo",
    ///
    /// ### Why is this bad?
    ///
    /// A `useMemo` callback that returns nothing, or whose result is never
    /// used, is not memoizing anything — it is usually a side effect in
    /// disguise, which belongs in an event handler or effect instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useMemo } from 'react';
    /// function Component({ a }) {
    ///   useMemo(() => {
    ///     console.log(a); // returns nothing, result unused
    ///   }, [a]);
    ///   return <div>{a}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useMemo } from 'react';
    /// function Component({ a }) {
    ///   const x = useMemo(() => a + 1, [a]);
    ///   return <div>{x}</div>;
    /// }
    /// ```
    VoidUseMemo,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Validates that `useMemo()` callbacks return a value and the result is used.",
);

impl Rule for VoidUseMemo {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::VoidUseMemo);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
import {useMemo} from 'react';
function Component({a}) {
  const x = useMemo(() => a + 1, [a]);
  return <div>{x}</div>;
}
",
    ];

    let fail = vec![
        "
import {useMemo} from 'react';
function Component() {
  const value = useMemo(() => {}, []);
  return <div>{value}</div>;
}",
    ];

    Tester::new(VoidUseMemo::NAME, VoidUseMemo::PLUGIN, pass, fail).test_and_snapshot();
}
