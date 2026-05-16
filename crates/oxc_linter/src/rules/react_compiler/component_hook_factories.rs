use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct ComponentHookFactories;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against higher order functions defining nested components or hooks. Components and hooks should be defined at the module level
    ComponentHookFactories,
    react_compiler,
    correctness,
    version = "next",
);

impl Rule for ComponentHookFactories {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(
            ctx,
            ErrorCategory::Factories,
            <Self as crate::rule::RuleMeta>::NAME,
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Normal component at module scope — no nested component/hook
        r"
        function Component(props) {
          return <div>{props.value}</div>;
        }
        ",
        // A helper function that returns a plain object, not a component
        r"
        function getConfig(a) {
          return { value: a };
        }
        ",
        // A hook defined at module scope — not nested inside another function
        r"
        function useMyHook(a) {
          const [state, setState] = useState(a);
          return state;
        }
        ",
    ];

    let fail = vec![
        // Derived from: error.nested-component-in-normal-function.js
        // A non-component, non-hook function that defines a nested component inside.
        r"
        export function getInput(a) {
          const Wrapper = () => {
            const handleChange = () => {
              a.onChange();
            };

            return <input onChange={handleChange} />;
          };

          return Wrapper;
        }
        ",
    ];

    Tester::new(ComponentHookFactories::NAME, ComponentHookFactories::PLUGIN, pass, fail)
        .test_and_snapshot();
}
