use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::{
    ast::{ChainElement, ComputedMemberExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use regex::Regex;

fn new_cap_diagnostic(span: Span, cap: &GetCapResult) -> OxcDiagnostic {
    let msg = if *cap == GetCapResult::Lower {
        "A constructor name should not start with a lowercase letter."
    } else {
        "A function with a name starting with an uppercase letter should only be used as a constructor."
    };

    OxcDiagnostic::warn(msg).with_label(span)
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

    value.as_bool().unwrap_or(true)
}

fn vec_str_serde_value(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    default_value: Vec<CompactStr>,
) -> Vec<CompactStr> {
    let Some(value) = map.get(key) else {
        return default_value; // default value
    };

    let Some(array_value) = value.as_array() else {
        return default_value; // default value
    };

    array_value
        .iter()
        .map(|value| CompactStr::new(value.as_str().unwrap_or_default()))
        .collect::<Vec<CompactStr>>()
}

fn regex_serde_value(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<Regex> {
    let value = map.get(key)?;
    let regex_string = value.as_str()?;

    if let Ok(regex) = Regex::new(regex_string) {
        return Some(regex);
    }

    None
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
            new_is_cap_exception_pattern: regex_serde_value(config, "newIsCapExceptionPattern"),
            cap_is_new_exceptions: vec_str_serde_value(config, "capIsNewExceptions", vec![]),
            cap_is_new_exception_pattern: regex_serde_value(config, "capIsNewExceptionPattern"),
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
    /// This rule requires constructor names to begin with a capital letter.
    ///
    /// ### Why is this bad?
    ///
    /// The new operator in JavaScript creates a new instance of a particular type of object.
    /// That type of object is represented by a constructor function.
    /// Since constructor functions are just regular functions, the only defining characteristic
    /// is that new is being used as part of the call.
    /// Native JavaScript functions begin with an uppercase letter to distinguish those functions
    /// that are to be used as constructors from functions that are not.
    /// Many style guides recommend following this pattern
    /// to more easily determine which functions are to be used as constructors.
    ///
    /// **Warning**:
    /// The option `newIsCapExceptionPattern` and `capIsNewExceptionPattern` are implemented with
    /// the [rust regex syntax](https://docs.rs/regex/latest/regex/). Many JavaScript features
    /// are not supported (Lookahead, Lookbehinds, ...).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function foo(arg) {
    ///     return Boolean(arg);
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "newIsCap": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCap": true }]*/
    ///
    /// var friend = new person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "newIsCap": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCap": true }]*/
    ///
    /// var friend = new Person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "newIsCap": false }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCap": false }]*/
    ///
    /// var friend = new person();
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "capIsNew": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNew": true }]*/
    ///
    /// var colleague = Person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "capIsNew": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNew": true }]*/
    ///
    /// var colleague = new Person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "capIsNew": false }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNew": false }]*/
    ///
    /// var colleague = Person();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "newIsCapExceptions": ["events"] }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCapExceptions": ["events"] }]*/
    ///
    /// var events = require('events');
    ///
    /// var emitter = new events();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "newIsCapExceptionPattern": "^person\\.." }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCapExceptionPattern": "^person\\.." }]*/
    ///
    /// var friend = new person.acquaintance();
    ///
    /// var bestFriend = new person.friend();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "newIsCapExceptionPattern": "\\.bar$" }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCapExceptionPattern": "\\.bar$" }]*/
    ///
    /// var friend = new person.bar();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptions": ["Person"] }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptions": ["Person"] }]*/
    ///
    /// function foo(arg) {
    ///     return Person(arg);
    /// }
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptionPattern": "^person\\.." }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptionPattern": "^person\\.." }]*/
    ///
    /// var friend = person.Acquaintance();
    /// var bestFriend = person.Friend();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptionPattern": "\\.Bar$" }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptionPattern": "\\.Bar$" }]*/
    ///
    /// foo.Bar();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptionPattern": "^Foo" }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptionPattern": "^Foo" }]*/
    ///
    /// var x = Foo(42);
    ///
    /// var y = Foobar(42);
    ///
    /// var z = Foo.Bar(42);
    /// ```
    ///
    /// ### properties
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "properties": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "properties": true }]*/
    ///
    /// var friend = new person.acquaintance();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "properties": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "properties": true }]*/
    ///
    /// var friend = new person.Acquaintance();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "properties": false }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "properties": false }]*/
    ///
    /// var friend = new person.acquaintance();
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "newIsCap": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCap": true }]*/
    ///
    /// var friend = new person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "newIsCap": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCap": true }]*/
    ///
    /// var friend = new Person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "newIsCap": false }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCap": false }]*/
    ///
    /// var friend = new person();
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "capIsNew": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNew": true }]*/
    ///
    /// var colleague = Person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "capIsNew": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNew": true }]*/
    ///
    /// var colleague = new Person();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "capIsNew": false }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNew": false }]*/
    ///
    /// var colleague = Person();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "newIsCapExceptions": ["events"] }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCapExceptions": ["events"] }]*/
    ///
    /// var events = require('events');
    ///
    /// var emitter = new events();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "newIsCapExceptionPattern": "^person\\.." }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCapExceptionPattern": "^person\\.." }]*/
    ///
    /// var friend = new person.acquaintance();
    ///
    /// var bestFriend = new person.friend();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "newIsCapExceptionPattern": "\\.bar$" }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "newIsCapExceptionPattern": "\\.bar$" }]*/
    ///
    /// var friend = new person.bar();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptions": ["Person"] }` option:
    ///
    /// ::: correct
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptions": ["Person"] }]*/
    ///
    /// function foo(arg) {
    ///     return Person(arg);
    /// }
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptionPattern": "^person\\.." }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptionPattern": "^person\\.." }]*/
    ///
    /// var friend = person.Acquaintance();
    /// var bestFriend = person.Friend();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptionPattern": "\\.Bar$" }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptionPattern": "\\.Bar$" }]*/
    ///
    /// foo.Bar();
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the `{ "capIsNewExceptionPattern": "^Foo" }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "capIsNewExceptionPattern": "^Foo" }]*/
    ///
    /// var x = Foo(42);
    ///
    /// var y = Foobar(42);
    ///
    /// var z = Foo.Bar(42);
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "properties": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "properties": true }]*/
    ///
    /// var friend = new person.acquaintance();
    /// ```
    ///
    ///
    /// Examples of **correct** code for this rule with the default `{ "properties": true }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "properties": true }]*/
    ///
    /// var friend = new person.Acquaintance();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "properties": false }` option:
    ///
    /// ```js
    /// /*eslint new-cap: ["error", { "properties": false }]*/
    ///
    /// var friend = new person.acquaintance();
    /// ```
    NewCap,
    eslint,
    style,
    pending  // TODO: maybe?
);

