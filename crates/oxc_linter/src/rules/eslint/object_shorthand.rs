use oxc_ast::{
    ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn expected_all_properties_shorthanded(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected shorthand for all properties.")
        .with_labels([span0.into()])
}

fn expected_literal_method_longform(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint(object-shorthand): Expected longform method syntax for string literal keys.",
    )
    .with_labels([span0.into()])
}

fn expected_property_shorthand(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected property shorthand.")
        .with_labels([span0.into()])
}

fn expected_property_longform(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected longform property syntax.")
        .with_labels([span0.into()])
}

fn expected_method_shorthand(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected method shorthand.")
        .with_labels([span0.into()])
}

fn expected_method_longform(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected longform method syntax.")
        .with_labels([span0.into()])
}

fn unexpected_mix(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint(object-shorthand): Unexpected mix of shorthand and non-shorthand properties.",
    )
    .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct ObjectShorthand(Box<ObjectShorthandConfig>);

#[derive(Debug, Default, Clone)]
pub struct ObjectShorthandConfig {
    apply_to_methods: bool,
    apply_to_properties: bool,
    apply_never: bool,
    apply_consistent: bool,
    apply_consistent_as_needed: bool,

    avoid_quotes: bool,
    ignore_constructors: bool,
    avoid_explicit_return_arrows: bool,
    methods_ignore_pattern: Option<String>,
}

impl std::ops::Deref for ObjectShorthand {
    type Target = ObjectShorthandConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// doc: https://github.com/eslint/eslint/blob/main/docs/src/rules/object-shorthand.md
// code: https://github.com/eslint/eslint/blob/main/lib/rules/object-shorthand.js
// test: https://github.com/eslint/eslint/blob/main/tests/lib/rules/object-shorthand.js

declare_oxc_lint!(
    /// ### What it does
    /// Require or disallow method and property shorthand syntax for object literals
    ///
    /// ### Why is this bad?
    /// Stylistic preference
    ///
    /// ### Example
    /// Here are a few common examples using the ES5 syntax:
    ///
    /// ```javascript
    /// var properties = { x: x, y: y, z: z, };
    /// var methods = { a: function() {}, b: function() {} };
    /// ```
    ///
    /// Now here are ES6 equivalents:
    ///
    /// ```javascript
    /// var properties = { x, y, z };
    /// var methods = { a() {}, b() {} };
    /// ```
    ObjectShorthand,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc-project.github.io/docs/contribute/linter.html#rule-category> for details
);

impl Rule for ObjectShorthand {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj1 = value.get(0);
        let obj2 = value.get(1);

        let shorthand_type =
            obj1.and_then(serde_json::Value::as_str).map(ShorthandType::from).unwrap_or_default();

