use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct MemoDependencies;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates that `useMemo()` and `useCallback()` declare comprehensive
    /// dependency lists without extraneous values.
    ///
    unlinked_upstream = "memo-dependencies",
    ///
    /// ### Why is this bad?
    ///
    /// Missing dependencies produce stale memoized values; extraneous ones
    /// cause unnecessary recomputation.
    MemoDependencies,
    react,
    suspicious,
    version = "1.79.0",
    short_description = "Validates that `useMemo()` and `useCallback()` dependencies are comprehensive, without extraneous values.",
);

impl Rule for MemoDependencies {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::MemoDependencies);
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
import {useMemo} from 'react';
function Component({value}) {
  return useMemo(() => value, [value]);
}
",
    ];

    let fail = vec![
        // Missing dependency.
        "
import {useMemo} from 'react';
function Component({value}) {
  return useMemo(() => value, []);
}
",
        // Extraneous dependency.
        "
import {useMemo} from 'react';
function Component({value, extra}) {
  return useMemo(() => value, [value, extra]);
}
",
    ];

    Tester::new(MemoDependencies::NAME, MemoDependencies::PLUGIN, pass, fail).test_and_snapshot();
}