impl Rule for NewCap {
    fn from_configuration(value: serde_json::Value) -> Self {
        NewCap::from(&value)
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(expression) if self.new_is_cap => {
                let callee = expression.callee.without_parentheses();

                let Some(short_name) = &extract_name_from_expression(callee) else {
                    return;
                };

                let Some(name) = &extract_name_deep_from_expression(callee) else {
                    return;
                };

                let capitalization = &get_cap(short_name);

                let allowed = *capitalization != GetCapResult::Lower
                    || is_cap_allowed_expression(
                        short_name,
                        name,
                        &self.new_is_cap_exceptions,
                        self.new_is_cap_exception_pattern.as_ref(),
                    )
                    || (!self.properties && short_name != name);

                if !allowed {
                    ctx.diagnostic(new_cap_diagnostic(callee.span(), capitalization));
                }
            }
            AstKind::CallExpression(expression) if self.cap_is_new => {
                let callee = expression.callee.without_parentheses();

                let Some(short_name) = &extract_name_from_expression(callee) else {
                    return;
                };

                let Some(name) = &extract_name_deep_from_expression(callee) else {
                    return;
                };

                let capitalization = &get_cap(short_name);

                let mut caps_is_new_exceptions = self.cap_is_new_exceptions.clone();
                caps_is_new_exceptions.append(&mut caps_allowed_vec());

                let allowed = *capitalization != GetCapResult::Upper
                    || is_cap_allowed_expression(
                        short_name,
                        name,
                        &caps_is_new_exceptions,
                        self.cap_is_new_exception_pattern.as_ref(),
                    )
                    || (!self.properties && short_name != name);

                if !allowed {
                    ctx.diagnostic(new_cap_diagnostic(callee.span(), capitalization));
                }
            }
            _ => (),
        }
    }
}

