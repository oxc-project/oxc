use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct NoDerivingStateInEffects;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows deriving values from state inside an effect and storing them
    /// back into state; derived values should be computed during render
    /// instead.
    ///
    unlinked_upstream = "no-deriving-state-in-effects",
    ///
    /// ### Why is this bad?
    ///
    /// Deriving state in effects causes a second render pass per update and
    /// lets the derived copy fall out of sync with its source.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { useEffect, useState } from 'react';
    /// function Component() {
    ///   const [firstName] = useState('Taylor');
    ///   const [lastName] = useState('Swift');
    ///   const [fullName, setFullName] = useState('');
    ///   useEffect(() => {
    ///     setFullName(firstName + ' ' + lastName);
    ///   }, [firstName, lastName]);
    ///   return <div>{fullName}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component({ firstName, lastName }) {
    ///   const fullName = firstName + ' ' + lastName;
    ///   return <div>{fullName}</div>;
    /// }
    /// ```
    NoDerivingStateInEffects,
    react,
    perf,
    version = "1.79.0",
    short_description = "Disallow deriving values from state in an effect instead of computing them during render.",
);

impl Rule for NoDerivingStateInEffects {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::EffectDerivationsOfState);
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
function Component({firstName, lastName}) {
  const fullName = firstName + ' ' + lastName;
  return <div>{fullName}</div>;
}
",
    ];

    let fail = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
import {useEffect, useState} from 'react';
function Component() {
  const [firstName] = useState('Taylor');
  const [lastName] = useState('Swift');
  const [fullName, setFullName] = useState('');
  useEffect(() => {
    setFullName(firstName + ' ' + lastName);
  }, [firstName, lastName]);
  return <div>{fullName}</div>;
}
",
    ];

    Tester::new(NoDerivingStateInEffects::NAME, NoDerivingStateInEffects::PLUGIN, pass, fail)
        .test_and_snapshot();
}
