use rustc_hash::FxHashSet;

use oxc_allocator::ArenaBox;
use oxc_ast::{
    AstKind, StaticPropertyName,
    ast::{
        BindingPattern, Expression, IdentifierReference, MemberExpression, ObjectPattern,
        match_member_expression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, SymbolId};
use oxc_span::Span;
use oxc_str::JSStr;

use crate::{context::LintContext, rule::Rule};

const GLOBAL_THIS: &str = "globalThis";

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
    recommended,
    version = "0.0.7",
    short_description = "Disallow calling some global objects as functions.",
);

fn is_global_obj(s: &str) -> bool {
    matches!(s, "Atomics" | "Intl" | "JSON" | "Math" | "Reflect")
}

fn is_global_this(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    let Expression::Identifier(ident) = expr.get_inner_expression() else { return false };
    ident.name == GLOBAL_THIS && ctx.is_reference_to_global_variable(ident)
}

fn global_this_member<'a>(
    expr: &'a MemberExpression<'_>,
    ctx: &LintContext<'_>,
) -> Option<&'a str> {
    if is_global_this(expr.object(), ctx) {
        expr.static_property_name().and_then(JSStr::as_str)
    } else {
        None
    }
}

fn global_this_destructured_property<'a>(
    pattern: &'a ObjectPattern<'a>,
    symbol_id: SymbolId,
) -> Option<&'a str> {
    let property = pattern.properties.iter().find(|property| {
        property.value.get_binding_identifier().is_some_and(|ident| ident.symbol_id() == symbol_id)
    })?;
    match property.key.static_name()? {
        StaticPropertyName::Borrowed(name) => name.as_str(),
        StaticPropertyName::Owned(_) => None,
    }
}

fn resolve_global_binding<'a>(
    ident: &'a ArenaBox<'a, IdentifierReference<'a>>,
    ctx: &LintContext<'a>,
) -> Option<&'a str> {
    let symbols = ctx.scoping();
    let mut ident = ident;
    let mut visited_symbols = FxHashSet::default();

    loop {
        let Some(symbol_id) = symbols.get_reference(ident.reference_id()).symbol_id() else {
            // Unresolved references are globals unless disabled through config.
            return ctx.is_reference_to_global_variable(ident).then_some(ident.name.as_str());
        };

        let decl = ctx.nodes().get_node(symbols.symbol_declaration(symbol_id));
        let AstKind::VariableDeclarator(decl) = decl.kind() else { return None };
        let init = decl.init.as_ref()?;

        match &decl.id {
            BindingPattern::BindingIdentifier(_) => match init {
                // handles "let a = JSON; let b = a; a();"
                Expression::Identifier(init_ident) => {
                    if !visited_symbols.insert(symbol_id) {
                        return None;
                    }
                    ident = init_ident;
                }
                // handles "let a = globalThis.JSON; let b = a; a();"
                _ if init.is_member_expression() => {
                    return global_this_member(init.to_member_expression(), ctx);
                }
                _ => return None,
            },
            // handles "const { JSON: j } = globalThis; j();"
            BindingPattern::ObjectPattern(pattern) if is_global_this(init, ctx) => {
                return global_this_destructured_property(pattern, symbol_id);
            }
            _ => return None,
        }
    }
}

impl Rule for NoObjCalls {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(expr) => check_callee(&expr.callee, expr.span, ctx),
            AstKind::CallExpression(expr) => check_callee(&expr.callee, expr.span, ctx),
            _ => {}
        }
    }
}

fn check_callee<'a>(callee: &'a Expression, span: Span, ctx: &LintContext<'a>) {
    match callee {
        Expression::Identifier(ident) => {
            // handle new Math(), Math(), etc
            if let Some(top_level_reference) = resolve_global_binding(ident, ctx)
                && is_global_obj(top_level_reference)
            {
                ctx.diagnostic(no_obj_calls_diagnostic(ident.name.as_str(), span));
            }
        }

        match_member_expression!(Expression) => {
            // handle new globalThis.Math(), globalThis.Math(), etc
            if let Some(global_member) = global_this_member(callee.to_member_expression(), ctx)
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
        "const { Math } = { Math() {} }; Math();",
        "const { Math } = JSON; Math();",
        "const [Math] = [() => {}]; Math();",
        "for (const { Math } of items) Math();",
        "var a = b; var b = a; a = () => {}; a();",
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
        "const { Math } = globalThis; Math();",
        "const { Math: M } = globalThis; M();",
        "const { ['JSON']: J } = globalThis; new J();",
        "const { Reflect = () => {} } = globalThis; Reflect();",
    ];

    Tester::new(NoObjCalls::NAME, NoObjCalls::PLUGIN, pass, fail).test_and_snapshot();

    let pass = vec![("Math();", None, Some(serde_json::json!({ "globals": { "Math": "off" } })))];
    let fail = vec![("Math();", None, None)];
    Tester::new(NoObjCalls::NAME, NoObjCalls::PLUGIN, pass, fail).test();
}

#[test]
fn shadowed_global_this() {
    use crate::tester::Tester;

    let mut pass = vec![
        "const globalThis = { Math() {} }; globalThis.Math();".to_string(),
        "import globalThis from 'custom-globals'; globalThis.Math();".to_string(),
        "try {} catch (globalThis) { globalThis.Math(); }".to_string(),
    ];
    let mut fail = Vec::new();
    for name in ["Math", "JSON", "Reflect", "Atomics", "Intl"] {
        pass.push(format!("function f(globalThis) {{ globalThis.{name}(); }}"));
        pass.push(format!("function f(globalThis) {{ new globalThis.{name}(); }}"));
        pass.push(format!("function f(globalThis) {{ globalThis['{name}'](); }}"));
        pass.push(format!("function f(globalThis) {{ globalThis?.{name}(); }}"));
        pass.push(format!(
            "function f(globalThis) {{ const alias = globalThis.{name}; alias(); }}"
        ));
        pass.push(format!(
            "function f(globalThis) {{ const alias = globalThis.{name}; new alias(); }}"
        ));
        pass.push(format!(
            "function f(globalThis) {{ const {{ {name}: alias }} = globalThis; alias(); }}"
        ));
        fail.push(format!("globalThis.{name}();"));
        fail.push(format!("new globalThis.{name}();"));
        fail.push(format!("const alias = globalThis.{name}; alias();"));
        fail.push(format!("const {{ {name}: alias }} = globalThis; alias();"));
    }
    Tester::new(NoObjCalls::NAME, NoObjCalls::PLUGIN, pass, fail).test();
}

#[test]
fn wrapped_global_this() {
    use crate::tester::Tester;

    let mut pass = Vec::new();
    let mut fail = Vec::new();
    for object in [
        "(globalThis)",
        "(globalThis as any)",
        "(globalThis satisfies any)",
        "globalThis!",
        "(globalThis<string>)",
    ] {
        for call in [
            format!("{object}.Math();"),
            format!("new {object}.Math();"),
            format!("const alias = {object}.Math; alias();"),
        ] {
            pass.push(format!("function f(globalThis: any) {{ {call} }}"));
            fail.push(call);
        }
    }
    Tester::new(NoObjCalls::NAME, NoObjCalls::PLUGIN, pass, fail).test();
}
