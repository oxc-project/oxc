use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{
    GlobalContext,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};

use crate::{
    AstNode, context::LintContext, fixer::RuleFixer, rule::Rule, utils::pad_fix_with_token_boundary,
};

fn prefer_math_trunc_diagnostic(span: Span, bad_op: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Math.trunc()` over instead of `{bad_op} 0`."))
        .with_label(span)
}

fn prefer_math_trunc_parse_int_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `Math.trunc` instead of `{name}(String(...), 10)`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferMathTrunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers use of [`Math.trunc()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/trunc) instead of bitwise operations or `parseInt(String(x), 10)` for clarity and more reliable results.
    ///
    /// It prevents the use of the following patterns:
    /// - `x | 0` ([`bitwise OR`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_OR) with 0)
    /// - `~~x` (two [`bitwise NOT`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_NOT))
    /// - `x >> 0` ([`Signed Right Shift`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Right_shift) with 0)
    /// - `x << 0` ([`Left Shift`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Left_shift) with 0)
    /// - `x ^ 0` ([`bitwise XOR Shift`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_XOR) with 0)
    /// - `parseInt(String(x), 10)` and `Number.parseInt(String(x), 10)`
    ///
    /// ### Why is this bad?
    ///
    /// Using bitwise operations or `parseInt(String(x), 10)` to truncate numbers is not clear and does not work reliably in [some cases](https://stackoverflow.com/a/34706108/11687747).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = 1.1 | 0;
    /// const bar = parseInt(String(foo), 10);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = Math.trunc(1.1);
    /// const bar = Math.trunc(foo);
    /// ```
    PreferMathTrunc,
    unicorn,
    pedantic,
    recommended,
    suggestion,
    version = "0.0.18",
    short_description = "Prefer `Math.trunc()` for truncating numbers.",
);

impl Rule for PreferMathTrunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (operator, argument_span, is_assignment, lhs_has_side_effect) = match node.kind() {
            AstKind::CallExpression(call_expr) => {
                check_parse_int_call(call_expr, ctx);
                return;
            }
            AstKind::UnaryExpression(unary_expr) => {
                if !matches!(unary_expr.operator, UnaryOperator::BitwiseNot) {
                    return;
                }
                let Expression::UnaryExpression(inner_unary_expr) = &unary_expr.argument else {
                    return;
                };
                if !matches!(inner_unary_expr.operator, UnaryOperator::BitwiseNot) {
                    return;
                }

                if let Expression::UnaryExpression(inner_inner_unary_expr) =
                    &inner_unary_expr.argument
                    && matches!(inner_inner_unary_expr.operator, UnaryOperator::BitwiseNot)
                {
                    return;
                }

                (UnaryOperator::BitwiseNot.as_str(), inner_unary_expr.argument.span(), false, false)
            }
            AstKind::BinaryExpression(bin_expr) => {
                let Expression::NumericLiteral(right_num_lit) = &bin_expr.right else {
                    return;
                };
                if right_num_lit.value != 0.0 {
                    return;
                }
                if !matches!(
                    bin_expr.operator,
                    BinaryOperator::BitwiseOR
                        | BinaryOperator::ShiftRight
                        | BinaryOperator::ShiftLeft
                        | BinaryOperator::BitwiseXOR
                ) {
                    return;
                }

                (bin_expr.operator.as_str(), bin_expr.left.span(), false, false)
            }
            AstKind::AssignmentExpression(assignment_expr) => {
                let Expression::NumericLiteral(right_num_lit) = &assignment_expr.right else {
                    return;
                };

                if right_num_lit.value != 0.0 {
                    return;
                }

                if !matches!(
                    assignment_expr.operator,
                    AssignmentOperator::BitwiseOR
                        | AssignmentOperator::ShiftRight
                        | AssignmentOperator::ShiftLeft
                        | AssignmentOperator::BitwiseXOR
                ) {
                    return;
                }

                (
                    assignment_expr.operator.as_str(),
                    assignment_expr.left.span(),
                    true,
                    assignment_expr
                        .left
                        .may_have_side_effects(&PreferMathTruncSideEffectsContext { ctx }),
                )
            }
            _ => return,
        };

        let span = node.kind().span();
        let diagnostic = prefer_math_trunc_diagnostic(span, operator);
        if is_assignment && lhs_has_side_effect {
            ctx.diagnostic(diagnostic);
            return;
        }
        ctx.diagnostic_with_suggestion(diagnostic, |fixer: RuleFixer<'_, 'a>| {
            let argument_text = ctx.source_range(argument_span);
            let mut replacement = if is_assignment {
                // `x |= 0` -> `x = Math.trunc(x)`
                format!("{argument_text} = Math.trunc({argument_text})")
            } else {
                // `x | 0` or `~~x` -> `Math.trunc(x)`
                format!("Math.trunc({argument_text})")
            };
            pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
            fixer.replace(span, replacement)
        });
    }
}

