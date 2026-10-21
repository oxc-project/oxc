use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Expression, MemberExpression, RegExpFlags, VariableDeclarator,
    },
};
use oxc_codegen::CodegenOptions;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::ast::Term;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;

use crate::{
    AstNode,
    ast_util::{extract_regex_flags, iter_outer_expressions},
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::is_regexp_callee,
};

fn string_literal(span: Span, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("This pattern can be replaced with `{replacement}`."))
        .with_label(span)
}

fn use_replace_all(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `String#replaceAll()` over `String#replace()` when using a regex with the global flag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStringReplaceAll;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers [`String#replaceAll()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll) over [`String#replace()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace) when using a regex with the global flag.
    ///
    /// ### Why is this bad?
    ///
    /// The [`String#replaceAll()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll) method is both faster and safer as you don't have to use a regex and remember to escape it if the string is not a literal. And when used with a regex, it makes the intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo.replace(/a/g, bar)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo.replace(/a/, bar)
    /// foo.replaceAll('a', bar)
    /// foo.replaceAll(/a|b/g, bar)
    ///
    /// const pattern = "not-a-regexp"
    /// foo.replace(pattern, bar)
    /// ```
    PreferStringReplaceAll,
    unicorn,
    pedantic,
    recommended,
    fix,
    version = "0.0.18",
    short_description = "Prefers `String#replaceAll()` over `String#replace()` when using a regex with the global flag.",
);

impl Rule for PreferStringReplaceAll {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => Self::check_call(call, ctx, false),
            AstKind::VariableDeclarator(declarator) => {
                if let Some(init) = &declarator.init
                    && is_reg_exp_with_global_flag(init, ctx)
                {
                    Self::check_variable(declarator, ctx);
                }
            }
            _ => {}
        }
    }
}

