use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, BinaryOperator, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_template_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected string concatenation.")
        .with_help("Use template literals instead of string concatenation.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTemplate;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require template literals instead of string concatenation.
    ///
    /// ### Why is this bad?
    ///
    /// In ES2015 (ES6), we can use template literals instead of string concatenation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const str = "Hello, " + name + "!";
    /// const str1 = "Time: " + (12 * 60 * 60 * 1000);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const str = "Hello World!";
    /// const str2 = `Time: ${12 * 60 * 60 * 1000}`;
    /// const str4 = "Hello, " + "World!";
    /// ```
    PreferTemplate,
    eslint,
    style,
    fix
);

impl Rule for PreferTemplate {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };
        if !matches!(expr.operator, BinaryOperator::Addition) {
            return;
        }
        // check is the outermost binary expression
        let not_outermost = ctx.nodes().ancestor_kinds(node.id()).any(
            |v| matches!(v, AstKind::BinaryExpression(e) if e.operator == BinaryOperator::Addition),
        );
        if not_outermost {
            return;
        }
        if check_should_report(expr) {
            let expr_span = expr.span;

            // Pre-compute the fix outside the closure to avoid lifetime issues
            let source = ctx.source_text();
            let mut parts: Vec<Part> = Vec::new();
            flatten_binary_expr(expr, source, &mut parts);

            let has_unhandled = parts.iter().any(|p| matches!(p, Part::Unhandled));

            if has_unhandled {
                ctx.diagnostic(prefer_template_diagnostic(expr.span));
            } else {
                let mut result = String::from('`');
                for part in &parts {
                    match part {
                        Part::StringContent(s) => {
                            result.push_str(s);
                        }
                        Part::TemplatePart(s) => {
                            result.push_str(s);
                        }
                        Part::Expression(s) => {
                            result.push_str("${");
                            result.push_str(s);
                            result.push('}');
                        }
                        Part::Unhandled => unreachable!(),
                    }
                }
                result.push('`');
                ctx.diagnostic_with_fix(prefer_template_diagnostic(expr.span), |fixer| {
                    fixer.replace(expr_span, result)
                });
            }
        }
    }
}

enum Part {
    /// Content from a string literal (already unescaped from JS string to raw text)
    StringContent(String),
    /// Content from a template literal (already valid template content)
    TemplatePart(String),
    /// An expression to be wrapped in ${}
    Expression(String),
    /// An expression we can't safely autofix
    Unhandled,
}

fn contains_string_literal(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            contains_string_literal(&bin.left) || contains_string_literal(&bin.right)
        }
        _ => false,
    }
}

fn flatten_binary_expr<'a>(expr: &'a BinaryExpression<'a>, source: &str, parts: &mut Vec<Part>) {
    // Flatten left - only recurse into + if it contains string literals
    let left = expr.left.get_inner_expression();
    match left {
        Expression::BinaryExpression(bin)
            if bin.operator == BinaryOperator::Addition && contains_string_literal(left) =>
        {
            flatten_binary_expr(bin, source, parts);
        }
        _ => collect_part(left, source, parts),
    }

    // Flatten right - only recurse into + if it contains string literals
    let right = expr.right.get_inner_expression();
    match right {
        Expression::BinaryExpression(bin)
            if bin.operator == BinaryOperator::Addition && contains_string_literal(right) =>
        {
            flatten_binary_expr(bin, source, parts);
        }
        _ => collect_part(right, source, parts),
    }
}

fn collect_part(expr: &Expression<'_>, source: &str, parts: &mut Vec<Part>) {
    match expr {
        Expression::StringLiteral(lit) => {
            let raw = &lit.value;
            // Check for octal/special escapes that we can't safely convert
            let source_text = lit.span.source_text(source);
            if has_unsafe_escapes(source_text) {
                parts.push(Part::Unhandled);
                return;
            }
            // Unescape the string for template literal context
            // The lit.value is already the interpreted value
            let content = raw.replace('`', "\\`").replace("${", "\\${");
            parts.push(Part::StringContent(content));
        }
        Expression::TemplateLiteral(tmpl) => {
            // Extract the content between backticks
            let inner_span = Span::new(tmpl.span.start + 1, tmpl.span.end - 1);
            let content = inner_span.source_text(source);
            parts.push(Part::TemplatePart(content.to_string()));
        }
        _ => {
            let expr_text = expr.span().source_text(source);
            parts.push(Part::Expression(expr_text.to_string()));
        }
    }
}

