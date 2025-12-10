use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{ParsedExpectFnCall, PossibleJestNode, is_equality_matcher, parse_expect_jest_fn_call},
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(files["length"]).toBe(1);
    /// expect(files["length"]).toBe(1,);
    /// expect(files["length"])["not"].toBe(1)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(files).toHaveLength(1);
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-to-have-length.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-to-have-length": "error"
    ///   }
    /// }
    /// ```
    PreferToHaveLength,
    jest,
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
                if let MemberExpression::PrivateFieldExpression(_) = mem_expr {
                    return;
                }
                let Expression::CallExpression(expr_call_expr) = mem_expr.object() else {
                    return;
                };
                Self::check_and_fix(
                    call_expr,
                    expr_call_expr,
                    &parsed_expect_call,
                    Some(mem_expr),
                    ctx,
                );
            }
            Expression::CallExpression(expr_call_expr) => {
                Self::check_and_fix(call_expr, expr_call_expr, &parsed_expect_call, None, ctx);
            }
            _ => (),
        }
    }

    fn check_and_fix<'a>(
        call_expr: &CallExpression<'a>,
        expr_call_expr: &CallExpression<'a>,
        parsed_expect_call: &ParsedExpectFnCall<'a>,
        super_mem_expr: Option<&MemberExpression<'a>>,
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
            let code = Self::build_code(fixer.source_text(), static_mem_expr, super_mem_expr);
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
        source: &str,
        mem_expr: &MemberExpression<'a>,
        super_mem_expr: Option<&MemberExpression<'a>>,
    ) -> String {
        let l = Span::new(mem_expr.span().start, mem_expr.object().span().end).source_text(source);
        let r = super_mem_expr.map(|mem_expr| {
            Span::new(mem_expr.object().span().end, mem_expr.span().end).source_text(source)
        });

        let mut code = String::with_capacity(8 + l.len() + r.map_or(0, str::len) + 13);
        code.push_str("expect(");
        code.push_str(l);
        code.push(')');
        if let Some(r) = r {
            code.push_str(r);
        }
        code.push_str(".toHaveLength");
        code
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

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
        ("expect().toBe();", None),
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
        (r#"expect(files["length"]).toBe(1,);"#, "expect(files).toHaveLength(1,);", None),
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

    Tester::new(PreferToHaveLength::NAME, PreferToHaveLength::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
