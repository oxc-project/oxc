use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_computed_key_diagnostic(span: Span, raw: Option<Atom>) -> OxcDiagnostic {
    // false positive, if we remove the closure, `borrowed data escapes outside of function `raw` escapes the function body here`
    #[expect(clippy::redundant_closure)]
    let key = raw.unwrap_or_else(|| Atom::empty());
    OxcDiagnostic::warn(format!("Unnecessarily computed property `{key}` found."))
        .with_help("Replace the computed property with a plain identifier or string literal")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoUselessComputedKey {
    enforce_for_class_members: bool,
}

impl Default for NoUselessComputedKey {
    fn default() -> Self {
        Self { enforce_for_class_members: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary computed property keys in objects and classes
    ///
    /// ### Why is this bad?
    ///
    /// It’s unnecessary to use computed properties with literals such as:
    /// ```js
    /// const foo = {["a"]: "b"};
    /// ```
    ///
    /// The code can be rewritten as:
    /// ```js
    /// const foo = {"a": "b"};
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const a = { ['0']: 0 };
    /// const b = { ['0+1,234']: 0 };
    /// const c = { [0]: 0 };
    /// const e = { ['x']() {} };
    ///
    /// class Foo {
    ///     ["foo"] = "bar";
    ///     [0]() {}
    ///     static ["foo"] = "bar";
    ///     get ['b']() {}
    ///     set ['c'](value) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = { 'a': 0 };
    /// const b = { 0: 0 };
    /// const c = { x() {} };
    /// const e = { '0+1,234': 0 };
    ///
    /// class Foo {
    ///     "foo" = "bar";
    ///     0() {}
    ///     'a'() {}
    ///     static "foo" = "bar";
    /// }
    /// ```
    ///
    /// Examples of additional **correct** code for this rule:
    /// ```js
    ///
    /// const c = {
    ///     "__proto__": foo, // defines object's prototype
    ///     ["__proto__"]: bar // defines a property named "__proto__"
    /// };
    /// class Foo {
    ///     ["constructor"]; // instance field named "constructor"
    ///     "constructor"() {} // the constructor of this class
    ///     static ["constructor"]; // static field named "constructor"
    ///     static ["prototype"]; // runtime error, it would be a parsing error without `[]`
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### enforceForClassMembers
    ///
    /// `{ type: boolean, default: true }`
    ///
    /// The `enforceForClassMembers` option controls whether the rule applies to
    /// class members (methods and properties).
    ///
    /// Examples of **correct** code for this rule with the `{ "enforceForClassMembers": false }` option:
    /// ```js
    /// class SomeClass {
    ///     ["foo"] = "bar";
    ///     [42] = "baz";
    ///     get ['b']() {}
    ///     set ['c'](value) {}
    ///     static ["foo"] = "bar";
    /// }
    /// ```
    NoUselessComputedKey,
    eslint,
    style,
    pending
);

impl Rule for NoUselessComputedKey {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);
        Self {
            enforce_for_class_members: obj
                .and_then(|v| v.get("enforceForClassMembers"))
                .and_then(Value::as_bool)
                .unwrap_or(true),
        }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectProperty(property) if property.computed => {
                if let Some(expr) =
                    property.key.as_expression().map(Expression::get_inner_expression)
                {
                    check_computed_class_member(
                        ctx,
                        property.key.span(),
                        expr,
                        false,
                        &[],
                        &["__proto__"],
                    );
                }
            }
            AstKind::BindingProperty(binding_prop) if binding_prop.computed => {
                if let Some(expr) =
                    binding_prop.key.as_expression().map(Expression::get_inner_expression)
                {
                    check_computed_class_member(ctx, binding_prop.span, expr, false, &[], &[]);
                }
            }
            AstKind::PropertyDefinition(prop_def)
                if self.enforce_for_class_members && prop_def.computed =>
            {
                if let Some(expr) =
                    prop_def.key.as_expression().map(Expression::get_inner_expression)
                {
                    check_computed_class_member(
                        ctx,
                        prop_def.key.span(),
                        expr,
                        prop_def.r#static,
                        &["prototype", "constructor"],
                        &["constructor"],
                    );
                }
            }
            AstKind::MethodDefinition(method_def)
                if self.enforce_for_class_members && method_def.computed =>
            {
                if let Some(expr) =
                    method_def.key.as_expression().map(Expression::get_inner_expression)
                {
                    check_computed_class_member(
                        ctx,
                        method_def.span,
                        expr,
                        method_def.r#static,
                        &["prototype"],
                        &["constructor"],
                    );
                }
            }
            _ => {}
        }
    }
}

