use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Immutability;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows mutating props, state, hook arguments, hook return values,
    /// and other values that are immutable by the Rules of React.
    ///
    upstream = "immutability",
    ///
    /// ### Why is this bad?
    ///
    /// React relies on immutability to know when to re-render; mutating these
    /// values causes stale UI and lost updates.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useState } from 'react';
    /// function Component() {
    ///   const [state] = useState({ a: 0 });
    ///   state.a = 1; // mutates state directly
    ///   return <div>{state.a}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useState } from 'react';
    /// function Component() {
    ///   const [state, setState] = useState({ a: 0 });
    ///   return <div onClick={() => setState({ a: state.a + 1 })}>{state.a}</div>;
    /// }
    /// ```
    Immutability,
    react,
    correctness,
    version = "1.79.0",
    short_description = "Disallow mutating props, state, and other values that are immutable by the Rules of React.",
);

impl Rule for Immutability {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Immutability);
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
import {useState} from 'react';
function Component(props) {
  const [state, setState] = useState({a: 0});
  return <div onClick={() => setState({a: state.a + 1})}>{props.foo}</div>;
}
",
    ];

    let fail = vec![
        // ---- ReactCompilerRuleTypescript-test.ts ----
        // Mutating useState value
        "
        import { useState } from 'react';
        function Component(props) {
          // typescript syntax that hermes-parser doesn't understand yet
          const x: `foo${1}` = 'foo1';
          const [state, setState] = useState({a: 0});
          state.a = 1;
          return <div>{props.foo}</div>;
        }
      ",
        // A recursive callback should point at the access, not the entire
        // callback initializer.
        "
        import { useCallback } from 'react';
        function Component() {
          const applyFilter = useCallback(() => {
            setTimeout(() => {
              applyFilter();
            });
          }, []);
          return null;
        }
      ",
        // Mutating an argument should recommend updating its owner rather than
        // suggesting a local copy that would not update the UI.
        "
        function Component({model}) {
          model.value = 'next';
          return <div>{model.value}</div>;
        }
      ",
    ];

    Tester::new(Immutability::NAME, Immutability::PLUGIN, pass, fail).test_and_snapshot();
}
