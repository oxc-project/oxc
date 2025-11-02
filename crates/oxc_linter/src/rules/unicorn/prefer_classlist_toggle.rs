use oxc_ast::{
    AstKind,
    ast::{CallExpression, ChainElement, Expression, IfStatement, MemberExpression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_same_expression,
};

fn prefer_classlist_toggle_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `classList.toggle()` over `classList.add()` and `classList.remove()`",
    )
    .with_help("Use `classList.toggle()` instead")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferClasslistToggle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers the use of `element.classList.toggle(className, condition)` over
    /// conditional add/remove patterns.
    ///
    /// ### Why is this bad?
    ///
    /// The `toggle()` method is more concise and expressive than using conditional
    /// logic to switch between `add()` and `remove()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (condition) {
    ///     element.classList.add('className');
    /// } else {
    ///     element.classList.remove('className');
    /// }
    ///
    /// condition ? element.classList.add('className') : element.classList.remove('className');
    ///
    /// element.classList[condition ? 'add' : 'remove']('className');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// element.classList.toggle('className', condition);
    /// ```
    PreferClasslistToggle,
    unicorn,
    style,
    fix
);

impl Rule for PreferClasslistToggle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                check_if_statement(if_stmt, node, ctx);
            }
            AstKind::ConditionalExpression(cond_expr) => {
                check_conditional_add_remove_call_expr(
                    &cond_expr.test,
                    &cond_expr.consequent,
                    &cond_expr.alternate,
                    node,
                    ctx,
                );

                let parent = ctx.nodes().parent_node(node.id());
                let grand_parent = ctx.nodes().parent_node(parent.id());
                if let AstKind::ComputedMemberExpression(_) = parent.kind()
                    && let AstKind::CallExpression(call_expr) = grand_parent.kind()
                {
                    check_computed_member_call(call_expr, ctx);
                }
            }
            _ => {}
        }
    }
}

fn check_if_statement<'a>(if_stmt: &'a IfStatement<'a>, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let Some(alternate) = &if_stmt.alternate else {
        return;
    };

    let Some(consequent) = extract_single_expression_from_statement(&if_stmt.consequent) else {
        return;
    };

    let Some(alternate_expr) = extract_single_expression_from_statement(alternate) else {
        return;
    };

    check_add_remove_calls(&if_stmt.test, consequent, alternate_expr, node, ctx);
}

fn extract_call_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expr {
        Expression::ChainExpression(chain) => {
            if let ChainElement::CallExpression(call) = &chain.expression {
                Some(call)
            } else {
                None
            }
        }
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}

fn check_add_remove_calls<'a>(
    test: &'a Expression<'a>,
    consequent: &'a Expression<'a>,
    alternate: &'a Expression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(consequent_call) = extract_call_expression(consequent) else {
        return;
    };
    let Some(alternate_call) = extract_call_expression(alternate) else {
        return;
    };

    if let Some((add_call, _remove_call, is_add_first)) =
        identify_add_remove_pair(consequent_call, alternate_call, ctx)
    {
        ctx.diagnostic_with_fix(prefer_classlist_toggle_diagnostic(node.span()), |fixer| {
            fix_if_statement(fixer, node, test, add_call, is_add_first, ctx)
        });
    }
}

fn check_conditional_add_remove_call_expr<'a>(
    test: &'a Expression<'a>,
    consequent: &'a Expression<'a>,
    alternate: &'a Expression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(consequent_call) = extract_call_expression(consequent) else {
        return;
    };
    let Some(alternate_call) = extract_call_expression(alternate) else {
        return;
    };

    if let Some((add_call, _remove_call, is_add_first)) =
        identify_add_remove_pair(consequent_call, alternate_call, ctx)
    {
        let span = consequent.span().merge(alternate.span());
        ctx.diagnostic_with_fix(prefer_classlist_toggle_diagnostic(span), |fixer| {
            fix_conditional_expression(fixer, node, test, add_call, is_add_first, ctx)
        });
    }
}