fn check_computed_class_member(
    ctx: &LintContext<'_>,
    span: Span,
    expr: &Expression,
    is_static: bool,
    allow_static: &[&str],
    allow_non_static: &[&str],
) {
    match expr {
        Expression::StringLiteral(lit) => {
            let key_name = lit.value.as_str();
            let allowed = if is_static {
                allow_static.contains(&key_name)
            } else {
                allow_non_static.contains(&key_name)
            };
            if !allowed {
                ctx.diagnostic(no_useless_computed_key_diagnostic(span, lit.raw));
            }
        }
        Expression::NumericLiteral(number_lit) => {
            ctx.diagnostic(no_useless_computed_key_diagnostic(span, number_lit.raw));
        }
        _ => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("({ 'a': 0, b(){} })", None),
        ("({ [x]: 0 });", None),
        ("({ a: 0, [b](){} })", None),
        ("({ ['__proto__']: [] })", None),
        ("var { 'a': foo } = obj", None),
        ("var { [a]: b } = obj;", None),
        ("var { a } = obj;", None),
        ("var { a: a } = obj;", None),
        ("var { a: b } = obj;", None),
        ("class Foo { a() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        ("class Foo { 'a'() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        ("class Foo { [x]() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        (
            "class Foo { ['constructor']() {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "class Foo { static ['prototype']() {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        ("(class { 'a'() {} })", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        ("(class { [x]() {} })", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        (
            "(class { ['constructor']() {} })",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "(class { static ['prototype']() {} })",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        ("class Foo { 'x'() {} }", None),
        ("(class { [x]() {} })", None),
        ("class Foo { static constructor() {} }", None),
        ("class Foo { prototype() {} }", None),
        (
            "class Foo { ['x']() {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": false }])),
        ),
        ("(class { ['x']() {} })", Some(serde_json::json!([{ "enforceForClassMembers": false }]))),
        (
            "class Foo { static ['constructor']() {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": false }])),
        ),
        (
            "class Foo { ['prototype']() {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": false }])),
        ),
        ("class Foo { a }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        (
            "class Foo { ['constructor'] }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "class Foo { static ['constructor'] }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "class Foo { static ['prototype'] }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        ("({ [99999999999999999n]: 0 })", None), // { "ecmaVersion": 2020 }
    ];

    let fail = vec![
        ("({ ['0']: 0 })", None),
        ("var { ['0']: a } = obj", None),
        ("({ ['0+1,234']: 0 })", None),
        ("({ [0]: 0 })", None),
        ("var { [0]: a } = obj", None),
        ("({ ['x']: 0 })", None),
        ("var { ['x']: a } = obj", None),
        ("var { ['__proto__']: a } = obj", None),
        ("({ ['x']() {} })", None),
        ("({ [/* this comment prevents a fix */ 'x']: 0 })", None),
        ("({ ['x' /* this comment also prevents a fix */]: 0 })", None),
        ("({ [('x')]: 0 })", None),
        ("var { [('x')]: a } = obj", None),
        ("({ *['x']() {} })", None),
        ("({ async ['x']() {} })", None), // { "ecmaVersion": 8 },
        ("({ get[.2]() {} })", None),
        ("({ set[.2](value) {} })", None),
        ("({ async[.2]() {} })", None), // { "ecmaVersion": 8 },
        ("({ [2]() {} })", None),
        ("({ get [2]() {} })", None),
        ("({ set [2](value) {} })", None),
        ("({ async [2]() {} })", None), // { "ecmaVersion": 8 },
        ("({ get[2]() {} })", None),
        ("({ set[2](value) {} })", None),
        ("({ async[2]() {} })", None), // { "ecmaVersion": 8 },
        ("({ get['foo']() {} })", None),
        ("({ *[2]() {} })", None),
        ("({ async*[2]() {} })", None),
        ("({ ['constructor']: 1 })", None),
        ("({ ['prototype']: 1 })", None),
        ("class Foo { ['0']() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        ("class Foo { ['0+1,234']() {} }", Some(serde_json::json!([{}]))),
        ("class Foo { ['x']() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        ("class Foo { [/* this comment prevents a fix */ 'x']() {} }", None),
        ("class Foo { ['x' /* this comment also prevents a fix */]() {} }", None),
        ("class Foo { [('x')]() {} }", None),
        ("class Foo { *['x']() {} }", None),
        ("class Foo { async ['x']() {} }", None), // { "ecmaVersion": 8 },
        ("class Foo { get[.2]() {} }", None),
        ("class Foo { set[.2](value) {} }", None),
        ("class Foo { async[.2]() {} }", None), // { "ecmaVersion": 8 },
        ("class Foo { [2]() {} }", None),
        ("class Foo { get [2]() {} }", None),
        ("class Foo { set [2](value) {} }", None),
        ("class Foo { async [2]() {} }", None), // { "ecmaVersion": 8 },
        ("class Foo { get[2]() {} }", None),
        ("class Foo { set[2](value) {} }", None),
        ("class Foo { async[2]() {} }", None), // { "ecmaVersion": 8 },
        ("class Foo { get['foo']() {} }", None),
        ("class Foo { *[2]() {} }", None),
        ("class Foo { async*[2]() {} }", None),
        ("class Foo { static ['constructor']() {} }", None),
        ("class Foo { ['prototype']() {} }", None),
        ("(class { ['x']() {} })", None),
        ("(class { ['__proto__']() {} })", None),
        ("(class { static ['__proto__']() {} })", None),
        ("(class { static ['constructor']() {} })", None),
        ("(class { ['prototype']() {} })", None),
        ("class Foo { ['0'] }", None),
        ("class Foo { ['0'] = 0 }", None),
        ("class Foo { static[0] }", None),
        ("class Foo { ['#foo'] }", None),
        ("(class { ['__proto__'] })", None),
        ("(class { static ['__proto__'] })", None),
        ("(class { ['prototype'] })", None),
    ];

    Tester::new(NoUselessComputedKey::NAME, NoUselessComputedKey::PLUGIN, pass, fail)
        .test_and_snapshot();
}