fn extract_name_deep_from_expression(expression: &Expression) -> Option<CompactStr> {
    if let Some(identifier) = expression.get_identifier_reference() {
        return Some(identifier.name.into());
    }

    match expression.without_parentheses() {
        Expression::StaticMemberExpression(expression) => {
            let prop_name = expression.property.name.into_compact_str();
            let obj_name =
                extract_name_deep_from_expression(expression.object.without_parentheses());

            if let Some(obj_name) = obj_name {
                let new_name = format!("{obj_name}.{prop_name}");
                return Some(CompactStr::new(&new_name));
            }

            Some(prop_name)
        }
        Expression::ComputedMemberExpression(expression) => {
            let prop_name = get_computed_member_name(expression)?;
            let obj_name =
                extract_name_deep_from_expression(expression.object.without_parentheses());

            if let Some(obj_name) = obj_name {
                let new_name = format!("{obj_name}.{prop_name}");
                return Some(CompactStr::new(&new_name));
            }

            Some(prop_name)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => extract_name_deep_from_expression(&call.callee),
            ChainElement::TSNonNullExpression(non_null) => {
                extract_name_deep_from_expression(&non_null.expression)
            }
            ChainElement::StaticMemberExpression(expression) => {
                let prop_name = expression.property.name.into_compact_str();
                let obj_name =
                    extract_name_deep_from_expression(expression.object.without_parentheses());

                if let Some(obj_name) = obj_name {
                    let new_name = format!("{obj_name}.{prop_name}");
                    return Some(CompactStr::new(&new_name));
                }

                Some(prop_name)
            }
            ChainElement::ComputedMemberExpression(expression) => {
                let prop_name = get_computed_member_name(expression)?;
                let obj_name =
                    extract_name_deep_from_expression(expression.object.without_parentheses());

                if let Some(obj_name) = obj_name {
                    let new_name = format!("{obj_name}.{prop_name}");
                    return Some(CompactStr::new(&new_name));
                }

                Some(prop_name)
            }
            ChainElement::PrivateFieldExpression(_) => None,
        },
        _ => None,
    }
}

fn get_computed_member_name(computed_member: &ComputedMemberExpression) -> Option<CompactStr> {
    let expression = computed_member.expression.without_parentheses();

    match &expression {
        Expression::StringLiteral(lit) => Some(lit.value.as_ref().into()),
        Expression::TemplateLiteral(lit) if lit.expressions.is_empty() && lit.quasis.len() == 1 => {
            Some(lit.quasis[0].value.raw.as_ref().into())
        }
        Expression::RegExpLiteral(lit) => lit.raw.as_ref().map(|&x| x.into_compact_str()),
        _ => None,
    }
}

fn extract_name_from_expression(expression: &Expression) -> Option<CompactStr> {
    if let Some(identifier) = expression.get_identifier_reference() {
        return Some(identifier.name.into());
    }

    match expression.without_parentheses() {
        Expression::StaticMemberExpression(expression) => {
            Some(expression.property.name.into_compact_str())
        }
        Expression::ComputedMemberExpression(expression) => get_computed_member_name(expression),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => extract_name_from_expression(&call.callee),
            ChainElement::TSNonNullExpression(non_null) => {
                extract_name_from_expression(&non_null.expression)
            }
            ChainElement::StaticMemberExpression(expression) => {
                Some(expression.property.name.into_compact_str())
            }
            ChainElement::ComputedMemberExpression(expression) => {
                get_computed_member_name(expression)
            }
            ChainElement::PrivateFieldExpression(_) => None,
        },
        _ => None,
    }
}

fn is_cap_allowed_expression(
    short_name: &CompactStr,
    name: &CompactStr,
    exceptions: &[CompactStr],
    patterns: Option<&Regex>,
) -> bool {
    if exceptions.contains(name) || exceptions.contains(short_name) {
        return true;
    }

    if name == "Date.UTC" {
        return true;
    }

    if let Some(pattern) = &patterns {
        return pattern.find(name).is_some();
    };

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
        ("var x = Date.UTC(2000, 0)", None),
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
        ("var x = Foo(42);", Some(serde_json::json!([{ "capIsNewExceptionPattern": "^Foo" }]))),
        (
            "var x = new foo(42);",
            Some(serde_json::json!([{ "newIsCap": true, "newIsCapExceptions": ["foo"] }])),
        ),
        ("var x = new foo(42);", Some(serde_json::json!([{ "newIsCapExceptionPattern": "^foo" }]))),
        ("var x = Object(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Foo"] }]))),
        ("var x = Foo.Bar(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Bar"] }]))),
        ("var x = Foo.Bar(42);", Some(serde_json::json!([{ "capIsNewExceptions": ["Foo.Bar"] }]))),
        (
            "var x = Foo.Bar(42);",
            Some(serde_json::json!([{ "capIsNewExceptionPattern": "^Foo\\.." }])),
        ),
        ("var x = new foo.bar(42);", Some(serde_json::json!([{ "newIsCapExceptions": ["bar"] }]))),
        (
            "var x = new foo.bar(42);",
            Some(serde_json::json!([{ "newIsCapExceptions": ["foo.bar"] }])),
        ),
        (
            "var x = new foo.bar(42);",
            Some(serde_json::json!([{ "newIsCapExceptionPattern": "^foo\\.." }])),
        ),
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
        // (
        //     "var a = b[`\\
        // 	Foo`]();",
        //     None,
        // ), // { "ecmaVersion": 6 },
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

    Tester::new(NewCap::NAME, NewCap::PLUGIN, pass, fail).test_and_snapshot();
}
