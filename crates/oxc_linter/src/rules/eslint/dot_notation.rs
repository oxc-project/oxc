use crate::{AstNode, context::LintContext, rule::Rule};
use lazy_static::lazy_static;
use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, MemberExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::{Regex, RegexBuilder};

fn dot_notation_use_dot_diagnostic(span: Span, incorrect: &str, correct: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("[{}] is better written in dot notation", incorrect))
        .with_help(format!("Write it as .{}", correct))
        .with_label(span)
}

fn dot_notation_use_brackets_diagnostic(span: Span, key: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(".{} is a syntax error", key))
        .with_help(format!("Write it using bracket notation [\"{}\"]", key))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DotNotation(Box<DotNotationConfig>);

#[derive(Debug, Clone)]
pub struct DotNotationConfig {
    allow_keywords: bool,
    allow_pattern: Option<Regex>,
}

impl Default for DotNotationConfig {
    fn default() -> Self {
        Self { allow_keywords: true, allow_pattern: None }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule enforces dot notation whenever possible.
    ///
    /// ### Why is this bad?
    /// In JavaScript, one can access properties using the dot notation (foo.bar) or square-bracket
    /// notation (foo["bar"]). However, the dot notation is often preferred because it is easier to
    /// read, less verbose, and works better with aggressive JavaScript minimizers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const x = foo["bar"];
    ///
    /// // ** For sample config { "allowKeywords": false } **
    /// const foo = { "class": "CS 101" }
    /// const x = foo["class"]; // Property name is a reserved word, square-bracket notation
    ///                         // required
    ///
    /// class C {
    ///     #in;
    ///     foo() {
    ///         this.#in; // Dot notation is required for private identifiers
    ///     }
    /// }
    ///
    /// // ** For sample config { "allowPattern": "^[a-z]+(_[a-z]+)+$" } **
    /// const data = {};
    /// data["fooBar"] = 42;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const x = foo.bar;
    /// const y = foo[bar];
    ///
    /// // For sample config { "allowPattern": "^[a-z]+(_[a-z]+)+$" }
    /// const data = {};
    /// data["foo_bar"] = 42;
    /// ```
    DotNotation,
    eslint,
    style,
    // Fixes are possible
    // See <https://github.com/eslint/eslint/blob/main/lib/rules/dot-notation.js>
    pending
);

/// List of ES3 keywords.
const KEYWORDS: &[&str] = &[
    "abstract",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "function",
    "goto",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "volatile",
    "while",
    "with",
];
lazy_static! {
    static ref VALID_IDENTIFER: Regex = Regex::new(r"^[a-zA-Z_$][a-zA-Z0-9_$]*$").unwrap();
}

impl Rule for DotNotation {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut cfg = DotNotationConfig::default();

        if let Some(config) = value.get(0) {
            if let Some(val) = config.get("allowPattern").and_then(serde_json::Value::as_str) {
                cfg.allow_pattern = RegexBuilder::new(val).build().ok();
            }
            if let Some(val) = config.get("allowKeywords").and_then(serde_json::Value::as_bool) {
                cfg.allow_keywords = val;
            }
        }

