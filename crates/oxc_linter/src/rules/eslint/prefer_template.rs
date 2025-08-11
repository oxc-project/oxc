use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, BinaryOperator, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_template_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected string concatenation.")
        .with_help("Unexpected string concatenation.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTemplate;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    PreferTemplate,
    eslint,
    style,
    pending
);

impl Rule for PreferTemplate {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };
        if !matches!(expr.operator, BinaryOperator::Addition) {
            return;
        }
        // check is the topest binary expression
        let is_topest = ctx
            .nodes()
            .ancestor_kinds(node.id())
            .find(|v| {
                matches!(v, AstKind::BinaryExpression(e) if e.operator == BinaryOperator::Addition)
            })
            .is_none();
        if !is_topest {
            return;
        }
        // println!("expr = {:?}", expr);
        if check_should_report(expr) {
            ctx.diagnostic(prefer_template_diagnostic(expr.span));
        }
    }
}

fn has_none_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            has_none_string_literal(binary.left.without_parentheses())
                && has_none_string_literal(binary.right.without_parentheses())
        }
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => false,
        _ => true,
    }
}

fn has_none_string_literal_v2(expr: &Expression) -> bool {
    match expr {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            has_none_string_literal_v2(binary.left.without_parentheses())
                || has_none_string_literal_v2(binary.right.without_parentheses())
        }
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => false,
        _ => true,
    }
}

fn check_should_report(expr: &BinaryExpression) -> bool {
    if expr.operator != BinaryOperator::Addition {
        return false;
    }

    let left = expr.left.without_parentheses();
    let right = expr.right.without_parentheses();

    let left_is_string = check_is_string_literal(left);
    let right_is_string = check_is_string_literal(right);
    // println!("left_is_string = {left_is_string}, right_is_string = {right_is_string}");

    if left_is_string && right_is_string {
        return false;
    }

    // 'nnar' + (functin() {})
    // 'a' + ('a' + v)
    if left_is_string && !right_is_string {
        let r = has_none_string_literal_v2(right);
        return r;
    }

    if right_is_string && !left_is_string {
        let l = has_none_string_literal_v2(left);
        return l;
    }

    // (bar + 'a') + baz;
    if !left_is_string && !right_is_string {
        if !has_none_string_literal(left) {
            return true;
        }
        if !has_none_string_literal(right) {
            return true;
        }
    }

    false
}

