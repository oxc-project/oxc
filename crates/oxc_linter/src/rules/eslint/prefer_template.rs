use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, BinaryOperator, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};

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
    fix,
    version = "1.12.0",
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
            ctx.diagnostic_with_fix(prefer_template_diagnostic(expr.span), |fixer| {
                let source_text = fixer.source_text();

                // Octal escape sequences (e.g. \033) and non-octal decimal escapes (\8, \9)
                // are valid in string literals but illegal in template literals.
                // Report the diagnostic but skip the autofix (matches ESLint's output: null).
                if has_octal_or_non_octal_decimal_escape_in_binary(source_text, expr) {
                    return fixer.noop();
                }

                let result = build_template_for_binary(source_text, &fixer, expr, None, None);
                fixer.replace(expr.span, result)
            });
        }
    }
}

// ---- Fix helpers ----

/// Build the template literal replacement for a binary `+` expression.
fn build_template_for_binary(
    source_text: &str,
    fixer: &RuleFixer<'_, '_>,
    binary: &BinaryExpression<'_>,
    text_before: Option<&str>,
    text_after: Option<&str>,
) -> String {
    let left = &binary.left;
    let right = &binary.right;

    // Find the + operator between left and right
    let plus_offset = fixer.find_next_token_from(left.span().end, "+").unwrap_or(0);
    let plus_pos = left.span().end + plus_offset;
    let text_before_plus = &source_text[left.span().end as usize..plus_pos as usize];
    let text_after_plus = &source_text[plus_pos as usize + 1..right.span().start as usize];

    let left_ends_with_curly = ends_with_template_curly(left);
    let right_starts_with_curly = starts_with_template_curly(right);

    if left_ends_with_curly {
        let combined_text = format!("{text_before_plus}{text_after_plus}");
        let left_str =
            build_template_for_expr(source_text, fixer, left, text_before, Some(&combined_text));
        let right_str = build_template_for_expr(source_text, fixer, right, None, text_after);
        // Merge: remove closing backtick of left and opening backtick of right
        return format!("{}{}", &left_str[..left_str.len() - 1], &right_str[1..]);
    }

    if right_starts_with_curly {
        let combined_text = format!("{text_before_plus}{text_after_plus}");
        let left_str = build_template_for_expr(source_text, fixer, left, text_before, None);
        let right_str =
            build_template_for_expr(source_text, fixer, right, Some(&combined_text), text_after);
        return format!("{}{}", &left_str[..left_str.len() - 1], &right_str[1..]);
    }

    // Neither side has a curly to merge into — keep as separate template literals joined by +
    let left_str = build_template_for_expr(source_text, fixer, left, text_before, None);
    let right_str = build_template_for_expr(source_text, fixer, right, text_after, None);
    format!("{left_str}{text_before_plus}+{text_after_plus}{right_str}")
}

/// Recursively build a template literal string for the given expression node.
/// `text_before` / `text_after` are comment/whitespace text to inject into `${}` wrappers.
fn build_template_for_expr(
    source_text: &str,
    fixer: &RuleFixer<'_, '_>,
    node: &Expression<'_>,
    text_before: Option<&str>,
    text_after: Option<&str>,
) -> String {
    let inner = node.without_parentheses();

    if let Expression::StringLiteral(lit) = inner {
        let raw = &source_text[lit.span.start as usize + 1..lit.span.end as usize - 1];
        let quote_char = source_text.as_bytes()[lit.span.start as usize];
        let escaped = escape_string_for_template(raw, quote_char as char);
        return format!("`{escaped}`");
    }

    if matches!(inner, Expression::TemplateLiteral(_)) {
        return source_text[inner.span().start as usize..inner.span().end as usize].to_string();
    }

    if let Expression::BinaryExpression(binary) = inner
        && binary.operator == BinaryOperator::Addition
        && has_string_literal(inner)
    {
        return build_template_for_binary(source_text, fixer, binary, text_before, text_after);
    }

    // Default: wrap expression in ${}
    // Use the inner (unparenthesized) expression's source text
    let expr_text = &source_text[inner.span().start as usize..inner.span().end as usize];
    format!("`${{{}{}{}}}`", text_before.unwrap_or(""), expr_text, text_after.unwrap_or(""))
}

