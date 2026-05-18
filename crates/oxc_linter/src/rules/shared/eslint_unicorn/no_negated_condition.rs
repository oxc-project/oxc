use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, context::LintContext};

fn no_negated_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated condition.")
        .with_help("Remove the negation operator and switch the consequent and alternate branches.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallow negated conditions.

### Why is this bad?

Negated conditions are more difficult to understand. Code can be made more readable by inverting the condition.

### Examples

Examples of **incorrect** code for this rule:
```javascript
if (!a) {
	doSomethingC();
} else {
	doSomethingB();
}

!a ? doSomethingC() : doSomethingB()
```

Examples of **correct** code for this rule:
```javascript
if (a) {
	doSomethingB();
} else {
	doSomethingC();
}

a ? doSomethingB() : doSomethingC()
```
";

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    match node.kind() {
        AstKind::IfStatement(if_stmt) => {
            let Some(if_stmt_alternate) = &if_stmt.alternate else {
                return;
            };

            if matches!(if_stmt_alternate, Statement::IfStatement(_)) {
                return;
            }

            let test = if_stmt.test.without_parentheses();
            if is_negated_expression(test) {
                ctx.diagnostic(no_negated_condition_diagnostic(test.span()));
            }
        }
        AstKind::ConditionalExpression(conditional_expr) => {
            let test = conditional_expr.test.without_parentheses();
            if is_negated_expression(test) {
                ctx.diagnostic(no_negated_condition_diagnostic(test.span()));
            }
        }
        _ => {}
    }
}

fn is_negated_expression(expr: &Expression) -> bool {
    match expr {
        Expression::UnaryExpression(unary_expr) => unary_expr.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary_expr) => matches!(
            binary_expr.operator,
            BinaryOperator::Inequality | BinaryOperator::StrictInequality
        ),
        _ => false,
    }
}