fn check_is_string_literal(node: &Expression) -> bool {
    matches!(node, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
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
        "foo + 'unescapes an escaped single quote in a single-quoted string: \''",
        r#"foo + "unescapes an escaped double quote in a double-quoted string: """#,
        r#"foo + 'does not unescape an escaped double quote in a single-quoted string: "'"#,
        r#"foo + "does not unescape an escaped single quote in a double-quoted string: \'""#,
        "foo + 'handles unicode escapes correctly: \x27'",
        "foo + 'does not autofix octal escape sequence' + '\033'",
        r"foo + 'does not autofix non-octal decimal escape sequence' + '\8'",
        "foo + '\n other text \033'",
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

    //     let _fix = vec![
    //         ("var foo = 'hello, ' + name + '!';", "var foo = `hello, ${  name  }!`;", None),
    // ("var foo = bar + 'baz';", "var foo = `${bar  }baz`;", None),
    // ("var foo = bar + `baz`;", "var foo = `${bar  }baz`;", None),
    // ("var foo = +100 + 'yen';", "var foo = `${+100  }yen`;", None),
    // ("var foo = 'bar' + baz;", "var foo = `bar${  baz}`;", None),
    // ("var foo = '￥' + (n * 1000) + '-'", "var foo = `￥${  n * 1000  }-`", None),
    // ("var foo = 'aaa' + aaa; var bar = 'bbb' + bbb;", "var foo = `aaa${  aaa}`; var bar = `bbb${  bbb}`;", None),
    // ("var string = (number + 1) + 'px';", "var string = `${number + 1  }px`;", None),
    // ("var foo = 'bar' + baz + 'qux';", "var foo = `bar${  baz  }qux`;", None),
    // // ("var foo = '0 backslashes: ${bar}' + baz;", "var foo = `0 backslashes: \${bar}${  baz}`;", None),
    // // ("var foo = '1 backslash: \${bar}' + baz;", "var foo = `1 backslash: \${bar}${  baz}`;", None),
    // // ("var foo = '2 backslashes: \\${bar}' + baz;", "var foo = `2 backslashes: \\\${bar}${  baz}`;", None),
    // // ("var foo = '3 backslashes: \\\${bar}' + baz;", "var foo = `3 backslashes: \\\${bar}${  baz}`;", None),
    // // ("var foo = bar + 'this is a backtick: `' + baz;", "var foo = `${bar  }this is a backtick: \`${  baz}`;", None),
    // // ("var foo = bar + 'this is a backtick preceded by a backslash: \`' + baz;", "var foo = `${bar  }this is a backtick preceded by a backslash: \`${  baz}`;", None),
    // // ("var foo = bar + 'this is a backtick preceded by two backslashes: \\`' + baz;", "var foo = `${bar  }this is a backtick preceded by two backslashes: \\\`${  baz}`;", None),
    // ("var foo = bar + `${baz}foo`;", "var foo = `${bar  }${baz}foo`;", None),
    // ("var foo = bar + baz + 'qux';", "var foo = `${bar + baz  }qux`;", None),
    // ("var foo = /* a */ 'bar' /* b */ + /* c */ baz /* d */ + 'qux' /* e */ ;", "var foo = /* a */ `bar${ /* b */  /* c */ baz /* d */  }qux` /* e */ ;", None),
    // ("var foo = bar + ('baz') + 'qux' + (boop);", "var foo = `${bar  }baz` + `qux${  boop}`;", None),
    // ("foo + 'unescapes an escaped single quote in a single-quoted string: \''", "`${foo  }unescapes an escaped single quote in a single-quoted string: '`", None),
    // (r#"foo + "unescapes an escaped double quote in a double-quoted string: """#, r#"`${foo  }unescapes an escaped double quote in a double-quoted string: "`"#, None),
    // (r#"foo + 'does not unescape an escaped double quote in a single-quoted string: "'"#, r#"`${foo  }does not unescape an escaped double quote in a single-quoted string: "`"#, None),
    // (r#"foo + "does not unescape an escaped single quote in a double-quoted string: \'""#, "`${foo  }does not unescape an escaped single quote in a double-quoted string: \'`", None),
    // ("foo + 'handles unicode escapes correctly: \x27'", "`${foo  }handles unicode escapes correctly: \x27`", None),
    // ("foo + '\\033'", "`${foo  }\\033`", None),
    // ("foo + '\0'", "`${foo  }\0`", None),
    // (r#""default-src 'self' https://*.google.com;"
    // 			            + "frame-ancestors 'none';"
    // 			            + "report-to " + foo + ";""#, "`default-src 'self' https://*.google.com;`
    // 			            + `frame-ancestors 'none';`
    // 			            + `report-to ${  foo  };`", None),
    // ("'a' + 'b' + foo", "`a` + `b${  foo}`", None),
    // ("'a' + 'b' + foo + 'c' + 'd'", "`a` + `b${  foo  }c` + `d`", None),
    // ("'a' + 'b + c' + foo + 'd' + 'e'", "`a` + `b + c${  foo  }d` + `e`", None),
    // ("'a' + 'b' + foo + ('c' + 'd')", "`a` + `b${  foo  }c` + `d`", None),
    // ("'a' + 'b' + foo + ('a' + 'b')", "`a` + `b${  foo  }a` + `b`", None),
    // ("'a' + 'b' + foo + ('c' + 'd') + ('e' + 'f')", "`a` + `b${  foo  }c` + `d` + `e` + `f`", None),
    // ("foo + ('a' + 'b') + ('c' + 'd')", "`${foo  }a` + `b` + `c` + `d`", None),
    // ("'a' + foo + ('b' + 'c') + ('d' + bar + 'e')", "`a${  foo  }b` + `c` + `d${  bar  }e`", None),
    // ("foo + ('b' + 'c') + ('d' + bar + 'e')", "`${foo  }b` + `c` + `d${  bar  }e`", None),
    // ("'a' + 'b' + foo + ('c' + 'd' + 'e')", "`a` + `b${  foo  }c` + `d` + `e`", None),
    // ("'a' + 'b' + foo + ('c' + bar + 'd')", "`a` + `b${  foo  }c${  bar  }d`", None),
    // ("'a' + 'b' + foo + ('c' + bar + ('d' + 'e') + 'f')", "`a` + `b${  foo  }c${  bar  }d` + `e` + `f`", None),
    // ("'a' + 'b' + foo + ('c' + bar + 'e') + 'f' + test", "`a` + `b${  foo  }c${  bar  }e` + `f${  test}`", None),
    // ("'a' + foo + ('b' + bar + 'c') + ('d' + test)", "`a${  foo  }b${  bar  }c` + `d${  test}`", None),
    // ("'a' + foo + ('b' + 'c') + ('d' + bar)", "`a${  foo  }b` + `c` + `d${  bar}`", None),
    // ("foo + ('a' + bar + 'b') + 'c' + test", "`${foo  }a${  bar  }b` + `c${  test}`", None),
    // // ("'a' + '`b`' + c", "`a` + `\`b\`${  c}`", None),
    // // ("'a' + '`b` + `c`' + d", "`a` + `\`b\` + \`c\`${  d}`", None),
    // // ("'a' + b + ('`c`' + '`d`')", "`a${  b  }\`c\`` + `\`d\``", None),
    // // ("'`a`' + b + ('`c`' + '`d`')", "`\`a\`${  b  }\`c\`` + `\`d\``", None),
    // // ("foo + ('`a`' + bar + '`b`') + '`c`' + test", "`${foo  }\`a\`${  bar  }\`b\`` + `\`c\`${  test}`", None),
    // ("'a' + ('b' + 'c') + d", "`a` + `b` + `c${  d}`", None),
    // // ("'a' + ('`b`' + '`c`') + d", "`a` + `\`b\`` + `\`c\`${  d}`", None),
    // ("a + ('b' + 'c') + d", "`${a  }b` + `c${  d}`", None),
    // ("a + ('b' + 'c') + (d + 'e')", "`${a  }b` + `c${  d  }e`", None),
    // // ("a + ('`b`' + '`c`') + d", "`${a  }\`b\`` + `\`c\`${  d}`", None),
    // // ("a + ('`b` + `c`' + '`d`') + e", "`${a  }\`b\` + \`c\`` + `\`d\`${  e}`", None),
    // ("'a' + ('b' + 'c' + 'd') + e", "`a` + `b` + `c` + `d${  e}`", None),
    // ("'a' + ('b' + 'c' + 'd' + (e + 'f') + 'g' +'h' + 'i') + j", "`a` + `b` + `c` + `d${  e  }fg` +`h` + `i${  j}`", None),
    // ("a + (('b' + 'c') + 'd')", "`${a  }b` + `c` + `d`", None),
    // ("(a + 'b') + ('c' + 'd') + e", "`${a  }b` + `c` + `d${  e}`", None),
    // (r#"var foo = "Hello " + "world " + "another " + test"#, "var foo = `Hello ` + `world ` + `another ${  test}`", None),
    // (r#"'Hello ' + '"world" ' + test"#, r#"`Hello ` + `"world" ${  test}`"#, None),
    // (r#""Hello " + "'world' " + test"#, "`Hello ` + `'world' ${  test}`", None)
    //     ];

    //     let _fix = vec![
    //         ("var foo = 'hello, ' + name + '!';", "var foo = `hello, ${  name  }!`;", None),
    // ("var foo = bar + 'baz';", "var foo = `${bar  }baz`;", None),
    // ("var foo = bar + `baz`;", "var foo = `${bar  }baz`;", None),
    // ("var foo = +100 + 'yen';", "var foo = `${+100  }yen`;", None),
    // ("var foo = 'bar' + baz;", "var foo = `bar${  baz}`;", None),
    // ("var foo = '￥' + (n * 1000) + '-'", "var foo = `￥${  n * 1000  }-`", None),
    // ("var foo = 'aaa' + aaa; var bar = 'bbb' + bbb;", "var foo = `aaa${  aaa}`; var bar = `bbb${  bbb}`;", None),
    // ("var string = (number + 1) + 'px';", "var string = `${number + 1  }px`;", None),
    // ("var foo = 'bar' + baz + 'qux';", "var foo = `bar${  baz  }qux`;", None),
    // // ("var foo = '0 backslashes: ${bar}' + baz;", "var foo = `0 backslashes: \${bar}${  baz}`;", None),
    // // ("var foo = '1 backslash: \${bar}' + baz;", "var foo = `1 backslash: \${bar}${  baz}`;", None),
    // // ("var foo = '2 backslashes: \\${bar}' + baz;", "var foo = `2 backslashes: \\\${bar}${  baz}`;", None),
    // // ("var foo = '3 backslashes: \\\${bar}' + baz;", "var foo = `3 backslashes: \\\${bar}${  baz}`;", None),
    // // ("var foo = bar + 'this is a backtick: `' + baz;", "var foo = `${bar  }this is a backtick: \`${  baz}`;", None),
    // // ("var foo = bar + 'this is a backtick preceded by a backslash: \`' + baz;", "var foo = `${bar  }this is a backtick preceded by a backslash: \`${  baz}`;", None),
    // // ("var foo = bar + 'this is a backtick preceded by two backslashes: \\`' + baz;", "var foo = `${bar  }this is a backtick preceded by two backslashes: \\\`${  baz}`;", None),
    // ("var foo = bar + `${baz}foo`;", "var foo = `${bar  }${baz}foo`;", None),
    // ("var foo = bar + baz + 'qux';", "var foo = `${bar + baz  }qux`;", None),
    // ("var foo = /* a */ 'bar' /* b */ + /* c */ baz /* d */ + 'qux' /* e */ ;", "var foo = /* a */ `bar${ /* b */  /* c */ baz /* d */  }qux` /* e */ ;", None),
    // ("var foo = bar + ('baz') + 'qux' + (boop);", "var foo = `${bar  }baz` + `qux${  boop}`;", None),
    // ("foo + 'unescapes an escaped single quote in a single-quoted string: \''", "`${foo  }unescapes an escaped single quote in a single-quoted string: '`", None),
    // (r#"foo + "unescapes an escaped double quote in a double-quoted string: """#, r#"`${foo  }unescapes an escaped double quote in a double-quoted string: "`"#, None),
    // (r#"foo + 'does not unescape an escaped double quote in a single-quoted string: "'"#, r#"`${foo  }does not unescape an escaped double quote in a single-quoted string: "`"#, None),
    // (r#"foo + "does not unescape an escaped single quote in a double-quoted string: \'""#, "`${foo  }does not unescape an escaped single quote in a double-quoted string: \'`", None),
    // ("foo + 'handles unicode escapes correctly: \x27'", "`${foo  }handles unicode escapes correctly: \x27`", None),
    // ("foo + '\\033'", "`${foo  }\\033`", None),
    // ("foo + '\0'", "`${foo  }\0`", None),
    // (r#""default-src 'self' https://*.google.com;"
    // 			            + "frame-ancestors 'none';"
    // 			            + "report-to " + foo + ";""#, "`default-src 'self' https://*.google.com;`
    // 			            + `frame-ancestors 'none';`
    // 			            + `report-to ${  foo  };`", None),
    // ("'a' + 'b' + foo", "`a` + `b${  foo}`", None),
    // ("'a' + 'b' + foo + 'c' + 'd'", "`a` + `b${  foo  }c` + `d`", None),
    // ("'a' + 'b + c' + foo + 'd' + 'e'", "`a` + `b + c${  foo  }d` + `e`", None),
    // ("'a' + 'b' + foo + ('c' + 'd')", "`a` + `b${  foo  }c` + `d`", None),
    // ("'a' + 'b' + foo + ('a' + 'b')", "`a` + `b${  foo  }a` + `b`", None),
    // ("'a' + 'b' + foo + ('c' + 'd') + ('e' + 'f')", "`a` + `b${  foo  }c` + `d` + `e` + `f`", None),
    // ("foo + ('a' + 'b') + ('c' + 'd')", "`${foo  }a` + `b` + `c` + `d`", None),
    // ("'a' + foo + ('b' + 'c') + ('d' + bar + 'e')", "`a${  foo  }b` + `c` + `d${  bar  }e`", None),
    // ("foo + ('b' + 'c') + ('d' + bar + 'e')", "`${foo  }b` + `c` + `d${  bar  }e`", None),
    // ("'a' + 'b' + foo + ('c' + 'd' + 'e')", "`a` + `b${  foo  }c` + `d` + `e`", None),
    // ("'a' + 'b' + foo + ('c' + bar + 'd')", "`a` + `b${  foo  }c${  bar  }d`", None),
    // ("'a' + 'b' + foo + ('c' + bar + ('d' + 'e') + 'f')", "`a` + `b${  foo  }c${  bar  }d` + `e` + `f`", None),
    // ("'a' + 'b' + foo + ('c' + bar + 'e') + 'f' + test", "`a` + `b${  foo  }c${  bar  }e` + `f${  test}`", None),
    // ("'a' + foo + ('b' + bar + 'c') + ('d' + test)", "`a${  foo  }b${  bar  }c` + `d${  test}`", None),
    // ("'a' + foo + ('b' + 'c') + ('d' + bar)", "`a${  foo  }b` + `c` + `d${  bar}`", None),
    // ("foo + ('a' + bar + 'b') + 'c' + test", "`${foo  }a${  bar  }b` + `c${  test}`", None),
    // // ("'a' + '`b`' + c", "`a` + `\`b\`${  c}`", None),
    // // ("'a' + '`b` + `c`' + d", "`a` + `\`b\` + \`c\`${  d}`", None),
    // // ("'a' + b + ('`c`' + '`d`')", "`a${  b  }\`c\`` + `\`d\``", None),
    // // ("'`a`' + b + ('`c`' + '`d`')", "`\`a\`${  b  }\`c\`` + `\`d\``", None),
    // // ("foo + ('`a`' + bar + '`b`') + '`c`' + test", "`${foo  }\`a\`${  bar  }\`b\`` + `\`c\`${  test}`", None),
    // ("'a' + ('b' + 'c') + d", "`a` + `b` + `c${  d}`", None),
    // // ("'a' + ('`b`' + '`c`') + d", "`a` + `\`b\`` + `\`c\`${  d}`", None),
    // ("a + ('b' + 'c') + d", "`${a  }b` + `c${  d}`", None),
    // ("a + ('b' + 'c') + (d + 'e')", "`${a  }b` + `c${  d  }e`", None),
    // // ("a + ('`b`' + '`c`') + d", "`${a  }\`b\`` + `\`c\`${  d}`", None),
    // // ("a + ('`b` + `c`' + '`d`') + e", "`${a  }\`b\` + \`c\`` + `\`d\`${  e}`", None),
    // ("'a' + ('b' + 'c' + 'd') + e", "`a` + `b` + `c` + `d${  e}`", None),
    // ("'a' + ('b' + 'c' + 'd' + (e + 'f') + 'g' +'h' + 'i') + j", "`a` + `b` + `c` + `d${  e  }fg` +`h` + `i${  j}`", None),
    // ("a + (('b' + 'c') + 'd')", "`${a  }b` + `c` + `d`", None),
    // ("(a + 'b') + ('c' + 'd') + e", "`${a  }b` + `c` + `d${  e}`", None),
    // (r#"var foo = "Hello " + "world " + "another " + test"#, "var foo = `Hello ` + `world ` + `another ${  test}`", None),
    // (r#"'Hello ' + '"world" ' + test"#, r#"`Hello ` + `"world" ${  test}`"#, None),
    // (r#""Hello " + "'world' " + test"#, "`Hello ` + `'world' ${  test}`", None)
    //     ];
    Tester::new(PreferTemplate::NAME, PreferTemplate::PLUGIN, pass, fail).test_and_snapshot();
}
