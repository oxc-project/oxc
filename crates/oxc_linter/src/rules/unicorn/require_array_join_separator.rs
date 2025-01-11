use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_prototype_property,
    AstNode,
};

fn require_array_join_separator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce using the separator argument with Array#join()")
        .with_help("Missing the separator argument.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireArrayJoinSeparator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using the separator argument with Array#join()
    ///
    /// ### Why is this bad?
    ///
    /// It's better to make it clear what the separator is when calling Array#join(),
    /// instead of relying on the default comma (',') separator.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.join()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo.join(",")
    /// ```
    RequireArrayJoinSeparator,
    unicorn,
    style,
    conditional_fix
);

fn is_array_prototype_property(member_expr: &MemberExpression, property: &str) -> bool {
    is_prototype_property(member_expr, property, Some("Array"))
}

impl Rule for RequireArrayJoinSeparator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        // foo.join()
        if is_method_call(call_expr, None, Some(&["join"]), Some(0), Some(0))
            && !call_expr.optional
            && !matches!(member_expr, MemberExpression::ComputedMemberExpression(_))
        {
            ctx.diagnostic_with_fix(
                require_array_join_separator_diagnostic(Span::new(
                    member_expr.span().end,
                    call_expr.span.end,
                )),
                |fixer| {
                    // after end of `join`, find the `(` and insert `","`
                    let open_bracket = ctx
                        .source_range(call_expr.span)
                        .chars()
                        .skip(member_expr.span().size() as usize)
                        .position(|c| c == '(');

                    if let Some(open_bracket) = open_bracket {
                        #[allow(clippy::cast_possible_truncation)]
                        fixer.insert_text_after_range(
                            Span::new(
                                0,
                                call_expr.span.start
                                    + member_expr.span().size()
                                    + open_bracket as u32
                                    + 1,
                            ),
                            r#"",""#,
                        )
                    } else {
                        fixer.noop()
                    }
                },
            );
        }

        // `[].join.call(foo)` and `Array.prototype.join.call(foo)`
        if let Some(member_expr_obj) = member_expr.object().as_member_expression() {
            if is_method_call(call_expr, None, Some(&["call"]), Some(1), Some(1))
                && !member_expr.optional()
                && !call_expr.optional
                && !call_expr.arguments.iter().any(oxc_ast::ast::Argument::is_spread)
                && is_array_prototype_property(member_expr_obj, "join")
            {
                ctx.diagnostic_with_fix(
                    require_array_join_separator_diagnostic(Span::new(
                        member_expr.span().end,
                        call_expr.span.end,
                    )),
                    |fixer| {
                        // after the end of the first argument, insert `","`
                        let first_arg = call_expr.arguments.first().unwrap();
                        fixer.insert_text_after_range(
                            Span::new(first_arg.span().end, first_arg.span().end),
                            r#", ",""#,
                        )
                    },
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"foo.join(",")"#, None),
        (r"join()", None),
        (r"foo.join(...[])", None),
        (r"foo.join?.()", None),
        (r"foo?.join?.()", None),
        (r"foo[join]()", None),
        (r#"foo["join"]()"#, None),
        (r#"[].join.call(foo, ",")"#, None),
        (r"[].join.call()", None),
        (r"[].join.call(...[foo])", None),
        (r"[].join?.call(foo)", None),
        (r"[]?.join.call(foo)", None),
        (r"[].join[call](foo)", None),
        (r"[][join].call(foo)", None),
        (r"[,].join.call(foo)", None),
        (r"[].join.notCall(foo)", None),
        (r"[].notJoin.call(foo)", None),
        (r#"Array.prototype.join.call(foo, "")"#, None),
        (r"Array.prototype.join.call()", None),
        (r"Array.prototype.join.call(...[foo])", None),
        (r"Array.prototype.join?.call(foo)", None),
        (r"Array.prototype?.join.call(foo)", None),
        (r"Array?.prototype.join.call(foo)", None),
        (r#"Array.prototype.join[call](foo, "")"#, None),
        (r"Array.prototype[join].call(foo)", None),
        (r"Array[prototype].join.call(foo)", None),
        (r"Array.prototype.join.notCall(foo)", None),
        (r"Array.prototype.notJoin.call(foo)", None),
        (r"Array.notPrototype.join.call(foo)", None),
        (r"NotArray.prototype.join.call(foo)", None),
        (r#"path.join(__dirname, "./foo.js")"#, None),
    ];

    let fail = vec![
        (r"foo.join()", None),
        (r"[].join.call(foo)", None),
        (r"[].join.call(foo,)", None),
        (r"[].join.call(foo , );", None),
        (r"Array.prototype.join.call(foo)", None),
        (r"Array.prototype.join.call(foo, )", None),
        (r"foo?.join()", None),
    ];

    let fix = vec![
        (r"foo.join()", r#"foo.join(",")"#),
        (r"foo.join                 ()", r#"foo.join                 (",")"#),
        (r"[].join.call(foo)", r#"[].join.call(foo, ",")"#),
        (r"[].join.call(foo,)", r#"[].join.call(foo, ",",)"#),
        (r"[].join.call(foo , );", r#"[].join.call(foo, "," , );"#),
        (r"Array.prototype.join.call(foo)", r#"Array.prototype.join.call(foo, ",")"#),
        (r"Array.prototype.join.call(foo, )", r#"Array.prototype.join.call(foo, ",", )"#),
        (r"foo?.join()", r#"foo?.join(",")"#),
    ];

    Tester::new(RequireArrayJoinSeparator::NAME, RequireArrayJoinSeparator::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
