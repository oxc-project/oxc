use crate::{ast_util::get_static_property_name, context::LintContext, rule::Rule, AstNode};
use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use regex::Regex;

fn new_cap_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NewCap(Box<NewCapConfig>);

#[derive(Debug, Default, Clone)]
pub struct NewCapConfig {
    new_is_cap: bool,
    cap_is_new: bool,
    new_is_cap_exceptions: Vec<CompactStr>,
    new_is_cap_exception_pattern: Option<Regex>,
    cap_is_new_exceptions: Vec<CompactStr>,
    cap_is_new_exception_pattern: Option<Regex>,
    properties: bool,
}

impl std::ops::Deref for NewCap {
    type Target = NewCapConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn bool_serde_value(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> bool {
    let Some(value) = map.get(key) else {
        return true; // default value
    };

    let err = format!("eslint/new-cap: expect configuration option '{key}' to be a boolean.");

    value.as_bool().expect(&err)
}

fn vec_str_serde_value(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    default_value: Vec<CompactStr>,
) -> Vec<CompactStr> {
    let Some(value) = map.get(key) else {
        return default_value; // default value
    };
    let err = format!("eslint/new-cap: expect configuration option '{key}' to be an array.");
    let err2 = format!(
        "eslint/new-cap: expect array configuration option '{key}' to only contain strings."
    );

    value
        .as_array()
        .expect(&err)
        .iter()
        .map(|value| CompactStr::new(value.as_str().expect(&err2)))
        .collect::<Vec<CompactStr>>()
}

impl From<&serde_json::Value> for NewCap {
    fn from(raw: &serde_json::Value) -> Self {
        let Some(config_entry) = raw.get(0) else {
            return Self(Box::new(NewCapConfig {
                new_is_cap: true,
                cap_is_new: true,
                new_is_cap_exceptions: caps_allowed_vec(),
                new_is_cap_exception_pattern: None,
                cap_is_new_exceptions: vec![],
                cap_is_new_exception_pattern: None,
                properties: true,
            }));
        };

        let config = config_entry
            .as_object()
            .map_or_else(
                || {
                    Err(OxcDiagnostic::warn(
                        "eslint/new-cap: invalid configuration, expected object.",
                    ))
                },
                Ok,
            )
            .unwrap();

        Self(Box::new(NewCapConfig {
            new_is_cap: bool_serde_value(config, "newIsCap"),
            cap_is_new: bool_serde_value(config, "capIsNew"),
            new_is_cap_exceptions: vec_str_serde_value(
                config,
                "newIsCapExceptions",
                caps_allowed_vec(),
            ),
            new_is_cap_exception_pattern: None,
            cap_is_new_exceptions: vec_str_serde_value(config, "capIsNewExceptions", vec![]),
            cap_is_new_exception_pattern: None,
            properties: bool_serde_value(config, "properties"),
        }))
    }
}

const CAPS_ALLOWED: [&str; 11] = [
    "Array", "Boolean", "Date", "Error", "Function", "Number", "Object", "RegExp", "String",
    "Symbol", "BigInt",
];

fn caps_allowed_vec() -> Vec<CompactStr> {
    CAPS_ALLOWED.iter().map(|x| CompactStr::new(x)).collect::<Vec<CompactStr>>()
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
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
    NewCap,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NewCap {
    fn from_configuration(value: serde_json::Value) -> Self {
        NewCap::from(&value)
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(expression) if self.new_is_cap => {
                let Some(short_name) =
                    extract_name_from_new_expression(&expression.callee, &node.kind())
                else {
                    return;
                };

                let Some(name) =
                    &extract_name_deep_from_new_expression(&expression.callee, &node.kind())
                else {
                    return;
                };

                let capitalization = get_cap(&short_name);

                let allowed = capitalization != GetCapResult::Lower
                    || is_cap_allowed_expression(
                        &expression.callee,
                        name,
                        &self.new_is_cap_exceptions,
                        &self.new_is_cap_exception_pattern,
                    );

                if !allowed {
                    ctx.diagnostic(new_cap_diagnostic(expression.span));
                }
            }
            AstKind::CallExpression(expression) if self.cap_is_new => {
                let Some(short_name) =
                    extract_name_from_new_expression(&expression.callee, &node.kind())
                else {
                    return;
                };

                let Some(name) =
                    &extract_name_deep_from_new_expression(&expression.callee, &node.kind())
                else {
                    return;
                };

                let capitalization = get_cap(&short_name);

                let mut caps_is_new_exceptions = self.cap_is_new_exceptions.clone();
                caps_is_new_exceptions.append(&mut caps_allowed_vec());

                let allowed = capitalization != GetCapResult::Upper
                    || is_cap_allowed_expression(
                        &expression.callee,
                        name,
                        &caps_is_new_exceptions,
                        &self.cap_is_new_exception_pattern,
                    );

                if !allowed {
                    ctx.diagnostic(new_cap_diagnostic(expression.span));
                }
            }
            _ => (),
        }
    }
}

fn extract_name_deep_from_new_expression(
    expression: &Expression,
    kind: &AstKind,
) -> Option<CompactStr> {
    if let Some(identifier) = expression.get_identifier_reference() {
        return Some(identifier.name.clone().into());
    }

    match expression {
        Expression::StaticMemberExpression(expression) => {
            let obj_name = extract_name_deep_from_new_expression(&expression.object, kind);
            let prop_name = expression.property.name.clone().into_compact_str();

            if let Some(obj_name) = obj_name {
                let new_name = format!("{obj_name}.{prop_name}");
                return Some(CompactStr::new(&new_name));
            }

            Some(prop_name)
        }
        _ => get_static_property_name(kind).map(std::convert::Into::into),
    }
}

fn extract_name_from_new_expression(expression: &Expression, kind: &AstKind) -> Option<CompactStr> {
    if let Some(identifier) = expression.get_identifier_reference() {
        return Some(identifier.name.clone().into());
    }

    get_static_property_name(kind).map(std::convert::Into::into)
}

fn is_cap_allowed_expression(
    expression: &Expression<'_>,
    name: &CompactStr,
    exceptions: &Vec<CompactStr>,
    patterns: &Option<Regex>,
) -> bool {
    if exceptions.contains(name) {
        return true;
    }

    false
}

#[derive(PartialEq, Debug)]
enum GetCapResult {
    Upper,
    Lower,
    NonAlpha,
}

fn get_cap(string: &CompactStr) -> GetCapResult {
    let first_char = string.chars().next().unwrap();

    if !first_char.is_alphabetic() {
        return GetCapResult::NonAlpha;
    }

    if first_char.is_lowercase() {
        return GetCapResult::Lower;
    }

    GetCapResult::Upper
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = new Constructor();", None),
        ("var x = new a.b.Constructor();", None),
        ("var x = new a.b['Constructor']();", None),
        ("var x = new a.b[Constructor]();", None),
        ("var x = new a.b[constructor]();", None),
        ("var x = new function(){};", None),
        ("var x = new _;", None),
        ("var x = new $;", None),
        ("var x = new Σ;", None),
        ("var x = new _x;", None),
        ("var x = new $x;", None),
        ("var x = new this;", None),
        ("var x = Array(42)", None),
        ("var x = Boolean(42)", None),
        ("var x = Date(42)", None),
        // ("var x = Date.UTC(2000, 0)", None),
        ("var x = Error('error')", None),
        ("var x = Function('return 0')", None),
        ("var x = Number(42)", None),
        ("var x = Object(null)", None),
        ("var x = RegExp(42)", None),
        ("var x = String(42)", None),
        ("var x = Symbol('symbol')", None),
        ("var x = BigInt('1n')", None),
        ("var x = _();", None),
        ("var x = $();", None),
        ("var x = Foo(42)", Some(serde_json::json!([{ "capIsNew": false }]))),
        ("var x = bar.Foo(42)", Some(serde_json::json!([{ "capIsNew": false }]))),
        ("var x = Foo.bar(42)", Some(serde_json::json!([{ "capIsNew": false }]))),
        ("var x = bar[Foo](42)", None),
        ("var x = bar['Foo'](42)", Some(serde_json::json!([{ "capIsNew": false }]))),
        ("var x = Foo.bar(42)", None),
        ("var x = new foo(42)", Some(serde_json::json!([{ "newIsCap": false }]))),
        ("var o = { 1: function() {} }; o[1]();", None),
        ("var o = { 1: function() {} }; new o[1]();", None),
        (
            "var x = Foo(42);",
            Some(serde_json::json!([{ "capIsNew": true, "capIsNewExceptions": ["Foo"] }])),
        ),
        // ("var x = Foo(42);", Some(serde_json::json!([{ "capIsNewExceptionPattern": "^Foo" }]))),
        (
            "var x = new foo(42);",
            Some(serde_json::json!([{ "newIsCap": true, "newIsCapExceptions": ["foo"] }])),
        ),
        // ("var x = new foo(42);", Some(serde_json::json!([{ "newIsCapExceptionPattern": "^foo" }]))),
        ("var x = Object(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Foo"] }]))),
        ("var x = Foo.Bar(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Bar"] }]))),
        ("var x = Foo.Bar(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Foo.Bar"] }]))),
        // (
        //     "var x = Foo.Bar(42);",
        //     Some(serde_json::json!([{ "capIsNewExceptionPattern": "^Foo\\.." }])),
        // ),
        ("var x = new foo.bar(42);", Some(serde_json::json!([{ "newIsCapExceptions": ["bar"] }]))),
        (
            "var x = new foo.bar(42);",
            Some(serde_json::json!([{ "newIsCapExceptions": ["foo.bar"] }])),
        ),
        // (
        //     "var x = new foo.bar(42);",
        //     Some(serde_json::json!([{ "newIsCapExceptionPattern": "^foo\\.." }])),
        // ),
        ("var x = new foo.bar(42);", Some(serde_json::json!([{ "properties": false }]))),
        ("var x = Foo.bar(42);", Some(serde_json::json!([{ "properties": false }]))),
        (
            "var x = foo.Bar(42);",
            Some(serde_json::json!([{ "capIsNew": false, "properties": false }])),
        ),
        ("foo?.bar();", None),       // { "ecmaVersion": 2020 },
        ("(foo?.bar)();", None),     // { "ecmaVersion": 2020 },
        ("new (foo?.Bar)();", None), // { "ecmaVersion": 2020 },
        ("(foo?.Bar)();", Some(serde_json::json!([{ "properties": false }]))), // { "ecmaVersion": 2020 },
        ("new (foo?.bar)();", Some(serde_json::json!([{ "properties": false }]))), // { "ecmaVersion": 2020 },
        ("Date?.UTC();", None),   // { "ecmaVersion": 2020 },
        ("(Date?.UTC)();", None), // { "ecmaVersion": 2020 }
    ];

    let fail = vec![
        ("var x = new c();", None),
        ("var x = new φ;", None),
        ("var x = new a.b.c;", None),
        ("var x = new a.b['c'];", None),
        ("var b = Foo();", None),
        ("var b = a.Foo();", None),
        ("var b = a['Foo']();", None),
        ("var b = a.Date.UTC();", None),
        ("var b = UTC();", None),
        ("var a = B.C();", None),
        (
            "var a = B
			.C();",
            None,
        ),
        ("var a = new B.c();", None),
        (
            "var a = new B.
			c();",
            None,
        ),
        ("var a = new c();", None),
        ("var a = new b[ ( 'foo' ) ]();", None), // { "ecmaVersion": 6 },
        ("var a = new b[`foo`];", None),         // { "ecmaVersion": 6 },
        (
            "var a = b[`\\
			Foo`]();",
            None,
        ), // { "ecmaVersion": 6 },
        ("var x = Foo.Bar(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Foo"] }]))),
        (
            "var x = Bar.Foo(42);",
            Some(serde_json::json!([{ "capIsNewExceptionPattern": "^Foo\\.." }])),
        ),
        ("var x = new foo.bar(42);", Some(serde_json::json!([{ "newIsCapExceptions": ["foo"] }]))),
        (
            "var x = new bar.foo(42);",
            Some(serde_json::json!([{ "newIsCapExceptionPattern": "^foo\\.." }])),
        ),
        ("new (foo?.bar)();", None), // { "ecmaVersion": 2020 },
        ("foo?.Bar();", None),       // { "ecmaVersion": 2020 },
        ("(foo?.Bar)();", None),     // { "ecmaVersion": 2020 }
    ];

    Tester::new(NewCap::NAME, NewCap::CATEGORY, pass, fail).test_and_snapshot();
}