// ---- Octal escape detection ----

/// Check if a binary expression tree contains any string literals with octal
/// or non-octal decimal escape sequences.
fn has_octal_or_non_octal_decimal_escape_in_binary(
    source_text: &str,
    expr: &BinaryExpression,
) -> bool {
    has_octal_or_non_octal_decimal_escape_in_expr(source_text, &expr.left)
        || has_octal_or_non_octal_decimal_escape_in_expr(source_text, &expr.right)
}

fn has_octal_or_non_octal_decimal_escape_in_expr(source_text: &str, expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            has_octal_or_non_octal_decimal_escape_in_binary(source_text, binary)
        }
        Expression::StringLiteral(lit) => {
            let raw = &source_text[lit.span.start as usize + 1..lit.span.end as usize - 1];
            check_octal_escape(raw)
        }
        _ => false,
    }
}

/// Check if a raw string (content between quotes) contains octal or non-octal decimal
/// escape sequences. `\0` alone is allowed, but `\0` followed by a digit is not.
/// `\1`-`\7` are octal. `\8` and `\9` are non-octal decimal escapes.
fn check_octal_escape(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                b'1'..=b'9' => return true,
                b'0' => {
                    if i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
                        return true;
                    }
                }
                b'\\' => {
                    // Skip escaped backslash — the next char is not part of an escape
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
    false
}

// ---- Template curly detection ----

/// Check if a node contains any string literal (including template literals)
/// in its concatenation tree.
fn has_string_literal(node: &Expression) -> bool {
    let node = node.without_parentheses();
    match node {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            has_string_literal(&binary.right) || has_string_literal(&binary.left)
        }
        _ => node.is_string_literal(),
    }
}

/// Will this node start with `${}` when converted to a template literal?
fn starts_with_template_curly(node: &Expression) -> bool {
    let node = node.without_parentheses();
    match node {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            starts_with_template_curly(&binary.left)
        }
        Expression::TemplateLiteral(tl) => {
            !tl.expressions.is_empty()
                && !tl.quasis.is_empty()
                && tl.quasis[0].span.start == tl.quasis[0].span.end
        }
        Expression::StringLiteral(_) => false,
        _ => true,
    }
}

/// Will this node end with `${}` when converted to a template literal?
fn ends_with_template_curly(node: &Expression) -> bool {
    let node = node.without_parentheses();
    match node {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            starts_with_template_curly(&binary.right)
        }
        Expression::TemplateLiteral(tl) => {
            if let Some(last) = tl.quasis.last() {
                !tl.expressions.is_empty() && last.span.start == last.span.end
            } else {
                false
            }
        }
        Expression::StringLiteral(_) => false,
        _ => true,
    }
}

// ---- String escaping ----

/// Escape a raw string literal content (between quotes) for use inside a template literal.
///
/// Ports ESLint's two-step approach:
/// 1. Escape unescaped `${` and backticks (preserving already-escaped ones)
/// 2. Unescape the original quote character (no longer needed in template literals)
fn escape_string_for_template(raw: &str, quote_char: char) -> String {
    // Step 1: Escape ${ and ` that aren't already properly escaped
    let step1 = escape_dollar_and_backtick(raw);
    // Step 2: Unescape the quote character (\' → ' or \" → ")
    unescape_quote(&step1, quote_char)
}

