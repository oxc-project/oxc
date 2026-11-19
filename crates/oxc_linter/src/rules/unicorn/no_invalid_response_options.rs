use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectPropertyKind, PropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{StringToNumber, ToInt32};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::GetSpan;
use oxc_syntax::{number::ToJsString, operator::UnaryOperator};

use crate::{AstNode, LintContext, ast_util::variable_declaration_kind, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct NoInvalidResponseOptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow invalid status and body combinations in `Response` APIs.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid response statuses and bodies throw at runtime.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new Response("body", { status: 204 });
    /// Response.redirect(url, 200);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// new Response(undefined, { status: 204 });
    /// Response.redirect(url, 302);
    /// ```
    NoInvalidResponseOptions,
    unicorn,
    nursery,
    suggestion,
    version = "next",
    short_description = "Disallow invalid options in Response APIs.",
);

// Only values whose conversion cannot invoke user code are represented here.
#[derive(Clone)]
enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    UnpairedString,
    BigInt(bool),
    NonNullish,
}

impl Value {
    fn number(&self) -> Option<f64> {
        match self {
            Self::Undefined | Self::UnpairedString => Some(f64::NAN),
            Self::Null => Some(0.0),
            Self::Bool(value) => Some(f64::from(*value)),
            Self::Number(value) => Some(*value),
            Self::String(value) => Some(value.as_str().string_to_number()),
            Self::BigInt(_) | Self::NonNullish => None,
        }
    }

    fn truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Bool(value) => *value,
            Self::Number(value) => *value != 0.0 && !value.is_nan(),
            Self::String(value) => !value.is_empty(),
            Self::BigInt(nonzero) => *nonzero,
            Self::UnpairedString | Self::NonNullish => true,
        }
    }
}

// Resolve at most one immutable initializer and only after its declaration.
fn value<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>, resolve: bool) -> Option<Value> {
    let expr = expr.get_inner_expression();
    match expr {
        Expression::NullLiteral(_) => Some(Value::Null),
        Expression::BooleanLiteral(lit) => Some(Value::Bool(lit.value)),
        Expression::NumericLiteral(lit) => Some(Value::Number(lit.value)),
        Expression::StringLiteral(lit) => {
            Some(lit.value.as_str().map_or(Value::UnpairedString, |s| Value::String(s.to_owned())))
        }
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::RegExpLiteral(_) => Some(Value::NonNullish),
        Expression::BigIntLiteral(lit) => Some(Value::BigInt(!lit.is_zero())),
        Expression::Identifier(id) => {
            if id.is_global_reference(ctx.scoping()) {
                return match id.name.as_str() {
                    "undefined" => Some(Value::Undefined),
                    "NaN" => Some(Value::Number(f64::NAN)),
                    "Infinity" => Some(Value::Number(f64::INFINITY)),
                    _ => None,
                };
            }
            if !resolve {
                return None;
            }
            let symbol = ctx.scoping().get_reference(id.reference_id()).symbol_id()?;
            let decl = ctx.symbol_declaration(symbol);
            let AstKind::VariableDeclarator(declarator) = decl.kind() else {
                return None;
            };
            if !declarator.id.is_binding_identifier()
                || !variable_declaration_kind(declarator, ctx.nodes())?.is_const()
                || declarator.span.end >= id.span.start
            {
                return None;
            }
            value(declarator.init.as_ref()?, ctx, false)
        }
        Expression::UnaryExpression(unary) => {
            // `void` always produces undefined, even if its operand has effects.
            if unary.operator == UnaryOperator::Void {
                return Some(Value::Undefined);
            }
            let arg = unary.argument.get_inner_expression();
            if !matches!(
                arg,
                Expression::NullLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::StringLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::Identifier(_)
            ) {
                return None;
            }
            let v = value(arg, ctx, false)?;
            match unary.operator {
                UnaryOperator::UnaryPlus => Some(Value::Number(v.number()?)),
                UnaryOperator::UnaryNegation => match v {
                    Value::BigInt(nonzero) => Some(Value::BigInt(nonzero)),
                    _ => Some(Value::Number(-v.number()?)),
                },
                UnaryOperator::LogicalNot => Some(Value::Bool(!v.truthy())),
                UnaryOperator::BitwiseNot => {
                    Some(Value::Number(f64::from(!v.number()?.to_int_32())))
                }
                UnaryOperator::Typeof => Some(Value::String(
                    match v {
                        Value::Undefined => "undefined",
                        Value::Null | Value::NonNullish => "object",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) | Value::UnpairedString => "string",
                        Value::BigInt(_) => "bigint",
                    }
                    .to_owned(),
                )),
                _ => None,
            }
        }
        Expression::TemplateLiteral(template) => {
            let mut result = String::new();
            let mut unpaired = false;
            for (index, quasi) in template.quasis.iter().enumerate() {
                if let Some(s) = quasi.value.cooked?.as_str() {
                    result.push_str(s);
                } else {
                    unpaired = true;
                }
                if let Some(expression) = template.expressions.get(index) {
                    let part = match expression.get_inner_expression() {
                        Expression::StringLiteral(lit) => {
                            if let Some(s) = lit.value.as_str() {
                                s.to_owned()
                            } else {
                                unpaired = true;
                                String::new()
                            }
                        }
                        Expression::NumericLiteral(lit) => lit.value.to_js_string(),
                        Expression::BooleanLiteral(lit) => lit.value.to_string(),
                        Expression::BigIntLiteral(lit) => lit.value.to_string(),
                        Expression::NullLiteral(_) => "null".to_owned(),
                        _ => return None,
                    };
                    result.push_str(&part);
                }
            }
            Some(if unpaired { Value::UnpairedString } else { Value::String(result) })
        }
        _ => None,
    }
}

