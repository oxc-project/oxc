use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

use super::react_compiler_rule::ReactCompilerConfig;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Fire(Box<ReactCompilerConfig>);

impl std::ops::Deref for Fire {
    type Target = ReactCompilerConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Validates usage of `fire`
    Fire,
    react_compiler,
    nursery,
    config = ReactCompilerConfig,
    version = "next",
);

impl Rule for Fire {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<ReactCompilerConfig>>(value)
            .map(|c| Self(Box::new(c.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(ctx, &self.0);
        super::cache::report_for_category(ctx, ErrorCategory::Fire);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    // Rule config that enables the `fire` transform.
    let rule_config = Some(json!([{ "environment": { "enableFire": true } }]));

    let pass = vec![
        // Derived from: transform-fire/basic.js
        // A valid fire() call: single callback invocation, no extra arguments.
        (
            r#"
            import { fire } from 'react';
            function Component(props) {
              const foo = props => {
                console.log(props);
              };
              useEffect(() => {
                fire(foo(props));
              });
              return null;
            }
            "#,
            rule_config.clone(),
        ),
        // A component that does not use fire() at all
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
        // Derived from: transform-fire/error.invalid-multiple-args.js
        // fire() is called with extra arguments beyond the single callback invocation,
        // which is invalid — reports a Fire error.
        (
            r#"
            import { fire } from 'react';
            function Component({ bar, baz }) {
              const foo = () => {
                console.log(bar, baz);
              };
              useEffect(() => {
                fire(foo(bar), baz);
              });
              return null;
            }
            "#,
            rule_config.clone(),
        ),
    ];

    Tester::new(Fire::NAME, Fire::PLUGIN, pass, fail).test_and_snapshot();
}