/// Check if a string literal source contains escape sequences that can't be
/// safely converted to a template literal (octal escapes, \0 followed by digit, etc.)
fn has_unsafe_escapes(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut i = 1; // skip opening quote
    let end = bytes.len().saturating_sub(1); // skip closing quote
    while i < end {
        if bytes[i] == b'\\' {
            i += 1;
            if i >= end {
                break;
            }
            match bytes[i] {
                // Octal escapes (other than \0 not followed by digit)
                b'0' => {
                    if i + 1 < end && bytes[i + 1].is_ascii_digit() {
                        return true;
                    }
                }
                b'1'..=b'7' => return true,
                b'8' | b'9' => return true, // non-octal decimal escape
                b'x' => {
                    // \xNN - hex escape, safe to keep
                }
                _ => {}
            }
        }
        i += 1;
    }
    false
}

fn check_should_report(expr: &BinaryExpression) -> bool {
    if expr.operator != BinaryOperator::Addition {
        return false;
    }

    let left = expr.left.get_inner_expression();
    let right = expr.right.get_inner_expression();

    let left_is_string =
        matches!(left, Expression::StringLiteral(_) | Expression::TemplateLiteral(_));
    let right_is_string =
        matches!(right, Expression::StringLiteral(_) | Expression::TemplateLiteral(_));

    match (left_is_string, right_is_string) {
        // 'a' + 'v'
        (true, true) => false,
        // 'a' + (v + '3')
        (true, false) => any_none_string_literal(right),
        // (v + 'a') + 'c'
        (false, true) => any_none_string_literal(left),
        // a + b
        (false, false) => !all_none_string_literal(left) || !all_none_string_literal(right),
    }
}

fn all_none_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            all_none_string_literal(binary.left.get_inner_expression())
                && all_none_string_literal(binary.right.get_inner_expression())
        }
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => false,
        _ => true,
    }
}

