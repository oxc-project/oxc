use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct PreserveManualMemoization;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates that existing manual memoization (`useMemo`, `useCallback`,
    /// `React.memo`) is preserved by the React Compiler: the compiler only
    /// compiles code whose inferred dependencies match or exceed the manually
    /// specified ones.
    ///
    upstream = "preserve-manual-memoization",
    ///
    /// ### Why is this bad?
    ///
    /// When the compiler cannot prove that existing manual memoization is
    /// preserved, it skips optimizing that code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useCallback } from 'react';
    /// function useFoo(props) {
    ///   const values = [];
    ///   values.push(props);
    ///   return useCallback(() => values, [values]);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useMemo } from 'react';
    /// function Component({ propA }) {
    ///   return useMemo(() => propA.x, [propA]);
    /// }
    /// ```
    PreserveManualMemoization,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Validates that existing manual memoization is preserved by the React Compiler.",
);

impl Rule for PreserveManualMemoization {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::PreserveManualMemo);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
import {useMemo} from 'react';
function Component({propA}) {
  return useMemo(() => propA.x, [propA]);
}
",
    ];

    let fail = vec![
        // Identify the manual memoization without highlighting the callback body.
        "
import {useCallback} from 'react';
function useFoo(props) {
  const x = [];
  useHook();
  x.push(props);
  return useCallback(() => {
    doSomething();
    doSomethingElse();
    return [x];
  }, [x]);
}
",
    ];

    Tester::new(PreserveManualMemoization::NAME, PreserveManualMemoization::PLUGIN, pass, fail)
        .test_and_snapshot();
}
