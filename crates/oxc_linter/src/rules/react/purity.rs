use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Purity;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates that components and hooks are pure by checking that they do
    /// not call known-impure functions such as `Math.random()`, `Date.now()`,
    /// or `performance.now()` during render.
    ///
    upstream = "purity",
    ///
    /// ### Why is this bad?
    ///
    /// Impure renders return different output for the same props and state,
    /// breaking memoization, concurrent rendering, and replayability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component() {
    ///   const rand = Math.random();
    ///   return <div>{rand}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useState } from 'react';
    /// function Component() {
    ///   const [rand, setRand] = useState(0);
    ///   return <button onClick={() => setRand(Math.random())}>{rand}</button>;
    /// }
    /// ```
    Purity,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Validates that components and hooks do not call known-impure functions.",
);

impl Rule for Purity {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Purity);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ---- RustBackend-test.ts ----
        // Basic component compiles without errors
        "
function Component(props) {
  return <div>{props.text}</div>;
}
",
    ];

    let fail = vec![
        // ---- ImpureFunctionCallsRule-test.ts ----
        // Known impure function calls are caught
        "
function Component() {
  const date = Date.now();
  const now = performance.now();
  const rand = Math.random();
  return <Foo date={date} now={now} rand={rand} />;
}
",
    ];

    Tester::new(Purity::NAME, Purity::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn skips_node_modules() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![(
        "function Component() { return <div>{Date.now()}</div>; }",
        None,
        None,
        Some(PathBuf::from("node_modules/package/Component.tsx")),
    )];

    Tester::new(Purity::NAME, Purity::PLUGIN, pass, vec![]).test();
}