impl PreferStringReplaceAll {
    fn check_call<'a>(
        call_expr: &CallExpression<'a>,
        ctx: &LintContext<'a>,
        resolved_pattern: bool,
    ) {
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let MemberExpression::StaticMemberExpression(static_member_expr) = member_expr else {
            return;
        };

        let method_name_str = static_member_expr.property.name.as_str();

        if !matches!(method_name_str, "replace" | "replaceAll") {
            return;
        }

        if call_expr.arguments.len() != 2 {
            return;
        }

        let pattern = &call_expr.arguments[0];
        match method_name_str {
            "replaceAll" => {
                if let Some(k) = get_pattern_replacement(pattern) {
                    ctx.diagnostic_with_fix(string_literal(pattern.span(), &k), |fixer| {
                        // foo.replaceAll(/hello world/g, bar) => foo.replaceAll('hello world', bar)
                        fixer.replace(pattern.span(), generate_string_literal(fixer, &k))
                    });
                }
            }
            "replace"
                if resolved_pattern
                    || pattern
                        .as_expression()
                        .is_some_and(|expr| is_reg_exp_with_global_flag(expr, ctx)) =>
            {
                let diagnostic = use_replace_all(static_member_expr.property.span);

                if let Some(k) = get_pattern_replacement(pattern) {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        let string_literal = generate_string_literal(fixer, &k);

                        let fixer = fixer.for_multifix();
                        let mut fix = fixer.new_fix_with_capacity(2);
                        fix.push(fixer.replace(static_member_expr.property.span, "replaceAll"));
                        fix.push(fixer.replace(pattern.span(), string_literal));
                        fix.with_message("Replace `replace` with `replaceAll`.")
                    });
                } else {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fixer.replace(static_member_expr.property.span, "replaceAll")
                    });
                }
            }
            _ => {}
        }
    }

    fn check_variable<'a>(declarator: &VariableDeclarator<'a>, ctx: &LintContext<'a>) {
        let Some(identifier) = declarator.id.get_binding_identifier() else {
            return;
        };
        let mut pending: smallvec::SmallVec<[SymbolId; 4]> =
            smallvec::smallvec![identifier.symbol_id()];
        while let Some(symbol_id) = pending.pop() {
            if binding_has_writes(symbol_id, ctx) {
                continue;
            }
            for reference in ctx.scoping().get_resolved_references(symbol_id) {
                let reference_span = ctx.nodes().get_node(reference.node_id()).span();
                match iter_outer_expressions(ctx.nodes(), reference.node_id()).next() {
                    Some(AstKind::CallExpression(call)) => {
                        if call.arguments.first().and_then(Argument::as_expression).is_some_and(
                            |pattern| pattern.get_inner_expression().span() == reference_span,
                        ) {
                            Self::check_call(call, ctx, true);
                        }
                    }
                    Some(AstKind::VariableDeclarator(alias)) => {
                        if alias.init.as_ref().is_some_and(|init| {
                            init.get_inner_expression().span() == reference_span
                        }) && let Some(identifier) = alias.id.get_binding_identifier()
                        {
                            pending.push(identifier.symbol_id());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn generate_string_literal(fixer: RuleFixer<'_, '_>, value: &str) -> String {
    let mut codegen =
        fixer.codegen().with_options(CodegenOptions { single_quote: true, ..Default::default() });
    codegen.print_string(value);
    codegen.into_source_text()
}

fn is_reg_exp_with_global_flag<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr.get_inner_expression() {
        Expression::RegExpLiteral(reg_exp_literal) => {
            reg_exp_literal.regex.flags.contains(RegExpFlags::G)
        }
        Expression::NewExpression(new_expr) => {
            is_regexp_callee(&new_expr.callee, ctx)
                && !matches!(new_expr.arguments.first(), Some(Argument::SpreadElement(_)))
                && extract_regex_flags(&new_expr.arguments)
                    .is_some_and(|flags| flags.contains(RegExpFlags::G))
        }
        _ => false,
    }
}

fn binding_has_writes(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    !ctx.scoping().symbol_redeclarations(symbol_id).is_empty()
        || ctx.scoping().get_resolved_references(symbol_id).any(|reference| {
            let reference_span = ctx.nodes().get_node(reference.node_id()).span();
            reference.is_write()
                || iter_outer_expressions(ctx.nodes(), reference.node_id())
                    .find(|kind| match kind {
                        AstKind::StaticMemberExpression(member) => {
                            !member.object.span().contains_inclusive(reference_span)
                        }
                        AstKind::ComputedMemberExpression(member) => {
                            !member.object.span().contains_inclusive(reference_span)
                        }
                        AstKind::PrivateFieldExpression(member) => {
                            !member.object.span().contains_inclusive(reference_span)
                        }
                        _ => true,
                    })
                    .is_some_and(|kind| match kind {
                        AstKind::AssignmentExpression(assignment) => {
                            assignment.left.span().contains_inclusive(reference_span)
                        }
                        AstKind::UpdateExpression(_) => true,
                        _ => false,
                    })
        })
}

fn get_pattern_replacement<'a>(expr: &'a Argument<'a>) -> Option<CompactStr> {
    let Argument::RegExpLiteral(reg_exp_literal) = expr else {
        return None;
    };

    if !reg_exp_literal.regex.flags.contains(RegExpFlags::G) {
        return None;
    }

    if reg_exp_literal.regex.flags.intersects(
        RegExpFlags::I | RegExpFlags::M | RegExpFlags::S | RegExpFlags::D | RegExpFlags::Y,
    ) {
        return None;
    }

    let pattern_terms = reg_exp_literal
        .regex
        .pattern
        .pattern
        .as_deref()
        .filter(|pattern| pattern.body.body.len() == 1)
        .and_then(|pattern| pattern.body.body.first().map(|it| &it.body))?;

    // Convert the regex pattern to a string by extracting character values
    // from the parsed AST instead of using the raw source text.
    // This ensures escape sequences are properly handled.
    let mut result = String::new();
    for term in pattern_terms {
        let Term::Character(ch) = term else {
            return None;
        };

        match char::from_u32(ch.value) {
            Some(c) => result.push(c),
            // Invalid unicode character, fall back to source text
            None => return Some(CompactStr::new(reg_exp_literal.regex.pattern.text.as_str())),
        }
    }

    Some(CompactStr::new(&result))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"foo.replace(/a/, bar)",
        r"foo.replaceAll(/a/, bar)",
        r"foo.replaceAll(/a|b/g, bar)",
        r#"foo.replace("string", bar)"#,
        r#"foo.replaceAll("string", bar)"#,
        r"foo.replace(/a/g)",
        r"foo.replaceAll(/a/g)",
        r"foo.replace(/\\./g)",
        r"foo.replaceAll(/\\./g)",
        r"new foo.replace(/a/g, bar)",
        r"new foo.replaceAll(/a/g, bar)",
        r"replace(/a/g, bar)",
        r"replaceAll(/a/g, bar)",
        r"foo[replace](/a/g, bar);",
        r"foo[replaceAll](/a/g, bar);",
        r"foo.methodNotReplace(/a/g, bar);",
        r"foo['replace'](/a/g, bar)",
        r"foo['replaceAll'](/a/g, bar)",
        r"foo.replace(/a/g, bar, extra);",
        r"foo.replaceAll(/a/g, bar, extra);",
        r"foo.replace();",
        r"foo.replaceAll();",
        r"foo.replace(...argumentsArray, ...argumentsArray2)",
        r"foo.replaceAll(...argumentsArray, ...argumentsArray2)",
        r"foo.replace(unknown, bar)",
        r#"const pattern = new RegExp("foo", unknown); foo.replace(pattern, bar)"#,
        r#"const pattern = new RegExp("foo"); foo.replace(pattern, bar)"#,
        r"const pattern = new RegExp(); foo.replace(pattern, bar)",
        r#"const pattern = "string"; foo.replace(pattern, bar)"#,
        r#"const pattern = new RegExp("foo", "g"); foo.replace(...[pattern], bar)"#,
        r#"const pattern = "not-a-regexp"; foo.replace(pattern, bar)"#,
        r#"const pattern = new RegExp("foo", "i"); foo.replace(pattern, bar)"#,
        r#"foo.replace(new NotRegExp("foo", "g"), bar)"#,
        // https://github.com/oxc-project/oxc/issues/21188
        // Should not suggest replacing regex with string when flags other than g/u/v are present
        r"foo.replaceAll(/foo/gi, bar)",
        r"foo.replaceAll(/foo/gm, bar)",
        r"foo.replaceAll(/foo/gs, bar)",
        r"foo.replaceAll(/foo/gim, bar)",
        r"const pattern = /a/; foo.replace(pattern, bar)",
        r"let pattern = /a/g; pattern = /a/; foo.replace(pattern, bar)",
        r"const pattern = /a/g; foo.replaceAll(pattern, bar)",
        r"const pattern = /a/g; function f(pattern) { foo.replace(pattern, bar); }",
        r"const a = b; const b = a; foo.replace(a, bar)",
        r"const {pattern} = /a/g; foo.replace(pattern, bar)",
        r#"function f(RegExp) { const pattern = new RegExp("foo", "g"); foo.replace(pattern, bar); }"#,
        r"let pattern = /a/g; const alias = pattern; pattern = /a/; foo.replace(alias, bar)",
        r#"var RE = /a/g; var RE = /b/; "aba".replace(RE, "x")"#,
        r#"const RE = /a/g; RE[Symbol.replace] = undefined; "x/a/gx/a/g".replace(RE, "b")"#,
        r"const RE = /a/g; RE.lastIndex++; foo.replace(RE, bar)",
        r"const RE = /a/g; const alias = RE; RE[Symbol.replace] = undefined; foo.replace(alias, bar)",
        r#"const RE = new RegExp(...patterns, "g"); foo.replace(RE, bar)"#,
        r"const RE = /a/g; foo.replace(unknown, RE)",
        r"const RE = /a/g; foo.replace(...RE, bar)",
        r"const RE = /a/g; const alias = RE.source; foo.replace(alias, bar)",
    ];

    let fail = vec![
        r"foo.replace(/a/g, bar)",
        r#"foo.replace(/"'/g, '\'')"#,
        r"foo.replace(/\./g, bar)",
        r"foo.replace(/\\\./g, bar)",
        r"foo.replace(/\|/g, bar)",
        r"foo.replace(/a/gu, bar)",
        r"foo.replace(/a/ug, bar)",
        r"foo.replace(/[a]/g, bar)",
        r"foo.replace(/a?/g, bar)",
        r"foo.replace(/.*/g, bar)",
        r"foo.replace(/a|b/g, bar)",
        r"foo.replace(/\W/g, bar)",
        r"foo.replace(/\u{61}/g, bar)",
        r"foo.replace(/\u{61}/gu, bar)",
        r"foo.replace(/\u{61}/gv, bar)",
        r#"foo.replace(/]/g, "bar")"#,
        r"foo.replace(/a/gi, bar)",
        r"foo.replace(/a/gui, bar)",
        r"foo.replace(/a/uig, bar)",
        r"foo.replace(/a/vig, bar)",
        r#"foo.replace(new RegExp("foo", "g"), bar)"#,
        r"foo.replace(/a]/g, _)",
        r"foo.replace(/[a]/g, _)",
        r"foo.replace(/a{1/g, _)",
        r"foo.replace(/a{1}/g, _)",
        r"foo.replace(/\u0022/g, _)",
        r"foo.replace(/\u0027/g, _)",
        r"foo.replace(/\cM\cj/g, _)",
        r"foo.replace(/\x22/g, _)",
        r"foo.replace(/\x27/g, _)",
        r"foo.replace(/\uD83D\ude00/g, _)",
        r"foo.replace(/\u{1f600}/gu, _)",
        r"foo.replace(/\n/g, _)",
        r"foo.replace(/\u{20}/gu, _)",
        r"foo.replace(/\u{20}/gv, _)",
        r"foo.replaceAll(/a]/g, _)",
        // we need a regex parser to handle this
        // r"foo.replaceAll(/\r\n\u{1f600}/gu, _)",
        // r"foo.replaceAll(/\r\n\u{1f600}/gv, _)",
        r"foo.replaceAll(/a very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long string/g, _)",
        r#"foo.replace(/(?!a)+/g, "")"#,
        // https://github.com/oxc-project/oxc/issues/1790
        // report error as `/world/g` can be replaced with string literal
        r#""Hello world".replaceAll(/world/g, 'world!');"#,
        // https://github.com/oxc-project/oxc/issues/21188
        // u/v flags are allowed, so replaceAll should still suggest string replacement
        r"foo.replaceAll(/foo/gu, bar)",
        r"foo.replaceAll(/foo/gv, bar)",
        r#"const RE = /a/g;
export const out = s.replace(RE, "b");
export const out2 = s.replace(/a/g, "b");"#,
        r#"const pattern = new RegExp("foo", "g"); foo.replace(pattern, bar)"#,
        r"let pattern = /a/g; foo.replace(pattern, bar)",
        r"var pattern = /a/g; foo.replace(pattern, bar)",
        r"const pattern = (/a/g); foo.replace((pattern), bar)",
        r"const pattern = /a/g; const alias = pattern; foo.replace(alias, bar)",
        r"const pattern = /a/g; obj[pattern] = value; foo.replace(pattern, bar)",
        r"const RE = /a/g; const first = RE; const second = (first); foo.replace(first, bar); foo.replace(second, bar); foo.replace(RE, bar)",
    ];

    let fix = vec![
        (r"foo.replace(/a/g, bar)", r"foo.replaceAll('a', bar)"),
        (r"foo?.replace(/a/g, bar)", r"foo?.replaceAll('a', bar)"),
        (
            r"foo?.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');",
            r"foo?.replaceAll(/[.*+?^${}()|[\]\\]/g, '\\$&');",
        ),
        (
            r"foo/* comment 1 */
	.replace/* comment 2 */(
		/* comment 3 */
		/a/g // comment 4
		,
		bar
	)",
            r"foo/* comment 1 */
	.replaceAll/* comment 2 */(
		/* comment 3 */
		'a' // comment 4
		,
		bar
	)",
        ),
        (r#"foo.replace(/"'/g, '\'')"#, r#"foo.replaceAll('"\'', '\'')"#),
        (r"foo.replace(/\./g, bar)", r"foo.replaceAll('.', bar)"),
        (r"foo.replace(/\\\./g, bar)", r"foo.replaceAll('\\.', bar)"),
        (r"foo.replace(/\|/g, bar)", r"foo.replaceAll('|', bar)"),
        (r"foo.replace(/a/gu, bar)", r"foo.replaceAll('a', bar)"),
        (r"foo.replace(/a/ug, bar)", r"foo.replaceAll('a', bar)"),
        (r"foo.replace(/[)-|.*+?^$]/g, '\\$&')", r"foo.replaceAll(/[)-|.*+?^$]/g, '\\$&')"),
        (
            r"foo.replace(/[.*+?^${}()|]\[\]\\]/g, '\\$&')",
            r"foo.replaceAll(/[.*+?^${}()|]\[\]\\]/g, '\\$&')",
        ),
        (r"foo.replace(/a?/g, bar)", r"foo.replaceAll(/a?/g, bar)"),
        (r"foo.replace(/.*/g, bar)", r"foo.replaceAll(/.*/g, bar)"),
        (r"foo.replace(/a|b/g, bar)", r"foo.replaceAll(/a|b/g, bar)"),
        (r"foo.replace(/\W/g, bar)", r"foo.replaceAll(/\W/g, bar)"),
        (r"foo.replace(/\u{61}/g, bar)", r"foo.replaceAll(/\u{61}/g, bar)"),
        (r#"foo.replace(/]/g, "bar")"#, r#"foo.replaceAll(']', "bar")"#),
        (r"foo.replace(/a/gi, bar)", r"foo.replaceAll(/a/gi, bar)"),
        (r"foo.replace(/a/dgims, bar)", r"foo.replaceAll(/a/dgims, bar)"),
        (r"foo.replace(/a/gy, bar)", r"foo.replaceAll(/a/gy, bar)"),
        (r"foo.replace(/./gs, bar)", r"foo.replaceAll(/./gs, bar)"),
        (r"foo.replace(/^a/gm, bar)", r"foo.replaceAll(/^a/gm, bar)"),
        (r"foo.replace(/a/gui, bar)", r"foo.replaceAll(/a/gui, bar)"),
        (r"foo.replace(/a/uig, bar)", r"foo.replaceAll(/a/uig, bar)"),
        (r"foo.replace(/a/vig, bar)", r"foo.replaceAll(/a/vig, bar)"),
        (
            r#"foo.replace(new RegExp("foo", "g"), bar)"#,
            r#"foo.replaceAll(new RegExp("foo", "g"), bar)"#,
        ),
        (r"foo.replace(/a]/g, _)", r"foo.replaceAll('a]', _)"),
        (r"foo.replace(/[ab]/g, _)", r"foo.replaceAll(/[ab]/g, _)"),
        (r"foo.replace(/[a-z]/g, _)", r"foo.replaceAll(/[a-z]/g, _)"),
        (r"foo.replace(/[^a]/g, _)", r"foo.replaceAll(/[^a]/g, _)"),
        (r"foo.replace(/a{1/g, _)", r"foo.replaceAll('a{1', _)"),
        (r"foo.replace(/(a)/g, _)", r"foo.replaceAll(/(a)/g, _)"),
        (r"foo.replace(/(?:a|b)/g, _)", r"foo.replaceAll(/(?:a|b)/g, _)"),
        (r"foo.replace(/\n/g, _)", r"foo.replaceAll('\n', _)"),
        (r"foo.replace(/\8/g, _)", r"foo.replaceAll('8', _)"),
        (r"foo.replace(/\c_/g, _)", r"foo.replaceAll('\\c_', _)"),
        (r"foo.replaceAll(/a]/g, _)", r"foo.replaceAll('a]', _)"),
        (r"foo?.replaceAll(/a/g, _)", r"foo?.replaceAll('a', _)"),
        (
            r"foo.replaceAll(/a very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long string/g, _)",
            r"foo.replaceAll('a very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long string', _)",
        ),
        (r#"foo.replace(/(?!a)+/g, "")"#, r#"foo.replaceAll(/(?!a)+/g, "")"#),
        (r"foo.replaceAll(/a/g, bar)", r"foo.replaceAll('a', bar)"),
        (r"text.replaceAll(/\\`/g, '`')", r"text.replaceAll('\\`', '`')"),
        (
            r"JSON.stringify(data).replace(/'/g, '&#39')",
            r"JSON.stringify(data).replaceAll('\'', '&#39')",
        ),
        // (
        //     r"foo.replace(/[a]/g, bar)",
        //     r"foo.replaceAll('a', bar)",
        // ),
        // (
        //     r"foo.replace(/[.]/g, bar)",
        //     r"foo.replaceAll('.', bar)",
        // ),
        // (
        //     r"foo.replace(/[\n]/g, bar)",
        //     r"foo.replaceAll('\n', bar)",
        // ),
        // (
        //     r"foo.replace(/\u{61}/gu, bar)",
        //     r"foo.replaceAll('\u{61}', bar)",
        // ),
        // (
        //     r"foo.replace(/\u{61}/gv, bar)",
        //     r"foo.replaceAll('\u{61}', bar)",
        // ),
        // (
        //     r"str.replace(/\u200B/g, '')",
        //     r"str.replaceAll('\u200B', '')",
        // ),
        // (
        //     r"str.replace(/\x20/g, '')",
        //     r"str.replaceAll('\x20', '')",
        // ),
        // (
        //     r"foo.replace(/a/gs, bar)",
        //     r"foo.replaceAll('a', bar)",
        // ),
        // (
        //     r"foo.replace(/\./gs, bar)",
        //     r"foo.replaceAll('.', bar)",
        // ),
        // (
        //     r"foo.replace(/a/gm, bar)",
        //     r"foo.replaceAll('a', bar)",
        // ),
        // (
        //     r"foo.replace(/a/dg, bar)",
        //     r"foo.replaceAll('a', bar)",
        // ),
        // (
        //     r"foo.replace(/a/dgms, bar)",
        //     r"foo.replaceAll('a', bar)",
        // ),
        // (
        //     r"foo.replaceAll(/a/gms, bar)",
        //     r"foo.replaceAll('a', bar)",
        // ),
        (
            r#"const pattern = new RegExp("foo", "g"); foo.replace(pattern, bar)"#,
            r#"const pattern = new RegExp("foo", "g"); foo.replaceAll(pattern, bar)"#,
        ),
        (
            r"const pattern = /a/g; foo.replace(pattern, bar)",
            r"const pattern = /a/g; foo.replaceAll(pattern, bar)",
        ),
        (
            r"const pattern = /a/g; const alias = pattern; foo.replace(alias, bar)",
            r"const pattern = /a/g; const alias = pattern; foo.replaceAll(alias, bar)",
        ),
        // (
        //     r"foo.replace(/a{1}/g, _)",
        //     r"foo.replaceAll('a', _)",
        // ),
        // (
        //     r"foo.replace(/[a]{1}/g, _)",
        //     r"foo.replaceAll('a', _)",
        // ),
        // (
        //     r"foo.replace(/(?:a)/g, _)",
        //     r"foo.replaceAll('a', _)",
        // ),
        // (
        //     r"foo.replace(/(?:[a])/g, _)",
        //     r"foo.replaceAll('a', _)",
        // ),
        // (
        //     r"foo.replace(/(?:ab)/g, _)",
        //     r"foo.replaceAll('ab', _)",
        // ),
        // (
        //     r"foo.replace(/(?:a)(?:b)/g, _)",
        //     r"foo.replaceAll('ab', _)",
        // ),
        // (
        //     r"foo.replace(/(?:a){1}/g, _)",
        //     r"foo.replaceAll('a', _)",
        // ),
        // (
        //     r"foo.replace(/\u0022/g, _)",
        //     r"foo.replaceAll('\u0022', _)",
        // ),
        // (
        //     r"foo.replace(/\u0027/g, _)",
        //     r"foo.replaceAll('\u0027', _)",
        // ),
        // (
        //     r"foo.replace(/\cM\cj\cI/g, _)",
        //     r"foo.replaceAll('\r\n\t', _)",
        // ),
        // (
        //     r"foo.replace(/\cZ/g, _)",
        //     r"foo.replaceAll('\u{1A}', _)",
        // ),
        // (
        //     r"foo.replace(/\377/g, _)",
        //     r"foo.replaceAll('\u{FF}', _)",
        // ),
        // (
        //     r"foo.replace(/\x0d\x0a\x09/g, _)",
        //     r"foo.replaceAll('\x0d\x0a\x09', _)",
        // ),
        // (
        //     r"foo.replace(/\u000d\u000a\u0009/g, _)",
        //     r"foo.replaceAll('\u000d\u000a\u0009', _)",
        // ),
        // (
        //     r"foo.replace(/\x22/g, _)",
        //     r"foo.replaceAll('\x22', _)",
        // ),
        // (
        //     r"foo.replace(/\x27/g, _)",
        //     r"foo.replaceAll('\x27', _)",
        // ),
        // (
        //     r"foo.replace(/\uD83D\ude00/g, _)",
        //     r"foo.replaceAll('\uD83D\ude00', _)",
        // ),
        // (
        //     r"foo.replace(/\u{1f600}/gu, _)",
        //     r"foo.replaceAll('\u{1f600}', _)",
        // ),
        // (
        //     r"foo.replace(/\u{20}/gu, _)",
        //     r"foo.replaceAll('\u{20}', _)",
        // ),
        // (
        //     r"foo.replace(/\u{20}/gv, _)",
        //     r"foo.replaceAll('\u{20}', _)",
        // ),
        // (
        //     r"foo.replace(/\1/g, _)",
        //     r"foo.replaceAll('\u{1}', _)",
        // ),
        // (
        //     r"foo.replace(/\00/g, _)",
        //     r"foo.replaceAll('\u{0}', _)",
        // ),
        // (
        //     r"foo.replace(/\08/g, _)",
        //     r"foo.replaceAll('\u{0}8', _)",
        // ),
        // (
        //     r"foo.replaceAll(/\r\n\u{1f600}/gu, _)",
        //     r"foo.replaceAll('\r\n\u{1f600}', _)",
        // ),
        // (
        //     r"foo.replaceAll(/\r\n\u{1f600}/gv, _)",
        //     r"foo.replaceAll('\r\n\u{1f600}', _)",
        // ),
        // (
        //     r#"foo.split("a").join("b")"#,
        //     r"foo.replaceAll('a', 'b')",
        // ),
        // (
        //     r#"foo.split(`a`).join("b")"#,
        //     r"foo.replaceAll('a', 'b')",
        // ),
        // (
        //     r#"foo.split("a").join(`b`)"#,
        //     r"foo.replaceAll('a', 'b')",
        // ),
        // (
        //     r#"foo.split("_").join("$&")"#,
        //     r"foo.replaceAll('_', '$$&')",
        // ),
        // (
        //     r#"foo.split("_").join("$1")"#,
        //     r"foo.replaceAll('_', '$$1')",
        // ),
        // (
        //     r#"foo.split("_").join("$$")"#,
        //     r"foo.replaceAll('_', '$$$$')",
        // ),
        // (
        //     r#"(foo).split("a").join("b")"#,
        //     r"(foo).replaceAll('a', 'b')",
        // ),
        // (
        //     r#"foo.split(/a+/).join("b")"#,
        //     r"foo.replaceAll(/a+/g, 'b')",
        // ),
        // (
        //     r#"foo.split(/(?:a)/).join("b")"#,
        //     r"foo.replaceAll(/(?:a)/g, 'b')",
        // ),
        // (
        //     r#"foo.split(/[ab]+/).join("b")"#,
        //     r"foo.replaceAll(/[ab]+/g, 'b')",
        // ),
        // (
        //     r#"foo.split(/\s+/).join("b")"#,
        //     r"foo.replaceAll(/\s+/g, 'b')",
        // ),
        // (
        //     r#"foo.split(/a|b/).join("b")"#,
        //     r"foo.replaceAll(/a|b/g, 'b')",
        // ),
        // (
        //     r#"foo.split(/a/i).join("b")"#,
        //     r"foo.replaceAll(/a/gi, 'b')",
        // ),
        // (
        //     r#"foo.split(/a/g).join("b")"#,
        //     r"foo.replaceAll(/a/g, 'b')",
        // ),
        // (
        //     r#"foo.split(/a/gi).join("b")"#,
        //     r"foo.replaceAll(/a/gi, 'b')",
        // ),
        (
            r"const RE = /a/g; const first = RE; const second = (first); foo.replace(first, bar); foo.replace(second, bar); foo.replace(RE, bar)",
            r"const RE = /a/g; const first = RE; const second = (first); foo.replaceAll(first, bar); foo.replaceAll(second, bar); foo.replaceAll(RE, bar)",
        ),
    ];

    Tester::new(PreferStringReplaceAll::NAME, PreferStringReplaceAll::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
