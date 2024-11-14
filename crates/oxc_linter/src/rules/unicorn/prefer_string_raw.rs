use oxc_ast::{
    ast::{Expression, JSXAttributeValue, PropertyKey},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::StringCharAt;
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct PreferStringRaw;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```javascript
    /// TODO
    /// ```
    PreferStringRaw,
    restriction,
);

fn unescape_backslash(input: &str, quote: &char) -> Option<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.peek() {
                if *next == '\\' || *next == *quote {
                    result.push(*next);
                    chars.next();
                    continue;
                }
            }
        }

        result.push(c);
    }

    Some(result)
}

impl Rule for PreferStringRaw {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StringLiteral(string_literal) = node.kind() else {
            return;
        };

        let parent_node = ctx.nodes().parent_node(node.id());

        dbg!(&string_literal);

        if let Some(parent_node) = parent_node {
            match parent_node.kind() {
                AstKind::Directive(_) => {
                    return;
                }
                AstKind::ImportDeclaration(decl) => {
                    if string_literal.span == decl.source.span {
                        return;
                    }
                }
                AstKind::ExportNamedDeclaration(decl) => {
                    if let Some(source) = &decl.source {
                        if string_literal.span == source.span {
                            return;
                        }
                    }
                }
                AstKind::ExportAllDeclaration(decl) => {
                    if string_literal.span == decl.source.span {
                        return;
                    }
                }
                AstKind::ObjectProperty(prop) => {
                    dbg!(&string_literal);
                    dbg!(&prop);
                    let PropertyKey::StringLiteral(key) = &prop.key else {
                        return;
                    };

                    dbg!(&key);

                    if !prop.computed && string_literal.span == key.span {
                        return;
                    }
                }
                AstKind::PropertyKey(key) => {
                    let PropertyKey::StringLiteral(key) = &key else {
                        return;
                    };

                    if string_literal.span == key.span {
                        return;
                    }
                }
                AstKind::JSXAttributeItem(attr) => {
                    let Some(attr) = attr.as_attribute() else {
                        return;
                    };

                    let Some(JSXAttributeValue::StringLiteral(value)) = &attr.value else {
                        return;
                    };

                    dbg!(&value);

                    if value.span == string_literal.span {
                        return;
                    }
                }
                AstKind::TSEnumMember(member) => {
                    println!("!!!!!");
                    let Some(Expression::StringLiteral(value)) = &member.initializer else {
                        return;
                    };

                    println!("~~~~ {:?}", value.span);
                    println!("^^^^ {:?}", string_literal.span);

                    if value.span == string_literal.span {
                        println!("SAME");
                        return;
                    }
                }
                _ => {}
            }
        }

        println!("AFTER");

        let raw = ctx.source_range(string_literal.span);

        let last_char_index = raw.len() - 2;
        if let Some('\\') = raw.char_at(Some(last_char_index as f64)) {
            return;
        };

        if !raw.contains("\\\\") || raw.contains("`") || raw.contains("${") {
            return;
        }

        let trimmed = ctx.source_range(string_literal.span.shrink(1));

        let Some(quote) = raw.char_at(Some(0.0)) else {
            return;
        };

        let Some(unescaped) = unescape_backslash(trimmed, &quote) else {
            return;
        };

        // dbg!(&unescaped);
        // dbg!(string_literal.value.as_ref());
        dbg!(string_literal.value.as_ref() != unescaped);

        if unescaped != string_literal.value.as_ref() {
            return;
        }

        ctx.diagnostic(
            OxcDiagnostic::warn(r"`String.raw` should be used to avoid escaping `\\`.")
                .with_label(string_literal.span),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<&str> = vec![
        // r"a = '\''",
        // r"'a\\b'",
        // r#"import foo from "./foo\\bar.js";"#,
        // r#"export {foo} from "./foo\\bar.js";"#,
        // r#"export * from "./foo\\bar.js";"#,
        // r"a = {'a\\b': 123}",
        // "
        //  	a = '\\\\a \\
        //  		b'
        //  ",
        // r"a = 'a\\b\u{51}c'",
        // "a = 'a\\\\b`'",
        // "a = 'a\\\\b${foo}'",
        // r#"<Component attribute="a\\b" />"#,
        // r#"
        //      enum Files {
        //      	Foo = "C:\\\\path\\\\to\\\\foo.js",
        //      }
        //  "#,
        r#"
             enum Foo {
             	"\\\\a\\\\b" = "baz",
             }
         "#,
    ];

    let fail = vec![
        // r"a = 'a\\b'",
        // r"a = {['a\\b']: b}",
        // r"function a() {return'a\\b'}",
        // r"const foo = 'foo \\x46';",
    ];

    Tester::new(PreferStringRaw::NAME, pass, fail).test_and_snapshot();
}