#[expect(clippy::float_cmp)]
fn check_parse_int_call<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if !call_expr.optional
        && call_expr.arguments.len() == 2
        && let Some(parse_int_name) = get_parse_int_name(call_expr, ctx)
        && let Some(first_arg) = call_expr.arguments[0].as_expression()
        && let Some(radix_arg) = call_expr.arguments[1].as_expression()
        && let Expression::NumericLiteral(radix) = radix_arg.without_parentheses()
        && radix.value == 10.0
        && let Expression::CallExpression(string_call) = first_arg.without_parentheses()
        && !string_call.optional
        && string_call.arguments.len() == 1
        && let Some(inner_arg) = string_call.arguments[0].as_expression()
        && let Expression::Identifier(string_ident) = string_call.callee.without_parentheses()
        && string_ident.name == "String"
        && ctx.is_reference_to_global_variable(string_ident)
    {
        let span = call_expr.span;
        let diagnostic = prefer_math_trunc_parse_int_diagnostic(span, parse_int_name);

        let search_start = string_call
            .type_arguments
            .as_ref()
            .map_or_else(|| string_call.callee.span().end, |ta| ta.span.end);
        let Some(open_paren) = (search_start < inner_arg.span().start)
            .then(|| ctx.find_next_token_within(search_start, inner_arg.span().start, "("))
            .flatten()
            .map(|offset| search_start + offset)
        else {
            ctx.diagnostic(diagnostic);
            return;
        };
        let close_paren = string_call.span.end.saturating_sub(1);
        let bytes = ctx.source_text().as_bytes();
        if close_paren <= open_paren || bytes.get(close_paren as usize) != Some(&b')') {
            ctx.diagnostic(diagnostic);
            return;
        }

        let has_comments_outside = ctx.has_comments_between(Span::new(span.start, open_paren))
            || ctx.has_comments_between(Span::new(close_paren, span.end));
        if has_comments_outside {
            ctx.diagnostic(diagnostic);
            return;
        }

        ctx.diagnostic_with_suggestion(diagnostic, |fixer: RuleFixer<'_, 'a>| {
            let argument_text = ctx.source_range(Span::new(open_paren + 1, close_paren));
            let mut replacement = format!("Math.trunc({argument_text})");
            pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
            fixer.replace(span, replacement)
        });
    }
}

fn get_parse_int_name<'a>(
    call_expr: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'static str> {
    match call_expr.callee.without_parentheses() {
        Expression::Identifier(ident)
            if ident.name == "parseInt" && ctx.is_reference_to_global_variable(ident) =>
        {
            Some("parseInt")
        }
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "parseInt" =>
        {
            let Expression::Identifier(obj) = member.object.without_parentheses() else {
                return None;
            };
            if obj.name == "Number" && ctx.is_reference_to_global_variable(obj) {
                Some("Number.parseInt")
            } else {
                None
            }
        }
        _ => None,
    }
}

struct PreferMathTruncSideEffectsContext<'c, 'a> {
    ctx: &'c LintContext<'a>,
}

