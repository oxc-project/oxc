use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct UseMemo;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates usage of the `useMemo()` hook against common mistakes, such
    /// as passing an async or generator callback or misusing its arguments.
    ///
    upstream = "use-memo",
    ///
    /// ### Why is this bad?
    ///
    /// An async or generator callback makes `useMemo` memoize a promise or
    /// iterator instead of the intended value, and misused arguments prevent
    /// memoization from working at all.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useMemo } from 'react';
    /// function Component({ a }) {
    ///   const x = useMemo(async () => {
    ///     await a;
    ///   }, [a]);
    ///   return <div>{x}</div>;
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
    UseMemo,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Validates usage of the `useMemo()` hook against common mistakes.",
);

impl Rule for UseMemo {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::UseMemo);
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
        // Async callback.
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
import {useMemo} from 'react';
function Component({a}) {
  const x = useMemo(async () => {
    await a;
  }, [a]);
  return <div>{x}</div>;
}
",
    ];

    Tester::new(UseMemo::NAME, UseMemo::PLUGIN, pass, fail).test_and_snapshot();
}