fn status<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>, default: u16) -> Option<u16> {
    let v = value(expr, ctx, true)?;
    if matches!(v, Value::Undefined) {
        return Some(default);
    }
    let number = v.number()?;
    if !number.is_finite() {
        return Some(0);
    }
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(number.trunc().rem_euclid(65536.0) as u16)
}

fn status_property<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let Expression::ObjectExpression(object) = expr.get_inner_expression() else {
        return None;
    };
    for property in object.properties.iter().rev() {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        let is_status = if property.computed {
            match value(property.key.as_expression()?, ctx, true)? {
                Value::String(name) => name == "status",
                Value::UnpairedString | Value::Number(_) => false,
                _ => return None,
            }
        } else {
            property.key.static_name().is_some_and(|name| name == "status")
        };
        if is_status {
            return (property.kind == PropertyKind::Init && !property.method)
                .then_some(&property.value);
        }
    }
    None
}

impl Rule for NoInvalidResponseOptions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (args, constructor, redirect) = match node.kind() {
            AstKind::NewExpression(new) => {
                let Expression::Identifier(id) = new.callee.get_inner_expression() else {
                    return;
                };
                if id.name != "Response" || !id.is_global_reference(ctx.scoping()) {
                    return;
                }
                (&new.arguments, true, false)
            }
            AstKind::CallExpression(call) => {
                if call.optional {
                    return;
                }
                let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression()
                else {
                    return;
                };
                if member.optional {
                    return;
                }
                let Expression::Identifier(id) = member.object.get_inner_expression() else {
                    return;
                };
                if id.name != "Response" || !id.is_global_reference(ctx.scoping()) {
                    return;
                }
                match member.property.name.as_str() {
                    "json" => (&call.arguments, false, false),
                    "redirect" => (&call.arguments, false, true),
                    _ => return,
                }
            }
            _ => return,
        };
        let Some(body) = args.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        let Some(options) = args.get(1).and_then(|arg| arg.as_expression()) else {
            return;
        };
        if redirect {
            if status(options, ctx, 302).is_some_and(|s| !matches!(s, 301 | 302 | 303 | 307 | 308))
            {
                ctx.diagnostic(
                    OxcDiagnostic::warn(
                        "`Response.redirect()` status must be 301, 302, 303, 307, or 308.",
                    )
                    .with_label(options.span()),
                );
            }
            return;
        }
        let Some(property) = status_property(options, ctx) else {
            return;
        };
        let Some(status) = status(property, ctx, 200) else {
            return;
        };
        if !(200..=599).contains(&status) {
            ctx.diagnostic(
                OxcDiagnostic::warn("Response status must be in the range 200 to 599.")
                    .with_label(property.span()),
            );
            return;
        }
        if !matches!(status, 204 | 205 | 304) {
            return;
        }
        let body_value = value(body, ctx, true);
        let definitely_body =
            matches!(
                body.get_inner_expression(),
                Expression::ObjectExpression(_)
                    | Expression::ArrayExpression(_)
                    | Expression::NewExpression(_)
                    | Expression::TemplateLiteral(_)
                    | Expression::RegExpLiteral(_)
            ) || body_value.as_ref().is_some_and(|v| !matches!(v, Value::Null | Value::Undefined));
        if constructor && !definitely_body {
            return;
        }
        let span = if constructor { body.span() } else { property.span() };
        let diagnostic =
            OxcDiagnostic::warn(format!("A response with status {status} cannot have a body."))
                .with_label(span);
        if constructor
            && body_value.is_some_and(|v| {
                matches!(
                    v,
                    Value::String(_)
                        | Value::UnpairedString
                        | Value::Number(_)
                        | Value::Bool(_)
                        | Value::BigInt(_)
                )
            })
            && ctx.comments_range(span.start..span.end).next().is_none()
        {
            ctx.diagnostic_with_suggestion(
                diagnostic.with_help("Replace the response body with `void 0`."),
                |fixer| fixer.replace(span, "void 0"),
            );
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function f(Response) { return new Response('', {status: 204}); }",
        "function f(undefined) { return new Response(undefined, {status: 204}); }",
        "new Response('', {status: 204, ...options})",
        "new Response('', {status: 204, [key]: 200})",
        "new Response('', {status: 204, get status() { return 200; }})",
        "new Response('', {status: 204, status: 200})",
        "Response.redirect(url, 302.9)",
        "Response.redirect(url, 65838)",
        "new Response('', {status: 599.9})",
        "new Response()",
        r#"new Response("")"#,
        r#"new Response("", {})"#,
        "Response.json(data)",
        "Response.json(data, {})",
        "Response.redirect(url)",
        "Response.redirect(url, undefined)",
        "Response.redirect(url, void sideEffect())",
        r#"Response.redirect(url, "302")"#,
        "const status = +302; Response.redirect(url, status)",
        "new Response(undefined, {status: -65_336})",
        "const body = null; new Response(body, {status: 204})",
        "const body = undefined; new Response(body, {status: 204})",
        "new Response(body, {status: 204})",
        "new Response(getBody(), {status: 204})",
        r#"new Response(condition ? null : "", {status: 204})"#,
        "new Response(JSON.stringify(data), {status: 204})",
        r#"let status = 204; status = 200; new Response("", {status})"#,
        "let status = 200; status = 204; Response.json(data, {status})",
        r#"const options = {status: 204}; options.status = 200; new Response("", options)"#,
        r#"const object = {status: 204}; object.status = 200; new Response("", {status: object.status})"#,
        r#"new Response("", {status: (sideEffect(), 204)})"#,
        r#"const alias = condition; var condition = true; new Response("", {status: alias ? 204 : 200})"#,
        r#"new Response("", {status}); const status = 204;"#,
        r#"new Response(body, {status: 204}); const body = "";"#,
        r#"new Response("", {[key]: 204}); const key = "status";"#,
        r#"new Response("", {status: 204, [key]: 200}); const key = "headers";"#,
        r#"let body = ""; body = null; new Response(body, {status: 204})"#,
        "const status = getStatus(); Response.redirect(url, status)",
        "Response.redirect(url, status)",
        "Response.redirect(url, {})",
        "Response.redirect(url, 200n)",
        "Response.redirect(url, Symbol())",
        r#"Response("", {status: 204})"#,
        r#"new OtherResponse("", {status: 204})"#,
        r#"new globalThis.Response("", {status: 204})"#,
        "OtherResponse.json(data, {status: 204})",
        "globalThis.Response.json(data, {status: 204})",
        "new Response.json(data, {status: 204})",
        "Response.json?.(data, {status: 204})",
        "Response?.json(data, {status: 204})",
        r#"Response["json"](data, {status: 204})"#,
        "Response.redirect?.(url, 200)",
        "Response?.redirect(url, 200)",
        r#"Response["redirect"](url, 200)"#,
        "new Response(...bodies, {status: 204})",
        r#"new Response("", ...options)"#,
        "Response.json(...data, {status: 204})",
        "Response.json(data, ...options)",
        "Response.redirect(...urls, 200)",
        "Response.redirect(url, ...statuses)",
        r#"new Response("", {statusText: "\n"})"#,
        r#"Response.json(data, {headers: {"invalid header": "value"}})"#,
        r#"Response.redirect("http://:", 302)"#,
        r#"new Response(true ? "" : null, {status: 204})"#,
        "const condition = true; Response.json(data, {status: condition ? 204 : 200})",
        r#"const originalStatus = 204; const status = originalStatus; new Response("", {status})"#,
        r#"const object = {}; mutate(object); new Response(true ? `${object}` : "", {status: 204})"#,
        r#"const object = {}; mutate(object); new Response((`${object}`, "body"), {status: 204})"#,
        r#"const object = {}; Object.defineProperty(object, "valueOf", {value() { return 302; }}); Response.redirect(url, +object)"#,
        r#"const object = {}; Object.defineProperty(object, "toString", {value() { return "302"; }}); Response.redirect(url, `${object}`)"#,
        r#"const object = {}; Object.defineProperty(object, "valueOf", {value() { return 302; }}); const status = +object; Response.redirect(url, status)"#,
        r#"const object = {}; Object.defineProperty(object, "valueOf", {value() { return 1; }}); new Response(+object ? null : "", {status: 204})"#,
        r#"const object = {}; Object.defineProperty(object, "toString", {value() { return "status"; }}); new Response("", {status: 204, [`${object}`]: 200})"#,
        r#"const status = undefined; new Response("", {status})"#,
    ];

    let fail = vec![
        r"new Response('x', {status: '\uD800'})",
        r"new Response('\uD800', {status: 204})",
        "const body = {}; new Response(body, {status: 204})",
        "const body = []; new Response(body, {status: 204})",
        "const body = /x/; new Response(body, {status: 204})",
        "new Response(-1n, {status: 204})",
        "new Response(!0n, {status: 204})",
        "new Response('', {status: 200, status: 204})",
        "new Response('', {...options, status: 204})",
        "new Response('', {[key]: 200, status: 204})",
        "new Response('', {get status() { return 200; }, status: 204})",
        "new Response('', {status: 204, headers: {}})",
        "new Response([], {status: 205})",
        "new Response(new Blob([]), {status: 304})",
        "new Response(undefined, {status: null})",
        "new Response(undefined, {status: false})",
        "new Response(undefined, {status: true})",
        "new Response(undefined, {status: -1})",
        "new Response(undefined, {status: Infinity})",
        "new Response(undefined, {status: 65536})",
        "new Response(undefined, {status: 'invalid'})",
        "Response.redirect(url, `${200}`)",
        "Response.json(null, {status: 304})",
        "Response.json(undefined, {status: 204})",
        r#"const body = ""; new Response(body, {status: 204})"#,
        r#"new Response("", {status: "204"})"#,
        r#"new Response("", {status: 204.9})"#,
        r#"new Response("", {status: +204})"#,
        r#"new Response("", {status: 65_740})"#,
        r#"const status = 205; new Response("", {status})"#,
        "const status = 304; Response.json(data, {status})",
        "const status = 200; Response.redirect(url, status)",
        "Response.redirect(url, null)",
        "Response.redirect(url, false)",
        "Response.redirect(url, NaN)",
        r#"Response.redirect(url, "200")"#,
        r#"const key = "status"; new Response("", {[key]: 204})"#,
        r#"new Response("", {[`status`]: 204})"#,
        r#"new Response("", {status: 204, [0]: 200})"#,
        "Response.redirect(url, 200, ...extraArguments)",
        r#"new ((Response))((("")), (({status: 204})))"#,
        "Response.json(data, {status: 304}).clone()",
        r#"consume(new Response("", {status: 204}))"#,
        r#"new Response(/* before */ "" /* after */, {status: 204})"#,
        "new Response({/* keep */}, {status: 204})",
        "new Response(`body ${object.value}`, {status: 204})",
        "new Response(`body ${sideEffect()}`, {status: 204})",
        r#"<Component response={new Response("", {status: 204})} />"#, // {"parserOptions": {"ecmaFeatures": {"jsx": true}}},
        r#"new Response("", {status: 204});"#,
        "new Response(`body ${1}`, {status: 204});",
        "new Response(1n, {status: 204});",
        "new Response(/body/, {status: 204});",
        r#"const object = {}; Object.defineProperty(object, "toString", {get() { sideEffect(); return String; }}); new Response(`${object}`, {status: 204})"#,
    ];

    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, pass, fail)
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}

