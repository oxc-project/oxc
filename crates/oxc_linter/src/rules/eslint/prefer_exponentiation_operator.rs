use oxc_ast::{
    AstKind,
    ast::{BinaryOperator, Expression},
    match_member_expression,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_identifier_part,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    AstNode,
    ast_util::{could_be_asi_hazard, is_method_call},
    context::LintContext,
    globals::GLOBAL_OBJECT_NAMES,
    rule::Rule,
    utils::get_precedence,
};

#[derive(Debug, Default, Clone)]
pub struct PreferExponentiationOperator;

fn prefer_exponentian_operator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `**` over `Math.pow`.")
        .with_help("Replace `Math.pow(a, b)` with `a ** b`.")
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of `Math.pow` in favor of the `**` operator.
    ///
    /// ### Why is this bad?
    ///
    /// Introduced in ES2016, the infix exponentiation operator `**` is an alternative for the
    /// standard `Math.pow` function. Infix notation is considered to be more readable and thus more
    /// preferable than the function notation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Math.pow(a, b)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// a ** b
    /// ```
    PreferExponentiationOperator,
    eslint,
    style,
    fix,
    version = "0.4.0",
    short_description = "Disallow the use of `Math.pow` in favor of the `**` operator.",
);

impl Rule for PreferExponentiationOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["pow"]), Some(2), Some(2)) {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        let member_expor_obj = member_expr.object();

        match member_expor_obj {
            Expression::Identifier(ident) => {
                if ident.name.as_str() != "Math" || !ctx.is_reference_to_global_variable(ident) {
                    return;
                }
            }
            match_member_expression!(Expression) => {
                let member_expr = member_expor_obj.to_member_expression();
                let Some(static_prop_name) = member_expr.static_property_name() else {
                    return;
                };
                if static_prop_name != "Math" {
                    return;
                }

                if let Expression::Identifier(ident) = member_expr.object().without_parentheses()
                    && GLOBAL_OBJECT_NAMES.contains(&ident.name.as_str())
                    && ctx.is_reference_to_global_variable(ident)
                {
                } else {
                    return;
                }
            }
            _ => return,
        }

        ctx.diagnostic_with_fix(prefer_exponentian_operator_diagnostic(call_expr.span), |fixer| {
            if let AstKind::CallExpression(call_expr) = node.kind()
                && !ctx.has_comments_between(call_expr.span)
                && call_expr.arguments.len() == 2
                && let Some(base) = call_expr.arguments[0].as_expression()
                && let Some(exponent) = call_expr.arguments[1].as_expression()
            {
                let base_text = if does_base_need_parens(base) {
                    format!("({})", base.span().source_text(ctx.source_text()))
                } else {
                    base.span().source_text(ctx.source_text()).to_string()
                };
                let exponent_text =
                    get_exponent_text(fixer.source_range(exponent.span()), exponent);

                let replacement = format!("{base_text} ** {exponent_text}");

                // Check if we need to wrap the entire expression in parentheses based on parent context
                let needs_parens = needs_parens_for_parent(node, ctx)
                    || match replacement_position(node, ctx) {
                        Some(ReplacementPosition::StatementStart) => {
                            base_text.starts_with('{') || starts_with_function_or_class(&base_text)
                        }
                        Some(ReplacementPosition::ArrowBody) => base_text.starts_with('{'),
                        Some(ReplacementPosition::ExportDefault) => {
                            starts_with_function_or_class(&base_text)
                        }
                        None => false,
                    };

                let mut replacement = if needs_parens {
                    format!("({replacement})")
                } else {
                    pad_adjacent_tokens(ctx.source_text(), call_expr.span, replacement)
                };
                if replacement.starts_with(['[', '(', '/', '+', '-', '`'])
                    && could_be_asi_hazard(node, ctx)
                {
                    replacement.insert(0, ';');
                }

                fixer.replace(call_expr.span, replacement)
            } else {
                fixer.noop()
            }
        });
    }
}

/// Pads `replacement` with a space on either side where its edge would merge with the adjacent
/// source character into a different token, e.g. `a+Math.pow(++b, c)` or `Math.pow(a, b)in c`.
fn pad_adjacent_tokens(source_text: &str, span: Span, mut replacement: String) -> String {
    if source_text[..span.start as usize]
        .chars()
        .next_back()
        .zip(replacement.chars().next())
        .is_some_and(|(before, first)| tokens_merge(before, first))
    {
        replacement.insert(0, ' ');
    }
    if replacement
        .chars()
        .next_back()
        .zip(source_text[span.end as usize..].chars().next())
        .is_some_and(|(last, after)| tokens_merge(last, after))
    {
        replacement.push(' ');
    }
    replacement
}

