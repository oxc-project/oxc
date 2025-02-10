use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

use crate::{context::LintContext, globals::PRE_DEFINE_VAR, rule::Rule};

fn no_shadow_restricted_names_diagnostic(shadowed_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Shadowing of global properties such as 'undefined' is not allowed.")
        .with_help(format!("Shadowing of global properties '{shadowed_name}'."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoShadowRestrictedNames;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow redefine the global variables like 'undefined', 'NaN', 'Infinity', 'eval', 'arguments'.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// function NaN(){}
    ///
    /// !function(Infinity){};
    ///
    /// var undefined = 5;
    ///
    /// try {} catch(eval){}
    /// ```
    NoShadowRestrictedNames,
    eslint,
    correctness
);

impl Rule for NoShadowRestrictedNames {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let name = ctx.symbols().get_name(symbol_id);

        if !PRE_DEFINE_VAR.contains_key(name) {
            return;
        }

        if name == "undefined" {
            // Allow to declare `undefined` variable but not allow to assign value to it.
            let node_id = ctx.semantic().symbols().get_declaration(symbol_id);
            if let AstKind::VariableDeclarator(declarator) = ctx.nodes().kind(node_id) {
                if declarator.init.is_none()
                    && ctx
                        .symbols()
                        .get_resolved_references(symbol_id)
                        .all(|reference| !reference.is_write())
                {
                    return;
                }
            }
        }

        let span = ctx.symbols().get_span(symbol_id);
        ctx.diagnostic(no_shadow_restricted_names_diagnostic(name, span));

        for &span in ctx.symbols().get_redeclarations(symbol_id) {
            ctx.diagnostic(no_shadow_restricted_names_diagnostic(name, span));
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
        ("try {} catch(e: undefined) {}", None),
        (
            "export default function() {}",
            Some(json!({
                "parserOptions": {
                    "ecmaVersion": 6,
                    "sourceType": "module"
                }
            })),
        ),
        (
            "try {} catch {}",
            Some(json!({
                "parserOptions": { "ecmaVersion": 2019 }
            })),
        ),
        ("var undefined;", None),
        ("var undefined;var undefined", None),
        (
            "let undefined",
            Some(json!({
                "parserOptions": { "ecmaVersion": 2015 }
            })),
        ),
        ("var normal, undefined;", None),
        ("var undefined; doSomething(undefined);", None),
        ("class foo { undefined() { } }", None),
        (
            "class foo { #undefined() { } }",
            Some(json!({
                "parserOptions": { "ecmaVersion": 2019 }
            })),
        ),
        ("var normal, undefined; var undefined;", None),
    ];

    let fail = vec![
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
            Some(json!({
                "parserOptions": {
                    "ecmaVersion": 6
                }
            })),
        ),
        (
            "var {undefined} = obj; var {a: undefined} = obj; var {a: {b: {undefined}}} = obj; var {a, ...undefined} = obj;",
            Some(json!({
                "parserOptions": {
                    "ecmaVersion": 9
                }
            })),
        ),
        ("var normal, undefined; undefined = 5;", None),
        ("try {} catch(undefined: undefined) {}", None),
        (
            "var [undefined] = [1]",
            Some(json!({
                "parserOptions": {
                    "ecmaVersion": 6
                }
            })),
        ),
        ("class undefined { }", None),
        ("class foo { undefined(undefined) { } }", None),
        ("class foo { #undefined(undefined) { } }", None),
    ];

    Tester::new(NoShadowRestrictedNames::NAME, NoShadowRestrictedNames::PLUGIN, pass, fail)
        .test_and_snapshot();
}