#[test]
fn typescript_and_suggestions() {
    use crate::{fixer::FixKind, tester::Tester};
    let pass = vec![
        "new Response(null as BodyInit | null, {status: 204} as ResponseInit)",
        "new Response(body as string, {status: 204})",
        "Response.json(data, {status: (200 as number)} satisfies ResponseInit)",
    ];
    let fail = vec![
        "new Response('' as string, {status: 204} as ResponseInit)",
        "new Response(<string>'', <ResponseInit>{status: <number>204})",
        "new Response('' satisfies string, {status: 204} satisfies ResponseInit)",
        "new Response(''!, {status: 204}!)",
        "Response.json(data, {['status' as string]: 304})",
        "new Response('', {status: +(204 as number)})",
        "Response.redirect(url, `${200 as number}`)",
        "new Response({/* keep */}, {status: 204})",
        "new Response('' as /* keep */ string, {status: 204})",
    ];
    let fixes = vec![
        (
            "new Response('', {status: 204})",
            "new Response(void 0, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Response(`body ${1}`, {status: 204})",
            "new Response(void 0, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Response(1n, {status: 204})",
            "new Response(void 0, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Response({/* keep */}, {status: 204})",
            "new Response({/* keep */}, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Response('' as /* keep */ string, {status: 204})",
            "new Response('' as /* keep */ string, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
    ];
    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("typescript")
        .expect_fix(fixes)
        .test_and_snapshot();
}

#[test]
fn status_matrix() {
    use crate::tester::Tester;
    let mut pass = Vec::new();
    let mut fail = Vec::new();
    for constructor in ["new Response", "Response.json"] {
        for status in ["200", "299", "300", "404", "599", "599.9", "65736"] {
            pass.push(format!("{constructor}('', {{status: {status}}})"));
        }
        for status in [
            "0",
            "101",
            "103",
            "199",
            "600",
            "65536",
            "-1",
            "NaN",
            "Infinity",
            "-Infinity",
            "null",
            "false",
            "true",
            "'invalid'",
        ] {
            fail.push(format!("{constructor}(undefined, {{status: {status}}})"));
        }
        for status in [204, 205, 304] {
            fail.push(format!("{constructor}('', {{status: {status}}})"));
        }
    }
    for status in ["301", "302", "303", "307", "308", "302.9", "65838"] {
        pass.push(format!("Response.redirect(url, {status})"));
    }
    for status in ["200", "300", "304", "305", "306", "309", "600"] {
        fail.push(format!("Response.redirect(url, {status})"));
    }
    for status in [204, 205, 304] {
        for body in ["undefined", "null", "void sideEffect()"] {
            pass.push(format!("new Response({body}, {{status: {status}}})"));
        }
    }
    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, pass, fail)
        .intentionally_allow_no_fix_tests()
        .test();
}

