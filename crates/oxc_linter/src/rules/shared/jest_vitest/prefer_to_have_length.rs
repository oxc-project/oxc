use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{ParsedExpectFnCall, PossibleJestNode, is_equality_matcher, parse_expect_jest_fn_call},
};

fn use_to_have_length(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toHaveLength()`.").with_label(span)
}

pub const DOCUMENTATION: &str = r#"### What it does

In order to have a better failure message, `toHaveLength()` should be used upon
asserting expectations on objects length property.

### Why is this bad?

This rule triggers a warning if `toBe()`, `toEqual()` or `toStrictEqual()` is
used to assert objects length property.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(files["length"]).toBe(1);
expect(files["length"]).toBe(1,);
expect(files["length"])["not"].toBe(1)
```

Examples of **correct** code for this rule:
```javascript
expect(files).toHaveLength(1);
```
"#;

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(parsed_expect_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
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
            check_and_fix(call_expr, expr_call_expr, &parsed_expect_call, Some(mem_expr), ctx);
        }
        Expression::CallExpression(expr_call_expr) => {
            check_and_fix(call_expr, expr_call_expr, &parsed_expect_call, None, ctx);
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
        let code = build_code(fixer.source_text(), static_mem_expr, super_mem_expr);
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
