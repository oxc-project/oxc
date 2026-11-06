use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct SetStateInEffect;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows calling `setState` synchronously inside an effect body.
    ///
    upstream = "set-state-in-effect",
    ///
    /// ### Why is this bad?
    ///
    /// Calling `setState` synchronously in an effect triggers an immediate
    /// extra render pass and usually indicates non-local derived data, a
    /// derived-event pattern, or improper external-data synchronization.
    /// Values that can be computed from props and state should be computed
    /// during render instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useEffect, useState } from 'react';
    /// function Component() {
    ///   const [state, setState] = useState(0);
    ///   useEffect(() => {
    ///     setState(s => s + 1);
    ///   });
    ///   return state;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component({ value }) {
    ///   const doubled = value * 2;
    ///   return <div>{doubled}</div>;
    /// }
    /// ```
    SetStateInEffect,
    react,
    correctness,
    recommended,
    version = "1.79.0",
    short_description = "Disallow calling `setState` synchronously inside an effect.",
);

impl Rule for SetStateInEffect {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::EffectSetState);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Effects that synchronize with external systems are fine.
        "
import {useEffect, useState} from 'react';
function Component({onChange}) {
  const [state, setState] = useState(0);
  useEffect(() => {
    onChange(state);
  }, [onChange, state]);
  return <div onClick={() => setState(state + 1)}>{state}</div>;
}
",
    ];

    let fail = vec![
        // Derived from `crates/oxc_react_compiler/fixtures` setState-in-effect cases.
        "
import {useEffect, useState} from 'react';
function Component() {
  const [state, setState] = useState(0);
  useEffect(() => {
    setState(s => s + 1);
  });
  return state;
}
",
        // An `exhaustive-deps` suppression on the effect does not hide this rule (oxc#26277)
        "
import {useEffect, useState} from 'react';
function Form({item}) {
  const [name, setName] = useState('');
  useEffect(() => {
    if (!item) return;
    setName(item.name);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [item?.id]);
  return <input value={name} onChange={(e) => setName(e.target.value)} />;
}
",
    ];

    Tester::new(SetStateInEffect::NAME, SetStateInEffect::PLUGIN, pass, fail).test_and_snapshot();
}