// Cases expanded from https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v79.0.0/test/no-invalid-response-options.js
#[test]
fn upstream_v79_batch_0() {
    use crate::tester::Tester;
    let valid: Vec<&str> = vec![
        r"new Response()",
        r#"new Response("")"#,
        r#"new Response("", {})"#,
        r"Response.json(data)",
        r"Response.json(data, {})",
        r"Response.redirect(url)",
        r"Response.redirect(url, undefined)",
        r"Response.redirect(url, void sideEffect())",
        r#"new Response("", {status: 200})"#,
        r#"new Response("", {status: 299})"#,
        r#"new Response("", {status: 300})"#,
        r#"new Response("", {status: 404})"#,
        r#"new Response("", {status: 599})"#,
        r#"new Response("", {status: 599.9})"#,
        r#"new Response("", {status: 65736})"#,
        r#"Response.json("", {status: 200})"#,
        r#"Response.json("", {status: 299})"#,
        r#"Response.json("", {status: 300})"#,
        r#"Response.json("", {status: 404})"#,
        r#"Response.json("", {status: 599})"#,
        r#"Response.json("", {status: 599.9})"#,
        r#"Response.json("", {status: 65736})"#,
        r"Response.redirect(url, 301)",
        r"Response.redirect(url, 302)",
        r"Response.redirect(url, 303)",
        r"Response.redirect(url, 307)",
        r"Response.redirect(url, 308)",
        r"Response.redirect(url, 302.9)",
        r"Response.redirect(url, 65838)",
        r#"Response.redirect(url, "302")"#,
        r"const status = +302; Response.redirect(url, status)",
        r"new Response(undefined, {status: -65_336})",
        r#"new Response("", {status: undefined})"#,
        r#"new Response("", {status: void 0})"#,
        r#"new Response("", {status: "200"})"#,
        r#"new Response("", {status})"#,
        r#"new Response("", options)"#,
        r#"new Response("", {status: {valueOf() { return 204; }}})"#,
        r#"new Response("", {status: 204n})"#,
        r#"new Response("", {status: Symbol()})"#,
        r#"new Response("", {status: 204, status: 200})"#,
        r#"new Response("", {status: 204, ...options})"#,
        r#"new Response("", {status: 204, [key]: 200})"#,
        r#"new Response("", {get status() { return 204; }})"#,
        r#"new Response("", {status: 204, set status(value) {}})"#,
        r#"new Response("", {status() { return 204; }})"#,
        r#"new Response("", {status: 204, get status() { return 200; }})"#,
        r#"new Response("", {...{status: 204}})"#,
        r#"Response.json("", {status: undefined})"#,
        r#"Response.json("", {status: void 0})"#,
        r#"Response.json("", {status: "200"})"#,
        r#"Response.json("", {status})"#,
        r#"Response.json("", options)"#,
        r#"Response.json("", {status: {valueOf() { return 204; }}})"#,
        r#"Response.json("", {status: 204n})"#,
        r#"Response.json("", {status: Symbol()})"#,
        r#"Response.json("", {status: 204, status: 200})"#,
        r#"Response.json("", {status: 204, ...options})"#,
        r#"Response.json("", {status: 204, [key]: 200})"#,
        r#"Response.json("", {get status() { return 204; }})"#,
        r#"Response.json("", {status: 204, set status(value) {}})"#,
        r#"Response.json("", {status() { return 204; }})"#,
        r#"Response.json("", {status: 204, get status() { return 200; }})"#,
        r#"Response.json("", {...{status: 204}})"#,
        r"new Response(undefined, {status: 204})",
        r"new Response(null, {status: 204})",
        r"new Response(void sideEffect(), {status: 204})",
        r"new Response(undefined, {status: 205})",
        r"new Response(null, {status: 205})",
        r"new Response(void sideEffect(), {status: 205})",
        r"new Response(undefined, {status: 304})",
        r"new Response(null, {status: 304})",
        r"new Response(void sideEffect(), {status: 304})",
        r"const body = null; new Response(body, {status: 204})",
        r"const body = undefined; new Response(body, {status: 204})",
        r"new Response(body, {status: 204})",
        r"new Response(getBody(), {status: 204})",
        r#"new Response(condition ? null : "", {status: 204})"#,
        r"new Response(JSON.stringify(data), {status: 204})",
        r#"let status = 204; status = 200; new Response("", {status})"#,
        r"let status = 200; status = 204; Response.json(data, {status})",
        r#"const options = {status: 204}; options.status = 200; new Response("", options)"#,
        r#"const object = {status: 204}; object.status = 200; new Response("", {status: object.status})"#,
        r#"new Response("", {status: (sideEffect(), 204)})"#,
        r#"const alias = condition; var condition = true; new Response("", {status: alias ? 204 : 200})"#,
        r#"new Response("", {status}); const status = 204;"#,
        r#"new Response(body, {status: 204}); const body = "";"#,
        r#"new Response("", {[key]: 204}); const key = "status";"#,
        r#"new Response("", {status: 204, [key]: 200}); const key = "headers";"#,
        r#"let body = ""; body = null; new Response(body, {status: 204})"#,
        r"const status = getStatus(); Response.redirect(url, status)",
        r"Response.redirect(url, status)",
        r"Response.redirect(url, {})",
        r"Response.redirect(url, 200n)",
        r"Response.redirect(url, Symbol())",
        r#"Response("", {status: 204})"#,
        r#"new OtherResponse("", {status: 204})"#,
        r#"new globalThis.Response("", {status: 204})"#,
        r"OtherResponse.json(data, {status: 204})",
        r"globalThis.Response.json(data, {status: 204})",
        r"new Response.json(data, {status: 204})",
        r"Response.json?.(data, {status: 204})",
        r"Response?.json(data, {status: 204})",
        r#"Response["json"](data, {status: 204})"#,
        r"Response.redirect?.(url, 200)",
        r"Response?.redirect(url, 200)",
        r#"Response["redirect"](url, 200)"#,
        r"new Response(...bodies, {status: 204})",
        r#"new Response("", ...options)"#,
        r"Response.json(...data, {status: 204})",
        r"Response.json(data, ...options)",
        r"Response.redirect(...urls, 200)",
        r"Response.redirect(url, ...statuses)",
        r#"new Response("", {statusText: "\n"})"#,
        r#"Response.json(data, {headers: {"invalid header": "value"}})"#,
        r#"Response.redirect("http://:", 302)"#,
        r#"new Response(true ? "" : null, {status: 204})"#,
        r"const condition = true; Response.json(data, {status: condition ? 204 : 200})",
        r#"const originalStatus = 204; const status = originalStatus; new Response("", {status})"#,
        r#"const object = {}; mutate(object); new Response(true ? `${object}` : "", {status: 204})"#,
        r#"const object = {}; mutate(object); new Response((`${object}`, "body"), {status: 204})"#,
        r#"const object = {}; Object.defineProperty(object, "valueOf", {value() { return 302; }}); Response.redirect(url, +object)"#,
        r#"const object = {}; Object.defineProperty(object, "toString", {value() { return "302"; }}); Response.redirect(url, `${object}`)"#,
        r#"const object = {}; Object.defineProperty(object, "valueOf", {value() { return 302; }}); const status = +object; Response.redirect(url, status)"#,
        r#"const object = {}; Object.defineProperty(object, "valueOf", {value() { return 1; }}); new Response(+object ? null : "", {status: 204})"#,
        r#"const object = {}; Object.defineProperty(object, "toString", {value() { return "status"; }}); new Response("", {status: 204, [`${object}`]: 200})"#,
        r#"const status = undefined; new Response("", {status})"#,
    ];
    let invalid: Vec<&str> = vec![
        r#"new Response("", {status: 204})"#,
        r"Response.json(data, {status: 204})",
        r#"new Response("", {status: 205})"#,
        r"Response.json(data, {status: 205})",
        r#"new Response("", {status: 304})"#,
        r"Response.json(data, {status: 304})",
        r"Response.json(null, {status: 304})",
        r"Response.json(undefined, {status: 204})",
        r#"new Response("body", {status: 204})"#,
        r"new Response(false, {status: 204})",
        r"new Response(0, {status: 204})",
        r"new Response([], {status: 204})",
        r"new Response({}, {status: 204})",
        r"new Response(new Uint8Array(), {status: 204})",
        r"new Response(new Blob([]), {status: 204})",
        r"new Response(`body ${value}`, {status: 204})",
        r"new Response([sideEffect()], {status: 204})",
        r"new Response({value: object.value}, {status: 204})",
        r#"const body = ""; new Response(body, {status: 204})"#,
        r#"new Response("", {status: "204"})"#,
        r#"new Response("", {status: 204.9})"#,
        r#"new Response("", {status: +204})"#,
        r#"new Response("", {status: 65_740})"#,
        r#"const status = 205; new Response("", {status})"#,
        r"const status = 304; Response.json(data, {status})",
        r"const status = 200; Response.redirect(url, status)",
        r"new Response(undefined, {status: 0})",
        r"new Response(undefined, {status: 101})",
        r"new Response(undefined, {status: 103})",
        r"new Response(undefined, {status: 199})",
        r"new Response(undefined, {status: 600})",
        r"new Response(undefined, {status: 65_536})",
        r"new Response(undefined, {status: -1})",
        r"new Response(undefined, {status: NaN})",
        r"new Response(undefined, {status: Infinity})",
        r"new Response(undefined, {status: -Infinity})",
        r"new Response(undefined, {status: null})",
        r"new Response(undefined, {status: false})",
        r"new Response(undefined, {status: true})",
        r#"new Response(undefined, {status: "invalid"})"#,
        r"Response.json(undefined, {status: 0})",
        r"Response.json(undefined, {status: 101})",
        r"Response.json(undefined, {status: 103})",
        r"Response.json(undefined, {status: 199})",
        r"Response.json(undefined, {status: 600})",
        r"Response.json(undefined, {status: 65_536})",
        r"Response.json(undefined, {status: -1})",
        r"Response.json(undefined, {status: NaN})",
        r"Response.json(undefined, {status: Infinity})",
        r"Response.json(undefined, {status: -Infinity})",
        r"Response.json(undefined, {status: null})",
        r"Response.json(undefined, {status: false})",
        r"Response.json(undefined, {status: true})",
        r#"Response.json(undefined, {status: "invalid"})"#,
        r"Response.redirect(url, 200)",
        r"Response.redirect(url, 300)",
        r"Response.redirect(url, 304)",
        r"Response.redirect(url, 305)",
        r"Response.redirect(url, 306)",
        r"Response.redirect(url, 309)",
        r"Response.redirect(url, 600)",
        r"Response.redirect(url, null)",
        r"Response.redirect(url, false)",
        r"Response.redirect(url, NaN)",
        r#"Response.redirect(url, "200")"#,
        r#"const key = "status"; new Response("", {[key]: 204})"#,
        r#"new Response("", {[`status`]: 204})"#,
        r#"new Response("", {status: 204, [0]: 200})"#,
        r#"new Response("", {status: 200, status: 204})"#,
        r#"new Response("", {get status() { return 200; }, status: 204})"#,
        r#"new Response("", {...options, status: 204})"#,
        r#"new Response("", {[key]: 200, status: 204})"#,
        r#"new Response("", {"status": 204})"#,
        r#"new Response("", {["status"]: 204})"#,
        r#"new Response("", {status: 204, headers: {}})"#,
        r#"new Response("", {status: 204}, ...extraArguments)"#,
        r#"Response.json("", {status: 200, status: 204})"#,
        r#"Response.json("", {get status() { return 200; }, status: 204})"#,
        r#"Response.json("", {...options, status: 204})"#,
        r#"Response.json("", {[key]: 200, status: 204})"#,
        r#"Response.json("", {"status": 204})"#,
        r#"Response.json("", {["status"]: 204})"#,
        r#"Response.json("", {status: 204, headers: {}})"#,
        r#"Response.json("", {status: 204}, ...extraArguments)"#,
        r"Response.redirect(url, 200, ...extraArguments)",
        r#"new ((Response))((("")), (({status: 204})))"#,
        r"Response.json(data, {status: 304}).clone()",
        r#"consume(new Response("", {status: 204}))"#,
        r#"new Response(/* before */ "" /* after */, {status: 204})"#,
        r"new Response({/* keep */}, {status: 204})",
        r"new Response(`body ${object.value}`, {status: 204})",
        r"new Response(`body ${sideEffect()}`, {status: 204})",
    ];
    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, valid, invalid)
        .change_rule_path_extension("tsx")
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("upstream_v79_0")
        .test_and_snapshot();
}

