use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

use super::react_compiler_rule::ReactCompilerConfig;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AutomaticEffectDependencies(Box<ReactCompilerConfig>);

impl std::ops::Deref for AutomaticEffectDependencies {
    type Target = ReactCompilerConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Verifies that automatic effect dependencies are compiled if opted-in
    AutomaticEffectDependencies,
    react_compiler,
    nursery,
    config = ReactCompilerConfig,
    version = "next",
);

impl Rule for AutomaticEffectDependencies {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<ReactCompilerConfig>>(value)
            .map(|c| Self(Box::new(c.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(ctx, &self.0);
        super::cache::report_for_category(ctx, ErrorCategory::AutomaticEffectDependencies);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    // Rule config that enables inferEffectDependencies for React's useEffect at index 1.
    // Matches the upstream ESLint plugin test harness configuration from TestUtils.ts.
    let rule_config = Some(json!([{
        "environment": {
            "inferEffectDependencies": [
                {
                    "function": {
                        "source": "react",
                        "importSpecifierName": "useEffect"
                    },
                    "autodepsIndex": 1
                }
            ]
        }
    }]));

    let pass = vec![
        // A component using a regular (non-AUTODEPS) deps array — compiles cleanly
        (
            r"
            import { useEffect } from 'react';
            function Component({ foo }) {
              useEffect(() => {
                console.log(foo);
              }, [foo]);
            }
            ",
            rule_config.clone(),
        ),
        // A component that does not use effects at all
        (
            r"
            function Component(props) {
              return <div>{props.value}</div>;
            }
            ",
            rule_config.clone(),
        ),
    ];

    let fail = vec![
        // Derived from: infer-effect-dependencies/error.wrong-index-no-func.js
        // useEffect is called with AUTODEPS as the only argument (no callback), so
        // the compiler cannot infer dependencies — reports AutomaticEffectDependencies.
        (
            r"
            import { useEffect, AUTODEPS } from 'react';
            function Component({ foo }) {
              useEffect(AUTODEPS);
            }
            ",
            rule_config.clone(),
        ),
    ];

    Tester::new(AutomaticEffectDependencies::NAME, AutomaticEffectDependencies::PLUGIN, pass, fail)
        .test_and_snapshot();
}