/// Escape `${` and backticks that aren't already escaped by preceding backslashes.
/// Ports: `raw.replace(/\\*(\$\{|`)/gu, matched => { ... })`
fn escape_dollar_and_backtick(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len() + 4);
    let bytes = raw.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'\\' {
                i += 1;
            }
            let num_backslashes = i - start;

            // Check if followed by ${ or `
            let is_backtick = i < bytes.len() && bytes[i] == b'`';
            let is_dollar_brace =
                i < bytes.len() && bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{';

            if is_backtick || is_dollar_brace {
                // Push all the original backslashes
                result.push_str(&raw[start..i]);
                if num_backslashes % 2 == 0 {
                    // Even backslashes → target is unescaped → add escape
                    result.push('\\');
                }
                // Consume the target character(s)
                if is_backtick {
                    result.push('`');
                    i += 1;
                } else {
                    result.push('$');
                    result.push('{');
                    i += 2;
                }
            } else {
                // Not followed by a target — keep backslashes as-is
                result.push_str(&raw[start..i]);
            }
            continue;
        }

        if bytes[i] == b'`' {
            // Bare backtick (no preceding backslashes) — escape it
            result.push('\\');
            result.push('`');
            i += 1;
            continue;
        }

        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            // Bare ${ — escape it
            result.push('\\');
            result.push('$');
            result.push('{');
            i += 2;
            continue;
        }

        // Regular character (handle multi-byte UTF-8)
        if bytes[i] > 127 {
            let ch = raw[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Unescape the original quote character: `\'` → `'` or `\"` → `"`.
/// Ports: `.replace(new RegExp(\`\\\\${quote}\`, "gu"), quote)`
#[expect(clippy::disallowed_methods)]
fn unescape_quote(s: &str, quote_char: char) -> String {
    let escaped = format!("\\{quote_char}");
    s.replace(&*escaped, &quote_char.to_string())
}

fn check_should_report(expr: &BinaryExpression) -> bool {
    if expr.operator != BinaryOperator::Addition {
        return false;
    }

    let left = expr.left.get_inner_expression();
    let right = expr.right.get_inner_expression();

    let left_is_string = left.is_string_literal();
    let right_is_string = right.is_string_literal();

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
    if let Expression::BinaryExpression(binary) = expr
        && binary.operator == BinaryOperator::Addition
    {
        return all_none_string_literal(binary.left.get_inner_expression())
            && all_none_string_literal(binary.right.get_inner_expression());
    }

    !expr.is_string_literal()
}

fn any_none_string_literal(expr: &Expression) -> bool {
    if let Expression::BinaryExpression(binary) = expr
        && binary.operator == BinaryOperator::Addition
    {
        return any_none_string_literal(binary.left.get_inner_expression())
            || any_none_string_literal(binary.right.get_inner_expression());
    }

    !expr.is_string_literal()
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
        ("var foo = 'hello, ' + name + '!';", "var foo = `hello, ${  name  }!`;", None),
        ("var foo = bar + 'baz';", "var foo = `${bar  }baz`;", None),
        ("var foo = bar + `baz`;", "var foo = `${bar  }baz`;", None),
        ("var foo = +100 + 'yen';", "var foo = `${+100  }yen`;", None),
        ("var foo = 'bar' + baz;", "var foo = `bar${  baz}`;", None),
        ("var foo = '￥' + (n * 1000) + '-'", "var foo = `￥${  n * 1000  }-`", None),
        (
            "var foo = 'aaa' + aaa; var bar = 'bbb' + bbb;",
            "var foo = `aaa${  aaa}`; var bar = `bbb${  bbb}`;",
            None,
        ),
        ("var string = (number + 1) + 'px';", "var string = `${number + 1  }px`;", None),
        ("var foo = 'bar' + baz + 'qux';", "var foo = `bar${  baz  }qux`;", None),
        (
            "var foo = '0 backslashes: ${bar}' + baz;",
            "var foo = `0 backslashes: \\${bar}${  baz}`;",
            None,
        ),
        (
            r"var foo = '1 backslash: \${bar}' + baz;",
            "var foo = `1 backslash: \\${bar}${  baz}`;",
            None,
        ),
        (
            // Note: Rust "\\${bar}" = JS `\${bar}` (1 backslash, not 2 as the test name says)
            "var foo = '2 backslashes: \\${bar}' + baz;",
            "var foo = `2 backslashes: \\${bar}${  baz}`;",
            None,
        ),
        (
            r"var foo = '3 backslashes: \\\${bar}' + baz;",
            "var foo = `3 backslashes: \\\\\\${bar}${  baz}`;",
            None,
        ),
        (
            "var foo = bar + 'this is a backtick: `' + baz;",
            "var foo = `${bar  }this is a backtick: \\`${  baz}`;",
            None,
        ),
        (
            r"var foo = bar + 'this is a backtick preceded by a backslash: \`' + baz;",
            "var foo = `${bar  }this is a backtick preceded by a backslash: \\`${  baz}`;",
            None,
        ),
        (
            // Note: Rust "\\`" = JS `\`` (1 backslash, not 2 as the test name says)
            "var foo = bar + 'this is a backtick preceded by two backslashes: \\`' + baz;",
            "var foo = `${bar  }this is a backtick preceded by two backslashes: \\`${  baz}`;",
            None,
        ),
        ("var foo = bar + `${baz}foo`;", "var foo = `${bar  }${baz}foo`;", None),
        ("var foo = bar + baz + 'qux';", "var foo = `${bar + baz  }qux`;", None),
        (
            "var foo = /* a */ 'bar' /* b */ + /* c */ baz /* d */ + 'qux' /* e */ ;",
            "var foo = /* a */ `bar${ /* b */  /* c */ baz /* d */  }qux` /* e */ ;",
            None,
        ),
        (
            "var foo = bar + ('baz') + 'qux' + (boop);",
            "var foo = `${bar  }baz` + `qux${  boop}`;",
            None,
        ),
        (
            r"foo + 'unescapes an escaped single quote in a single-quoted string: \''",
            "`${foo  }unescapes an escaped single quote in a single-quoted string: '`",
            None,
        ),
        (
            // Note: r#"..."# ends at the first `"#`, so this string has no escaped quote
            r#"foo + "unescapes an escaped double quote in a double-quoted string: ""#,
            "`${foo  }unescapes an escaped double quote in a double-quoted string: `",
            None,
        ),
        (
            r#"foo + 'does not unescape an escaped double quote in a single-quoted string: "'"#,
            r#"`${foo  }does not unescape an escaped double quote in a single-quoted string: "`"#,
            None,
        ),
        (
            r#"foo + "does not unescape an escaped single quote in a double-quoted string: \'""#,
            "`${foo  }does not unescape an escaped single quote in a double-quoted string: \\'`",
            None,
        ),
        (
            r"foo + 'handles unicode escapes correctly: \x27'",
            "`${foo  }handles unicode escapes correctly: \\x27`",
            None,
        ),
        // No fix for non-octal decimal escape sequences
        (
            r"foo + 'does not autofix non-octal decimal escape sequence' + '\8'",
            r"foo + 'does not autofix non-octal decimal escape sequence' + '\8'",
            None,
        ),
        (r"foo + '\0\1'", r"foo + '\0\1'", None),
        (r"foo + '\\033'", r"`${foo  }\\033`", None),
        (r"foo + '\0'", r"`${foo  }\0`", None),
        (
            r#""default-src 'self' https://*.google.com;"
            + "frame-ancestors 'none';"
            + "report-to " + foo + ";""#,
            r"`default-src 'self' https://*.google.com;`
            + `frame-ancestors 'none';`
            + `report-to ${  foo  };`",
            None,
        ),
        ("'a' + 'b' + foo", "`a` + `b${  foo}`", None),
        ("'a' + 'b' + foo + 'c' + 'd'", "`a` + `b${  foo  }c` + `d`", None),
        ("'a' + 'b + c' + foo + 'd' + 'e'", "`a` + `b + c${  foo  }d` + `e`", None),
        ("'a' + 'b' + foo + ('c' + 'd')", "`a` + `b${  foo  }c` + `d`", None),
        ("'a' + 'b' + foo + ('a' + 'b')", "`a` + `b${  foo  }a` + `b`", None),
        (
            "'a' + 'b' + foo + ('c' + 'd') + ('e' + 'f')",
            "`a` + `b${  foo  }c` + `d` + `e` + `f`",
            None,
        ),
        ("foo + ('a' + 'b') + ('c' + 'd')", "`${foo  }a` + `b` + `c` + `d`", None),
        (
            "'a' + foo + ('b' + 'c') + ('d' + bar + 'e')",
            "`a${  foo  }b` + `c` + `d${  bar  }e`",
            None,
        ),
        ("foo + ('b' + 'c') + ('d' + bar + 'e')", "`${foo  }b` + `c` + `d${  bar  }e`", None),
        ("'a' + 'b' + foo + ('c' + 'd' + 'e')", "`a` + `b${  foo  }c` + `d` + `e`", None),
        ("'a' + 'b' + foo + ('c' + bar + 'd')", "`a` + `b${  foo  }c${  bar  }d`", None),
        (
            "'a' + 'b' + foo + ('c' + bar + ('d' + 'e') + 'f')",
            "`a` + `b${  foo  }c${  bar  }d` + `e` + `f`",
            None,
        ),
        (
            "'a' + 'b' + foo + ('c' + bar + 'e') + 'f' + test",
            "`a` + `b${  foo  }c${  bar  }e` + `f${  test}`",
            None,
        ),
        (
            "'a' + foo + ('b' + bar + 'c') + ('d' + test)",
            "`a${  foo  }b${  bar  }c` + `d${  test}`",
            None,
        ),
        ("'a' + foo + ('b' + 'c') + ('d' + bar)", "`a${  foo  }b` + `c` + `d${  bar}`", None),
        ("foo + ('a' + bar + 'b') + 'c' + test", "`${foo  }a${  bar  }b` + `c${  test}`", None),
        ("'a' + '`b`' + c", "`a` + `\\`b\\`${  c}`", None),
        ("'a' + '`b` + `c`' + d", "`a` + `\\`b\\` + \\`c\\`${  d}`", None),
        ("'a' + b + ('`c`' + '`d`')", "`a${  b  }\\`c\\`` + `\\`d\\``", None),
        ("'`a`' + b + ('`c`' + '`d`')", "`\\`a\\`${  b  }\\`c\\`` + `\\`d\\``", None),
        (
            "foo + ('`a`' + bar + '`b`') + '`c`' + test",
            "`${foo  }\\`a\\`${  bar  }\\`b\\`` + `\\`c\\`${  test}`",
            None,
        ),
        ("'a' + ('b' + 'c') + d", "`a` + `b` + `c${  d}`", None),
        ("'a' + ('`b`' + '`c`') + d", "`a` + `\\`b\\`` + `\\`c\\`${  d}`", None),
        ("a + ('b' + 'c') + d", "`${a  }b` + `c${  d}`", None),
        ("a + ('b' + 'c') + (d + 'e')", "`${a  }b` + `c${  d  }e`", None),
        ("a + ('`b`' + '`c`') + d", "`${a  }\\`b\\`` + `\\`c\\`${  d}`", None),
        ("a + ('`b` + `c`' + '`d`') + e", "`${a  }\\`b\\` + \\`c\\`` + `\\`d\\`${  e}`", None),
        ("'a' + ('b' + 'c' + 'd') + e", "`a` + `b` + `c` + `d${  e}`", None),
        (
            "'a' + ('b' + 'c' + 'd' + (e + 'f') + 'g' +'h' + 'i') + j",
            "`a` + `b` + `c` + `d${  e  }fg` +`h` + `i${  j}`",
            None,
        ),
        ("a + (('b' + 'c') + 'd')", "`${a  }b` + `c` + `d`", None),
        ("(a + 'b') + ('c' + 'd') + e", "`${a  }b` + `c` + `d${  e}`", None),
        (
            r#"var foo = "Hello " + "world " + "another " + test"#,
            "var foo = `Hello ` + `world ` + `another ${  test}`",
            None,
        ),
        (r#"'Hello ' + '"world" ' + test"#, r#"`Hello ` + `"world" ${  test}`"#, None),
        (r#""Hello " + "'world' " + test"#, "`Hello ` + `'world' ${  test}`", None),
    ];

    Tester::new(PreferTemplate::NAME, PreferTemplate::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
