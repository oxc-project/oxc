use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct SetStateInEffect;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against setState in effect bodies.
    SetStateInEffect,
    react_compiler,
    correctness,
);

impl Rule for SetStateInEffect {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::EffectSetState);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Valid: setState in event handler is fine
        r#"
        function Component(props) {
          const [state, setState] = useState(0);
          return <div onClick={() => setState(1)}>{state}</div>;
        }
        "#,
        // Cross-category: conditional hook triggers Hooks, not SetStateInEffect
        r#"
        function useConditional() {
          if (cond) {
            useConditionalHook();
          }
        }
        "#,
    ];
    let fail = vec![
        // setState in useLayoutEffect
        r#"
        import { useState, useLayoutEffect } from 'react';
        function Component(props) {
          const [state, setState] = useState(props.initial);
          useLayoutEffect(() => {
            setState(derive(props.value));
          }, [props.value]);
          return <div>{state}</div>;
        }
        "#,
    ];
    Tester::new(SetStateInEffect::NAME, SetStateInEffect::PLUGIN, pass, fail).test_and_snapshot();
}