        Self(Box::new(ObjectShorthandConfig {
            apply_to_methods: matches!(
                shorthand_type,
                ShorthandType::Methods | ShorthandType::Always
            ),
            apply_to_properties: matches!(
                shorthand_type,
                ShorthandType::Properties | ShorthandType::Always
            ),
            apply_never: matches!(shorthand_type, ShorthandType::Never),
            apply_consistent: matches!(shorthand_type, ShorthandType::Consistent),
            apply_consistent_as_needed: matches!(shorthand_type, ShorthandType::ConsistentAsNeeded),

            avoid_quotes: obj2
                .and_then(|v| v.get("avoidQuotes"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            ignore_constructors: obj2
                .and_then(|v| v.get("ignoreConstructors"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            avoid_explicit_return_arrows: obj2
                .and_then(|v| v.get("avoidExplicitReturnArrows"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            methods_ignore_pattern: obj2
                .and_then(|v| v.get("methodsIgnorePattern"))
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ObjectProperty(property) = node.kind() {
            let is_concise_property = property.shorthand || property.method;

            if !can_property_have_shorthand(property) {
                return;
            }

            if is_concise_property {
                if property.method
                    && (self.apply_never
                        || self.avoid_quotes && is_property_key_string_literal(property))
                {
                    // from { x() {} } to { x: function() {} }
                    // TODO: implement
                } else if self.apply_never {
                    // from { x } to { x: x }
                    // TODO: implement
                }
            } else if self.apply_to_methods && is_property_value_anonymous_function(property) {
                // from { x: function() {} }   to { x() {} }
                // from { [x]: function() {} } to { [x]() {} }
                // from { x: () => {} }        to { x() {} }
                // from { [x]: () => {} }      to { [x]() {} }
                Self::check_longform_methods(&self, property, ctx);
            } else if self.apply_to_properties {
                // from { x: x }   to { x }
                // from { "x": x } to { x }
                Self::check_longform_properties(&self, property, ctx);
            }
        } else if let AstKind::ObjectExpression(obj_expr) = node.kind() {
            if self.apply_consistent {
                Self::check_consistency(obj_expr, false, ctx);
            } else if self.apply_consistent_as_needed {
                Self::check_consistency(obj_expr, true, ctx);
            }
        }
    }
}

impl ObjectShorthand {
    fn check_longform_methods(&self, property: &ObjectProperty, ctx: &LintContext<'_>) {
        // TODO: self.ignore_constructors
        // TODO: self.methods_ignore_pattern

        if self.avoid_quotes && is_property_key_string_literal(property) {
            return;
        }

        if let Expression::FunctionExpression(func) = &property.value.without_parenthesized() {
            ctx.diagnostic(expected_method_shorthand(func.span));
        }

        if self.avoid_explicit_return_arrows {
            if let Expression::ArrowFunctionExpression(func) =
                &property.value.without_parenthesized()
            {
                if !func.expression {
                    ctx.diagnostic(expected_method_shorthand(func.span));
                }
            }
        }
    }

    fn check_longform_properties(&self, property: &ObjectProperty, ctx: &LintContext<'_>) {
        let Expression::Identifier(value_identifier) = &property.value.without_parenthesized()
        else {
            return;
        };

        if let Some(property_name) = property.key.name() {
            // from { x: x } to { x }
            // from { "x": x } to { x }
            if !self.avoid_quotes && property_name == value_identifier.name {
                if ctx.semantic().trivias().has_comments_between(Span::new(
                    property.key.span().start,
                    value_identifier.span.end,
                )) {
                    ctx.diagnostic(expected_property_shorthand(property.span));
                } else {
                    // TODO: fixer
                    ctx.diagnostic(expected_property_shorthand(property.span));
                }
            }
        }
    }

    fn check_consistency(
        obj_expr: &ObjectExpression,
        check_redundancy: bool,
        ctx: &LintContext<'_>,
    ) {
        let properties =
            obj_expr.properties.iter().filter_map(|property_kind| match property_kind {
                ObjectPropertyKind::ObjectProperty(property) => {
                    can_property_have_shorthand(property).then(|| property)
                }
                _ => None,
            });

        if properties.clone().count() > 0 {
            let shorthand_properties = properties.clone().filter(|p| is_shorthand_property(p));

            if shorthand_properties.clone().count() != properties.clone().count() {
                if shorthand_properties.count() > 0 {
                    ctx.diagnostic(unexpected_mix(obj_expr.span));
                } else if check_redundancy {
                    if properties.clone().all(|p| is_redundant_property(p)) {
                        ctx.diagnostic(expected_all_properties_shorthanded(obj_expr.span));
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
enum ShorthandType {
    #[default]
    Always,
    Methods,
    Properties,
    Consistent,
    ConsistentAsNeeded,
    Never,
}

impl ShorthandType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "methods" => Self::Methods,
            "properties" => Self::Properties,
            "consistent" => Self::Consistent,
            "consistent-as-needed" => Self::ConsistentAsNeeded,
            "never" => Self::Never,
            _ => Self::Always,
        }
    }
}

fn is_property_value_function(property: &ObjectProperty) -> bool {
    matches!(
        property.value,
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

fn is_property_value_anonymous_function(property: &ObjectProperty) -> bool {
    match &property.value {
        Expression::FunctionExpression(func) => func.id.is_none(),
        Expression::ArrowFunctionExpression(_) => true,
        _ => false,
    }
}

fn is_property_key_string_literal(property: &ObjectProperty) -> bool {
    matches!(property.key.as_expression(), Some(Expression::StringLiteral(_)))
}

fn is_shorthand_property(property: &ObjectProperty) -> bool {
    property.shorthand || property.method
}

fn is_redundant_property(property: &ObjectProperty) -> bool {
    match &property.value {
            Expression::FunctionExpression(func) => func.id.is_none(),
            Expression::Identifier(value_identifier) => {
                if let Some(property_name) = property.key.name() {
                    property_name == value_identifier.name
                } else {
                    false
                }
            }
        _ => false,
    }
}

fn can_property_have_shorthand(property: &ObjectProperty) -> bool {
    // Ignore getters and setters
    if property.kind != PropertyKind::Init {
        return false;
    }

    // Ignore computed properties, unless they are functions
    if property.computed && !is_property_value_function(property) {
        return false;
    }

    return true;
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("var x = {y() {}}", None),
        ("var x = {y}", None),
        ("var x = {a: b}", None),
        ("var x = {a: 'a'}", None),
        ("var x = {'a': 'a'}", None),
        ("var x = {'a': b}", None),
        ("var x = {y(x) {}}", None),
        ("var {x,y,z} = x", None),
        ("var {x: {y}} = z", None),
        ("var x = {*x() {}}", None),
        ("var x = {x: y}", None),
        ("var x = {x: y, y: z}", None),
        ("var x = {x: y, y: z, z: 'z'}", None),
        ("var x = {x() {}, y: z, l(){}}", None),
        ("var x = {x: y, y: z, a: b}", None),
        ("var x = {x: y, y: z, 'a': b}", None),
        ("var x = {x: y, y() {}, z: a}", None),
        ("var x = {[y]: y}", None),
        ("doSomething({x: y})", None),
        ("doSomething({'x': y})", None),
        ("doSomething({x: 'x'})", None),
        ("doSomething({'x': 'x'})", None),
        ("doSomething({y() {}})", None),
        ("doSomething({x: y, y() {}})", None),
        ("doSomething({y() {}, z: a})", None),
        ("!{ a: function a(){} };", None),
        // arrow functions are still alright by default
        ("var x = {y: (x)=>x}", None),
        ("doSomething({y: (x)=>x})", None),
        ("var x = {y: (x)=>x, y: a}", None),
        ("doSomething({x, y: (x)=>x})", None),
        ("({ foo: x => { return; }})", None),
        ("({ foo: (x) => { return; }})", None),
        ("({ foo: () => { return; }})", None),
        // getters and setters
        ("var x = {get y() {}}", None),
        ("var x = {set y(z) {}}", None),
        ("var x = {get y() {}, set y(z) {}}", None),
        ("doSomething({get y() {}})", None),
        ("doSomething({set y(z) {}})", None),
        ("doSomething({get y() {}, set y(z) {}})", None),
        // object literal computed properties
        ("var x = {[y]: y}", Some(json!(["properties"]))),
        ("var x = {['y']: 'y'}", Some(json!(["properties"]))),
        ("var x = {['y']: y}", Some(json!(["properties"]))),
        // object literal computed methods
        ("var x = {[y]() {}}", Some(json!(["methods"]))),
        ("var x = {[y]: function x() {}}", Some(json!(["methods"]))),
        ("var x = {[y]: y}", Some(json!(["methods"]))),
        // options
        ("var x = {y() {}}", Some(json!(["methods"]))),
        ("var x = {x, y() {}, a:b}", Some(json!(["methods"]))),
        ("var x = {y}", Some(json!(["properties"]))),
        ("var x = {y: {b}}", Some(json!(["properties"]))),
        // consistent
        ("var x = {a: a, b: b}", Some(json!(["consistent"]))),
        ("var x = {a: b, c: d, f: g}", Some(json!(["consistent"]))),
        ("var x = {a, b}", Some(json!(["consistent"]))),
        ("var x = {a, b, get test() { return 1; }}", Some(json!(["consistent"]))),
        ("var x = {foo, bar, ...baz}", Some(json!(["consistent"]))),
        ("var x = {bar: baz, ...qux}", Some(json!(["consistent"]))),
        ("var x = {...foo, bar: bar, baz: baz}", Some(json!(["consistent"]))),
        // consistent-as-needed
        ("var x = {...bar}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a, b}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a, b, get test(){return 1;}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {0: 'foo'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {'key': 'baz'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo: 'foo'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {[foo]: foo}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo: function foo() {}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {[foo]: 'foo'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {bar, ...baz}", Some(json!(["consistent-as-needed"]))),
        ("var x = {bar: baz, ...qux}", Some(json!(["consistent-as-needed"]))),
        ("var x = {...foo, bar, baz}", Some(json!(["consistent-as-needed"]))),
    ];

    let fail = vec![
        ("var x = {a: /* comment */ a}", None),
        ("var x = {a /* comment */: a}", None),
        ("var x = {a: (a /* comment */)}", None),
        ("var x = {'a': /* comment */ a}", None),
        ("var x = {'a' /* comment */: a}", None),
        ("var x = {'a': (a /* comment */)}", None),
        ("var x = {f: /* comment */ function() {}}", None),
        ("var x = {f /* comment */: function() {}}", None),
        ("var x = {a: a, b}", Some(json!(["consistent"]))),
        ("var x = {b, c: d, f: g}", Some(json!(["consistent"]))),
        ("var x = {foo, bar: baz, ...qux}", Some(json!(["consistent"]))),
        ("var x = {a: a, b: b}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a, z: function z(){}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo: function() {}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a: a, b: b, ...baz}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo, bar: bar, ...qux}", Some(json!(["consistent-as-needed"]))),
    ];

    let fix = vec![
        ("var x = {x: x}", "var x = {x}", None),
        ("var x = {'x': x}", "var x = {x}", None),
        ("var x = {y: y, x: x}", "var x = {y, x}", None),
        ("var x = {y: z, x: x, a: b}", "var x = {y: z, x, a: b}", None),
        (
            "var x = {y: z,
              x: x,
              a: b
              // comment 
            }",
            "var x = {y: z,
              x,
              a: b
              // comment 
            }",
            None,
        ),
        (
            "var x = {y: z,
              a: b,
              // comment 
              f: function() {}}",
            "var x = {y: z,
              a: b,
              // comment 
              f() {}}",
            None,
        ),
        (
            "var x = {a: b,
              /* comment */
              y: y
            }",
            "var x = {a: b,
              /* comment */
              y
            }",
            None,
        ),
        (
            "var x = {
              a: b,
              /* comment */
              y: y
            }",
            "var x = {
              a: b,
              /* comment */
              y
            }",
            None,
        ),
        (
            "var x = {
              f: function() {
                /* comment */
                a(b);
                }
              }",
            "var x = {
              f() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              [f]: function() {
                /* comment */
                a(b);
                }
              }",
            "var x = {
              [f]() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              f: function*() {
                /* comment */
                a(b);
                }
              }",
            "var x = {
              *f() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        ("var x = {y: function() {}}", "var x = {y() {}}", None),
        ("var x = {y: function*() {}}", "var x = {*y() {}}", None),
        ("var x = {x: y, y: z, a: a}", "var x = {x: y, y: z, a}", None),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            "var x = {ConstructorFunction(){}, a: b}",
            None,
        ),
        (
            "var x = {x: y, y: z, a: function(){}, b() {}}",
            "var x = {x: y, y: z, a(){}, b() {}}",
            None,
        ),
        ("var x = {x: x, y: function() {}}", "var x = {x, y() {}}", None),
        ("doSomething({x: x})", "doSomething({x})", None),
        ("doSomething({'x': x})", "doSomething({x})", None),
        ("doSomething({a: 'a', 'x': x})", "doSomething({a: 'a', x})", None),
        ("doSomething({y: function() {}})", "doSomething({y() {}})", None),
        ("doSomething({[y]: function() {}})", "doSomething({[y]() {}})", None),
        ("doSomething({['y']: function() {}})", "doSomething({['y']() {}})", None),
        ("({ foo: async function () {} })", "({ async foo () {} })", None),
        ("({ 'foo': async function() {} })", "({ async 'foo'() {} })", None),
        ("({ [foo]: async function() {} })", "({ async [foo]() {} })", None),
        ("({ [foo.bar]: function*() {} })", "({ *[foo.bar]() {} })", None),
        ("({ [foo   ]: function() {} })", "({ [foo   ]() {} })", None),
        ("({ [ foo ]: async function() {} })", "({ async [ foo ]() {} })", None),
        ("({ foo: function *() {} })", "({ *foo() {} })", None),
        ("({ [  foo   ]: function() {} })", "({ [  foo   ]() {} })", None),
        ("({ [  foo]: function() {} })", "({ [  foo]() {} })", None),
        // options
        ("var x = {y: function() {}}", "var x = {y() {}}", Some(json!(["methods"]))),
        (
            "var x = {x, y() {}, z: function() {}}",
            "var x = {x, y() {}, z() {}}",
            Some(json!(["methods"])),
        ),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            "var x = {ConstructorFunction(){}, a: b}",
            Some(json!(["methods"])),
        ),
        ("var x = {[y]: function() {}}", "var x = {[y]() {}}", Some(json!(["methods"]))),
        ("({ [(foo)]: function() { return; } })", "({ [(foo)]() { return; } })", None),
        ("({ [(foo)]: async function() { return; } })", "({ async [(foo)]() { return; } })", None),
        (
            "({ [(((((((foo)))))))]: function() { return; } })",
            "({ [(((((((foo)))))))]() { return; } })",
            None,
        ),
        ("var x = {x: x}", "var x = {x}", Some(json!(["properties"]))),
        ("var x = {a, b, c(){}, x: x}", "var x = {a, b, c(){}, x}", Some(json!(["properties"]))),
        ("({ a: (function(){ return foo; }) })", "({ a(){ return foo; } })", None),
        ("({ a: async function*() {} })", "({ async *a() {} })", Some(json!(["always"]))),
    ];