impl<'a> GlobalContext<'a> for PreferMathTruncSideEffectsContext<'_, 'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        reference.is_global_reference(self.ctx.scoping())
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for PreferMathTruncSideEffectsContext<'_, 'a> {
    fn annotations(&self) -> bool {
        false
    }

    fn manual_pure_functions(&self, _callee: &Expression) -> bool {
        false
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        PropertyReadSideEffects::All
    }

    fn unknown_global_side_effects(&self) -> bool {
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 1 | 1;",
        "const foo = 0 | 1;",
        "const foo = 1.4 | +0;",
        "const foo = 1.4 | -0;",
        "const foo = 1.4 | (.5 - 0.5);",
        "const foo = 1.4 & 0xFFFFFFFF",
        "const foo = 1.4 & 0xFF",
        "const foo = 1.4 & 0x0",
        "const foo = 1.4 & 0",
        "const foo = ~3.9;",
        "const foo = 1.1 >> 1",
        "const foo = 1.1 >>> 0",
        "const foo = 0 << 1",
        "let foo = 0;
            foo |= 1;",
        "let foo = 1.1;
            foo >>>= 0;",
        "let foo = 1.2; // comment 1
            foo |= 1; // comment 2 and 1.2 | 0",
        "parseInt(value, 10);",
        "parseInt(String(value));",
        "parseInt(String(value), 16);",
        "Number.parseInt(String(value), 16);",
        "parseInt(String(value), \"10\");",
        "parseInt(String(value), 10n);",
        "parseInt(String(value), -10);",
        "parseInt?.(String(value), 10);",
        "parseInt(String?.(value), 10);",
        "parseInt(String(...value), 10);",
        "Number.parseInt(String(...value), 10);",
        "parseInt(String(value), ...[10]);",
        "Number.parseInt(String(value), ...[10]);",
        "parseInt(String(), 10);",
        "parseInt(String(value, extra), 10);",
        "parseInt(new String(value), 10);",
        "parseInt(String(value), 10, extra);",
        "Number.parseInt(String(value), 10, extra);",
        "Number.parseInt?.(String(value), 10);",
        "Number?.parseInt(String(value), 10);",
        "Number[\"parseInt\"](String(value), 10);",
        "const parseInt = value => value; parseInt(String(value), 10);",
        "const Number = { parseInt() {} }; Number.parseInt(String(value), 10);",
        "const String = value => value; parseInt(String(value), 10);",
        "const String = value => value; Number.parseInt(String(value), 10);",
    ];

    let fail = vec![
        "const foo = 1.1 | 0;",
        "const foo = 111 | 0;",
        "const foo = (1 + 2 / 3.4) | 0;",
        "const foo = bar((1.4 | 0) + 2);",
        "const foo = (0, 1.4) | 0;",
        "function foo() {return.1 | 0;}",
        "const foo = 1.4 | 0.;",
        "const foo = 1.4 | .0;",
        "const foo = 1.4 | 0.0000_0000_0000;",
        "const foo = 1.4 | 0b0;",
        "const foo = 1.4 | 0x0000_0000_0000;",
        "const foo = 1.4 | 0o0;",
        "const foo = 1.23 | 0 | 4;",
        "const foo = ~~3.9;",
        "const foo = ~~111;",
        "const foo = ~~(1 + 2 / 3.4);",
        "const foo = ~~1 + 2 / 3.4;",
        "const foo = ~~(0, 1.4);",
        "const foo = ~~~10.01;",
        "const foo = ~~(~10.01);",
        "const foo = ~(~~10.01);",
        "const foo = ~~-10.01;",
        "const foo = ~~~~10.01;",
        "function foo() {return~~3.9;}",
        "const foo = bar >> 0;",
        "const foo = bar << 0;",
        "const foo = bar ^ 0;",
        "function foo() {return.1 ^0;}",
        "foo.bar |= 0;",
        "foo[0] ^= 0;",
        "function foo() {return[foo][0] ^= 0;};",
        "obj[foo] |= 0;",
        // Unsafe assignment LHS: report but offer NO suggestion. Duplicating the LHS in
        // `x = Math.trunc(x)` would re-run side effects or observable property reads.
        "foo[i++] |= 0;",
        "foo[bar()] ^= 0;",
        "getFoo().bar >>= 0;",
        "obj.foo.bar |= 0;",
        "obj.foo[0] |= 0;",
        "const foo = /* first comment */ 3.4 | 0; // A B C",
        "const foo = /* first comment */ ~~3.4; // A B C",
        "const foo = /* will keep */ 3.4 /* will remove 1 */ | /* will remove 2 */ 0;",
        "const foo = /* will keep */ ~ /* will remove 1 */ ~ /* will remove 2 */ 3.4;",
        "const foo = ~~bar | 0;",
        "const foo = ~~(bar| 0);",
        "const foo = bar | 0 | 0;",
        "const foo = ~~~~((bar | 0 | 0) >> 0 >> 0 << 0 << 0 ^ 0 ^0);",
        "const foo = parseInt(String(number), 10);",
        "const foo = Number.parseInt(String(number), 10);",
        "const foo = (Number).parseInt(String(number), 10);",
        "const foo = parseInt((String(number)), 10);",
        "const foo = parseInt((String)(number), 10);",
        "const foo = parseInt(String(number), (10));",
        "const foo = parseInt(String(number), 10.0);",
        "const foo = parseInt(String(number), 0xa);",
        "const foo = parseInt(String((number)), 10);",
        "const foo = parseInt(String(number + 1), 10);",
        "const foo = parseInt(String(number,), 10);",
        "const foo = parseInt(String(/* comment */ number), 10);",
        "const foo = parseInt(String(number /* comment */), 10);",
        "const foo = parseInt(/* comment */ String(number), 10);",
        "const foo = parseInt(String /* comment */ (number), 10);",
        "const foo = parseInt(String(number) /* comment */, 10);",
        "const foo = parseInt(String(number), /* comment */ 10);",
        "const foo = parseInt(String((0, number)), 10);",
        "const foo = parseInt(String(number as number), 10);",
        "const foo = parseInt(String<() => number>(number), 10);",
        "const foo = parseInt(String(number!), 10);",
        "function foo() {return(parseInt)(String(read(/* keep */)), 10);}",
        "function foo() {throw(Number.parseInt)(String(read(/* keep */)), 10);}",
        "async function foo() {return await(parseInt)(String(read(/* keep */)), 10);}",
    ];

    let fix = vec![
        ("const foo = 1.1 | 0;", "const foo = Math.trunc(1.1);"),
        ("const foo = ~~3.9;", "const foo = Math.trunc(3.9);"),
        ("function foo() {return.1 | 0;}", "function foo() {return Math.trunc(.1);}"),
        ("function foo() {return~~3.9;}", "function foo() {return Math.trunc(3.9);}"),
        ("const foo = bar >> 0;", "const foo = Math.trunc(bar);"),
        ("const foo = bar << 0;", "const foo = Math.trunc(bar);"),
        ("const foo = bar ^ 0;", "const foo = Math.trunc(bar);"),
        ("foo.bar |= 0;", "foo.bar = Math.trunc(foo.bar);"),
        ("foo[0] ^= 0;", "foo[0] = Math.trunc(foo[0]);"),
        (
            "function foo() {return[foo][0] ^= 0;};",
            "function foo() {return[foo][0] = Math.trunc([foo][0]);};",
        ),
        ("obj[foo] |= 0;", "obj[foo] = Math.trunc(obj[foo]);"),
        ("const foo = parseInt(String(number), 10);", "const foo = Math.trunc(number);"),
        ("const foo = Number.parseInt(String(number), 10);", "const foo = Math.trunc(number);"),
        ("const foo = (Number).parseInt(String(number), 10);", "const foo = Math.trunc(number);"),
        ("const foo = parseInt((String(number)), 10);", "const foo = Math.trunc(number);"),
        ("const foo = parseInt((String)(number), 10);", "const foo = Math.trunc(number);"),
        ("const foo = parseInt(String(number), (10));", "const foo = Math.trunc(number);"),
        ("const foo = parseInt(String(number), 10.0);", "const foo = Math.trunc(number);"),
        ("const foo = parseInt(String(number), 0xa);", "const foo = Math.trunc(number);"),
        ("const foo = parseInt(String((number)), 10);", "const foo = Math.trunc((number));"),
        ("const foo = parseInt(String(number + 1), 10);", "const foo = Math.trunc(number + 1);"),
        ("const foo = parseInt(String(number,), 10);", "const foo = Math.trunc(number,);"),
        (
            "const foo = parseInt(String(/* comment */ number), 10);",
            "const foo = Math.trunc(/* comment */ number);",
        ),
        (
            "const foo = parseInt(String(number /* comment */), 10);",
            "const foo = Math.trunc(number /* comment */);",
        ),
        (
            "const foo = parseInt(/* comment */ String(number), 10);",
            "const foo = parseInt(/* comment */ String(number), 10);",
        ),
        (
            "const foo = parseInt(String /* comment */ (number), 10);",
            "const foo = parseInt(String /* comment */ (number), 10);",
        ),
        (
            "const foo = parseInt(String(number) /* comment */, 10);",
            "const foo = parseInt(String(number) /* comment */, 10);",
        ),
        (
            "const foo = parseInt(String(number), /* comment */ 10);",
            "const foo = parseInt(String(number), /* comment */ 10);",
        ),
        ("const foo = parseInt(String((0, number)), 10);", "const foo = Math.trunc((0, number));"),
        (
            "const foo = parseInt(String(number as number), 10);",
            "const foo = Math.trunc(number as number);",
        ),
        (
            "const foo = parseInt(String<() => number>(number), 10);",
            "const foo = Math.trunc(number);",
        ),
        ("const foo = parseInt(String(number!), 10);", "const foo = Math.trunc(number!);"),
        (
            "function foo() {return(parseInt)(String(read(/* keep */)), 10);}",
            "function foo() {return Math.trunc(read(/* keep */));}",
        ),
        (
            "function foo() {throw(Number.parseInt)(String(read(/* keep */)), 10);}",
            "function foo() {throw Math.trunc(read(/* keep */));}",
        ),
        (
            "async function foo() {return await(parseInt)(String(read(/* keep */)), 10);}",
            "async function foo() {return await Math.trunc(read(/* keep */));}",
        ),
    ];

    Tester::new(PreferMathTrunc::NAME, PreferMathTrunc::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
