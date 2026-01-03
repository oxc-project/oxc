use crate::{AstNode, context::LintContext, rule::Rule};
use lazy_regex::Regex;
use oxc_ast::AstKind;
use oxc_ast::ast::{ComputedMemberExpression, Expression, StaticMemberExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::number::NumberBase;
use schemars::JsonSchema;
use serde_json::Value;
use std::ops::Deref;

fn use_dot_notation_diagnostic(
    span: Span,
    identifier: &str,
    key: &str,
    optional: bool,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("[\"{key}\"] is better written in dot notation."))
        .with_help(format!(
            "Replace {identifier}{}[\"{key}\"] with {identifier}{}{key}",
            if optional { "?." } else { "" },
            if optional { "?." } else { "." },
        ))
        .with_label(span)
}

fn use_brackets_diagnostic(
    span: Span,
    identifier: &str,
    key: &str,
    optional: bool,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(".{key} is a syntax error."))
        .with_help(format!(
            "Replace {identifier}{}{key} with {identifier}{}[\"{key}\"]",
            if optional { "?." } else { "." },
            if optional { "?." } else { "" },
        ))
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema)]
#[schemars(rename_all = "camelCase")]
pub struct DotNotationConfig {
    /// Set the `allowKeywords` option to `false` (default is `true`)
    /// to follow ECMAScript version 3 compatible style,
    /// avoiding dot notation for reserved word properties.
    allow_keywords: bool,
    /// Set the `allowPattern` option to a regular expression string
    /// to allow bracket notation for property names that match a pattern
    /// (by default, no pattern is tested).
    #[serde(with = "Regex")]
    allow_pattern: Option<Regex>,
}

impl Default for DotNotationConfig {
    fn default() -> Self {
        Self { allow_keywords: true, allow_pattern: None }
    }
}

#[derive(Debug, Default, Clone, JsonSchema)]
pub struct DotNotation(Box<DotNotationConfig>);

impl Deref for DotNotation {
    type Target = DotNotationConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces dot notation to access object properties whenever possible.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, one can access properties using the dot notation (foo.bar)
    /// or square-bracket notation (foo["bar"]).
    /// However, the dot notation is often preferred because it is easier to read, less verbose,
    /// and works better with aggressive JavaScript minimizers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const x = foo["bar"];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const x = foo.bar;
    ///
    /// const y = foo[bar]; // Property name is a variable, square-bracket notation required
    /// ```
    ///
    /// Examples of **correct** code for the `{ "allowKeywords": false }` option:
    /// ```js
    /// const foo = { "class": "CS 101" }
    /// const x = foo["class"]; // Property name is a reserved word, square-bracket notation required
    /// ```
    ///
    /// Examples of additional **correct** code for the `{ "allowKeywords": false }` option:
    /// ```js
    /// class C {
    ///     #in;
    ///     foo() {
    ///         this.#in; // Dot notation is required for private identifiers
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for the sample `{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }`
    /// (pattern to find snake case named properties) option:
    /// ```js
    /// const data = {};
    /// data["fooBar"] = 42;
    /// ```
    ///
    /// Examples of **correct** code for the sample `{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }`
    /// (pattern to find snake case named properties) option:
    /// ```js
    /// const data = {};
    /// data["foo_bar"] = 42;
    /// ```
    DotNotation,
    eslint,
    style,
    fix,
    config = DotNotation,
);

impl Rule for DotNotation {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        let allow_keywords = config
            .and_then(|config| config.get("allowKeywords"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let allow_pattern =
            config.and_then(|config| config.get("allowPattern")).and_then(Value::as_str);

        Self(Box::new(DotNotationConfig {
            allow_keywords,
            allow_pattern: if let Some(allow_pattern) = &allow_pattern {
                Regex::new(allow_pattern).ok()
            } else {
                None
            },
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ComputedMemberExpression(node) => {
                self.check_computed_member_expression(node, ctx);
            }
            AstKind::StaticMemberExpression(node) => self.check_static_member_expression(node, ctx),
            _ => (),
        }
    }
}

impl DotNotation {
    fn check_computed_member_expression<'a>(
        &self,
        node: &ComputedMemberExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let value = match node.expression.get_inner_expression() {
            Expression::NullLiteral(x) => x.to_string(),
            Expression::BooleanLiteral(x) => x.to_string(),
            Expression::StringLiteral(x) => x.to_string(),
            Expression::TemplateLiteral(x) => {
                if x.expressions.is_empty() && x.quasis.len() == 1 {
                    x.quasis[0].value.cooked.unwrap().to_string()
                } else {
                    return;
                }
            }
            _ => return,
        };
        if !is_valid_identifier(&value) {
            return;
        }
        if !self.allow_keywords && is_keyword(value.as_str()) {
            return;
        }
        if let Some(allow_pattern) = &self.allow_pattern
            && allow_pattern.is_match(&value)
        {
            return;
        }
        let object_name = node.object.span().source_text(ctx.source_text());
        ctx.diagnostic_with_fix(
            use_dot_notation_diagnostic(node.span, object_name, &value, node.optional),
            |fixer| {
                let insert_whitespace_after_identifier =
                    if let Expression::NumericLiteral(num) = &node.object {
                        num.base == NumberBase::Decimal && !node.optional
                    } else {
                        false
                    };
                let next_token_start =
                    usize::try_from(node.span.end + 1).expect("span index does not fit into usize");
                let insert_whitespace_after_expression = ctx.source_text().len() > next_token_start
                    && !ctx
                        .source_text()
                        .as_bytes()
                        .get(next_token_start)
                        .unwrap()
                        .is_ascii_whitespace();

                fixer.replace(
                    node.span,
                    format!(
                        "{}{}{}.{}{}",
                        object_name,
                        if insert_whitespace_after_identifier { " " } else { "" },
                        if node.optional { "?" } else { "" },
                        value,
                        if insert_whitespace_after_expression { " " } else { "" },
                    ),
                )
            },
        );
    }

    fn check_static_member_expression<'a>(
        &self,
        node: &StaticMemberExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if !self.allow_keywords && is_keyword(node.property.name.as_str()) {
            let object_name = node.object.span().source_text(ctx.source_text());
            ctx.diagnostic_with_fix(
                use_brackets_diagnostic(node.span, object_name, &node.property.name, node.optional),
                |fixer| {
                    fixer.replace(
                        node.span,
                        format!(
                            "{}{}[\"{}\"]",
                            object_name,
                            if node.optional { "?." } else { "" },
                            node.property.name,
                        ),
                    )
                },
            );
        }
    }
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.bytes();
    chars.next().is_some_and(|c|
        /* a-zA-Z_$ */ c.is_ascii_alphabetic() || c == b'_' || c == b'$')
        && chars.all(|c| /* a-zA-Z0-9_$ */ c.is_ascii_alphanumeric() || c == b'_' || c == b'$')
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "abstract"
            | "boolean"
            | "break"
            | "byte"
            | "case"
            | "catch"
            | "char"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "double"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "final"
            | "finally"
            | "float"
            | "for"
            | "function"
            | "goto"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "int"
            | "interface"
            | "long"
            | "native"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "short"
            | "static"
            | "super"
            | "switch"
            | "synchronized"
            | "this"
            | "throw"
            | "throws"
            | "transient"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "volatile"
            | "while"
            | "with"
    )
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

    let fix = vec![
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
        (
            "(let?.true)",
            r#"(let?.["true"])"#,
            Some(serde_json::json!([{ "allowKeywords": false }])),
        ),
    ];
    Tester::new(DotNotation::NAME, DotNotation::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
