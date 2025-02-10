use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, BinaryExpression, BinaryOperator, Expression,
        StaticMemberExpression,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, fixer::Fix, rule::Rule, utils::is_same_expression, AstNode};

fn prefer_negative_index_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer negative index over .length - index when possible").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNegativeIndex;

#[derive(Debug)]
enum TypeOptions {
    String,
    Array,
    TypedArray,
    Literal,
    Unknown,
}

declare_oxc_lint!(
    /// ### What it does
    /// Prefer negative index over `.length` - index when possible
    ///
    /// ### Why is this bad?
    /// Conciseness and readability
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo.slice(foo.length - 2, foo.length - 1);
    /// foo.at(foo.length - 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo.slice(-2, -1);
    /// foo.at(-1);
    /// ```
    PreferNegativeIndex,
    unicorn,
    style,
    fix
);

impl Rule for PreferNegativeIndex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !call_expr.callee.is_member_expression() {
            return;
        }

        let callee_object_expr = call_expr.callee.to_member_expression().object();

        let Some(name) = call_expr.callee_name() else { return };
        let is_prototype_call = name == "call";
        let is_prototype_apply = name == "apply";
        let is_prototype =
            callee_object_expr.is_member_expression() && (is_prototype_call || is_prototype_apply);

        let Some(callee_name) = (if is_prototype {
            callee_object_expr.to_member_expression().static_property_name()
        } else {
            Some(name)
        }) else {
            return;
        };

        let callee_type = if is_prototype {
            get_prototype_callee_type(callee_object_expr.to_member_expression().object())
        } else {
            TypeOptions::Literal
        };

        let identifier_expr = match (callee_type, callee_name) {
            (TypeOptions::String, "slice" | "at")
            | (TypeOptions::TypedArray, "slice" | "at" | "with" | "subarray")
            | (TypeOptions::Array, "slice" | "at" | "splice" | "with" | "toSpliced")
            | (
                TypeOptions::Literal,
                "slice" | "at" | "splice" | "subarray" | "with" | "toSpliced",
            ) => {
                if is_prototype {
                    let Some(first_arg) =
                        call_expr.arguments.first().and_then(Argument::as_expression)
                    else {
                        return;
                    };
                    first_arg
                } else {
                    callee_object_expr
                }
            }
            _ => return,
        };

        let mut member_exprs: Vec<&StaticMemberExpression> = Vec::new();
        let range_increment = if matches!(callee_name, "slice" | "subarray") { 2 } else { 1 };
        let arg_range_start = usize::from(is_prototype);
        let arg_range_end = if is_prototype_apply {
            arg_range_start + 1
        } else {
            arg_range_start + range_increment
        };

        for (i, argument) in call_expr.arguments.iter().enumerate() {
            if i >= arg_range_end {
                break;
            }

            let Some(arg_expr) = argument.as_expression() else {
                continue;
            };

            match arg_expr {
                Expression::BinaryExpression(binary_expr) => {
                    let Some(member_expr) = get_binary_left_expr(binary_expr) else {
                        continue;
                    };

                    if is_same_node(identifier_expr, &member_expr.object, ctx) {
                        member_exprs.push(member_expr);
                    }
                }
                Expression::ArrayExpression(array_expr) => {
                    for (j, element) in array_expr.elements.iter().enumerate() {
                        if j >= range_increment {
                            break;
                        }
                        if let ArrayExpressionElement::BinaryExpression(binary_expr) = element {
                            let Some(el_member_expr) = get_binary_left_expr(binary_expr) else {
                                continue;
                            };

                            if is_same_node(identifier_expr, &el_member_expr.object, ctx) {
                                member_exprs.push(el_member_expr);
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        if !member_exprs.is_empty() {
            ctx.diagnostic_with_fix(prefer_negative_index_diagnostic(call_expr.span), |fixer| {
                let mut fixes = fixer.new_fix_with_capacity(member_exprs.len());

                for member_expr in member_exprs {
                    let member_expr_span = member_expr.span();
                    let member_expr_next_end = member_expr_span.end + 1;
                    let member_expr_with_next_span =
                        Span::new(member_expr_span.start, member_expr_next_end);
                    let member_expr_with_next_str = ctx.source_range(member_expr_with_next_span);

                    if member_expr_with_next_str.ends_with(' ') {
                        fixes.push(Fix::delete(member_expr_with_next_span));
                    } else {
                        fixes.push(Fix::delete(member_expr_span));
                    }
                }

                fixes
            });
        }
    }
}

fn is_same_node(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    if is_same_expression(left, right, ctx) {
        return true;
    }

    match (left, right) {
        (
            Expression::ComputedMemberExpression(left_computed_expr),
            Expression::ComputedMemberExpression(right_computed_expr),
        ) => is_same_node(&left_computed_expr.expression, &right_computed_expr.expression, ctx),
        (Expression::StringLiteral(left_lit), Expression::NumericLiteral(right_lit)) => {
            left_lit.to_string() == right_lit.to_string()
        }
        (Expression::NumericLiteral(left_lit), Expression::StringLiteral(right_lit)) => {
            left_lit.to_string() == right_lit.to_string()
        }
        (
            Expression::TemplateLiteral(left_template_lit),
            Expression::StringLiteral(right_string_lit),
        ) => {
            let Some(template_str) = left_template_lit.quasi() else {
                return false;
            };

            template_str.as_str() == right_string_lit.to_string()
        }
        (
            Expression::StringLiteral(left_string_lit),
            Expression::TemplateLiteral(right_template_lit),
        ) => {
            let Some(template_str) = right_template_lit.quasi() else {
                return false;
            };

            left_string_lit.to_string() == template_str.as_str()
        }
        _ => false,
    }
}

fn get_prototype_callee_type(expression: &Expression) -> TypeOptions {
    match expression {
        Expression::ArrayExpression(_) => TypeOptions::Array,
        Expression::StringLiteral(_) => TypeOptions::String,
        Expression::StaticMemberExpression(static_member_expr) => {
            let Some(identifier_ref) = static_member_expr.object.get_identifier_reference() else {
                return TypeOptions::Unknown;
            };

            if static_member_expr.property.name.as_str() != "prototype" {
                return TypeOptions::Unknown;
            }

            match identifier_ref.name.as_str() {
                "String" => TypeOptions::String,
                "Array" => TypeOptions::Array,
                "Int8Array" | "Uint8Array" | "Uint8ClampedArray" | "Int16Array" | "Uint16Array"
                | "Int32Array" | "Uint32Array" | "Float32Array" | "Float64Array"
                | "BigInt64Array" | "BigUint64Array" | "ArrayBuffer" => TypeOptions::TypedArray,
                _ => TypeOptions::Unknown,
            }
        }
        _ => TypeOptions::Unknown,
    }
}

fn get_binary_left_expr<'a>(
    binary_expr: &'a BinaryExpression,
) -> Option<&'a StaticMemberExpression<'a>> {
    if !matches!(binary_expr.operator, BinaryOperator::Subtraction)
        || !binary_expr.right.is_number_literal() | binary_expr.right.is_number_0()
    {
        return None;
    }

    match &binary_expr.left {
        Expression::ParenthesizedExpression(paren_expr) => {
            let Expression::BinaryExpression(paren_inner_binary_expr) = &paren_expr.expression
            else {
                return None;
            };

            get_binary_left_expr(paren_inner_binary_expr)
        }
        Expression::BinaryExpression(inner_binary_expr) => get_binary_left_expr(inner_binary_expr),
        Expression::StaticMemberExpression(member_expr) => {
            if member_expr.property.name == "length" {
                return Some(member_expr.as_ref());
            }

            None
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.slice(-2, -1)",
        "foo.splice(-1, 1)",
        "Array.prototype.slice.call(foo, -2, -1)",
        "Array.prototype.slice.apply(foo, [-2, -1])",
        "slice(foo.length - 1)",
        "foo.forEach(foo.length - 1)",
        "Array.prototype.forEach.call(foo, foo.length - 1)",
        "FOO.prototype.slice.apply(foo, [-2, -1])",
        r#"Array.prototype.slice.apply(foo, "")"#,
        "new Foo.forEach(Foo.length - 1)",
        "foo.slice(bar.length - 1)",
        "foo.slice(foo.length - 0)",
        r#"foo.slice(foo.length - "1")"#,
        "foo.slice(foo.length - (-1))",
        "foo.slice(foo.length + 1)",
        "foo.slice(foo.length - 2 + 1)",
        "foo.slice((foo.length - 1) + 1)",
        "foo.slice(foo.length - 1 / 1)",
        "[1, 2, 3].slice([1, 2, 3].length - 1)",
        "foo[bar++].slice(foo[bar++].length - 1)",
        "function foo() {return [].slice.apply(arguments);}",
        "String.prototype.toSpliced.call(foo, foo.length - 1)",
        "String.prototype.with.call(foo, foo.length - 1)",
        "Uint8Array.prototype.toSpliced.call(foo, foo.length - 1)",
        "Array.prototype.subarray.call(foo, foo.length - 1)",
    ];

    let fail = vec![
        "foo.slice(foo.length - 2, foo.length - 1)",
        "foo.splice(foo.length - 1, 1)",
        "Array.prototype.slice.call(foo, foo.length - 2, foo.length - 1)",
        "Array.prototype.slice.apply(foo, [foo.length - 2, foo.length - 1])",
        "foo.slice(foo.length - 1 - 1)",
        "foo.bar.slice(foo.bar.length - 1)",
        "foo['bar'].slice(foo['bar'].length - 1)",
        "foo[1].slice(foo[1].length - 1)",
        "foo.slice(foo.length/* comment */ - 1)",
        "
        							foo.slice(
        								// comment 1

        								foo.length

        								// comment 2
        								- 1
        								-1
        								,
        								foo.length // comment 3
        								- 1
        							)
        						",
        "foo.slice((foo.length - 1) - 1)",
        "foo.slice(/* will keep */(/* will keep 1 */foo.length - 1) - 1)",
        "
        							[].slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							[].splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							[].slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							[].splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							[NOT_EMPTY].slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							[NOT_EMPTY].splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							[NOT_EMPTY].slice.call(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							[NOT_EMPTY].splice.call(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						",
        "
        							''.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							''.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							''.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							''.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							'NOT_EMPTY'.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							'NOT_EMPTY'.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							'NOT_EMPTY'.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							'NOT_EMPTY'.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						",
        "
        							Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							String.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							String.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							ArrayBuffer.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							ArrayBuffer.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int8Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int8Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8ClampedArray.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8ClampedArray.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int16Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int16Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint16Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint16Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int32Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint32Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float32Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float64Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigInt64Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigInt64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigUint64Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigUint64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							NOT_SUPPORTED.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							NOT_SUPPORTED.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        						",
        "
        							Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							String.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							String.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							ArrayBuffer.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							ArrayBuffer.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int8Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int8Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8ClampedArray.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8ClampedArray.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int16Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int16Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint16Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint16Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int32Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint32Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float32Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float64Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigInt64Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigInt64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigUint64Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigUint64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							NOT_SUPPORTED.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							NOT_SUPPORTED.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						",
        "/**/foo.slice(foo.length - 2, foo.length - 1)",
        "/**/foo.splice(foo.length - 1, 1)",
        r#"foo.bar.slice(foo["bar"].length - 1)"#,
        r#"foo[`bar`].slice(foo["bar"].length - 1)"#,
        r#"foo[1].slice(foo["1"].length - 1)"#,
        r#"foo['bar'].slice(foo["bar"].length - 1)"#,
        "foo.toSpliced(foo.length - 3, foo.length - 6)",
        "Array.prototype.toSpliced.call(foo, foo.length - 3, foo.length - 6)",
        "[].toSpliced.call(foo, foo.length - 3, foo.length - 6)",
        "foo.with(foo.length - 3, foo.length - 6)",
        "Array.prototype.with.call(foo, foo.length - 3, foo.length - 6)",
        "foo.subarray(foo.length - 3, foo.length - 6)",
        "Uint8Array.prototype.subarray.call(foo, foo.length - 3, foo.length - 6)",
        "Uint8Array.prototype.subarray.apply(foo, [foo.length - 3, foo.length - 6])",
    ];

    let fix = vec![
        ("foo.slice(foo.length - 2, foo.length - 1)", "foo.slice(- 2, - 1)", None),
        ("foo.splice(foo.length - 1, 1)", "foo.splice(- 1, 1)", None),
        (
            "Array.prototype.slice.call(foo, foo.length - 2, foo.length - 1)",
            "Array.prototype.slice.call(foo, - 2, - 1)",
            None,
        ),
        (
            "Array.prototype.slice.apply(foo, [foo.length - 2, foo.length - 1])",
            "Array.prototype.slice.apply(foo, [- 2, - 1])",
            None,
        ),
        ("foo.slice(foo.length - 1 - 1)", "foo.slice(- 1 - 1)", None),
        ("foo.bar.slice(foo.bar.length - 1)", "foo.bar.slice(- 1)", None),
        ("foo['bar'].slice(foo['bar'].length - 1)", "foo['bar'].slice(- 1)", None),
        ("foo[1].slice(foo[1].length - 1)", "foo[1].slice(- 1)", None),
        ("foo.slice(foo.length/* comment */ - 1)", "foo.slice(/* comment */ - 1)", None),
        (
            "
        							foo.slice(
        								// comment 1

        								foo.length

        								// comment 2
        								- 1,
        								-1
        								,
        								foo.length // comment 3
        								- 1
        							)
        						",
            "
        							foo.slice(
        								// comment 1

        								

        								// comment 2
        								- 1,
        								-1
        								,
        								foo.length // comment 3
        								- 1
        							)
        						",
            None,
        ),
        ("foo.slice((foo.length - 1) - 1)", "foo.slice((- 1) - 1)", None),
        (
            "foo.slice(/* will keep */(/* will keep 1 */foo.length - 1) - 1)",
            "foo.slice(/* will keep */(/* will keep 1 */- 1) - 1)",
            None,
        ),
        (
            "
        							[].slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							[].splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							[].slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							[].splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						",
            "
        							[].slice.call(foo, - 1, - 2, foo.length - 3);
        							[].splice.call(foo, - 1, foo.length - 2, foo.length - 3);
        							[].slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							[].splice.apply(foo, [- 1, foo.length - 2, foo.length - 3]);
        						",
            None,
        ),
        ("
        							''.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							''.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							''.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							''.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						", "
        							''.slice.call(foo, - 1, - 2, foo.length - 3);
        							''.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							''.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							''.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						", None),
        ("
        							Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							String.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							String.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							ArrayBuffer.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							ArrayBuffer.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int8Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int8Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8ClampedArray.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8ClampedArray.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int16Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int16Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint16Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint16Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int32Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint32Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float32Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float64Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigInt64Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigInt64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigUint64Array.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigUint64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							NOT_SUPPORTED.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							NOT_SUPPORTED.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        						", "
        							Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Array.prototype.splice.call(foo, - 1, foo.length - 2, foo.length - 3);
        							String.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							String.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							ArrayBuffer.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							ArrayBuffer.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int8Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Int8Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Uint8Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint8ClampedArray.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Uint8ClampedArray.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int16Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Int16Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint16Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Uint16Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Int32Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Int32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Uint32Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Uint32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float32Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Float32Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							Float64Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							Float64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigInt64Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							BigInt64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							BigUint64Array.prototype.slice.call(foo, - 1, - 2, foo.length - 3);
        							BigUint64Array.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							NOT_SUPPORTED.prototype.slice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        							NOT_SUPPORTED.prototype.splice.call(foo, foo.length - 1, foo.length - 2, foo.length - 3);
        						", None),
        ("
        							Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							String.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							String.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							ArrayBuffer.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							ArrayBuffer.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int8Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int8Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8ClampedArray.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8ClampedArray.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int16Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int16Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint16Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint16Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int32Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint32Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float32Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float64Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigInt64Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigInt64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigUint64Array.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigUint64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							NOT_SUPPORTED.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							NOT_SUPPORTED.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						", "
        							Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Array.prototype.splice.apply(foo, [- 1, foo.length - 2, foo.length - 3]);
        							String.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							String.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							ArrayBuffer.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							ArrayBuffer.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int8Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Int8Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Uint8Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint8ClampedArray.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Uint8ClampedArray.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int16Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Int16Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint16Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Uint16Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Int32Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Int32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Uint32Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Uint32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float32Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Float32Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							Float64Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							Float64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigInt64Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							BigInt64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							BigUint64Array.prototype.slice.apply(foo, [- 1, - 2, foo.length - 3]);
        							BigUint64Array.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							NOT_SUPPORTED.prototype.slice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        							NOT_SUPPORTED.prototype.splice.apply(foo, [foo.length - 1, foo.length - 2, foo.length - 3]);
        						", None)
    ];
    Tester::new(PreferNegativeIndex::NAME, PreferNegativeIndex::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