fn any_none_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            any_none_string_literal(binary.left.get_inner_expression())
                || any_none_string_literal(binary.right.get_inner_expression())
        }
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => false,
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "'use strict';",
        "var foo = 'foo' + '\0';",
        "var foo = 'bar';",
        "var foo = 'bar' + 'baz';",
        "var foo = foo + +'100';",
        "var foo = `bar`;",
        "var foo = `hello, ${name}!`;",
        r#"var foo = `foo` + `bar` + "hoge";"#,
        r#"var foo = `foo` +
			    `bar` +
			    "hoge";"#,
    ];

    let fail = vec![
        "var foo = 'hello, ' + name + '!';",
        "var foo = bar + 'baz';",
        "var foo = bar + `baz`;",
        "var foo = +100 + 'yen';",
        "var foo = 'bar' + baz;",
        "var foo = '￥' + (n * 1000) + '-'",
        "var foo = 'aaa' + aaa; var bar = 'bbb' + bbb;",
        "var string = (number + 1) + 'px';",
        "var foo = 'bar' + baz + 'qux';",
        "var foo = '0 backslashes: ${bar}' + baz;",
        r"var foo = '1 backslash: \${bar}' + baz;",
        "var foo = '2 backslashes: \\${bar}' + baz;",
        r"var foo = '3 backslashes: \\\${bar}' + baz;",
        "var foo = bar + 'this is a backtick: `' + baz;",
        r"var foo = bar + 'this is a backtick preceded by a backslash: \`' + baz;",
        "var foo = bar + 'this is a backtick preceded by two backslashes: \\`' + baz;",
        "var foo = bar + `${baz}foo`;",
        "var foo = bar + baz + 'qux';",
        "var foo = /* a */ 'bar' /* b */ + /* c */ baz /* d */ + 'qux' /* e */ ;",
        "var foo = bar + ('baz') + 'qux' + (boop);",
        r"foo + 'unescapes an escaped single quote in a single-quoted string: \''",
        r#"foo + "unescapes an escaped double quote in a double-quoted string: ""#,
        r#"foo + 'does not unescape an escaped double quote in a single-quoted string: "'"#,
        r#"foo + "does not unescape an escaped single quote in a double-quoted string: \'""#,
        r"foo + 'handles unicode escapes correctly: \x27'",
        r"foo + 'does not autofix octal escape sequence' + '\x1b'",
        r"foo + 'does not autofix non-octal decimal escape sequence' + '\8'",
        r"foo + '\n other text \x1b'",
        r"foo + '\0\1'",
        "foo + '\08'",
        "foo + '\\033'",
        "foo + '\0'",
        r#""default-src 'self' https://*.google.com;"
			            + "frame-ancestors 'none';"
			            + "report-to " + foo + ";""#,
        "'a' + 'b' + foo",
        "'a' + 'b' + foo + 'c' + 'd'",
        "'a' + 'b + c' + foo + 'd' + 'e'",
        "'a' + 'b' + foo + ('c' + 'd')",
        "'a' + 'b' + foo + ('a' + 'b')",
        "'a' + 'b' + foo + ('c' + 'd') + ('e' + 'f')",
        "foo + ('a' + 'b') + ('c' + 'd')",
        "'a' + foo + ('b' + 'c') + ('d' + bar + 'e')",
        "foo + ('b' + 'c') + ('d' + bar + 'e')",
        "'a' + 'b' + foo + ('c' + 'd' + 'e')",
        "'a' + 'b' + foo + ('c' + bar + 'd')",
        "'a' + 'b' + foo + ('c' + bar + ('d' + 'e') + 'f')",
        "'a' + 'b' + foo + ('c' + bar + 'e') + 'f' + test",
        "'a' + foo + ('b' + bar + 'c') + ('d' + test)",
        "'a' + foo + ('b' + 'c') + ('d' + bar)",
        "foo + ('a' + bar + 'b') + 'c' + test",
        "'a' + '`b`' + c",
        "'a' + '`b` + `c`' + d",
        "'a' + b + ('`c`' + '`d`')",
        "'`a`' + b + ('`c`' + '`d`')",
        "foo + ('`a`' + bar + '`b`') + '`c`' + test",
        "'a' + ('b' + 'c') + d",
        "'a' + ('`b`' + '`c`') + d",
        "a + ('b' + 'c') + d",
        "a + ('b' + 'c') + (d + 'e')",
        "a + ('`b`' + '`c`') + d",
        "a + ('`b` + `c`' + '`d`') + e",
        "'a' + ('b' + 'c' + 'd') + e",
        "'a' + ('b' + 'c' + 'd' + (e + 'f') + 'g' +'h' + 'i') + j",
        "a + (('b' + 'c') + 'd')",
        "(a + 'b') + ('c' + 'd') + e",
        r#"var foo = "Hello " + "world " + "another " + test"#,
        r#"'Hello ' + '"world" ' + test"#,
        r#""Hello " + "'world' " + test"#,
    ];

    let fix = vec![
        ("var foo = 'hello, ' + name + '!';", "var foo = `hello, ${name}!`;", None),
        ("var foo = bar + 'baz';", "var foo = `${bar}baz`;", None),
        ("var foo = bar + `baz`;", "var foo = `${bar}baz`;", None),
        ("var foo = +100 + 'yen';", "var foo = `${+100}yen`;", None),
        ("var foo = 'bar' + baz;", "var foo = `bar${baz}`;", None),
        ("var foo = '￥' + (n * 1000) + '-'", "var foo = `￥${n * 1000}-`", None),
        (
            "var foo = 'aaa' + aaa; var bar = 'bbb' + bbb;",
            "var foo = `aaa${aaa}`; var bar = `bbb${bbb}`;",
            None,
        ),
        ("var string = (number + 1) + 'px';", "var string = `${number + 1}px`;", None),
        ("var foo = 'bar' + baz + 'qux';", "var foo = `bar${baz}qux`;", None),
        ("var foo = bar + `${baz}foo`;", "var foo = `${bar}${baz}foo`;", None),
        ("var foo = bar + baz + 'qux';", "var foo = `${bar + baz}qux`;", None),
    ];
    Tester::new(PreferTemplate::NAME, PreferTemplate::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
