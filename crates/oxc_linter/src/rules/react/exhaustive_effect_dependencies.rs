use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct ExhaustiveEffectDependencies;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates that effect dependency arrays are exhaustive and contain no
    /// extraneous values.
    ///
    unlinked_upstream = "exhaustive-effect-dependencies",
    ///
    /// ### Why is this bad?
    ///
    /// Missing effect dependencies capture stale values from a previous
    /// render; extraneous dependencies re-fire the effect needlessly.
    ExhaustiveEffectDependencies,
    react,
    suspicious,
    version = "1.79.0",
    short_description = "Validates that effect dependencies are exhaustive, without extraneous values.",
);

impl Rule for ExhaustiveEffectDependencies {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::EffectExhaustiveDependencies);
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
function Component(props) {
  return <div>{props.text}</div>;
}
",
        "
import {useEffect} from 'react';
function Component({value}) {
  useEffect(() => {
    log(value);
  }, [value]);
}
",
    ];

    let fail = vec![
        // Missing dependency.
        "
import {useEffect} from 'react';
function Component({value}) {
  useEffect(() => {
    log(value);
  }, []);
}
",
        // Extraneous dependency.
        "
import {useEffect} from 'react';
function Component({value, extra}) {
  useEffect(() => {
    log(value);
  }, [value, extra]);
}
",
    ];

    Tester::new(
        ExhaustiveEffectDependencies::NAME,
        ExhaustiveEffectDependencies::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
