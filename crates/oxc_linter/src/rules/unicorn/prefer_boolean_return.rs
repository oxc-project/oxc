use oxc_ast::{
    AstKind,
    ast::{Expression, ReturnStatement, Statement, StaticMemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::static_ident;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_boolean_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Return the boolean expression directly.")
        .with_help("Return the boolean expression directly instead of using an `if` statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferBooleanReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer directly returning boolean expressions over `if` statements that return
    /// `true` and `false` in opposite branches.
    ///
    /// ### Why is this bad?
    ///
    /// Returning a boolean expression directly is shorter and easier to read. It also
    /// avoids unnecessary branching when the condition already evaluates to a boolean.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function isEven(value) {
    ///   if (value % 2 === 0) {
    ///     return true;
    ///   } else {
    ///     return false;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function isEven(value) {
    ///   return value % 2 === 0;
    /// }
    /// ```
    PreferBooleanReturn,
    unicorn,
    style,
    recommended,
    conditional_fix,
    version = "next",
    short_description = "Prefer directly returning boolean expressions over `if` statements."
);

impl Rule for PreferBooleanReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        if matches!(if_stmt.test, Expression::ConditionalExpression(_)) {
            return;
        }

        let consequent_body = get_node_body(&if_stmt.consequent);
        let Some(consequent_value) = get_boolean_return_value(consequent_body) else {
            return;
        };

        let (replacement_span, alternate_return) = if let Some(alternate) = &if_stmt.alternate {
            let alternate_body = get_node_body(alternate);
            let Some(alternate_value) = get_boolean_return_value(alternate_body) else {
                return;
            };
            if consequent_value == alternate_value {
                return;
            }
            (if_stmt.span, as_return_statement(alternate_body))
        } else {
            let Some((previous, next)) = sibling_statements(node, ctx) else {
                return;
            };

            if let Some(previous) = previous
                && is_same_boolean_guard(previous, consequent_value)
            {
                return;
            }

            let Statement::ReturnStatement(next_return) = next else {
                return;
            };
            let Some(next_value) = get_boolean_return_value(next) else {
                return;
            };
            if consequent_value == next_value {
                return;
            }
            (Span::new(if_stmt.span.start, next_return.span.end), Some(next_return.as_ref()))
        };

        let should_negate = !consequent_value;

        let mut can_fix = true;

        let test_span = if_stmt.test.span();
        if ctx
            .comments_range(replacement_span.start..replacement_span.end)
            .any(|comment| comment.span.start < test_span.start || comment.span.end > test_span.end)
        {
            can_fix = false;
        }

        if can_fix
            && let Some(alternate_return) = alternate_return
            && has_trailing_same_line_comment(ctx, alternate_return.span.end)
        {
            can_fix = false;
        }

        let needs_boolean = !should_negate && !is_runtime_boolean_expression(&if_stmt.test, ctx);
        if can_fix && needs_boolean {
            let scope_id = node.scope_id();
            if let Some(symbol_id) = ctx.scoping().find_binding(scope_id, static_ident!("Boolean"))
                && !ctx.scoping().symbol_flags(symbol_id).is_type()
            {
                can_fix = false;
            }
        }

        let diagnostic = prefer_boolean_return_diagnostic(if_stmt.span);
        if can_fix {
            let replacement =
                format!("return {};", boolean_return_text(&if_stmt.test, should_negate, ctx));
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fixer.replace(replacement_span, replacement)
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

fn has_trailing_same_line_comment(ctx: &LintContext<'_>, stmt_end: u32) -> bool {
    let source = ctx.source_text();
    let start = stmt_end as usize;
    if start >= source.len() {
        return false;
    }
    let line_end = source[start..].find('\n').map_or(source.len(), |i| start + i);
    let Ok(line_end) = u32::try_from(line_end) else {
        return false;
    };
    let Some(comment) = ctx.comments_range(stmt_end..line_end).next() else {
        return false;
    };
    let between = &source[start..comment.span.start as usize];
    between.chars().all(char::is_whitespace)
}

fn get_node_body<'a, 'b>(stmt: &'b Statement<'a>) -> &'b Statement<'a> {
    let Statement::BlockStatement(block) = stmt else {
        return stmt;
    };

    let mut statements =
        block.body.iter().filter(|statement| !matches!(statement, Statement::EmptyStatement(_)));
    let Some(first) = statements.next() else {
        return stmt;
    };
    if statements.next().is_some() {
        return stmt;
    }
    get_node_body(first)
}

fn as_return_statement<'a, 'b>(stmt: &'b Statement<'a>) -> Option<&'b ReturnStatement<'a>> {
    if let Statement::ReturnStatement(ret) = stmt { Some(ret) } else { None }
}

fn get_boolean_return_value(stmt: &Statement<'_>) -> Option<bool> {
    let Statement::ReturnStatement(ret) = stmt else {
        return None;
    };
    let mut expression = ret.argument.as_ref()?;
    while let Some(inner) = unwrap_ts_expression(expression) {
        expression = inner;
    }
    if let Expression::BooleanLiteral(literal) = expression { Some(literal.value) } else { None }
}

fn unwrap_ts_expression<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b Expression<'a>> {
    match expression {
        Expression::TSAsExpression(expression) => Some(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => Some(&expression.expression),
        Expression::TSNonNullExpression(expression) => Some(&expression.expression),
        Expression::TSTypeAssertion(expression) => Some(&expression.expression),
        Expression::TSInstantiationExpression(expression) => Some(&expression.expression),
        _ => None,
    }
}

fn sibling_statements<'a, 'b>(
    node: &AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<(Option<&'b Statement<'a>>, &'b Statement<'a>)> {
    let parent = ctx.nodes().parent_node(node.id());
    let statements: &[Statement<'a>] = match parent.kind() {
        AstKind::BlockStatement(block) => &block.body,
        AstKind::FunctionBody(body) => &body.statements,
        AstKind::Program(program) => &program.body,
        AstKind::SwitchCase(case) => &case.consequent,
        AstKind::StaticBlock(block) => &block.body,
        _ => return None,
    };

    let span = node.span();
    let index = statements.iter().position(|statement| statement.span() == span)?;
    let previous = index.checked_sub(1).and_then(|index| statements.get(index));
    let next = statements.get(index + 1)?;
    Some((previous, next))
}

fn is_same_boolean_guard(stmt: &Statement<'_>, value: bool) -> bool {
    let Statement::IfStatement(if_stmt) = stmt else {
        return false;
    };
    if if_stmt.alternate.is_some() {
        return false;
    }
    get_boolean_return_value(get_node_body(&if_stmt.consequent)) == Some(value)
}

fn is_runtime_boolean_expression(expression: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    match expression {
        Expression::BooleanLiteral(_) => true,
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, UnaryOperator::LogicalNot | UnaryOperator::Delete)
        }
        Expression::BinaryExpression(binary) => matches!(
            binary.operator,
            BinaryOperator::Equality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::LessThan
                | BinaryOperator::LessEqualThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterEqualThan
                | BinaryOperator::In
                | BinaryOperator::Instanceof
        ),
        Expression::LogicalExpression(logical) => {
            is_runtime_boolean_expression(&logical.left, ctx)
                && is_runtime_boolean_expression(&logical.right, ctx)
        }
        Expression::CallExpression(call) => {
            if call.optional {
                return false;
            }

            match &call.callee {
                Expression::Identifier(identifier) => {
                    identifier.name == "Boolean" && ctx.is_reference_to_global_variable(identifier)
                }
                Expression::StaticMemberExpression(member) => is_array_is_array_call(member, ctx),
                _ => false,
            }
        }
        _ => false,
    }
}

fn is_array_is_array_call(member: &StaticMemberExpression<'_>, ctx: &LintContext<'_>) -> bool {
    if member.optional || member.property.name != "isArray" {
        return false;
    }
    let Expression::Identifier(object) = &member.object else {
        return false;
    };
    object.name == "Array" && ctx.is_reference_to_global_variable(object)
}

fn boolean_return_text(
    expression: &Expression<'_>,
    should_negate: bool,
    ctx: &LintContext<'_>,
) -> String {
    if should_negate {
        let inner = expression_text(expression, needs_parens_for_unary(expression), ctx);
        return format!("!{inner}");
    }

    if is_runtime_boolean_expression(expression, ctx) {
        return expression_text(expression, false, ctx);
    }

    let needs_parens = matches!(expression, Expression::SequenceExpression(_));
    format!("Boolean({})", expression_text(expression, needs_parens, ctx))
}

fn expression_text(
    expression: &Expression<'_>,
    needs_parens: bool,
    ctx: &LintContext<'_>,
) -> String {
    if let Expression::ParenthesizedExpression(parenthesized) = expression {
        let mut inner = &parenthesized.expression;
        while let Expression::ParenthesizedExpression(parenthesized) = inner {
            inner = &parenthesized.expression;
        }

        let outer_span = expression.span();
        let inner_span = inner.span();
        let has_comments = ctx.comments_range(outer_span.start..outer_span.end).any(|comment| {
            comment.span.start < inner_span.start || comment.span.end > inner_span.end
        });

        if has_comments {
            return ctx.source_range(outer_span).to_string();
        }

        let needs_parens = needs_parens || matches!(inner, Expression::SequenceExpression(_));
        return expression_text(inner, needs_parens, ctx);
    }

    let text = ctx.source_range(expression.span()).to_string();
    if needs_parens { format!("({text})") } else { text }
}

fn needs_parens_for_unary(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::SequenceExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::YieldExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::TSAsExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::TSNonNullExpression(_)
        | Expression::TSTypeAssertion(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            needs_parens_for_unary(&parenthesized.expression)
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tester::Tester;

    fn pass() -> Vec<&'static str> {
        vec![
            "function unicorn() { if (test) { return true; } doSomething(); return false; }",
            "function unicorn() { if (test) { return a; } return b; }",
            "function unicorn() { if (test) { return true; doSomething(); } return false; }",
            "function unicorn() { if (test) { return; } return false; }",
            "function unicorn() { if (a ? b : c) { return true; } return false; }",
            "function unicorn() { if (test) { return true; } else { return true; } }",
            "function unicorn() { if (a) { return false; } if (b) { return false; } return true; }",
            "function unicorn() { if (a) { return false; } if (b) { return false; } if (c) { return false; } return true; }",
            "function unicorn() { if (a) { return true; } if (b) { return true; } return false; }",
            "function unicorn() { if (a) return false; if (b) return false; return true; }",
            "function f() {\n\tif (a) {\n\t\treturn t!;\n\t}\n\n\treturn false;\n}",
            // Additional valid edge cases
            "if (test) { return false; } else { return false; }",
            "if (test) { return; } else { return false; }",
            "if (test) { return true; }",
            "if (test) { return false; }",
            "if (test) { return true; } else { foo(); return false; }",
            "if (test) { foo(); return true; } else { return false; }",
            "if (test) { return true; } foo(); return false;",
            "if (test) { return true; } if (foo) { return false; }",
            "if (test) { return false; if (foo) { return true; } } return false;",
        ]
    }

    fn fail() -> Vec<&'static str> {
        vec![
            "if (test) { return true; } else if (foo) { return false; } else { return true; }",
            "function unicorn() { if(test){ return true; } else{ return false; } }",
            "function unicorn() { if (test) return true; else return false; }",
            "function unicorn() { if(value > 0){ return true; } else{ return false; } }",
            "function unicorn() { if(items.length){ return false; } else{ return true; } }",
            "function unicorn(Boolean) { if(test){ return true; } else{ return false; } }",
            "function unicorn() { if(test){ return true; } else{ return false; /* comment */ } }",
            "function unicorn() { if(test){ return true; /* comment */ } else{ return false; } }",
            "function unicorn() { if (test) { return true; } return false; }",
            "function unicorn() { if (test) { return false; } return true; }",
            "function unicorn() { if (value > 0) { return true; } return false; }",
            "function unicorn() { if (Boolean(value)) { return true; } return false; }",
            "function unicorn() { if (a === b && c === d) { return true; } return false; }",
            "function unicorn() { if (a && b) { return true; } return false; }",
            "function unicorn() { if (Array.isArray(value)) { return true; } return false; }",
            "function unicorn(Array) { if (Array.isArray(value)) { return true; } return false; }",
            "function unicorn() { if (value > 0) { return false; } return true; }",
            "function unicorn() { if (test) return true; return false; }",
            "function unicorn() { if (items.length) { return true; } return false; }",
            "function unicorn() { if (items.length) { return false; } return true; }",
            "function unicorn() { if (object?.property) { return true; } return false; }",
            "function unicorn() { if (a, b) { return true; } return false; }",
            "function unicorn() { if ((a, b)) { return true; } return false; }",
            "function unicorn() { if ((test)) { return true; } return false; }",
            "function unicorn() { if (value as boolean) { return true; } return false; }",
            "function unicorn(value: boolean) { if (value) { return true; } return false; }",
            "function unicorn() { if (value satisfies boolean) { return true; } return false; }",
            "function unicorn() { if (value!) { return true; } return false; }",
            "function unicorn() { if ( value) { return true; } return false; }",
            "function unicorn() { if (value satisfies boolean) { return false; } return true; }",
            "function unicorn() {\n\tif (test) {\n\t\treturn true;\n\t}\n\n\t// comment\n\treturn false;\n}",
            "function unicorn() { if (test) { return true; } return false; // comment\n}",
            "function unicorn() { if (/* comment */ test) { return true; } return false; }",
            "function unicorn(Boolean) { if (test) { return true; } return false; }",
            "function unicorn(Boolean) { if (test) { return false; } return true; }",
            "function unicorn(Boolean) { if (value > 0) { return true; } return false; }",
            "function unicorn() { if (a) { return false; } if (b) { return true; } return false; }",
            "function unicorn() {\n\tif (test({\n\t\tmultiline: true,\n\t})) {\n\t\treturn true;\n\t}\n\n\treturn false;\n}",
            "function unicorn() { if (!test) { return true; } return false; }",
            "function unicorn() { if (delete object.key) { return true; } return false; }",
            "function unicorn() { if (typeof value) { return true; } return false; }",
            "function unicorn() { if (a === b || false) { return true; } return false; }",
            "function unicorn() { if ((value > 0)) { return false; } return true; }",
            "type Boolean = string; function unicorn() { if (test) { return true; } return false; }",
            "function unicorn() { if (value /* keep */ > 0) { return true; } else { return false; } }",
            "function unicorn() { if (value /* keep */ > 0) { return false; } return true; }",
            "function unicorn() { if (test(/* keep */ value)) { return true; } return false; }",
            "function unicorn() { if (((/* keep */ value))) { return true; } return false; }",
            "function unicorn() { if (((value /* keep */))) { return false; } else { return true; } }",
            "function unicorn() { if (value // keep\r\n > 0) { return true; } return false; }",
            "function unicorn() { if (value as /* keep */ boolean) { return true; } return false; }",
            "function run() { if (value satisfies /* keep */ boolean) { return false; } return true; }",
            "function unicorn() { if (value > 0 /* keep */) { return true; } return false; }",
            "function unicorn() { if (value /* copied */ > 0) { return /* keep */ true; } return false; }",
            "function unicorn() { if (value /* copied */ > 0) { return true; } /* keep */ return false; }",
            "function unicorn() { if (value /* copied */ > 0) { return true; } return false; // keep\n}",
            "function unicorn(Boolean) { if (test(/* keep */ value)) { return true; } return false; }",
            "function f() {\n\tif (a) {\n\t\treturn true as const;\n\t}\n\n\treturn false;\n}",
            "function f() {\n\tif (a) {\n\t\treturn true as boolean;\n\t}\n\n\treturn false;\n}",
            "function f() {\n\tif (a) {\n\t\treturn true satisfies boolean;\n\t}\n\n\treturn false;\n}",
            "function f() {\n\tif (a) {\n\t\treturn false as const;\n\t}\n\n\treturn true;\n}",
            "function f() {\n\tif (a) {\n\t\treturn false!;\n\t}\n\n\treturn true;\n}",
        ]
    }

    fn fix() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "if (test) { return true; } else if (foo) { return false; } else { return true; }",
                "if (test) { return true; } else return !foo;",
            ),
            (
                "function unicorn() { if(test){ return true; } else{ return false; } }",
                "function unicorn() { return Boolean(test); }",
            ),
            (
                "function unicorn() { if (test) return true; else return false; }",
                "function unicorn() { return Boolean(test); }",
            ),
            (
                "function unicorn() { if(value > 0){ return true; } else{ return false; } }",
                "function unicorn() { return value > 0; }",
            ),
            (
                "function unicorn() { if(items.length){ return false; } else{ return true; } }",
                "function unicorn() { return !items.length; }",
            ),
            (
                "function unicorn() { if (test) { return true; } return false; }",
                "function unicorn() { return Boolean(test); }",
            ),
            (
                "function unicorn() { if (test) { return false; } return true; }",
                "function unicorn() { return !test; }",
            ),
            (
                "function unicorn() { if (value > 0) { return true; } return false; }",
                "function unicorn() { return value > 0; }",
            ),
            (
                "function unicorn() { if (Boolean(value)) { return true; } return false; }",
                "function unicorn() { return Boolean(value); }",
            ),
            (
                "function unicorn() { if (a === b && c === d) { return true; } return false; }",
                "function unicorn() { return a === b && c === d; }",
            ),
            (
                "function unicorn() { if (a && b) { return true; } return false; }",
                "function unicorn() { return Boolean(a && b); }",
            ),
            (
                "function unicorn() { if (Array.isArray(value)) { return true; } return false; }",
                "function unicorn() { return Array.isArray(value); }",
            ),
            (
                "function unicorn(Array) { if (Array.isArray(value)) { return true; } return false; }",
                "function unicorn(Array) { return Boolean(Array.isArray(value)); }",
            ),
            (
                "function unicorn() { if (value > 0) { return false; } return true; }",
                "function unicorn() { return !(value > 0); }",
            ),
            (
                "function unicorn() { if (test) return true; return false; }",
                "function unicorn() { return Boolean(test); }",
            ),
            (
                "function unicorn() { if (items.length) { return true; } return false; }",
                "function unicorn() { return Boolean(items.length); }",
            ),
            (
                "function unicorn() { if (items.length) { return false; } return true; }",
                "function unicorn() { return !items.length; }",
            ),
            (
                "function unicorn() { if (object?.property) { return true; } return false; }",
                "function unicorn() { return Boolean(object?.property); }",
            ),
            (
                "function unicorn() { if (a, b) { return true; } return false; }",
                "function unicorn() { return Boolean((a, b)); }",
            ),
            (
                "function unicorn() { if ((a, b)) { return true; } return false; }",
                "function unicorn() { return Boolean((a, b)); }",
            ),
            (
                "function unicorn() { if ((test)) { return true; } return false; }",
                "function unicorn() { return Boolean(test); }",
            ),
            (
                "function unicorn() { if (value as boolean) { return true; } return false; }",
                "function unicorn() { return Boolean(value as boolean); }",
            ),
            (
                "function unicorn(value: boolean) { if (value) { return true; } return false; }",
                "function unicorn(value: boolean) { return Boolean(value); }",
            ),
            (
                "function unicorn() { if (value satisfies boolean) { return true; } return false; }",
                "function unicorn() { return Boolean(value satisfies boolean); }",
            ),
            (
                "function unicorn() { if (value!) { return true; } return false; }",
                "function unicorn() { return Boolean(value!); }",
            ),
            (
                "function unicorn() { if ( value) { return true; } return false; }",
                "function unicorn() { return Boolean(value); }",
            ),
            (
                "function unicorn() { if (value satisfies boolean) { return false; } return true; }",
                "function unicorn() { return !(value satisfies boolean); }",
            ),
            (
                "function unicorn(Boolean) { if (test) { return false; } return true; }",
                "function unicorn(Boolean) { return !test; }",
            ),
            (
                "function unicorn(Boolean) { if (value > 0) { return true; } return false; }",
                "function unicorn(Boolean) { return value > 0; }",
            ),
            (
                "function unicorn() { if (a) { return false; } if (b) { return true; } return false; }",
                "function unicorn() { if (a) { return false; } return Boolean(b); }",
            ),
            (
                "function unicorn() {\n\tif (test({\n\t\tmultiline: true,\n\t})) {\n\t\treturn true;\n\t}\n\n\treturn false;\n}",
                "function unicorn() {\n\treturn Boolean(test({\n\t\tmultiline: true,\n\t}));\n}",
            ),
            (
                "function unicorn() { if (!test) { return true; } return false; }",
                "function unicorn() { return !test; }",
            ),
            (
                "function unicorn() { if (delete object.key) { return true; } return false; }",
                "function unicorn() { return delete object.key; }",
            ),
            (
                "function unicorn() { if (typeof value) { return true; } return false; }",
                "function unicorn() { return Boolean(typeof value); }",
            ),
            (
                "function unicorn() { if (a === b || false) { return true; } return false; }",
                "function unicorn() { return a === b || false; }",
            ),
            (
                "function unicorn() { if ((value > 0)) { return false; } return true; }",
                "function unicorn() { return !(value > 0); }",
            ),
            (
                "type Boolean = string; function unicorn() { if (test) { return true; } return false; }",
                "type Boolean = string; function unicorn() { return Boolean(test); }",
            ),
            (
                "function unicorn() { if (value /* keep */ > 0) { return true; } else { return false; } }",
                "function unicorn() { return value /* keep */ > 0; }",
            ),
            (
                "function unicorn() { if (value /* keep */ > 0) { return false; } return true; }",
                "function unicorn() { return !(value /* keep */ > 0); }",
            ),
            (
                "function unicorn() { if (test(/* keep */ value)) { return true; } return false; }",
                "function unicorn() { return Boolean(test(/* keep */ value)); }",
            ),
            (
                "function unicorn() { if (((/* keep */ value))) { return true; } return false; }",
                "function unicorn() { return Boolean(((/* keep */ value))); }",
            ),
            (
                "function unicorn() { if (((value /* keep */))) { return false; } else { return true; } }",
                "function unicorn() { return !((value /* keep */)); }",
            ),
            (
                "function unicorn() { if (value // keep\r\n > 0) { return true; } return false; }",
                "function unicorn() { return value // keep\r\n > 0; }",
            ),
            (
                "function unicorn() { if (value as /* keep */ boolean) { return true; } return false; }",
                "function unicorn() { return Boolean(value as /* keep */ boolean); }",
            ),
            (
                "function run() { if (value satisfies /* keep */ boolean) { return false; } return true; }",
                "function run() { return !(value satisfies /* keep */ boolean); }",
            ),
            (
                "function f() {\n\tif (a) {\n\t\treturn true as const;\n\t}\n\n\treturn false;\n}",
                "function f() {\n\treturn Boolean(a);\n}",
            ),
            (
                "function f() {\n\tif (a) {\n\t\treturn true as boolean;\n\t}\n\n\treturn false;\n}",
                "function f() {\n\treturn Boolean(a);\n}",
            ),
            (
                "function f() {\n\tif (a) {\n\t\treturn true satisfies boolean;\n\t}\n\n\treturn false;\n}",
                "function f() {\n\treturn Boolean(a);\n}",
            ),
            (
                "function f() {\n\tif (a) {\n\t\treturn false as const;\n\t}\n\n\treturn true;\n}",
                "function f() {\n\treturn !a;\n}",
            ),
            (
                "function f() {\n\tif (a) {\n\t\treturn false!;\n\t}\n\n\treturn true;\n}",
                "function f() {\n\treturn !a;\n}",
            ),
        ]
    }

    #[test]
    fn test() {
        Tester::new(PreferBooleanReturn::NAME, PreferBooleanReturn::PLUGIN, pass(), fail())
            .expect_fix(fix())
            .test_and_snapshot();
    }
}
