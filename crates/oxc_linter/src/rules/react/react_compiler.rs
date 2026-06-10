use std::borrow::Cow;

use oxc_diagnostics::Severity;
use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::{PluginOptions, default_plugin_options};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ReactCompiler(Box<ReactCompilerConfig>);

impl std::ops::Deref for ReactCompiler {
    type Target = ReactCompilerConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactCompilerConfig {
    /// Also report compiler bail-outs — places where React Compiler skipped a
    /// component or hook (for example because of unsupported syntax) without
    /// finding a rule violation. These do not indicate incorrect code, only
    /// code that the compiler declined to optimize.
    report_all_bailouts: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Runs the React Compiler's analysis in lint-only mode and reports code
    /// that violates the Rules of React — for example calling hooks
    /// conditionally, calling setState during render, accessing refs during
    /// render, or mutating props and state.
    ///
    /// This rule surfaces the same diagnostics as `eslint-plugin-react-compiler`.
    ///
    /// ### Why is this bad?
    ///
    /// Code that breaks the Rules of React can behave unpredictably at runtime
    /// (stale UI, infinite re-render loops, lost state) and prevents React
    /// Compiler from optimizing the component. Following these rules keeps
    /// components correct and allows them to be automatically memoized.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   if (props.cond) {
    ///     useState(0); // hooks may not be called conditionally
    ///   }
    ///   return <div>{props.text}</div>;
    /// }
    /// ```
    ///
    /// ```jsx
    /// function Component() {
    ///   const ref = useRef(null);
    ///   return <div>{ref.current}</div>; // refs may not be read during render
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   const [state, setState] = useState(0);
    ///   return <button onClick={() => setState(state + 1)}>{props.text}</button>;
    /// }
    /// ```
    ReactCompiler,
    react,
    nursery,
    config = ReactCompilerConfig,
    version = "1.70.0",
);

impl Rule for ReactCompiler {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let program = ctx.nodes().program();
        let options = PluginOptions {
            filename: ctx.file_path().to_str().map(ToString::to_string),
            ..default_plugin_options()
        };

        let result = oxc_react_compiler::lint(program, options);

        let diagnostics = result.diagnostics.into_vec();
        let diagnostics = if self.report_all_bailouts {
            diagnostics
        } else {
            diagnostics
                .into_iter()
                .filter(|diagnostic| diagnostic.severity == Severity::Error)
                .collect::<Vec<_>>()
        };

        for mut diagnostic in diagnostics {
            // The rule code already identifies the source, so the prefix is noise here.
            if let Some(message) = diagnostic.message.strip_prefix("[ReactCompiler] ") {
                diagnostic.message = Cow::Owned(message.to_string());
            }
            ctx.diagnostic(diagnostic);
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            "function Component(props) {
                return <div onClick={() => props.onClick()}>{props.text}</div>;
            }",
            None,
        ),
        (
            "function add(a, b) {
                return a + b;
            }",
            None,
        ),
        (
            "function Component(props) {
                const [state, setState] = useState(0);
                return <button onClick={() => setState(state + 1)}>{props.text}</button>;
            }",
            None,
        ),
        // A bail-out (local named `fbt`) is not a violation, only a skipped
        // optimization; it is not reported by default.
        (
            "function Component() {
                const fbt = 'span';
                return <fbt desc='label'>Hello</fbt>;
            }",
            None,
        ),
    ];

    let fail = vec![
        // Conditional hook call — Rules of Hooks violation.
        (
            "function Component(props) {
                if (props.cond) {
                    useState(0);
                }
                return <div>{props.text}</div>;
            }",
            None,
        ),
        // setState called during render.
        (
            "function Component() {
                const [state, setState] = useState(0);
                setState(1);
                return <div>{state}</div>;
            }",
            None,
        ),
        // Ref read during render.
        (
            "function Component() {
                const ref = useRef(null);
                const value = ref.current;
                return <div>{value}</div>;
            }",
            None,
        ),
        // Bail-outs are reported when `reportAllBailouts` is enabled.
        (
            "function Component() {
                const fbt = 'span';
                return <fbt desc='label'>Hello</fbt>;
            }",
            Some(json!([{ "reportAllBailouts": true }])),
        ),
    ];

    Tester::new(ReactCompiler::NAME, ReactCompiler::PLUGIN, pass, fail).test_and_snapshot();
}