        Self(Box::new(cfg))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::MemberExpression(node) = &node.kind() {
            match &node {
                MemberExpression::ComputedMemberExpression(expr) => {
                    let inner_expr =
                        if let Expression::ParenthesizedExpression(inner_expr) = &expr.expression {
                            &inner_expr.expression
                        } else {
                            &expr.expression
                        };

                    if let Some((before, after)) = match &inner_expr {
                        Expression::NullLiteral(x) => Some((x.to_string(), x.to_string())),
                        Expression::BooleanLiteral(x) => Some((x.to_string(), x.to_string())),
                        Expression::StringLiteral(x) => Some((x.to_string(), x.to_string())),
                        Expression::TemplateLiteral(x) => {
                            if x.expressions.len() == 0 && x.quasis.len() == 1 {
                                let s = x.quasis[0].value.cooked.unwrap().into_string();
                                Some((s.clone(), s))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    } {
                        if VALID_IDENTIFER.is_match(&after)
                            && (self.0.allow_keywords || !KEYWORDS.contains(&after.as_str()))
                            && !(self.0.allow_pattern.is_some()
                                && self.0.allow_pattern.as_ref().unwrap().is_match(&after))
                        {
                            ctx.diagnostic(dot_notation_use_dot_diagnostic(
                                expr.span, &before, &after,
                            ))
                        }
                    }
                }
                MemberExpression::StaticMemberExpression(expr) => {
                    if !self.0.allow_keywords && KEYWORDS.contains(&expr.property.name.as_str()) {
                        ctx.diagnostic(dot_notation_use_brackets_diagnostic(
                            expr.span,
                            &expr.property.name,
                        ))
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("a.b;", None),
        ("a.b.c;", None),
        ("a['12'];", None),
        ("a[b];", None),
        ("a[0];", None),
        ("a.b.c;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.arguments;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.let;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.yield;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.eval;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a[0];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['while'];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['true'];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['null'];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a[true];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a[null];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.true;", Some(serde_json::json!([{ "allowKeywords": true }]))),
        ("a.null;", Some(serde_json::json!([{ "allowKeywords": true }]))),
        ("a['snake_case'];", Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }]))),
        (
            "a['lots_of_snake_case'];",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        ("a[`time${range}`];", None), // { "ecmaVersion": 6 },
        ("a[`while`];", Some(serde_json::json!([{ "allowKeywords": false }]))), // { "ecmaVersion": 6 },
        ("a[`time range`];", None), // { "ecmaVersion": 6 },
        ("a.true;", None),
        ("a.null;", None),
        ("a[undefined];", None),
        ("a[void 0];", None),
        ("a[b()];", None),
        ("a[/(?<zero>0)/];", None), // { "ecmaVersion": 2018 },
        ("class C { foo() { this['#a'] } }", None), // { "ecmaVersion": 2022 },
        (
            "class C { #in; foo() { this.#in; } }",
            Some(serde_json::json!([{ "allowKeywords": false }])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("a.true;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['true'];", None),
        ("a[`time`];", None), // { "ecmaVersion": 6 },
        ("a[null];", None),
        ("a[true];", None),
        ("a[false];", None),
        ("a['b'];", None),
        ("a.b['c'];", None),
        ("a['_dangle'];", Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }]))),
        ("a['SHOUT_CASE'];", Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }]))),
        ("foo[ /* comment */ 'bar' ]", None),
        ("foo[ 'bar' /* comment */ ]", None),
        ("foo[    'bar'    ];", None),
        ("foo. /* comment */ while", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("foo[('bar')]", None),
        ("foo[(null)]", None),
        ("(foo)['bar']", None),
        ("1['toString']", None),
        ("foo['bar']instanceof baz", None),
        ("let.if()", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("5['prop']", None),
        ("-5['prop']", None),
        ("01['prop']", None),
        ("01234567['prop']", None),
        ("08['prop']", None),
        ("090['prop']", None),
        ("018['prop']", None),
        ("5_000['prop']", None),       // { "ecmaVersion": 2021 },
        ("5_000_00['prop']", None),    // { "ecmaVersion": 2021 },
        ("5.000_000['prop']", None),   // { "ecmaVersion": 2021 },
        ("0b1010_1010['prop']", None), // { "ecmaVersion": 2021 },
        ("obj?.['prop']", None),       // { "ecmaVersion": 2020 },
        ("0?.['prop']", None),         // { "ecmaVersion": 2020 },
        ("obj?.true", Some(serde_json::json!([{ "allowKeywords": false }]))), // { "ecmaVersion": 2020 },
        ("let?.true", Some(serde_json::json!([{ "allowKeywords": false }]))), // { "ecmaVersion": 2020 }
    ];

    // TODO: Implement fixes
    let _fix = vec![
        ("a.true;", r#"a["true"];"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['true'];", "a.true;", None),
        ("a[`time`];", "a.time;", None),
        ("a[null];", "a.null;", None),
        ("a[true];", "a.true;", None),
        ("a[false];", "a.false;", None),
        ("a['b'];", "a.b;", None),
        ("a.b['c'];", "a.b.c;", None),
        (
            "a['_dangle'];",
            "a._dangle;",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        (
            "a['SHOUT_CASE'];",
            "a.SHOUT_CASE;",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        ("foo[    'bar'    ];", "foo.bar;", None),
        ("foo[('bar')]", "foo.bar", None),
        ("foo[(null)]", "foo.null", None),
        ("(foo)['bar']", "(foo).bar", None),
        ("1['toString']", "1 .toString", None),
        ("foo['bar']instanceof baz", "foo.bar instanceof baz", None),
        ("5['prop']", "5 .prop", None),
        ("-5['prop']", "-5 .prop", None),
        ("01['prop']", "01.prop", None),
        ("01234567['prop']", "01234567.prop", None),
        ("08['prop']", "08 .prop", None),
        ("090['prop']", "090 .prop", None),
        ("018['prop']", "018 .prop", None),
        ("5_000['prop']", "5_000 .prop", None),
        ("5_000_00['prop']", "5_000_00 .prop", None),
        ("5.000_000['prop']", "5.000_000.prop", None),
        ("0b1010_1010['prop']", "0b1010_1010.prop", None),
        ("obj?.['prop']", "obj?.prop", None),
        ("0?.['prop']", "0?.prop", None),
        ("obj?.true", r#"obj?.["true"]"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("let?.true", r#"let?.["true"]"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
    ];
    Tester::new(DotNotation::NAME, DotNotation::PLUGIN, pass, fail).test_and_snapshot();
}
