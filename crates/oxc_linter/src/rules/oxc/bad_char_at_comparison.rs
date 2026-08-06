use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression, TSType, VariableDeclarator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::is_method_call, ast_util::variable_declaration_kind, context::LintContext,
    rule::Rule,
};

fn bad_char_at_comparison_diagnostic(
    character_access: Span,
    compared_string: Span,
    len: usize,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid character comparison")
        .with_help("Character access returns a string of length at most 1. If the return value is compared with a string of length greater than 1, the comparison will always be false.")
        .with_labels([
            character_access.label("A single character is accessed here"),
            compared_string.label(format!("And compared with a string of length {len} here")),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct BadCharAtComparison;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when a character accessed with `charAt`, `at`, or bracket notation is
    /// compared with a string of length greater than 1.
    ///
    /// ### Why is this bad?
    ///
    /// Character access returns a string of length at most 1. If the return value is compared with
    /// a string of length greater than 1, the comparison will always be false.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// a.charAt(4) === 'a2';
    /// 'abc'.at(4) === 'a2';
    /// 'abc'[4] === 'a2';
    /// a.charAt(4) === '/n';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// a.charAt(4) === 'a'
    /// a.charAt(4) === '\n';
    /// ```
    BadCharAtComparison,
    oxc,
    correctness,
    version = "0.0.22",
    short_description = "Warns when a single-character string access is compared with a string of length greater than 1.",
);

impl Rule for BadCharAtComparison {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.kind() else {
            return;
        };

        if binary_expr.operator.is_equality()
            && let Some((character_access, compared_string, len)) =
                bad_char_at_comparison(binary_expr, ctx)
        {
            ctx.diagnostic(bad_char_at_comparison_diagnostic(
                character_access,
                compared_string,
                len,
            ));
        }
    }
}

fn bad_char_at_comparison(
    binary_expr: &BinaryExpression,
    ctx: &LintContext,
) -> Option<(Span, Span, usize)> {
    bad_char_at_comparison_operands(&binary_expr.left, &binary_expr.right, ctx)
        .or_else(|| bad_char_at_comparison_operands(&binary_expr.right, &binary_expr.left, ctx))
}

fn bad_char_at_comparison_operands(
    character_access: &Expression,
    compared_string: &Expression,
    ctx: &LintContext,
) -> Option<(Span, Span, usize)> {
    let (compared_string, len) = invalid_comparison_string(compared_string)?;
    let character_access = character_access.without_parentheses();

    is_single_character_access(character_access, ctx)
        .then(|| (character_access.span(), compared_string, len))
}

fn invalid_comparison_string(expr: &Expression) -> Option<(Span, usize)> {
    let expr = expr.without_parentheses();
    let value = match expr {
        Expression::StringLiteral(literal) => literal.value.as_str(),
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => {
            literal.quasis.first()?.value.cooked.as_deref()?
        }
        _ => return None,
    };
    let len = value.encode_utf16().count();

    (len > 1).then(|| (expr.span(), len))
}

fn is_single_character_access(expr: &Expression, ctx: &LintContext) -> bool {
    match expr {
        Expression::CallExpression(call_expr) => {
            is_method_call(call_expr, None, Some(&["charAt"]), Some(1), Some(1))
                || (!call_expr.optional
                    && is_method_call(call_expr, None, Some(&["at"]), Some(1), Some(1))
                    && call_expr.callee.get_member_expr().is_some_and(|member_expr| {
                        !member_expr.optional() && is_definitely_string(member_expr.object(), ctx)
                    }))
        }
        Expression::ComputedMemberExpression(member_expr) if !member_expr.optional => {
            is_static_string_index(&member_expr.expression)
                && is_definitely_string(&member_expr.object, ctx)
        }
        _ => false,
    }
}

fn is_definitely_string(expr: &Expression, ctx: &LintContext) -> bool {
    match expr.without_parentheses() {
        Expression::StringLiteral(_) => true,
        Expression::Identifier(identifier) => {
            let Some(symbol_id) =
                ctx.scoping().get_reference(identifier.reference_id()).symbol_id()
            else {
                return false;
            };
            let declaration =
                ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id)).kind();

            declaration
                .as_variable_declarator()
                .is_some_and(|declarator| is_definitely_string_declarator(declarator, ctx))
        }
        _ => false,
    }
}

fn is_definitely_string_declarator(declarator: &VariableDeclarator, ctx: &LintContext) -> bool {
    if declarator
        .type_annotation
        .as_ref()
        .is_some_and(|annotation| matches!(annotation.type_annotation, TSType::TSStringKeyword(_)))
    {
        return true;
    }

    if !variable_declaration_kind(declarator, ctx).is_const()
        || !declarator.id.is_binding_identifier()
    {
        return false;
    }

    declarator
        .init
        .as_ref()
        .is_some_and(|init| matches!(init.without_parentheses(), Expression::StringLiteral(_)))
}

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

fn is_static_string_index(expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::NumericLiteral(_) => true,
        Expression::StringLiteral(literal) => {
            let value = literal.value.as_str();
            value == "0"
                || (!value.starts_with('0')
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value.parse::<u64>().is_ok_and(|index| index <= MAX_SAFE_INTEGER))
        }
        _ => false,
    }
}

// Some test cases are adapted from `eslint-plugin-unicorn/no-invalid-character-comparison`:
// <https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v72.0.0/test/no-invalid-character-comparison.js>
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"a.charAt(4) === 'a'",
        "a.charAt(4) === '\\n'",
        "a.charAt(4) === '\t'",
        r"a.charAt(4) === '\ufeff'",
        r"a.charAt(4) !== '\ufeff'",
        r#""abc".charAt() === "ab""#,
        r#"[1, 2].at(0) === "ab""#,
        r#"const value = [1, 2]; value[0] === "ab";"#,
        "chatAt(4) === 'a2'",
        "new chatAt(4) === 'a'",
        r#"const value = 100; value[0] === "ab""#,
    ];

    let fail = vec![
        r"a.charAt(4) === 'aa'",
        "a.charAt(4) === '/n'",
        "a.charAt(3) === '/t'",
        r"a.charAt(4) === 'ac'",
        r"a.charAt(822) !== 'foo'",
        r"a.charAt(4) === '\\ukeff'",
        r#""abc".at(0) === "ab""#,
        r#""abc".at(-1) !== "ab""#,
        r#""abc"[0] === "ab""#,
        r#""abc"["0"] === "ab""#,
        r#"declare const value: string; value[0] === "ab";"#,
        r#""abc".charAt(0) === `ab`"#,
        r#"("abc".charAt(0)) === "ab""#,
        r#""abc".charAt(0) === ("ab")"#,
        r#""abc".charAt(0) === "😀""#,
        r#""abc".charAt(0) === "\u{1F600}""#,
        r#"const value = "abc"; value[0] === "ab""#,
        r#"const value = "abc"; value.at(0) === "ab""#,
    ];

    Tester::new(BadCharAtComparison::NAME, BadCharAtComparison::PLUGIN, pass, fail)
        .test_and_snapshot();
}