    Tester::new(ObjectShorthand::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_never() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("var x = {a: n, c: d, f: g}", Some(json!(["never"]))),
        ("var x = {a: function(){}, b: {c: d}}", Some(json!(["never"]))),
        ("let {a, b} = o;", Some(json!(["never"]))),
        ("var x = {foo: foo, bar: bar, ...baz}", Some(json!(["never"]))),
    ];

    let fail = vec![];

    let fix = vec![
        (
            "({ [(foo)]() { return; } })",
            "({ [(foo)]: function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ async [(foo)]() { return; } })",
            "({ [(foo)]: async function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ *[((foo))]() { return; } })",
            "({ [((foo))]: function*() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ [(((((((foo)))))))]() { return; } })",
            "({ [(((((((foo)))))))]: function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ 'foo bar'() { return; } })",
            "({ 'foo bar': function() { return; } })",
            Some(json!(["never"])),
        ),
        ("({ *foo() { return; } })", "({ foo: function*() { return; } })", Some(json!(["never"]))),
        (
            "({ async foo() { return; } })",
            "({ foo: async function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ *['foo bar']() { return; } })",
            "({ ['foo bar']: function*() { return; } })",
            Some(json!(["never"])),
        ),
        ("var x = {y() {}}", "var x = {y: function() {}}", Some(json!(["never"]))),
        ("var x = {*y() {}}", "var x = {y: function*() {}}", Some(json!(["never"]))),
        ("var x = {y}", "var x = {y: y}", Some(json!(["never"]))),
        (
            "var x = {y, a: b, *x(){}}",
            "var x = {y: y, a: b, x: function*(){}}",
            Some(json!(["never"])),
        ),
        ("var x = {y: {x}}", "var x = {y: {x: x}}", Some(json!(["never"]))),
        (
            "var x = {ConstructorFunction(){}, a: b}",
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(json!(["never"])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            "var x = {notConstructorFunction: function(){}, b: c}",
            Some(json!(["never"])),
        ),
        (
            "var x = {foo, bar: baz, ...qux}",
            "var x = {foo: foo, bar: baz, ...qux}",
            Some(json!(["never"])),
        ),
        ("({ async* a() {} })", "({ a: async function*() {} })", Some(json!(["never"]))),
    ];

    Tester::new(ObjectShorthand::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_ignore_constructors() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        ("({ [foo.bar]: () => {} })", Some(json!(["always", { "ignoreConstructors": true }]))),
    ];

    let fix = vec![
        (
            "var x = {y: function() {}}",
            "var x = {y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_y: function() {}}",
            "var x = {_y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$y: function() {}}",
            "var x = {$y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__y: function() {}}",
            "var x = {__y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0y: function() {}}",
            "var x = {_0y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, pass, vec![]).expect_fix(fix).test_and_snapshot();

    let pass = vec![
        ("var x = {ConstructorFunction: function(){}, a: b}", Some(json!(["never"]))),
        ("var x = {notConstructorFunction: function(){}, b: c}", Some(json!(["never"]))),
    ];

    Tester::new(ObjectShorthand::NAME, pass, vec![]).test();
}

#[test]
fn test_methods_ignore_pattern() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (
            "var x = { foo: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: function() {}  }",
            Some(json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: function*() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: async function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: () => { return 5; }  }",
            Some(
                json!(["always", { "methodsIgnorePattern": "^foo$", "avoidExplicitReturnArrows": true }]),
            ),
        ),
        (
            "var x = { 'foo': function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { ['foo']: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 123: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { afoob: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { afoob: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^.foo.$" }])),
        ),
        (
            "var x = { '👍foo👍': function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^.foo.$" }])),
        ),
    ];

    let fix = vec![
        (
            "var x = { afoob: function() {} }",
            "var x = { afoob() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { afoob: function() {} }",
            "var x = { afoob() {} }",
            Some(json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 'afoob': function() {} }",
            "var x = { 'afoob'() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 1234: function() {} }",
            "var x = { 1234() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { bar: function() {} }",
            "var x = { bar() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { [foo]: function() {} }",
            "var x = { [foo]() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { foo: foo }", // does not apply to properties
            "var x = { foo }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, pass, vec![]).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_avoid_quotes() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("var x = {'a': function(){}}", Some(json!(["always", { "avoidQuotes": true }]))),
        ("var x = {['a']: function(){}}", Some(json!(["methods", { "avoidQuotes": true }]))),
        ("var x = {'y': y}", Some(json!(["properties", { "avoidQuotes": true }]))),
    ];

    let fix = vec![
        ("var x = {a: a}", "var x = {a}", Some(json!(["always", { "avoidQuotes": true }]))),
        (
            "var x = {a: function(){}}",
            "var x = {a(){}}",
            Some(json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {[a]: function(){}}",
            "var x = {[a](){}}",
            Some(json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {'a'(){}}",
            "var x = {'a': function(){}}",
            Some(json!(["always", { "avoidQuotes": true }])),
        ),
        (
            "var x = {['a'](){}}",
            "var x = {['a']: function(){}}",
            Some(json!(["methods", { "avoidQuotes": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, pass, vec![]).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_avoid_explicit_return_arrows() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("({ x: () => foo })", Some(json!(["always", { "avoidExplicitReturnArrows": false }]))),
        (
            "({ x: () => { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": false }])),
        ),
        ("({ x: () => foo })", Some(json!(["always", { "avoidExplicitReturnArrows": true }]))),
        ("({ x() { return; } })", Some(json!(["always", { "avoidExplicitReturnArrows": true }]))),
        (
            "({ x() { return; }, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => foo })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => foo, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { this; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "function foo() { ({ x: () => { arguments; } }) }",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                class Foo extends Bar {
                    constructor() {
                        var foo = { x: () => { super(); } };
                    }
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                class Foo extends Bar {
                    baz() {
                        var foo = { x: () => { super.baz(); } };
                    }
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                function foo() {
                    var x = { x: () => { new.target; } };
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                function foo() {
                    var x = {
                        x: () => {
                            var y = () => { this; };
                        }
                    };
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                function foo() {
                    var x = {
                        x: () => {
                            var y = () => { this; };
                            function foo() { this; }
                        }
                    };
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                function foo() {
                    var x = {
                        x: () => {
                            return { y: () => { this; } };
                        }
                    };
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
    ];

    let fix = vec![
        (
            "({ x: (arg => { return; }) })",
            "({ x(arg) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; } })",
            "({ x() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => { return; } })",
            "({ x() { return; }, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => foo })",
            "({ x() { return; }, y: () => foo })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => { return; } })",
            "({ x() { return; }, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: foo => { return; } })",
            "({ x(foo) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: (foo = 1) => { return; } })",
            "({ x(foo = 1) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: ({ foo: bar = 1 } = {}) => { return; } })",
            "({ x({ foo: bar = 1 } = {}) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { this; } } })",
            "({ x() { function foo() { this; } } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { var foo = function() { arguments; } } })",
            "({ x() { var foo = function() { arguments; } } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { arguments; } } })",
            "({ x() { function foo() { arguments; } } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                ({
                    x: () => {
                        class Foo extends Bar {
                            constructor() {
                                super();
                            }
                        }
                    }
                })
            ",
            "
                ({
                    x() {
                        class Foo extends Bar {
                            constructor() {
                                super();
                            }
                        }
                    }
                })
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                ({
                    x: () => {
                        function foo() {
                            new.target;
                        }
                    }
                })
            ",
            "
                ({
                    x() {
                        function foo() {
                            new.target;
                        }
                    }
                })
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ 'foo bar': () => { return; } })",
            "({ 'foo bar'() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [foo]: () => { return; } })",
            "({ [foo]() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: 1, foo: async (bar = 1) => { return; } })",
            "({ a: 1, async foo(bar = 1) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [ foo ]: async bar => { return; } })",
            "({ async [ foo ](bar) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ key: (arg = () => {}) => {} })",
            "({ key(arg = () => {}) {} })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                function foo() {
                    var x = {
                        x: () => {
                            this;
                            return { y: () => { foo; } };
                        }
                    };
                }
            ",
            "
                function foo() {
                    var x = {
                        x: () => {
                            this;
                            return { y() { foo; } };
                        }
                    };
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                function foo() {
                    var x = {
                        x: () => {
                            ({ y: () => { foo; } });
                            this;
                        }
                    };
                }
            ",
            "
                function foo() {
                    var x = {
                        x: () => {
                            ({ y() { foo; } });
                            this;
                        }
                    };
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (() => { return foo; }) })",
            "({ a(){ return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg) => { return foo; }) })",
            "({ a(arg) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg, arg2) => { return foo; }) })",
            "({ a(arg, arg2) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async () => { return foo; }) })",
            "({ async a() { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg) => { return foo; }) })",
            "({ async a(arg) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg, arg2) => { return foo; }) })",
            "({ async a(arg, arg2) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                const test = {
                    key: <T>(): void => { },
                    key: async <T>(): Promise<void> => { },
                    key: <T>(arg: T): T => { return arg },
                    key: async <T>(arg: T): Promise<T> => { return arg },
                }
            ",
            "
                const test = {
                    key<T>(): void { },
                    async key<T>(): Promise<void> { },
                    key<T>(arg: T): T { return arg },
                    async key<T>(arg: T): Promise<T> { return arg },
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                const test = {
                    key: (): void => {x()},
                    key: ( (): void => {x()} ),
                    key: ( (): (void) => {x()} ),

                    key: (arg: t): void => {x()},
                    key: ( (arg: t): void => {x()} ),
                    key: ( (arg: t): (void) => {x()} ),

                    key: (arg: t, arg2: t): void => {x()},
                    key: ( (arg: t, arg2: t): void => {x()} ),
                    key: ( (arg: t, arg2: t): (void) => {x()} ),

                    key: async (): void => {x()},
                    key: ( async (): void => {x()} ),
                    key: ( async (): (void) => {x()} ),

                    key: async (arg: t): void => {x()},
                    key: ( async (arg: t): void => {x()} ),
                    key: ( async (arg: t): (void) => {x()} ),

                    key: async (arg: t, arg2: t): void => {x()},
                    key: ( async (arg: t, arg2: t): void => {x()} ),
                    key: ( async (arg: t, arg2: t): (void) => {x()} ),
                }
            ",
            "
                const test = {
                    key(): void {x()},
                    key(): void {x()},
                    key(): (void) {x()},

                    key(arg: t): void {x()},
                    key(arg: t): void {x()},
                    key(arg: t): (void) {x()},

                    key(arg: t, arg2: t): void {x()},
                    key(arg: t, arg2: t): void {x()},
                    key(arg: t, arg2: t): (void) {x()},

                    async key(): void {x()},
                    async key(): void {x()},
                    async key(): (void) {x()},

                    async key(arg: t): void {x()},
                    async key(arg: t): void {x()},
                    async key(arg: t): (void) {x()},

                    async key(arg: t, arg2: t): void {x()},
                    async key(arg: t, arg2: t): void {x()},
                    async key(arg: t, arg2: t): (void) {x()},
                }
            ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, pass, vec![]).expect_fix(fix).test_and_snapshot();
}
