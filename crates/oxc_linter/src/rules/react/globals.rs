use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Globals;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows assigning to or mutating variables declared outside a
    /// component or hook during render; side effects must run outside of
    /// render.
    ///
    upstream = "globals",
    ///
    /// ### Why is this bad?
    ///
    /// Components must be pure so React can render them at any time and in
    /// any order. Writing to a global during render makes the output depend
    /// on how often the component has rendered, and breaks under Strict Mode
    /// and concurrent rendering.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// let someGlobal = false;
    /// function Component() {
    ///   someGlobal = true; // assignment during render
    ///   return <div>{String(someGlobal)}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useEffect } from 'react';
    /// let someGlobal = false;
    /// function Component() {
    ///   useEffect(() => {
    ///     someGlobal = true;
    ///   }, []);
    ///   return <div />;
    /// }
    /// ```
    Globals,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Disallow assigning to or mutating globals during render.",
);

impl Rule for Globals {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Globals);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Globals may be reassigned in effects.
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
import {useEffect} from 'react';
let someGlobal = false;
function Component() {
  useEffect(() => {
    someGlobal = true;
  }, []);
  return <div />;
}
",
        // Not a component/hook and not a wrapper: the compiler prefilter must
        // skip this file without hiding a true positive elsewhere.
        "
let someGlobal = false;
function helper() {
  someGlobal = true;
  return someGlobal;
}
",
    ];

    let fail = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
let someGlobal = false;
function Component() {
  const setGlobal = () => {
    someGlobal = true;
  };
  setGlobal();
  return <div>{String(someGlobal)}</div>;
}
",
        // Hook named `useX` — a prefilter that only looked for JSX would miss this.
        "
let someGlobal = false;
function useFoo() {
  someGlobal = true;
  return useState(0);
}
",
        // Anonymous component via React.memo — a name-only skip would miss this.
        "
let someGlobal = false;
const Component = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});
",
    ];

    Tester::new(Globals::NAME, Globals::PLUGIN, pass, fail).test_and_snapshot();
}
