use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, MemberExpression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ScopeId};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

const GLOBAL_THIS: &str = "globalThis";
const NON_CALLABLE_GLOBALS: [&str; 5] = ["Atomics", "Intl", "JSON", "Math", "Reflect"];

fn no_obj_calls_diagnostic(obj_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{obj_name}` is not a function and cannot be called"))
        .with_help("This call will throw a TypeError at runtime.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoObjCalls;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow calling some global objects as functions.
    ///
    /// This rule can be disabled for TypeScript code, as the TypeScript compiler
    /// enforces this check.
    ///
    /// ### Why is this bad?
    ///
    /// Some global objects are not intended to be called as functions.
    /// Calling them as functions will usually result in a TypeError being thrown.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let math = Math();
    /// let newMath = new Math();
    ///
    /// let json = JSON();
    /// let newJson = new JSON();
    ///
    /// let atomics = Atomics();
    /// let newAtomics = new Atomics();
    ///
    /// let intl = Intl();
    /// let newIntl = new Intl();
    ///
    /// let reflect = Reflect();
    /// let newReflect = new Reflect();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let area = r => 2 * Math.PI * r * r;
    /// let object = JSON.parse("{}");
    /// let first = Atomics.load(sharedArray, 0);
    /// let segmenterFrom = Intl.Segmenter("fr", { granularity: "word" });
    /// ```
    NoObjCalls,
    eslint,
    correctness,
);

fn is_global_obj(s: &str) -> bool {
    NON_CALLABLE_GLOBALS.contains(&s)
}

fn global_this_member<'a>(expr: &'a MemberExpression<'_>) -> Option<&'a str> {
    if expr.object().is_specific_id(GLOBAL_THIS) { expr.static_property_name() } else { None }
}