fn extract_single_expression_from_statement<'a>(
    stmt: &'a Statement<'a>,
) -> Option<&'a Expression<'a>> {
    match stmt {
        Statement::ExpressionStatement(e) => Some(&e.expression),
        Statement::BlockStatement(block_stmt) if block_stmt.body.len() == 1 => {
            if let Statement::ExpressionStatement(e) = &block_stmt.body[0] {
                Some(&e.expression)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn check_computed_member_call<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if call_expr.optional {
        return;
    }

    if call_expr.arguments.len() != 1 {
        return;
    }

    if call_expr.arguments[0].is_spread() {
        return;
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return;
    };

    let MemberExpression::ComputedMemberExpression(computed) = member_expr else {
        return;
    };

    if computed.optional {
        return;
    }

    if !is_classlist_access(computed.object.get_inner_expression()) {
        return;
    }

    let Expression::ConditionalExpression(cond_expr) = &computed.expression else {
        return;
    };

    let Some(is_add_first) = check_add_remove_ternary(&cond_expr.consequent, &cond_expr.alternate)
    else {
        return;
    };

    let span = call_expr.span;
    ctx.diagnostic_with_fix(prefer_classlist_toggle_diagnostic(span), |fixer| {
        fix_computed_member_call(fixer, call_expr, &cond_expr.test, is_add_first, ctx)
    });
}

fn identify_add_remove_pair<'a>(
    first: &'a CallExpression<'a>,
    second: &'a CallExpression<'a>,
    ctx: &LintContext,
) -> Option<(&'a CallExpression<'a>, &'a CallExpression<'a>, bool)> {
    if first.arguments.len() != 1
        || second.arguments.len() != 1
        || first.optional
        || second.optional
    {
        return None;
    }
    let first_call_expr_arg = first.arguments[0].as_expression()?;
    let second_call_expr_arg = second.arguments[0].as_expression()?;

    if !is_same_expression(
        first_call_expr_arg.get_inner_expression(),
        second_call_expr_arg.get_inner_expression(),
        ctx,
    ) {
        return None;
    }

    let Expression::StaticMemberExpression(first_member) = first.callee.get_inner_expression()
    else {
        return None;
    };
    let Expression::StaticMemberExpression(second_member) = second.callee.get_inner_expression()
    else {
        return None;
    };

    if first_member.optional || second_member.optional {
        return None;
    }

    let (classlist_add_call, classlist_remove_call, is_add_first) =
        if first_member.property.name == "add" && second_member.property.name == "remove" {
            (first, second, true)
        } else if first_member.property.name == "remove" && second_member.property.name == "add" {
            (second, first, false)
        } else {
            return None;
        };

    let (classlist_add_callee, classlist_remove_callee) = if is_add_first {
        (&first_member, &second_member)
    } else {
        (&second_member, &first_member)
    };

    let Expression::StaticMemberExpression(classlist_add_member_expr) =
        classlist_add_callee.object.get_inner_expression()
    else {
        return None;
    };
    let Expression::StaticMemberExpression(classlist_remove_member_expr) =
        classlist_remove_callee.object.get_inner_expression()
    else {
        return None;
    };

    if classlist_add_member_expr.property.name != "classList"
        || classlist_remove_member_expr.property.name != "classList"
    {
        return None;
    }

    if !is_same_expression(
        classlist_add_member_expr.object.get_inner_expression(),
        classlist_remove_member_expr.object.get_inner_expression(),
        ctx,
    ) {
        return None;
    }

    Some((classlist_add_call, classlist_remove_call, is_add_first))
}

fn is_classlist_access(expr: &Expression) -> bool {
    match expr {
        Expression::StaticMemberExpression(static_member) => {
            static_member.property.name == "classList"
        }
        Expression::ChainExpression(chain) => {
            if let Some(member_expr) = chain.expression.as_member_expression() {
                member_expr.static_property_name() == Some("classList")
            } else {
                false
            }
        }
        _ => false,
    }
}

fn check_add_remove_ternary(consequent: &Expression, alternate: &Expression) -> Option<bool> {
    let (Expression::StringLiteral(cons_str), Expression::StringLiteral(alt_str)) =
        (consequent.get_inner_expression(), alternate.get_inner_expression())
    else {
        return None;
    };

    if cons_str.value == "add" && alt_str.value == "remove" {
        Some(true)
    } else if cons_str.value == "remove" && alt_str.value == "add" {
        Some(false)
    } else {
        None
    }
}

fn fix_if_statement<'a>(
    fixer: RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    test: &Expression<'a>,
    add_call: &CallExpression<'a>,
    is_add_first: bool,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let Some(member_expr) = add_call.callee.get_member_expr() else {
        return fixer.noop();
    };

    let classlist_expr = member_expr.object();
    let classlist_text = classlist_expr.span().source_text(ctx.source_text());

    let Some(class_name_arg) = add_call.arguments[0].as_expression() else {
        return fixer.noop();
    };
    let class_name_text = class_name_arg.span().source_text(ctx.source_text());

    let test_text = test.span().source_text(ctx.source_text());
    let condition = if is_add_first { test_text.to_string() } else { format!("!({test_text})") };

    let replacement = format!("{classlist_text}.toggle({class_name_text}, {condition});");
    fixer.replace(node.span(), replacement)
}

fn fix_conditional_expression<'a>(
    fixer: RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    test: &Expression<'a>,
    add_call: &CallExpression<'a>,
    is_add_first: bool,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let Some(member_expr) = add_call.callee.get_member_expr() else {
        return fixer.noop();
    };

    let classlist_expr = member_expr.object();
    let classlist_text = classlist_expr.span().source_text(ctx.source_text());

    let Some(class_name_arg) = add_call.arguments[0].as_expression() else {
        return fixer.noop();
    };
    let class_name_text = class_name_arg.span().source_text(ctx.source_text());

    let test_text = test.span().source_text(ctx.source_text());
    let condition = if is_add_first { test_text.to_string() } else { format!("!({test_text})") };

    let replacement = format!("{classlist_text}.toggle({class_name_text}, {condition})");
    fixer.replace(node.span(), replacement)
}

