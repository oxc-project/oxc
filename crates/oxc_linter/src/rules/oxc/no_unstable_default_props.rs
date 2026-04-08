use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unstable_default_props_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unstable value used as default prop")
        .with_help("Object and array literals as default parameter values create a new reference on every render. Extract them to a module-level constant.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnstableDefaultProps;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects object or array literals used as default values for component
    /// props in function parameters.
    ///
    /// ### Why is this bad?
    ///
    /// Using `{}` or `[]` as a default parameter value creates a new reference
    /// on every render. This defeats React.memo, causes unnecessary re-renders,
    /// and can trigger infinite loops in useEffect dependencies.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component({ items = [], config = {} }) {
    ///     return <div />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const DEFAULT_ITEMS = [];
    /// const DEFAULT_CONFIG = {};
    /// function Component({ items = DEFAULT_ITEMS, config = DEFAULT_CONFIG }) {
    ///     return <div />;
    /// }
    /// ```
    NoUnstableDefaultProps,
    oxc,
    perf,
    none
);

impl Rule for NoUnstableDefaultProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Function(func) = node.kind() else {
            return;
        };

        // Check if this looks like a React component (PascalCase name or arrow in PascalCase var)
        let is_component = func
            .id
            .as_ref()
            .is_some_and(|id| id.name.starts_with(|c: char| c.is_ascii_uppercase()));

        if !is_component {
            // Could be an arrow function assigned to a PascalCase variable,
            // but we can't check that easily from here. Just check the first param.
            // Skip non-PascalCase named functions.
            if func.id.is_some() {
                return;
            }
        }

        // Check first parameter for destructured props with default values
        let Some(first_param) = func.params.items.first() else {
            return;
        };

        let oxc_ast::ast::BindingPattern::ObjectPattern(obj_pattern) = &first_param.pattern else {
            return;
        };

        for prop in &obj_pattern.properties {
            if let oxc_ast::ast::BindingPattern::AssignmentPattern(assign) = &prop.value
                && is_unstable_value(&assign.right)
            {
                ctx.diagnostic(no_unstable_default_props_diagnostic(assign.right.span()));
            }
        }
    }
}

fn is_unstable_value(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::RegExpLiteral(_)
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function Component({ items = DEFAULT_ITEMS }) { return <div />; }",
        "function Component({ count = 0 }) { return <div />; }",
        "function Component({ name = 'default' }) { return <div />; }",
        "function Component({ enabled = true }) { return <div />; }",
        "function helper({ items = [] }) { return items; }",
    ];

    let fail = vec![
        "function Component({ items = [] }) { return <div />; }",
        "function Component({ config = {} }) { return <div />; }",
        "function Component({ onClick = () => {} }) { return <div />; }",
    ];

    Tester::new(NoUnstableDefaultProps::NAME, NoUnstableDefaultProps::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