fn resolve_global_binding<'a, 'b: 'a>(
    ident: &'a oxc_allocator::Box<'a, IdentifierReference<'a>>,
    scope_id: ScopeId,
    ctx: &LintContext<'a>,
) -> Option<&'a str> {
    let scope = ctx.scoping();
    let nodes = ctx.nodes();
    let symbols = ctx.scoping();

    if ctx.is_reference_to_global_variable(ident) {
        return Some(ident.name.as_str());
    }

    let Some(binding_id) = scope.find_binding(scope_id, ident.name) else {
        // Panic in debug builds, but fail gracefully in release builds.
        debug_assert!(
            false,
            "No binding id found for {}, but this IdentifierReference
                is not a global",
            &ident.name
        );
        return None;
    };

    let decl = nodes.get_node(symbols.symbol_declaration(binding_id));
    match decl.kind() {
        AstKind::VariableDeclarator(parent_decl) => {
            if !parent_decl.id.is_binding_identifier() {
                return Some(ident.name.as_str());
            }
            match &parent_decl.init {
                // handles "let a = JSON; let b = a; a();"
                Some(Expression::Identifier(parent_ident)) if parent_ident.name != ident.name => {
                    let decl_scope = decl.scope_id();
                    resolve_global_binding(parent_ident, decl_scope, ctx)
                }
                // handles "let a = globalThis.JSON; let b = a; a();"
                Some(parent_expr) if parent_expr.is_member_expression() => {
                    global_this_member(parent_expr.to_member_expression())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

impl Rule for NoObjCalls {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(expr) => check_callee(&expr.callee, expr.span, node, ctx),
            AstKind::CallExpression(expr) => check_callee(&expr.callee, expr.span, node, ctx),
            _ => {}
        }
    }
}

fn check_callee<'a>(callee: &'a Expression, span: Span, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    match callee {
        Expression::Identifier(ident) => {
            // handle new Math(), Math(), etc
            if let Some(top_level_reference) = resolve_global_binding(ident, node.scope_id(), ctx)
                && is_global_obj(top_level_reference)
            {
                ctx.diagnostic(no_obj_calls_diagnostic(ident.name.as_str(), span));
            }
        }

        match_member_expression!(Expression) => {
            // handle new globalThis.Math(), globalThis.Math(), etc
            if let Some(global_member) = global_this_member(callee.to_member_expression())
                && is_global_obj(global_member)
            {
                ctx.diagnostic(no_obj_calls_diagnostic(global_member, span));
            }
        }
        _ => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var x = Math;",
        "var x = Math.random();",
        "var x = Math.PI;",
        "var x = foo.Math();",
        "var x = new foo.Math();",
        "var x = new Math.foo;",
        "var x = new Math.foo();",
        "JSON.parse(foo)",
        "new JSON.parse",
        "Reflect.get(foo, 'x')", // { "ecmaVersion": 6 },
        "new Reflect.foo(a, b)", // { "ecmaVersion": 6 },
        "Atomics.load(foo, 0)",  // { "ecmaVersion": 2017 },
        "new Atomics.foo()",     // { "ecmaVersion": 2017 },
        "new Intl.Segmenter()",  // { "ecmaVersion": 2015 },
        "Intl.foo()",            // { "ecmaVersion": 2015 },
        // These only pass in ESLint because they're relying on older versions of ES, which we do not support.
        // "globalThis.Math();",         // { "ecmaVersion": 6 },
        // "var x = globalThis.Math();", // { "ecmaVersion": 6 },
        // "f(globalThis.Math());",                 // { "ecmaVersion": 6 },
        // "globalThis.Math().foo;",                // { "ecmaVersion": 6 },
        // "var x = globalThis.JSON();",            // { "ecmaVersion": 6 },
        // "x = globalThis.JSON(str);",             // { "ecmaVersion": 6 },
        // "globalThis.Math( globalThis.JSON() );", // { "ecmaVersion": 6 },
        // "var x = globalThis.Reflect();",         // { "ecmaVersion": 6 },
        // "var x = globalThis.Reflect();",         // { "ecmaVersion": 2017 },
        // "/*globals Reflect: true*/ globalThis.Reflect();", // { "ecmaVersion": 2017 }, // We do not support inline globals.
        // "var x = globalThis.Atomics();", // { "ecmaVersion": 2017 },
        // "var x = globalThis.Atomics();", // { "ecmaVersion": 2017, "globals": { "Atomics": false } },
        // "var x = globalThis.Intl();",    // { "ecmaVersion": 2015 },
        // non-existing variables
        // "/*globals Math: off*/ Math();", // We do not support inline globals.
        // "/*globals Math: off*/ new Math();", // We do not support inline globals.
        // We don't support these cases because they depend on configuring globals or defaulting to a very old version of the ES spec, so they won't pass in Oxlint:
        // "JSON();",     // { "globals": { "JSON": "off" }, },
        // "new JSON();", // { "globals": { "JSON": "off" }, },
        // "Reflect();",
        // "Atomics();",
        // "new Reflect();",
        // "new Atomics();",
        // "Atomics();", // { "ecmaVersion": 6 },
        // "Intl()",
        // "new Intl()",

        // Shadowing
        "var Math; Math();",
        "var Math; new Math();",
        "let JSON; JSON();",                          // { "ecmaVersion": 2015 },
        "let JSON; new JSON();",                      // { "ecmaVersion": 2015 },
        "if (foo) { const Reflect = 1; Reflect(); }", // { "ecmaVersion": 6 },
        "if (foo) { const Reflect = 1; new Reflect(); }", // { "ecmaVersion": 6 },
        "function foo(Math) { Math(); }",
        "function foo(JSON) { new JSON(); }",
        "function foo(Atomics) { Atomics(); }", // { "ecmaVersion": 2017 },
        "function foo() { if (bar) { let Atomics; if (baz) { new Atomics(); } } }", // { "ecmaVersion": 2017 },
        "function foo() { var JSON; JSON(); }",
        "function foo() { var Atomics = bar(); var baz = Atomics(5); }", // { "globals": { "Atomics": false } },
        r#"var construct = typeof Reflect !== "undefined" ? Reflect.construct : undefined; construct();"#, // { "globals": { "Reflect": false } },
        "function foo(Intl) { Intl(); }", // { "ecmaVersion": 2015 },
        "if (foo) { const Intl = 1; Intl(); }", // { "ecmaVersion": 2015 },
        "if (foo) { const Intl = 1; new Intl(); }", // { "ecmaVersion": 2015 }
        // https://github.com/oxc-project/oxc/pull/508#issuecomment-1618850742
        "{const Math = () => {}; {let obj = new Math();}}",
        "{const {parse} = JSON;parse('{}')}",
        // https://github.com/oxc-project/oxc/issues/4389
        r"export const getConfig = getConfig;
        getConfig();",
        // reference test cases
        "let j = JSON;
        function foo() {
            let j = x => x;
            return x();
        }",
    ];

    let fail = vec![
        "Math();",
        "var x = Math();",
        "f(Math());",
        "Math().foo;",
        "new Math;",
        "new Math();",
        "new Math(foo);",
        "new Math().foo;",
        "(new Math).foo();",
        "var x = JSON();",
        "x = JSON(str);",
        "var x = new JSON();",
        "Math( JSON() );",
        "var x = Reflect();",     // { "ecmaVersion": 6 },
        "var x = new Reflect();", // { "ecmaVersion": 6 },
        "var x = Reflect();",     // { "ecmaVersion": 2017 },
        // "/*globals Reflect: true*/ Reflect();", // We do not support inline globals.
        // "/*globals Reflect: true*/ new Reflect();", // We do not support inline globals.
        "var x = Atomics();",     // { "ecmaVersion": 2017 },
        "var x = new Atomics();", // { "ecmaVersion": 2017 },
        "var x = Atomics();",     // { "ecmaVersion": 2020 },
        "var x = Atomics();",     // { "globals": { "Atomics": false } },
        "var x = new Atomics();", // { "globals": { "Atomics": "writable" } },
        "var x = Intl();",        // { "ecmaVersion": 2015 },
        "var x = new Intl();",    // { "ecmaVersion": 2015 },
        // "/*globals Intl: true*/ Intl();", // We do not support inline globals.
        // "/*globals Intl: true*/ new Intl();", // We do not support inline globals.
        "var x = globalThis.Math();",     // { "ecmaVersion": 2020 },
        "var x = new globalThis.Math();", // { "ecmaVersion": 2020 },
        "f(globalThis.Math());",          // { "ecmaVersion": 2020 },
        "globalThis.Math().foo;",         // { "ecmaVersion": 2020 },
        "new globalThis.Math().foo;",     // { "ecmaVersion": 2020 },
        "var x = globalThis.JSON();",     // { "ecmaVersion": 2020 },
        "x = globalThis.JSON(str);",      // { "ecmaVersion": 2020 },
        "globalThis.Math( globalThis.JSON() );", // { "ecmaVersion": 2020 },
        "var x = globalThis.Reflect();",  // { "ecmaVersion": 2020 },
        "var x = new globalThis.Reflect;", // { "ecmaVersion": 2020 },
        // "/*globals Reflect: true*/ Reflect();", // { "ecmaVersion": 2020 }, // We do not support inline globals.
        "var x = globalThis.Atomics();", // { "ecmaVersion": 2020 },
        "var x = globalThis.Intl();",    // { "ecmaVersion": 2020 },
        "var x = new globalThis.Intl;",  // { "ecmaVersion": 2020 },
        // "/*globals Intl: true*/ Intl();", // { "ecmaVersion": 2020 }, // We do not support inline globals.
        // TODO: Fix these.
        // "var foo = bar ? baz: JSON; foo();",
        // "var foo = bar ? baz: JSON; new foo();",
        // "var foo = bar ? baz: globalThis.JSON; foo();", // { "ecmaVersion": 2020 },
        // "var foo = bar ? baz: globalThis.JSON; new foo();", // { "ecmaVersion": 2020 },
        // "var foo = window.Atomics; foo();", // { "ecmaVersion": 2020, "globals": globals.browser },
        // "var foo = window.Atomics; new foo;", // { "ecmaVersion": 2020, "globals": globals.browser },
        // "var foo = window.Intl; foo();", // { "ecmaVersion": 2020, "globals": globals.browser },
        // "var foo = window.Intl; new foo;", // { "ecmaVersion": 2020, "globals": globals.browser },
        "var x = globalThis?.Reflect();", // { "ecmaVersion": 2020 },
        // TODO: Fix.
        // "var x = (globalThis?.Reflect)();", // { "ecmaVersion": 2020 }
        // reference test cases
        "let j = JSON; j();",
        "let a = JSON; let b = a; let c = b; b();",
        "let m = globalThis.Math; new m();",
    ];

    Tester::new(NoObjCalls::NAME, NoObjCalls::PLUGIN, pass, fail).test_and_snapshot();
}
