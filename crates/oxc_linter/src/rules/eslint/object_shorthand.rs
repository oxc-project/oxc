use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum ObjectShorthandDiagnostic {
    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Expected shorthand for all properties.")]
    ExpectedAllPropertiesShorthanded(#[label] Span),

    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Expected longform method syntax for string literal keys.")]
    ExpectedLiteralMethodLongform(#[label] Span),

    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Expected property shorthand.")]
    ExpectedPropertyShorthand(#[label] Span),

    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Expected longform property syntax.")]
    ExpectedPropertyLongform(#[label] Span),

    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Expected method shorthand.")]
    ExpectedMethodShorthand(#[label] Span),

    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Expected longform method syntax.")]
    ExpectedMethodLongform(#[label] Span),

    #[diagnostic(severity(warning))]
    #[error("eslint(object-shorthand): Unexpected mix of shorthand and non-shorthand properties.")]
    UnexpectedMix(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct ObjectShorthand(Box<ObjectShorthandConfig>);

#[derive(Debug, Default, Clone)]
pub struct ObjectShorthandConfig {
    shorthand_type: ShorthandType,
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

        Self(Box::new(ObjectShorthandConfig {
            shorthand_type: obj1
                .and_then(serde_json::Value::as_str)
                .map(ShorthandType::from)
                .unwrap_or_default(),
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

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
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
        (
            "var x = {
          f: /* comment */ function() {
          }
          }",
            None,
        ),
        (
            "var x = {
         f /* comment */: function() {
          }
          }",
            None,
        ),
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