#[test]
fn upstream_v79_batch_1() {
    use crate::tester::Tester;
    let valid: Vec<&str> = vec![
        r"new Response(null as BodyInit | null, {status: 204} as ResponseInit)",
        r"new Response(undefined!, {status: 204})",
        r"new Response(body as string, {status: 204})",
        r"Response.json(data, {status: (200 as number)} satisfies ResponseInit)",
        r"const status = +(302 as number); Response.redirect(url, status)",
        r#"const object = {}; Object.defineProperty(object, "toString", {value() { return "302"; }}); Response.redirect(url, `${object as string}`)"#,
    ];
    let invalid: Vec<&str> = vec![
        r#"new Response("" as string, {status: 204} as ResponseInit)"#,
        r#"new Response(<string>"", <ResponseInit>{status: <number>204})"#,
        r#"new Response("" satisfies string, {status: 204} satisfies ResponseInit)"#,
        r#"new Response(""!, {status: 204}!)"#,
        r#"new Response(("" as string)!, {status: (204 as number)!})"#,
        r#"new Response("" as /* keep */ string, {status: 204})"#,
        r"Response.json(data, {status: 304} as ResponseInit)",
        r"Response.redirect(url, 200 as number)",
        r#"Response.json(data, {["status" as string]: 304})"#,
        r#"new Response("", {status: +(204 as number)})"#,
        r"Response.json(data, {status: -(1 as number)})",
        r"Response.redirect(url, `${200 as number}`)",
        r#"const status = +(204 satisfies number); new Response("", {status})"#,
        r"Response.redirect(url, -(Infinity as number))",
    ];
    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, valid, invalid)
        .change_rule_path_extension("ts")
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("upstream_v79_1")
        .test_and_snapshot();
}

