use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct UseMemo;

declare_oxc_lint!(
    /// ### What it does
    /// Validates usage of manual memoization.
    UseMemo,
    react_compiler,
    correctness,
);

impl Rule for UseMemo {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::UseMemo);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Valid: useMemo with proper callback (no parameters)
        r#"
        import {useMemo} from 'react';
        function Component(props) {
          const x = useMemo(() => props.a + props.b, [props.a, props.b]);
          return <div>{x}</div>;
        }
        "#,
        // Cross-category: conditional hook triggers Hooks, not UseMemo
        r#"
        function useConditional() {
          if (cond) {
            useConditionalHook();
          }
        }
        "#,
    ];
    let fail = vec![
        // useMemo callback should not accept parameters
        r#"
        import {useMemo} from 'react';
        function Component({value}) {
          const result = useMemo((x) => x + 1, [value]);
          return <div>{result}</div>;
        }
        "#,
    ];
    Tester::new(UseMemo::NAME, UseMemo::PLUGIN, pass, fail).test_and_snapshot();
}
