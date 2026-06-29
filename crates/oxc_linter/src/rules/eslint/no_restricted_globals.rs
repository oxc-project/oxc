use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::de::Error;
use serde_json::Value;

use crate::{AstNode, ast_util::iter_outer_expressions, context::LintContext, rule::Rule};

fn no_restricted_globals(global_name: &str, suffix: &str, span: Span) -> OxcDiagnostic {
    let warn_text = if suffix.is_empty() {
        format!("Unexpected use of '{global_name}'.")
    } else {
        format!("Unexpected use of '{global_name}'. {suffix}")
    };

    OxcDiagnostic::warn(warn_text)
        .with_help("Use a local variable or function parameter instead of the restricted global.")
        .with_label(span)
}

#[derive(Debug, Clone, Default)]
pub struct NoRestrictedGlobals(Box<NoRestrictedGlobalsConfig>);

impl Deref for NoRestrictedGlobals {
    type Target = NoRestrictedGlobalsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoRestrictedGlobalsConfig {
    /// Objects in the format
    /// `{ "name": "event", "message": "Use local parameter instead." }`, which define what globals
    /// are restricted from use.
    globals: FxHashMap<String, String>,
    /// A boolean option that enables detection of restricted globals accessed via global objects. Default is `false`.
    check_global_object: bool,
    /// An array option that specifies additional global object names to check when `checkGlobalObject` is enabled.
    /// By default, the rule checks these global objects: `globalThis`, `self`, and `window`.
    global_objects: Vec<CompactStr>,
}

fn default_globals_objects() -> Vec<CompactStr> {
    vec![CompactStr::new("globalThis"), CompactStr::new("self"), CompactStr::new("window")]
}

impl Default for NoRestrictedGlobalsConfig {
    fn default() -> Self {
        Self {
            globals: FxHashMap::default(),
            check_global_object: false,
            global_objects: default_globals_objects(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Specify global variable names that should not be used in your application.
    ///
    /// ### Why is this bad?
    ///
    /// Disallowing usage of specific global variables can be useful if you want to allow a set of global
    /// variables by enabling an environment, but still want to disallow some of those.
    ///
    /// For instance, early Internet Explorer versions exposed the current DOM event as a global variable
    /// `event`, but using this variable has been considered as a bad practice for a long time. Restricting
    /// this will make sure this variable isn't used in browser code.
    ///
    /// ### Examples
    ///
    /// If we have options:
    ///
    /// ```json
    /// "no-restricted-globals": ["error", "event"]
    /// ```
    ///
    /// The following patterns are considered problems:
    ///
    /// ```javascript
    /// function onClick() {
    ///    console.log(event);    // Unexpected global variable 'event'. Use local parameter instead.
    /// }
    /// ```
    NoRestrictedGlobals,
    eslint,
    restriction,
    config = NoRestrictedGlobalsConfig,
    version = "0.4.0",
    short_description = "Specify global variable names that should not be used in your application.",
);

impl Rule for NoRestrictedGlobals {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut config = NoRestrictedGlobalsConfig::default();
        let Value::Array(arr) = value else {
            return Ok(Self(Box::new(config)));
        };

        for item in arr {
            match item {
                // "no-restricted-globals": ["error", "event"]
                Value::String(name) => {
                    config.globals.insert(name.clone(), String::new());
                }
                Value::Object(obj) => {
                    match obj.get("globals") {
                        // "no-restricted-globals": ["error", { globalObjects: [], checkGlobalObject: true, "globals": [{"object": { "name": "event", "message": "Use local parameter instead." } }] }]
                        Some(Value::Array(globals)) => {
                            for global in globals {
                                match global {
                                    Value::String(name) => {
                                        config.globals.insert(name.clone(), String::new());
                                    }
                                    Value::Object(global_obj) => {
                                        let name = global_obj
                                            .get("name")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default();
                                        let message = global_obj
                                            .get("message")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default();
                                        config
                                            .globals
                                            .insert(name.to_string(), message.to_string());
                                    }
                                    _ => {
                                        return Err(serde_json::error::Error::custom(
                                            "Expected 'globals' array to contain either strings or objects",
                                        ));
                                    }
                                }
                            }

                            config.check_global_object = obj
                                .get("checkGlobalObject")
                                .and_then(Value::as_bool)
                                .unwrap_or(false);

                            if let Some(Value::Array(global_objects)) = obj.get("globalObjects") {
                                config.global_objects = global_objects
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .map(CompactStr::new)
                                    .collect();
                            }
                        }
                        Some(_) => {
                            return Err(serde_json::error::Error::custom(
                                "Expected 'object' property to be an array",
                            ));
                        }
                        _ => {
                            // "no-restricted-globals": ["error", { "name": "event", "message": "Use local parameter instead." }]
                            let name = obj.get("name").and_then(Value::as_str).unwrap_or_default();
                            let message =
                                obj.get("message").and_then(Value::as_str).unwrap_or_default();
                            config.globals.insert(name.to_string(), message.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        config.global_objects.extend(default_globals_objects());

        Ok(Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext) {
        // Prefer `run_once` only (not `run` + `run_once`): implementing both forces NODE_TYPES=None
        // and invokes the rule on every AST node.
        if self.globals.is_empty() {
            return;
        }
        let unresolved = ctx.scoping().root_unresolved_references();
        for (name, message) in &self.globals {
            let Some(ref_ids) = unresolved.get(name.as_str()) else {
                continue;
            };
            for &ref_id in ref_ids {
                let reference = ctx.scoping().get_reference(ref_id);
                if reference.symbol_id().is_some() || reference.is_type() {
                    continue;
                }
                let node = ctx.nodes().get_node(reference.node_id());
                let AstKind::IdentifierReference(ident) = node.kind() else {
                    continue;
                };
                if self.check_global_object
                    && is_ident_property(ident, &ctx.nodes().parent_kind(node.id()))
                {
                    continue;
                }
                ctx.diagnostic(no_restricted_globals(name, message, ident.span));
            }
        }

        // Restricted globals accessed as `window.event` / `globalThis['event']`.
        if !self.check_global_object {
            return;
        }
        // `global_objects` may contain duplicates (defaults are appended in from_configuration).
        let mut seen_global_objs = rustc_hash::FxHashSet::default();
        for global_obj in &self.global_objects {
            if !seen_global_objs.insert(global_obj.as_str()) {
                continue;
            }
            let Some(ref_ids) = unresolved.get(global_obj.as_str()) else {
                continue;
            };
            for &ref_id in ref_ids {
                let node = ctx.nodes().get_node(ctx.scoping().get_reference(ref_id).node_id());
                self.check_global_object_member_access(node, ctx);
            }
        }
    }
}

impl NoRestrictedGlobals {
    fn check_global_object_member_access<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let ident_span = node.kind().span();
        let Some(parent) = iter_outer_expressions(ctx.nodes(), node.id()).next() else {
            return;
        };
        match parent {
            AstKind::StaticMemberExpression(expression) => {
                let Some(obj_ident) = expression.object.get_identifier_reference() else {
                    return;
                };
                // Only when this identifier is the object, not the property.
                if obj_ident.span != ident_span {
                    return;
                }
                let Some(message) = self.globals.get(expression.property.name.as_str()) else {
                    return;
                };
                ctx.diagnostic(no_restricted_globals(
                    expression.property.name.as_str(),
                    message,
                    expression.property.span,
                ));
            }
            AstKind::ComputedMemberExpression(expression) => {
                let Some(obj_ident) = expression.object.get_identifier_reference() else {
                    return;
                };
                // Only when this identifier is the object, not the property.
                if obj_ident.span != ident_span {
                    return;
                }
                let property_name = match &expression.expression {
                    Expression::StringLiteral(str) => str.value.as_str(),
                    Expression::TemplateLiteral(template)
                        if template.is_no_substitution_template() =>
                    {
                        let Some(cooked) = &template.quasis[0].value.cooked else {
                            return;
                        };
                        cooked.as_str()
                    }
                    _ => return,
                };
                let Some(message) = self.globals.get(property_name) else {
                    return;
                };
                ctx.diagnostic(no_restricted_globals(
                    property_name,
                    message,
                    expression.expression.span(),
                ));
            }
            _ => {}
        }
    }
}

fn is_ident_property(ident: &IdentifierReference, kind: &AstKind) -> bool {
    match kind {
        AstKind::StaticMemberExpression(expression) => {
            expression.property.node_id() == ident.node_id()
        }
        AstKind::ComputedMemberExpression(expression) => {
            let Expression::Identifier(ident_ref) = &expression.expression else {
                return false;
            };

            ident_ref.node_id() == ident.node_id()
        }
        _ => false,
    }
}
#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    const CUSTOM_MESSAGE: &str = "Use bar instead.";

    let pass = vec![
        ("foo", None, None),
        ("foo", Some(serde_json::json!(["bar"])), None),
        ("var foo = 1;", Some(serde_json::json!(["foo"])), None),
        ("event", Some(serde_json::json!(["bar"])), Some(json!({"env": { "browser": true}}))),
        ("import foo from 'bar';", Some(serde_json::json!(["foo"])), None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() {}", Some(serde_json::json!(["foo"])), None),
        ("function fn() { var foo; }", Some(serde_json::json!(["foo"])), None),
        ("foo.bar", Some(serde_json::json!(["bar"])), None),
        ("foo", Some(serde_json::json!([{ "name": "bar", "message": "Use baz instead." }])), None),
        ("foo", Some(serde_json::json!([{ "globals": ["bar"] }])), None),
        ("const foo = 1", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        (
            "event",
            Some(serde_json::json!([{ "globals": ["bar"] }])),
            Some(json!({"env": { "browser": true}})),
        ),
        ("import foo from 'bar';", Some(serde_json::json!([{ "globals": ["foo"] }])), None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() {}", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        ("function fn() { let foo; }", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        ("foo.bar", Some(serde_json::json!([{ "globals": ["bar"] }])), None),
        (
            "foo",
            Some(
                serde_json::json!([ { "globals": [{ "name": "bar", "message": CUSTOM_MESSAGE }] }, ]),
            ),
            None,
        ),
        (
            "window.foo()",
            Some(serde_json::json!([{ "globals": ["foo"] }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "self.foo()",
            Some(serde_json::json!([{ "globals": ["foo"] }])),
            Some(json!({"env": { "browser": true}})),
        ),
        ("globalThis.foo()", Some(serde_json::json!([{ "globals": ["foo"] }])), None), // { "ecmaVersion": 2020 },
        (
            "myGlobal.foo()",
            Some(serde_json::json!([ { "globals": ["foo"], "globalObjects": ["myGlobal"], }, ])),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        // not sure why eslint doesn't report these cases when checkGlobalObject is true,
        // but we will report them since it's more intuitive.
        // (
        //     "window.foo()",
        //     Some(serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, }, ])),
        //     None,
        // ),
        // (
        //     "self.foo()",
        //     Some(serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, }, ])),
        //     None,
        // ),
        // (
        //     "globalThis.foo()",
        //     Some(serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, }, ])),
        //     None,
        // ), // { "ecmaVersion": 6 },
        // (
        //     "myGlobal.foo()",
        //     Some(
        //         serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
        //     ),
        //     None,
        // ),
        (
            "otherGlobal.foo()",
            Some(
                serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "otherGlobal": "readonly" }})),
        ),
        (
            "foo.window.bar()",
            Some(serde_json::json!([ { "globals": ["bar"], "checkGlobalObject": true, }, ])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo.self.bar()",
            Some(serde_json::json!([ { "globals": ["bar"], "checkGlobalObject": true, }, ])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo.globalThis.bar()",
            Some(serde_json::json!([ { "globals": ["bar"], "checkGlobalObject": true, }, ])),
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "foo.myGlobal.bar()",
            Some(
                serde_json::json!([ { "globals": ["bar"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        (
            "let window; window.foo()",
            Some(serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, }, ])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "let self; self.foo()",
            Some(serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, }, ])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "let globalThis; globalThis.foo()",
            Some(serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, }, ])),
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "let myGlobal; myGlobal.foo()",
            Some(
                serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        ("foo", None, None),
        ("foo", Some(serde_json::json!(["bar"])), None),
        ("const foo: number = 1;", Some(serde_json::json!(["foo"])), None),
        ("event", Some(serde_json::json!(["bar"])), Some(json!({"env": { "browser": true}}))),
        ("import foo from 'bar';", Some(serde_json::json!(["foo"])), None),
        ("function foo(): void {}", Some(serde_json::json!(["foo"])), None),
        ("function fn(): void { let foo; }", Some(serde_json::json!(["foo"])), None),
        ("foo.bar", Some(serde_json::json!(["bar"])), None),
        ("foo", Some(serde_json::json!([{ "name": "bar", "message": "Use baz instead." }])), None),
        (
            "
                        export default class Test {
                            private status: string;
                            getStatus() {
                                return this.status;
                            }
                        }",
            Some(serde_json::json!(["status"])),
            None,
        ),
        ("type Handler = (event: string) => any", Some(serde_json::json!(["event"])), None),
        ("let b: { c: Test }", Some(serde_json::json!(["Test"])), None),
        ("function foo(param: Test) {}", Some(serde_json::json!(["Test"])), None),
        ("1 as Test", Some(serde_json::json!(["Test"])), None),
        ("class Derived implements Test {}", Some(serde_json::json!(["Test"])), None),
        (
            "class Derived implements Test1, Test2 {}",
            Some(serde_json::json!(["Test1", "Test2"])),
            None,
        ),
        ("interface Derived extends Test {}", Some(serde_json::json!(["Test"])), None),
        ("type Intersection = Test & {}", Some(serde_json::json!(["Test"])), None),
        ("type Union = Test | {}", Some(serde_json::json!(["Test"])), None),
        ("let value: NS.Test", Some(serde_json::json!(["NS"])), None),
        ("let value: NS.Test", Some(serde_json::json!(["Test"])), None),
        ("let value: NS.Test", Some(serde_json::json!(["NS.Test"])), None),
        // ("let value: typeof Test", Some(serde_json::json!(["Test"])), None), TODO: @Sysix
        ("let value: Type<Test>", Some(serde_json::json!(["Type", "Test"])), None),
        ("type Intersection = Test<any>", Some(serde_json::json!(["Test", "any"])), None),
        ("type Intersection = Test<A, B>", Some(serde_json::json!(["Test", "A", "B"])), None),
        ("foo.bar", Some(serde_json::json!(["bar"])), None),
        ("foo.globalThis.bar", Some(serde_json::json!(["bar"])), None),
        ("foo.globalThis.bar()", Some(serde_json::json!(["bar"])), None),
        (
            "function handler(event) { return event.target; }",
            Some(serde_json::json!(["event"])),
            None,
        ),
        ("function handler(name) { return name.length; }", Some(serde_json::json!(["name"])), None),
    ];

    let fail = vec![
        ("foo", Some(serde_json::json!(["foo"])), None),
        ("function fn() { foo; }", Some(serde_json::json!(["foo"])), None),
        (
            "function fn() { foo; }",
            Some(serde_json::json!(["foo"])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(serde_json::json!(["foo", "event"])),
            Some(json!({"env": { "browser": true}})),
        ),
        ("foo", Some(serde_json::json!(["foo"])), Some(json!({"globals": { "foo": "readonly" }}))),
        ("foo()", Some(serde_json::json!(["foo"])), None),
        ("foo.bar()", Some(serde_json::json!(["foo"])), None),
        ("foo", Some(serde_json::json!([{ "name": "foo" }])), None),
        ("function fn() { foo; }", Some(serde_json::json!([{ "name": "foo" }])), None),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "name": "foo" }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(serde_json::json!(["foo", { "name": "event" }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(serde_json::json!([{ "name": "foo" }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        ("foo()", Some(serde_json::json!([{ "name": "foo" }])), None),
        ("foo.bar()", Some(serde_json::json!([{ "name": "foo" }])), None),
        ("foo", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])), None),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            None,
        ),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(
                serde_json::json!([ "foo", { "name": "event", "message": "Use local event parameter." }, ]),
            ),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        ("foo()", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])), None),
        (
            "foo.bar()",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            None,
        ),
        (
            "var foo = obj => hasOwnProperty(obj, 'name');",
            Some(serde_json::json!(["hasOwnProperty"])),
            None,
        ), // { "ecmaVersion": 6 },
        ("foo", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        ("function fn() { foo; }", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "globals": ["foo"] }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(serde_json::json!([{ "globals": ["foo", "event"] }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(serde_json::json!([{ "globals": ["foo"] }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        ("foo()", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        ("foo.bar()", Some(serde_json::json!([{ "globals": ["foo"] }])), None),
        ("foo", Some(serde_json::json!([{ "globals": [{ "name": "foo" }] }])), None),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "globals": [{ "name": "foo" }] }])),
            None,
        ),
        (
            "function fn() { foo; }",
            Some(serde_json::json!([{ "globals": [{ "name": "foo" }] }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(serde_json::json!([{ "globals": ["foo", { "name": "event" }] }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(serde_json::json!([{ "globals": [{ "name": "foo" }] }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        ("foo()", Some(serde_json::json!([{ "globals": [{ "name": "foo" }] }])), None),
        ("foo.bar()", Some(serde_json::json!([{ "globals": [{ "name": "foo" }] }])), None),
        (
            "foo",
            Some(
                serde_json::json!([{ "globals": [{ "name": "foo", "message": CUSTOM_MESSAGE }] }]),
            ),
            None,
        ),
        (
            "function fn() { foo; }",
            Some(
                serde_json::json!([{ "globals": [{ "name": "foo", "message": CUSTOM_MESSAGE }] }]),
            ),
            None,
        ),
        (
            "function fn() { foo; }",
            Some(
                serde_json::json!([{ "globals": [{ "name": "foo", "message": CUSTOM_MESSAGE }] }]),
            ),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(
                serde_json::json!([ { "globals": [ "foo", { "name": "event", "message": "Use local event parameter.", }, ], }, ]),
            ),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(
                serde_json::json!([{ "globals": [{ "name": "foo", "message": CUSTOM_MESSAGE }] }]),
            ),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "foo()",
            Some(
                serde_json::json!([{ "globals": [{ "name": "foo", "message": CUSTOM_MESSAGE }] }]),
            ),
            None,
        ),
        (
            "foo.bar()",
            Some(
                serde_json::json!([{ "globals": [{ "name": "foo", "message": CUSTOM_MESSAGE }] }]),
            ),
            None,
        ),
        (
            "var foo = obj => hasOwnProperty(obj, 'name');",
            Some(serde_json::json!([{ "globals": ["hasOwnProperty"] }])),
            None,
        ), // { "ecmaVersion": 6 },
        (
            "window.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "(window).foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "window!.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "self.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        // TODO: we do not supported nested global object access like window.window.foo()
        // (
        //     "window.window.foo()",
        //     Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
        //     Some(json!({"env": { "browser": true}})),
        // ),
        // (
        //     "self.self.foo()",
        //     Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
        //     Some(json!({"env": { "browser": true}})),
        // ),
        (
            "globalThis.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "(globalThis as any).foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            None,
        ), // { "ecmaVersion": 2020 },
        // TODO: we do not supported nested global object access like window.window.foo()
        // (
        //     "globalThis.globalThis.foo()",
        //     Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
        //     None,
        // ), // { "ecmaVersion": 2020 },
        (
            "myGlobal.foo()",
            Some(
                serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        // TODO: we do not supported nested global object access like window.window.foo()
        // (
        //     "myGlobal.myGlobal.foo()",
        //     Some(
        //         serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
        //     ),
        //     Some(json!({"globals": { "myGlobal": "readonly" }})),
        // ),
        (
            r#"window["foo"]"#,
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            r#"self["foo"]"#,
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            r#"globalThis["foo"]"#,
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            None,
        ), // { "ecmaVersion": 2020 },
        (
            r#"myGlobal["foo"]"#,
            Some(
                serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        (
            "window?.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "self?.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "window.foo(); myGlobal.foo()",
            Some(
                serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"env": { "browser": true}, "globals": { "myGlobal": "readonly" }})),
        ),
        (
            "myGlobal.foo(); myOtherGlobal.bar()",
            Some(
                serde_json::json!([ { "globals": ["foo", "bar"], "checkGlobalObject": true, "globalObjects": ["myGlobal", "myOtherGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly", "myOtherGlobal": "readonly" }})),
        ),
        (
            "foo(); window.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo(); self.foo()",
            Some(serde_json::json!([{ "globals": ["foo"], "checkGlobalObject": true }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo(); myGlobal.foo()",
            Some(
                serde_json::json!([ { "globals": ["foo"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        (
            "function onClick(event) { console.log(event); console.log(window.event); }",
            Some(serde_json::json!([ { "globals": ["event"], "checkGlobalObject": true, }, ])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "function onClick(event) { console.log(event); console.log(self.event); }",
            Some(serde_json::json!([ { "globals": ["event"], "checkGlobalObject": true, }, ])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "function onClick(event) { console.log(event); console.log(globalThis.event); }",
            Some(serde_json::json!([ { "globals": ["event"], "checkGlobalObject": true, }, ])),
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "function onClick(event) { console.log(event); console.log(myGlobal.event); }",
            Some(
                serde_json::json!([ { "globals": ["event"], "checkGlobalObject": true, "globalObjects": ["myGlobal"], }, ]),
            ),
            Some(json!({"globals": { "myGlobal": "readonly" }})),
        ),
        ("foo", Some(serde_json::json!(["foo"])), None),
        ("function fn(): void { foo; }", Some(serde_json::json!(["foo"])), None),
        (
            "function fn(): void { foo; }",
            Some(serde_json::json!(["foo"])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(serde_json::json!(["foo", "event"])),
            Some(json!({"env": { "browser": true}})),
        ),
        ("foo", Some(serde_json::json!(["foo"])), Some(json!({"globals": { "foo": "readonly" }}))),
        ("foo()", Some(serde_json::json!(["foo"])), None),
        ("foo.bar()", Some(serde_json::json!(["foo"])), None),
        ("foo", Some(serde_json::json!([{ "name": "foo" }])), None),
        ("function fn(): void { foo; }", Some(serde_json::json!([{ "name": "foo" }])), None),
        (
            "function fn(): void { foo; }",
            Some(serde_json::json!([{ "name": "foo" }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(serde_json::json!(["foo", { "name": "event" }])),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(serde_json::json!([{ "name": "foo" }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        ("foo()", Some(serde_json::json!([{ "name": "foo" }])), None),
        ("foo.bar()", Some(serde_json::json!([{ "name": "foo" }])), None),
        ("foo", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])), None),
        (
            "function fn(): void { foo; }",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            None,
        ),
        (
            "function fn(): void { foo; }",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        (
            "event",
            Some(
                serde_json::json!([ "foo", { "name": "event", "message": "Use local event parameter." }, ]),
            ),
            Some(json!({"env": { "browser": true}})),
        ),
        (
            "foo",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            Some(json!({"globals": { "foo": "readonly" }})),
        ),
        ("foo()", Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])), None),
        (
            "foo.bar()",
            Some(serde_json::json!([{ "name": "foo", "message": CUSTOM_MESSAGE }])),
            None,
        ),
        (
            "const foo = obj => hasOwnProperty(obj, 'name');",
            Some(serde_json::json!(["hasOwnProperty"])),
            None,
        ),
        ("const x: Promise<any> = Promise.resolve();", Some(serde_json::json!(["Promise"])), None),
    ];

    Tester::new(NoRestrictedGlobals::NAME, NoRestrictedGlobals::PLUGIN, pass, fail)
        .test_and_snapshot();
}
