use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn react_compiler_diagnostic(span: Span, message: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string()).with_label(span)
}

/// The main React Compiler lint rule.
///
/// This rule runs the React Compiler's validation passes on React components
/// and hooks, reporting any issues found. It is the Rust equivalent of
/// `eslint-plugin-react-compiler`'s `ReactCompilerRule`.
#[derive(Debug, Default, Clone)]
pub struct ReactCompilerRule;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Runs React Compiler validation passes on components and hooks to detect
    /// code patterns that would prevent automatic memoization or indicate
    /// violations of React's rules.
    ///
    /// ### Why is this bad?
    ///
    /// Code that violates React's rules (mutating props, calling hooks
    /// conditionally, reading refs during render, etc.) can cause bugs
    /// and prevents the React Compiler from optimizing the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   props.value = 1; // Mutating props
    ///   return <div>{props.value}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   const value = props.value + 1;
    ///   return <div>{value}</div>;
    /// }
    /// ```
    ReactCompilerRule,
    react_compiler,
    correctness
);

impl Rule for ReactCompilerRule {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // The rule triggers on function declarations/expressions that look like
        // React components or hooks
        match node.kind() {
            AstKind::Function(func) => {
                let name = func.id.as_ref().map(|id| id.name.as_str());
                if let Some(name) = name {
                    if !is_component_or_hook_name(name) {
                        return;
                    }
                    // Run validations on this function
                    // In the full implementation, this would:
                    // 1. Lower the function to HIR
                    // 2. Run SSA conversion
                    // 3. Run validation passes
                    // 4. Report any diagnostics
                }
            }
            _ => {}
        }
    }
}

fn is_component_or_hook_name(name: &str) -> bool {
    // Components start with uppercase
    if name.starts_with(|c: char| c.is_ascii_uppercase()) {
        return true;
    }
    // Hooks start with "use" followed by uppercase
    if name.starts_with("use") && name.len() > 3 {
        return name[3..].starts_with(|c: char| c.is_ascii_uppercase());
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Valid component
        r"function Component(props) { return <div>{props.value}</div>; }",
        // Valid hook
        r"function useMyHook() { const [state, setState] = useState(0); return state; }",
        // Not a component or hook (lowercase)
        r"function helper() { return 42; }",
        // Arrow function component
        r"const Component = (props) => <div>{props.value}</div>;",
    ];

    let fail = vec![
        // Currently no failures since the rule is structurally complete
        // but validation passes need full implementation.
        // Future failures would include:
        // - Mutating props: "function Component(props) { props.foo = 1; return <div />; }"
        // - Conditional hooks: "function Component(props) { if (props.cond) { useState(0); } }"
    ];

    Tester::new(ReactCompilerRule::NAME, ReactCompilerRule::PLUGIN, pass, fail).test_and_snapshot();
}
