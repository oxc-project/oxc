use super::NoShadow;
use crate::rule::RuleMeta;
use crate::tester::Tester;

#[test]
fn test() {
    let pass = vec![
        // Different names - no shadowing
        ("var x = 1; function foo() { var y = 2; }", None),
        // Same name in different functions - no shadowing
        ("function foo(x) { } function bar(x) { }", None),
        // Type vs value with same name (TypeScript) - default ignored
        ("type Foo = string; const Foo = 'bar';", None),
        // Interface vs value with same name
        ("interface Foo { x: number } const Foo = { x: 1 };", None),
        // Type parameter shadowing value (default ignored)
        ("const T = 1; function foo<T>() { }", None),
        // Enum member doesn't shadow
        ("const Red = 1; enum Color { Red, Green, Blue }", None),
        // Class with same name as type (declaration merging)
        ("interface Foo { x: number } class Foo { x = 1; }", None),
        // Namespace with same name as class
        ("class Foo { } namespace Foo { export const x = 1; }", None),
        // Import used only as type, value with same name
        ("import { Foo } from './foo'; type X = Foo; const Foo = 1;", None),
        // Allowed names
        ("var x = 1; function foo() { var x = 2; }", Some(serde_json::json!([{ "allow": ["x"] }]))),
        // Reassign
        ("let x = true; if (x) { x = false; }", Some(serde_json::json!([{ "allow": ["x"] }]))),
        // --- hoist = never: do NOT report if the outer declaration happens later ---
        (
            "function f() { { let x = 1; } let x = 2; }",
            Some(serde_json::json!([{ "hoist": "never" }])),
        ),
        // hoist = never: even if the outer is a function declaration, "never" should NOT report when it appears later
        (
            "function f() { { let x = 1; } function x() {} }",
            Some(serde_json::json!([{ "hoist": "never" }])),
        ),
        // --- hoist = functions: do NOT report if the outer declaration happens later and it is NOT a function declaration ---
        (
            "function f() { { let x = 1; } var x = 2; }",
            Some(serde_json::json!([{ "hoist": "functions" }])),
        ),
        (
            "function f() { { let C = 1; } class C {} }",
            Some(serde_json::json!([{ "hoist": "functions" }])),
        ),
        // --- allow: should suppress the diagnostic regardless of hoist setting ---
        (
            "function f() { { let x = 1; } let x = 2; }",
            Some(serde_json::json!([{ "hoist": "all", "allow": ["x"] }])),
        ),
        // allow multiple names
        (
            "let x = 1; function f(){ let x = 2; } let y = 1; function g(){ let y = 2; }",
            Some(serde_json::json!([{ "allow": ["x", "y"] }])),
        ),
        // allow applied to destructuring (you already have the failing version; this ensures the escape hatch works)
        ("const x = 1; { const { x } = { x: 2 }; }", Some(serde_json::json!([{ "allow": ["x"] }]))),
        // Outer is NOT a function declaration; it's a const variable initialized with a function expression.
        (
            "function f() { { let x = 1; } const x = function() {}; }",
            Some(serde_json::json!([{ "hoist": "functions" }])),
        ),
        // -------------------------
        // TypeScript specific pass cases
        // -------------------------
        // `allow` should also override type/value behavior even when ignoreTypeValueShadow=false.
        (
            "type Foo = string; { const Foo = 'bar'; }",
            Some(serde_json::json!([{
                "ignoreTypeValueShadow": false,
                "allow": ["Foo"]
            }])),
        ),
        // If ignoreTypeValueShadow=false, this would normally be reportable...
        // ...but with ignoreFunctionTypeParameterNameValueShadow=true it must be ignored (pass).
        (
            "const T = 1; function foo<T>() { }",
            Some(serde_json::json!([{
                "ignoreTypeValueShadow": false,
                "ignoreFunctionTypeParameterNameValueShadow": true
            }])),
        ),
    ];

    let fail = vec![
        // Basic shadowing
        ("var x = 1; function foo() { var x = 2; }", None),
        // Block scope shadowing
        ("const x = 1; { const x = 2; }", None),
        // Parameter shadowing outer variable
        ("var x = 1; function foo(x) { }", None),
        // Nested function shadowing
        ("function foo() { var x = 1; function bar() { var x = 2; } }", None),
        // Arrow function shadowing
        ("const x = 1; const foo = () => { const x = 2; };", None),
        // Class method shadowing
        ("const x = 1; class Foo { method() { const x = 2; } }", None),
        // Let shadowing
        ("let x = 1; { let x = 2; }", None),
        // Catch clause shadowing
        ("const e = 1; try { } catch (e) { }", None),
        // For loop variable shadowing
        ("const i = 1; for (let i = 0; i < 10; i++) { }", None),
        // Destructuring shadowing in nested scope
        ("const x = 1; { const { x } = { x: 2 }; }", None),
        // Array destructuring shadowing in nested scope
        ("const x = 1; { const [x] = [2]; }", None),
        ("let x = 1; { { let x = 3; } let x = 2; }", None),
        // Type shadowing type (not ignored)
        ("type Foo = string; { type Foo = number; }", None),
        // Interface shadowing interface
        ("interface Foo { x: number } { interface Foo { y: string } }", None),
        // --- hoist = all: DO report even if the outer declaration happens later ---
        (
            "function f() { { let x = 1; } let x = 2; }",
            Some(serde_json::json!([{ "hoist": "all" }])),
        ),
        // --- hoist = functions: DO report if the shadowed symbol is a function declaration even when it appears later ---
        (
            "function f() { { let x = 1; } function x() {} }",
            Some(serde_json::json!([{ "hoist": "functions" }])),
        ),
        // hoist = all: should also report when the outer is var and appears later (generalized hoisting behavior)
        (
            "function f() { { let x = 1; } var x = 2; }",
            Some(serde_json::json!([{ "hoist": "all" }])),
        ),
        // hoist = never: should still report the normal case (outer first, inner later)
        (
            "function f() { let x = 2; { let x = 1; } }",
            Some(serde_json::json!([{ "hoist": "never" }])),
        ),
        // --- allow: allowing only "x" must NOT allow "y" ---
        ("let y = 1; function g(){ let y = 2; }", Some(serde_json::json!([{ "allow": ["x"] }]))),
        // allow is case-sensitive
        ("let x = 1; function f(){ let x = 2; }", Some(serde_json::json!([{ "allow": ["X"] }]))),
        // -------------------------
        // TypeScript specific fail cases
        // -------------------------
        // ignoreTypeValueShadow = false => type/value with the same name is now reportable
        (
            "type Foo = string; { const Foo = 'bar'; }",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
        ),
        (
            "interface Foo { x: number }; { const Foo = { x: 1 } };",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
        ),
        // ignoreFunctionTypeParameterNameValueShadow = false
        // (and ignoreTypeValueShadow=false to ensure this option is the one deciding)
        // Now the function type parameter shadowing a value should be reported.
        (
            "const T = 1; function foo<T>() { }",
            Some(serde_json::json!([{
                "ignoreTypeValueShadow": false,
                "ignoreFunctionTypeParameterNameValueShadow": false
            }])),
        ),
        // Enum shadowing Enum (Enums are values)
        ("enum E { A } function foo() { enum E { B } }", None),
        // Class shadowing Class (Classes are values)
        ("class C { } function foo() { class C { } }", None),
        // Import (Value) shadowing Value
        // The import is used as a value, so it should be shadowed
        ("import { Foo } from './foo'; const x = Foo; function bar() { const Foo = 1; }", None),
        // Generic Parameter shadowing Generic Parameter
        ("function foo<T>() { function bar<T>() { } }", None),
    ];

    Tester::new(NoShadow::NAME, NoShadow::PLUGIN, pass, fail).test_and_snapshot();
}