fn tokens_merge(left: char, right: char) -> bool {
    (is_identifier_part(left) && (is_identifier_part(right) || right == '\\'))
        || matches!((left, right), ('+', '+') | ('-', '-'))
}

fn does_base_need_parens(expr: &Expression) -> bool {
    let expr = expr.without_parentheses();
    if matches!(expr, Expression::UnaryExpression(_) | Expression::AwaitExpression(_)) {
        return true;
    }
    if let Some(prec) = get_precedence(expr) {
        return prec <= Precedence::Exponentiation;
    }
    false
}

/// Determines if the exponent expression needs parentheses when used in `base ** exponent`.
///
/// Parentheses are needed only when the exponent has lower precedence than exponentiation.
/// Since `**` is right-associative, `a ** b ** c` equals `a ** (b ** c)`,
/// so `Math.pow(a, b ** c)` can safely become `a ** b ** c`.
fn does_exponent_need_parens(expr: &Expression) -> bool {
    let expr = expr.without_parentheses();

    if let Some(prec) = get_precedence(expr) {
        return prec < Precedence::Exponentiation;
    }

    false
}

/// Gets the text for the exponent, adding parentheses if needed.
fn get_exponent_text(source_text: &str, expr: &Expression) -> String {
    if does_exponent_need_parens(expr) {
        format!("({source_text})")
    } else {
        source_text.to_string()
    }
}

/// Grammar positions where the replacement's leading token determines how the
/// surrounding code is parsed.
#[derive(Debug, Clone, Copy)]
enum ReplacementPosition {
    /// The replacement begins an expression statement.
    StatementStart,
    /// The replacement begins the concise body of an arrow function.
    ArrowBody,
    /// The replacement begins the expression after `export default`.
    ExportDefault,
}

/// Walks up the ancestors that begin at the same offset as `node` (the same
/// walk as ESLint's `isStartOfExpressionStatement`) to find the grammar
/// position that the replacement will begin.
fn replacement_position(node: &AstNode, ctx: &LintContext) -> Option<ReplacementPosition> {
    let start = node.span().start;
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if ancestor.span().start != start {
            return match ancestor.kind() {
                AstKind::ArrowFunctionExpression(arrow)
                    if arrow.is_expression() && arrow.body.span().start == start =>
                {
                    Some(ReplacementPosition::ArrowBody)
                }
                AstKind::ExportDefaultDeclaration(decl)
                    if decl.declaration.span().start == start =>
                {
                    Some(ReplacementPosition::ExportDefault)
                }
                _ => None,
            };
        }
        if matches!(ancestor.kind(), AstKind::ExpressionStatement(_)) {
            return Some(ReplacementPosition::StatementStart);
        }
    }
    None
}

/// Returns `true` if `text` starts with a `function`, `class`, or
/// `async function` expression.
fn starts_with_function_or_class(text: &str) -> bool {
    fn keyword(text: &str, keyword: &str) -> bool {
        text.starts_with(keyword)
            && !text[keyword.len()..]
                .starts_with(|c: char| c.is_alphanumeric() || matches!(c, '_' | '$'))
    }
    keyword(text, "function")
        || keyword(text, "class")
        || (keyword(text, "async") && keyword(text["async".len()..].trim_start(), "function"))
}

