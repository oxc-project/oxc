use oxc_ast::{
    ast::{match_member_expression, CallExpression, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{is_equality_matcher, parse_expect_jest_fn_call, ParsedExpectFnCall, PossibleJestNode},
};

fn use_to_have_length(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toHaveLength()`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToHaveLength;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// In order to have a better failure message, `toHaveLength()` should be used upon
    /// asserting expectations on objects length property.
    ///
    /// ### Why is this bad?
    ///
    /// This rule triggers a warning if `toBe()`, `toEqual()` or `toStrictEqual()` is
    /// used to assert objects length property.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // valid
    /// expect.hasAssertions;
    /// expect.hasAssertions();
    /// expect(files).toHaveLength(1);
    /// expect(files.name).toBe('file');
    ///
    /// // invalid
    /// expect(files["length"]).toBe(1);
    /// expect(files["length"]).toBe(1,);
    /// expect(files["length"])["not"].toBe(1)
    /// ```
    ///
    PreferToHaveLength,
    style,
    fix
);

impl Rule for PreferToHaveLength {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferToHaveLength {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(parsed_expect_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Some(static_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        match static_expr.object() {
            expr @ match_member_expression!(Expression) => {
                let mem_expr = expr.to_member_expression();
                let Expression::CallExpression(expr_call_expr) = mem_expr.object() else {
                    return;
                };
                match mem_expr {
                    MemberExpression::ComputedMemberExpression(_) => Self::check_and_fix(
                        call_expr,
                        expr_call_expr,
                        &parsed_expect_call,
                        Some("ComputedMember"),
                        mem_expr.static_property_name(),
                        ctx,
                    ),
                    MemberExpression::StaticMemberExpression(_) => Self::check_and_fix(
                        call_expr,
                        expr_call_expr,
                        &parsed_expect_call,
                        Some("StaticMember"),
                        mem_expr.static_property_name(),
                        ctx,
                    ),
                    MemberExpression::PrivateFieldExpression(_) => (),
                };
            }
            Expression::CallExpression(expr_call_expr) => {
                Self::check_and_fix(
                    call_expr,
                    expr_call_expr,
                    &parsed_expect_call,
                    None,
                    None,
                    ctx,
                );
            }
            _ => (),
        }
    }

    fn check_and_fix<'a>(
        call_expr: &CallExpression<'a>,
        expr_call_expr: &CallExpression<'a>,
        parsed_expect_call: &ParsedExpectFnCall<'a>,
        kind: Option<&str>,
        property_name: Option<&str>,
        ctx: &LintContext<'a>,
    ) {
        let Some(argument) = expr_call_expr.arguments.first() else {
            return;
        };
        let Some(static_mem_expr) = argument.as_member_expression() else {
            return;
        };
        // Get property `name` field from expect(file.NAME) call
        let Some(expect_property_name) = static_mem_expr.static_property_name() else {
            return;
        };
        let Some(matcher) = parsed_expect_call.matcher() else {
            return;
        };
        if expect_property_name != "length" || !is_equality_matcher(matcher) {
            return;
        }

        ctx.diagnostic_with_fix(use_to_have_length(matcher.span), |fixer| {
            let code = Self::build_code(fixer, static_mem_expr, kind, property_name);
            let offset = u32::try_from(
                fixer
                    .source_range(Span::new(matcher.span.end, call_expr.span().end))
                    .find('(')
                    .unwrap(),
            )
            .unwrap();
            fixer.replace(Span::new(call_expr.span.start, matcher.span.end + offset), code)
        });
    }

    fn build_code<'a>(
        fixer: RuleFixer<'_, 'a>,
        mem_expr: &MemberExpression<'a>,
        kind: Option<&str>,
        property_name: Option<&str>,
    ) -> String {
        let mut formatter = fixer.codegen();

        formatter.print_str("expect(");
        formatter.print_str(
            fixer.source_range(Span::new(mem_expr.span().start, mem_expr.object().span().end)),
        );
        formatter.print_str(")");

        if let Some(kind_val) = kind {
            if kind_val == "ComputedMember" {
                let property = property_name.unwrap();
                formatter.print_str("[\"");
                formatter.print_str(property);
                formatter.print_str("\"]");
            } else if kind_val == "StaticMember" {
                formatter.print_str(".");
                let property = property_name.unwrap();
                formatter.print_str(property);
            }
        }

        formatter.print_str(".toHaveLength");
        formatter.into_source_text()
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect(files).toHaveLength(1);", None),
        ("expect(files.name).toBe('file');", None),
        ("expect(files[`name`]).toBe('file');", None),
        ("expect(users[0]?.permissions?.length).toBe(1);", None),
        ("expect(result).toBe(true);", None),
        ("expect(user.getUserName(5)).resolves.toEqual('Paul')", None),
        ("expect(user.getUserName(5)).rejects.toEqual('Paul')", None),
        ("expect(a);", None),
    ];

    let fail = vec![
        ("expect(files[\"length\"]).toBe(1);", None),
        ("expect(files[\"length\"]).toBe(1,);", None),
        ("expect(files[\"length\"])[\"not\"].toBe(1);", None),
        ("expect(files[\"length\"])[\"toBe\"](1);", None),
        ("expect(files[\"length\"]).not[\"toBe\"](1);", None),
        ("expect(files[\"length\"])[\"not\"][\"toBe\"](1);", None),
        ("expect(files.length).toBe(1);", None),
        ("expect(files.length).toEqual(1);", None),
        ("expect(files.length).toStrictEqual(1);", None),
        ("expect(files.length).not.toStrictEqual(1);", None),
        (
            "expect((meta.get('pages') as YArray<unknown>).length).toBe((originalMeta.get('pages') as YArray<unknown>).length);",
            None,
        ),
        (
            "expect(assetTypeContainer.getElementsByTagName('time').length).toEqual(
          0,
        );",
            None,
        ),
    ];

    let fix = vec![
        ("expect(files[\"length\"]).not.toBe(1);", "expect(files).not.toHaveLength(1);", None),
        (
            "expect(files[\"length\"])[\"resolves\"].toBe(1,);",
            "expect(files)[\"resolves\"].toHaveLength(1,);",
            None,
        ),
        (
            "expect(files[\"length\"])[\"not\"].toBe(1);",
            "expect(files)[\"not\"].toHaveLength(1);",
            None,
        ),
        ("expect(files[\"length\"])[\"toBe\"](1);", "expect(files).toHaveLength(1);", None),
        ("expect(files[\"length\"]).not[\"toBe\"](1);", "expect(files).not.toHaveLength(1);", None),
        (
            "expect(files[\"length\"])[\"not\"][\"toBe\"](1);",
            "expect(files)[\"not\"].toHaveLength(1);",
            None,
        ),
        ("expect(files.length).toBe(1);", "expect(files).toHaveLength(1);", None),
        ("expect(files.length).toEqual(1);", "expect(files).toHaveLength(1);", None),
        ("expect(files.length).toStrictEqual(1);", "expect(files).toHaveLength(1);", None),
        ("expect(files.length).not.toStrictEqual(1);", "expect(files).not.toHaveLength(1);", None),
        (
            "expect((meta.get('pages') as YArray<unknown>).length).toBe((originalMeta.get('pages') as YArray<unknown>).length);",
            "expect((meta.get('pages') as YArray<unknown>)).toHaveLength((originalMeta.get('pages') as YArray<unknown>).length);",
            None,
        ),
        (
            "expect(assetTypeContainer.getElementsByTagName('time').length).toEqual(
          0,
        );",
            "expect(assetTypeContainer.getElementsByTagName('time')).toHaveLength(
          0,
        );",
            None,
        ),
    ];

    Tester::new(PreferToHaveLength::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