fn fix_computed_member_call<'a>(
    fixer: RuleFixer<'_, 'a>,
    call_expr: &CallExpression<'a>,
    test: &Expression<'a>,
    is_add_first: bool,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return fixer.noop();
    };

    let MemberExpression::ComputedMemberExpression(computed) = member_expr else {
        return fixer.noop();
    };

    let classlist_text = computed.object.span().source_text(ctx.source_text());

    let Some(class_name_arg) = call_expr.arguments[0].as_expression() else {
        return fixer.noop();
    };
    let class_name_text = class_name_arg.span().source_text(ctx.source_text());

    let test_text = test.span().source_text(ctx.source_text());
    let condition = if is_add_first { test_text.to_string() } else { format!("!({test_text})") };

    let replacement = format!("{classlist_text}.toggle({class_name_text}, {condition})");
    fixer.replace(call_expr.span, replacement)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"element.classList.toggle("className", condition)"#,
        r#"element.classList.toggle("className")"#,
        "if (condition) {
				element.classList.add('className');
			} else if (other) {
				element.classList.remove('className');
			}",
        "if (condition) {
				element.classList.notAdd('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				element.classList.remove('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				element.classList.add('className');
			} else {
				element.classList.add('className');
			}",
        "if (condition) {
				element.classList.add('className1');
			} else {
				element.classList.remove('className2');
			}",
        "element.classList.add('className');
			element.classList.remove('className');",
        "if (condition) {
				element.classList.add?.('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				element.classList?.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				a.add('className');
			} else {
				a.remove('className');
			}",
        "if (condition) {
				element.notClassList.add('className');
			} else {
				element.notClassList.remove('className');
			}",
        "if (condition) {
				element.classList.add('className');
				foo();
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				{
					element.classList.add('className');
				}
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				element1.classList.add('className');
			} else {
				element2.classList.remove('className');
			}",
        "if (condition) {
				element.classList.add('className', extraArgument);
			} else {
				element.classList.remove('className', extraArgument);
			}",
        "if (condition) {
				element.classList.add();
			} else {
				element.classList.remove();
			}",
        "if (condition) {
				element.classList.remove('className');
				element.classList.add('className');
			}",
        "condition ? element.classList.add(className1) : element.classList.remove(className2)",
        "condition ? element.classList.add?.(className) : element.classList.remove(className)",
        "condition ? element.classList?.add(className) : element.classList.remove(className)",
        "condition ? element.classList.add(className) : element.classList.add(className)",
        "condition ? element.classList.notAdd(className) : element.classList.remove(className)",
        "condition ? element.notClassList.add(className) : element.notClassList.remove(className)",
        r#"element.classList[condition ? "add" : "remove"]"#,
        r#"element.classList[condition ? "add" : "remove"](className, extraArgument)"#,
        r#"element.classList[condition ? "add" : "remove"]()"#,
        r#"element.classList[condition ? "add" : "remove"]?.(className)"#,
        r#"element.classList[condition ? add : "remove"](className)"#,
        r#"element.classList[condition ? "add" : "add"](className)"#,
        r#"element.classList[condition ? "remove" : "remove"](className)"#,
        r#"(condition ? "add" : "remove").classList(className)"#,
        r#"element.classList.add(condition ? "add" : "remove")"#,
        r#"foo(element.classList[condition ? "add" : "remove"])"#,
    ];

    let fail = vec![
        "if (condition) {
				element.classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition)
				element.classList.add('className');
			else
				element.classList.remove('className');",
        "if (condition)
				element.classList.add('className');
			else {
				element.classList.remove('className');
			}",
        "if (condition) {
				element?.classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition)
				element.classList.add('className');
			else
				element?.classList.remove('className');",
        "if (condition) {
				element.classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (condition) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (condition) {
				(( element )).classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (0, condition) {
				element.classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (0, condition) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (condition) {
				element.classList.remove(((className)));
			} else {
				element.classList.add(((className)));
			}",
        "foo
			if (condition) {
				(( element )).classList.add('className')
			} else {
				element.classList.remove('className')
			}",
        "if (condition) {
				(( element )).classList.add('className')
			} else {
				element.classList.remove('className')
			}
			[].forEach(foo);",
        "if (element.classList.contains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element?.classList.contains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element.classList.contains?.('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element.classList?.contains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element.classList.notContains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element.classList.contains('not-same-class-name')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element.notClassList.contains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (contains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (notSameElement.classList.contains('className')) {
				element.classList.remove('className');
			} else {
				element.classList.add('className');
			}",
        "if (element.classList.contains('className')) {
				element.classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "if (!element.classList.contains('className')) {
				element.classList.add('className');
			} else {
				element.classList.remove('className');
			}",
        "condition ? element.classList.add(className) : element.classList.remove(className)",
        "condition ? element?.classList.add(className) : element.classList.remove(className)",
        "condition ? element.classList.add(className) : element?.classList.remove(className)",
        "condition ? element.classList.remove(className) : element.classList.add(className)",
        "if (condition ? element.classList.add(className) : element.classList.remove(className));",
        "function foo() {
				return!foo ? element.classList.add(className) : element.classList.remove(className)
			}",
        "foo
			condition ? (( element )).classList.add(className) : element.classList.remove(className);",
        "element.classList.contains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element?.classList.contains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element.classList.contains?.('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element.classList?.contains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element.classList.notContains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element.classList.contains('not-same-class-name')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element.notClassList.contains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "contains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "notSameElement.classList.contains('className')
				? element.classList.remove('className')
				: element.classList.add('className')",
        "element.classList.contains('className')
				? element.classList.add('className')
				: element.classList.remove('className')",
        "!element.classList.contains('className')
				? element.classList.add('className')
				: element.classList.remove('className')",
        r#"element.classList[condition ? "add" : "remove"](className)"#,
        r#"element.classList[condition ? "remove": "add"](className)"#,
        r#"element?.classList[condition ? "add" : "remove"](className)"#,
        r#"const toggle = (element) => element.classList[condition ? "add" : "remove"](className)"#,
        r#"element.classList[condition ? "add" : "remove"](((className)))"#,
        r#"element.classList[index % 2 ? "remove" : "add"](className)"#,
        r#"element.classList[(index % 2) ? "remove" : "add"](className)"#,
        r#"element.classList[(0, condition) ? "add" : "remove"](className)"#,
    ];

    let fix = vec![
        (
            "if (condition) { element.classList.add('foo'); } else { element.classList.remove('foo'); }",
            "element.classList.toggle('foo', condition);",
        ),
        (
            "if (condition) { element.classList.remove('foo'); } else { element.classList.add('foo'); }",
            "element.classList.toggle('foo', !(condition));",
        ),
        (
            "condition ? element.classList.add('foo') : element.classList.remove('foo')",
            "element.classList.toggle('foo', condition)",
        ),
        (
            "condition ? element.classList.remove('foo') : element.classList.add('foo')",
            "element.classList.toggle('foo', !(condition))",
        ),
        (
            r#"element.classList[condition ? "add" : "remove"]('foo')"#,
            "element.classList.toggle('foo', condition)",
        ),
        (
            r#"element.classList[condition ? "remove" : "add"]('foo')"#,
            "element.classList.toggle('foo', !(condition))",
        ),
        (
            "if (foo) element.classList.add('bar'); else element.classList.remove('bar');",
            "element.classList.toggle('bar', foo);",
        ),
    ];

    Tester::new(PreferClasslistToggle::NAME, PreferClasslistToggle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