fn needs_parens_for_parent(node: &AstNode, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(node.id());

    match parent.kind() {
        AstKind::BinaryExpression(bin_expr) => {
            if bin_expr.operator == BinaryOperator::Exponential {
                let AstKind::CallExpression(call_expr) = node.kind() else {
                    return true;
                };
                bin_expr.right.span() != call_expr.span
            } else {
                bin_expr.precedence() >= Precedence::Exponentiation
            }
        }
        AstKind::UnaryExpression(_)
        | AstKind::AwaitExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSSatisfiesExpression(_) => true,
        AstKind::StaticMemberExpression(_)
        | AstKind::ComputedMemberExpression(_)
        | AstKind::PrivateFieldExpression(_)
        | AstKind::CallExpression(_)
        | AstKind::TaggedTemplateExpression(_)
        | AstKind::TSNonNullExpression(_)
        | AstKind::TSInstantiationExpression(_) => parent.span().start == node.span().start,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Object.pow(a, b)",
        "Math.max(a, b)",
        "Math",
        "Math(a, b)",
        "pow",
        "pow(a, b)",
        "Math.pow",
        "Math.Pow(a, b)",
        "math.pow(a, b)",
        "foo.Math.pow(a, b)",
        "new Math.pow(a, b)",
        "Math[pow](a, b)",
        "globalThis.Object.pow(a, b)",
        "globalThis.Math.max(a, b)",
        // "/* globals Math:off*/ Math.pow(a, b)",
        "let Math; Math.pow(a, b);",
        "if (foo) { const Math = 1; Math.pow(a, b); }",
        "var x = function Math() { Math.pow(a, b); }",
        "function foo(Math) { Math.pow(a, b); }",
        "function foo() { Math.pow(a, b); var Math; }",
        "
			                var globalThis = bar;
			                globalThis.Math.pow(a, b)
			            ",
        "class C { #pow; foo() { Math.#pow(a, b); } }",
    ];

    let fail = vec![
        "globalThis.Math.pow(a, b)",
        "globalThis.Math['pow'](a, b)",
        "Math.pow(a, b) + Math.pow(c,
			 d)",
        "Math.pow(Math.pow(a, b), Math.pow(c, d))",
        "Math.pow(a, b)**Math.pow(c, d)",
        "Math.pow(a, b as any)",
        "Math.pow(a as any, b)",
        "Math.pow(a, b) as any",
        // With comments - no fix should be applied
        "Math.pow(a, b) + Math.pow(c, /* comment */ d)",
        "Math.pow(a, b).toFixed(2)",
        "Math.pow(a, b)?.toFixed(2)",
        "Math.pow(a, b)[0]",
        "Math.pow(a, b)()",
        "Math.pow(a, b)`x`",
        "Math.pow(a, b)!",
        "object[Math.pow(a, b)]",
        "fn(Math.pow(a, b))",
        "(Math.pow(a, b)).toFixed(2)",
        "Math.pow(a, b)<T>()",
        "Math.pow(a, b)<T>",
        "Math.pow({ a: 2 }[k], 2) > 3 && go()",
        "const f = () => Math.pow({ a: 2 }[k], 2)",
        "Math.pow(function (x) {}.length, 2) > 0 && f()",
        "Math.pow({ a: 1 }.a, 2)",
        "Math.pow(async function () {}, 2)",
        "Math.pow(class {}, 2)",
    ];

    let fix = vec![
        ("globalThis.Math.pow(a, b)", "a ** b"),
        ("globalThis.Math['pow'](a, b)", "a ** b"),
        // Nested Math.pow - only fixes outer call first (inner calls remain as arguments)
        ("Math.pow(Math.pow(a, b), Math.pow(c, d))", "Math.pow(a, b) ** Math.pow(c, d)"),
        // When Math.pow is the left operand of **, wrap in parens
        ("Math.pow(a, b)**Math.pow(c, d)", "(a ** b)**c ** d"),
        // TypeScript: exponent with type assertion needs parens
        ("Math.pow(a, b as any)", "a ** (b as any)"),
        // TypeScript: base with type assertion needs parens
        ("Math.pow(a as any, b)", "(a as any) ** b"),
        // TypeScript: entire expression cast needs parens around the exponentiation
        ("Math.pow(a, b) as any", "(a ** b) as any"),
        // Additional test cases
        ("Math.pow(a, b)", "a ** b"),
        ("Math.pow(2, 3)", "2 ** 3"),
        // Unary expressions in base need parens
        ("Math.pow(-a, b)", "(-a) ** b"),
        ("Math.pow(+a, b)", "(+a) ** b"),
        ("Math.pow(!a, b)", "(!a) ** b"),
        ("Math.pow(~a, b)", "(~a) ** b"),
        // Binary expressions with lower precedence need parens
        ("Math.pow(a + b, c)", "(a + b) ** c"),
        ("Math.pow(a * b, c)", "(a * b) ** c"),
        ("Math.pow(a, b + c)", "a ** (b + c)"),
        ("Math.pow(a, b * c)", "a ** (b * c)"),
        // Exponentiation in base needs parens (right-associativity)
        ("Math.pow(a ** b, c)", "(a ** b) ** c"),
        // Exponentiation in exponent doesn't need parens
        ("Math.pow(a, b ** c)", "a ** b ** c"),
        // Identifiers don't need parens
        ("Math.pow(foo, bar)", "foo ** bar"),
        // Member expressions don't need parens
        ("Math.pow(a.b, c.d)", "a.b ** c.d"),
        // Call expressions don't need parens
        ("Math.pow(f(), g())", "f() ** g()"),
        ("Math.pow(a, b).toFixed(2)", "(a ** b).toFixed(2)"),
        ("Math.pow(a, b)?.toFixed(2)", "(a ** b)?.toFixed(2)"),
        ("Math.pow(a, b)[0]", "(a ** b)[0]"),
        ("Math.pow(a, b)()", "(a ** b)()"),
        ("Math.pow(a, b)`x`", "(a ** b)`x`"),
        ("Math.pow(a, b)!", "(a ** b)!"),
        ("object[Math.pow(a, b)]", "object[a ** b]"),
        ("fn(Math.pow(a, b))", "fn(a ** b)"),
        ("(Math.pow(a, b)).toFixed(2)", "(a ** b).toFixed(2)"),
        ("Math.pow(a, b)<T>()", "(a ** b)<T>()"),
        ("Math.pow(a, b)<T>", "(a ** b)<T>"),
        ("foo\nMath.pow(a + b, c)", "foo\n;(a + b) ** c"),
        ("foo\nMath.pow(+a, b)", "foo\n;(+a) ** b"),
        ("foo\nMath.pow(-a, b)", "foo\n;(-a) ** b"),
        ("foo\nMath.pow((a).b, c)", "foo\n;(a).b ** c"),
        ("foo\nMath.pow([a, b].find(fn), c)", "foo\n;[a, b].find(fn) ** c"),
        ("foo\nMath.pow(/regex/, 2)", "foo\n;/regex/ ** 2"),
        ("foo\nMath.pow(`template literal`, 2)", "foo\n;`template literal` ** 2"),
        ("foo\n100 + Math.pow((a).b, c)", "foo\n100 + (a).b ** c"),
        ("foo\nMath.pow(a.b, c)", "foo\na.b ** c"),
        ("Math.pow((a).b, c)", "(a).b ** c"),
        ("foo;\nMath.pow((a).b, c)", "foo;\n(a).b ** c"),
        ("Math.pow({ a: 2 }[k], 2) > 3 && go()", "({ a: 2 }[k] ** 2) > 3 && go()"),
        ("const f = () => Math.pow({ a: 2 }[k], 2)", "const f = () => ({ a: 2 }[k] ** 2)"),
        (
            "Math.pow(function (x) {}.length, 2) > 0 && f()",
            "(function (x) {}.length ** 2) > 0 && f()",
        ),
        ("Math.pow({ a: 1 }.a, 2)", "({ a: 1 }.a ** 2)"),
        ("Math.pow(async function () {}, 2)", "(async function () {} ** 2)"),
        ("Math.pow(class {}, 2)", "(class {} ** 2)"),
        ("const x = Math.pow({ a: 1 }.a, 2)", "const x = { a: 1 }.a ** 2"),
        ("foo(Math.pow({ a: 1 }.a, 2))", "foo({ a: 1 }.a ** 2)"),
        ("const i = () => Math.pow({}.a, 2) + 1", "const i = () => ({}.a ** 2) + 1"),
        (
            "const g = () => Math.pow(function () {}.length, 2)",
            "const g = () => function () {}.length ** 2",
        ),
        (
            "export default Math.pow(function () {}.length, 2)",
            "export default (function () {}.length ** 2)",
        ),
        ("export default Math.pow(class {}.x, 2)", "export default (class {}.x ** 2)"),
        (
            "export default Math.pow(async function () {}.length, 2)",
            "export default (async function () {}.length ** 2)",
        ),
        ("export default Math.pow({}.a, 2)", "export default {}.a ** 2"),
        ("const result = Math.pow(a, b)in obj;", "const result = a ** b in obj;"),
        ("x = Math.pow(a, 2)instanceof B", "x = a ** 2 instanceof B"),
        ("a+Math.pow(++b, c)", "a+ ++b ** c"),
        ("a-Math.pow(--b, c)", "a- --b ** c"),
        ("a+Math.pow(--b, c)", "a+--b ** c"),
        ("Math.pow(a, b + c)in d", "a ** (b + c)in d"),
        ("x=Math.pow(a, b)", "x=a ** b"),
    ];

    Tester::new(
        PreferExponentiationOperator::NAME,
        PreferExponentiationOperator::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