#[test]
fn upstream_v79_batch_2() {
    use crate::tester::Tester;
    let valid: Vec<&str> = vec![];
    let invalid: Vec<&str> = vec![r#"<Component response={new Response("", {status: 204})} />"#];
    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, valid, invalid)
        .change_rule_path_extension("tsx")
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("upstream_v79_2")
        .test_and_snapshot();
}

#[test]
fn upstream_v79_batch_3() {
    use crate::{fixer::FixKind, tester::Tester};
    let valid: Vec<&str> = vec![];
    let invalid: Vec<&str> = vec![
        r#"new Response("", {status: 204});"#,
        r"new Response(`body ${1}`, {status: 204});",
        r"new Response(1n, {status: 204});",
        r"new Response(/body/, {status: 204});",
        r#"const object = {}; Object.defineProperty(object, "toString", {get() { sideEffect(); return String; }}); new Response(`${object}`, {status: 204})"#,
        r"new Response({}, {status: 204})",
        r"new Response([], {status: 204})",
        r"new Response(new Blob([]), {status: 204})",
        r"new Response([sideEffect()], {status: 204})",
        r"new Response({value: object.value}, {status: 204})",
        r"new Response(`body ${object}`, {status: 204})",
        r"new Response({} as /* keep */ BodyInit, {status: 204})",
        r#"new Response({toString() { sideEffect(); return "body"; }}, {status: 204})"#,
        r"new Response({get toString() { sideEffect(); return String; }}, {status: 204})",
        r"new Response([object], {status: 204})",
    ];
    let fixes = vec![
        (
            r#"new Response("", {status: 204});"#,
            r"new Response(void 0, {status: 204});",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response(`body ${1}`, {status: 204});",
            r"new Response(void 0, {status: 204});",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response(1n, {status: 204});",
            r"new Response(void 0, {status: 204});",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response(/body/, {status: 204});",
            r"new Response(/body/, {status: 204});",
            None,
            FixKind::Suggestion,
        ),
        (
            r#"const object = {}; Object.defineProperty(object, "toString", {get() { sideEffect(); return String; }}); new Response(`${object}`, {status: 204})"#,
            r#"const object = {}; Object.defineProperty(object, "toString", {get() { sideEffect(); return String; }}); new Response(`${object}`, {status: 204})"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response({}, {status: 204})",
            r"new Response({}, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response([], {status: 204})",
            r"new Response([], {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response(new Blob([]), {status: 204})",
            r"new Response(new Blob([]), {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response([sideEffect()], {status: 204})",
            r"new Response([sideEffect()], {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response({value: object.value}, {status: 204})",
            r"new Response({value: object.value}, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response(`body ${object}`, {status: 204})",
            r"new Response(`body ${object}`, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response({} as /* keep */ BodyInit, {status: 204})",
            r"new Response({} as /* keep */ BodyInit, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r#"new Response({toString() { sideEffect(); return "body"; }}, {status: 204})"#,
            r#"new Response({toString() { sideEffect(); return "body"; }}, {status: 204})"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response({get toString() { sideEffect(); return String; }}, {status: 204})",
            r"new Response({get toString() { sideEffect(); return String; }}, {status: 204})",
            None,
            FixKind::Suggestion,
        ),
        (
            r"new Response([object], {status: 204})",
            r"new Response([object], {status: 204})",
            None,
            FixKind::Suggestion,
        ),
    ];
    Tester::new(NoInvalidResponseOptions::NAME, NoInvalidResponseOptions::PLUGIN, valid, invalid)
        .change_rule_path_extension("ts")
        .expect_fix(fixes)
        .with_snapshot_suffix("upstream_v79_3")
        .test_and_snapshot();
}
