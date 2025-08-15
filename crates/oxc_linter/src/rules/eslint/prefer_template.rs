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
        // check is the outermost binary expression
        let not_outermost = ctx.nodes().ancestor_kinds(node.id()).any(
            |v| matches!(v, AstKind::BinaryExpression(e) if e.operator == BinaryOperator::Addition),
        );
        if not_outermost {
            return;
        }
        if check_should_report(expr) {
            ctx.diagnostic(prefer_template_diagnostic(expr.span));
        }
    }
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
