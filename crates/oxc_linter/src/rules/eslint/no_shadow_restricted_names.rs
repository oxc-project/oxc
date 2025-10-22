use crate::{context::LintContext, rule::Rule};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde_json::Value;

const PRE_DEFINE_VAR: [&str; 5] = ["Infinity", "NaN", "arguments", "eval", "undefined"];

fn no_shadow_restricted_names_diagnostic(shadowed_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Shadowing of global properties such as 'undefined' is not allowed.")
        .with_help(format!("Rename '{shadowed_name}' to avoid shadowing the global property."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoShadowRestrictedNames(Box<NoShadowRestrictedNamesConfig>);

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoShadowRestrictedNamesConfig {
    /// If true, also report shadowing of `globalThis`.
    report_global_this: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the redefining of global variables such as `undefined`, `NaN`, `Infinity`,
    /// `eval`, and `arguments`.
    ///
    /// ### Why is this bad?
    ///
    /// Value properties of the Global Object `NaN`, `Infinity`, `undefined` as well as the strict
    /// mode restricted identifiers `eval` and `arguments` are considered to be restricted names in
    /// JavaScript. Defining them to mean something else can have unintended consequences and
    /// confuse others reading the code. For example, there’s nothing preventing you from
    /// writing:
    ///
    /// ```javascript
    /// var undefined = "foo";
    /// ```
    ///
    /// Then any code used within the same scope would not get the global undefined, but rather the
    /// local version with a very different meaning.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function NaN(){}
    ///
    /// !function(Infinity){};
    ///
    /// var undefined = 5;
    ///
    /// try {} catch(eval){}
    /// ```
    ///
    /// ```javascript
    /// import NaN from "foo";
    ///
    /// import { undefined } from "bar";
    ///
    /// class Infinity {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var Object;
    ///
    /// function f(a, b){}
    ///
    /// // Exception: `undefined` may be shadowed if the variable is never assigned a value.
    /// var undefined;
    /// ```
    ///
    /// ```javascript
    /// import { undefined as undef } from "bar";
    /// ```
    NoShadowRestrictedNames,
    eslint,
    correctness,
    config = NoShadowRestrictedNamesConfig
);

impl Rule for NoShadowRestrictedNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(NoShadowRestrictedNamesConfig {
            report_global_this: value
                .get(0)
                .and_then(|x| x.get("reportGlobalThis"))
                .and_then(Value::as_bool)
                .unwrap_or_default(),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        for symbol_id in ctx.scoping().symbol_ids() {
            let name = ctx.scoping().symbol_name(symbol_id);

            if !(PRE_DEFINE_VAR.contains(&name)
                || self.0.report_global_this && name == "globalThis")
            {
                continue;
            }

            if name == "undefined" {
                // Allow to declare `undefined` variable but not allow to assign value to it.
                let node_id = ctx.scoping().symbol_declaration(symbol_id);
                if let AstKind::VariableDeclarator(declarator) = ctx.nodes().kind(node_id)
                    && declarator.init.is_none()
                    && ctx
                        .scoping()
                        .get_resolved_references(symbol_id)
                        .all(|reference| !reference.is_write())
                {
                    continue;
                }
            }

            let redeclarations = ctx.scoping().symbol_redeclarations(symbol_id);
            if redeclarations.is_empty() {
                let span = ctx.scoping().symbol_span(symbol_id);
                ctx.diagnostic(no_shadow_restricted_names_diagnostic(name, span));
            } else {
                for rd in redeclarations {
                    ctx.diagnostic(no_shadow_restricted_names_diagnostic(name, rd.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("function foo(bar){ var baz; }", None),
        ("!function foo(bar){ var baz; }", None),
        ("!function(bar){ var baz; }", None),
        ("try {} catch(e) {}", None),
        ("export default function() {}", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("try {} catch {}", None),              // { "ecmaVersion": 2019 },
        ("var undefined;", None),
        ("var undefined; doSomething(undefined);", None),
        ("var undefined; var undefined;", None),
        ("let undefined", None), // { "ecmaVersion": 2015 },
        ("import { undefined as undef } from 'foo';", None), // { "sourceType": "module", "ecmaVersion": 2015, },
        ("let globalThis;", None),                           // { "ecmaVersion": 2020 },
        ("class globalThis {}", None),                       // { "ecmaVersion": 2020 },
        ("import { baz as globalThis } from 'foo';", None), // { "ecmaVersion": 2020, "sourceType": "module", },
        ("globalThis.foo", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        ("const foo = globalThis", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        ("function foo() { return globalThis; }", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        ("import { globalThis as foo } from 'bar'", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020, "sourceType": "module" }
    ];

    let fail = vec![
        ("var undefined = 5;", None),
        ("function NaN(){}", None),
        ("try {} catch(eval){}", None),
        ("function NaN(NaN) { var NaN; !function NaN(NaN) { try {} catch(NaN) {} }; }", None),
        (
            "function undefined(undefined) { !function undefined(undefined) { try {} catch(undefined) {} }; }",
            None,
        ),
        (
            "function Infinity(Infinity) { var Infinity; !function Infinity(Infinity) { try {} catch(Infinity) {} }; }",
            None,
        ),
        (
            "function arguments(arguments) { var arguments; !function arguments(arguments) { try {} catch(arguments) {} }; }",
            None,
        ),
        ("function eval(eval) { var eval; !function eval(eval) { try {} catch(eval) {} }; }", None),
        (
            "var eval = (eval) => { var eval; !function eval(eval) { try {} catch(eval) {} }; }",
            None,
        ), // { "ecmaVersion": 6 },
        ("var [undefined] = [1]", None), // { "ecmaVersion": 6 },
        (
            "var {undefined} = obj; var {a: undefined} = obj; var {a: {b: {undefined}}} = obj; var {a, ...undefined} = obj;",
            None,
        ), // { "ecmaVersion": 9 },
        ("var undefined; undefined = 5;", None),
        ("class undefined {}", None),   // {				"ecmaVersion": 2015,			},
        ("(class undefined {})", None), // {				"ecmaVersion": 2015,			},
        ("import undefined from 'foo';", None), // {				"ecmaVersion": 2015,				"sourceType": "module",			},
        ("import { undefined } from 'foo';", None), // {				"ecmaVersion": 2015,				"sourceType": "module",			},
        ("import { baz as undefined } from 'foo';", None), // {				"ecmaVersion": 2015,				"sourceType": "module",			},
        ("import * as undefined from 'foo';", None), // {				"ecmaVersion": 2015,				"sourceType": "module",			},
        (
            "function globalThis(globalThis) { var globalThis; !function globalThis(globalThis) { try {} catch(globalThis) {} }; }",
            Some(json!([{ "reportGlobalThis": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function globalThis(globalThis) { var globalThis; !function globalThis(globalThis) { try {} catch(globalThis) {} }; }",
            Some(json!([{ "reportGlobalThis": true }])),
        ), // { "ecmaVersion": 2020 },
        ("const [globalThis] = [1]", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        (
            "var {globalThis} = obj; var {a: globalThis} = obj; var {a: {b: {globalThis}}} = obj; var {a, ...globalThis} = obj;",
            Some(json!([{ "reportGlobalThis": true }])),
        ), // { "ecmaVersion": 2020 },
        ("let globalThis; globalThis = 5;", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        ("class globalThis {}", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        ("(class globalThis {})", Some(json!([{ "reportGlobalThis": true }]))), // { "ecmaVersion": 2020 },
        ("import globalThis from 'foo';", Some(json!([{ "reportGlobalThis": true }]))), // {				"ecmaVersion": 2020,				"sourceType": "module",			},
        ("import { globalThis } from 'foo';", Some(json!([{ "reportGlobalThis": true }]))), // {				"ecmaVersion": 2020,				"sourceType": "module",			},
        ("import { baz as globalThis } from 'foo';", Some(json!([{ "reportGlobalThis": true }]))), // {				"ecmaVersion": 2020,				"sourceType": "module",			},
        ("import * as globalThis from 'foo';", Some(json!([{ "reportGlobalThis": true }]))), // {				"ecmaVersion": 2020,				"sourceType": "module",			}
    ];

    Tester::new(NoShadowRestrictedNames::NAME, NoShadowRestrictedNames::PLUGIN, pass, fail)
        .test_and_snapshot();
}
