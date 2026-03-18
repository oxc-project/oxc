use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct Immutability;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against mutations of props, hook arguments, and hook return values.
    Immutability,
    react_compiler,
    correctness,
);

impl Rule for Immutability {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::Immutability);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Valid: no mutation of props or hook return values
        r"
        function Component(props) {
          const [state, setState] = useState(0);
          return <div onClick={() => setState(state + 1)}>{props.value}</div>;
        }
        ",
        // Cross-category: conditional hook triggers Hooks, not Immutability
        r"
        function useConditional() {
          if (cond) {
            useConditionalHook();
          }
        }
        ",
    ];
    let fail = vec![
        // Local reassignment in callback
        r"
        function Component(props) {
          let x = props.value;
          const handler = () => { x = 1; };
          return <div onClick={handler}>{x}</div>;
        }
        ",
    ];
    Tester::new(Immutability::NAME, Immutability::PLUGIN, pass, fail).test_and_snapshot();
}
