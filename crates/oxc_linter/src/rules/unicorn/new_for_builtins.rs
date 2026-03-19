use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, context::LintContext, globals::GLOBAL_OBJECT_NAMES, rule::Rule};

fn enforce(span: Span, fn_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `new {fn_name}()` instead of `{fn_name}()`")).with_label(span)
}

fn disallow(span: Span, fn_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `{fn_name}()` instead of `new {fn_name}()`")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NewForBuiltins;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `new` for the following builtins: `Object`, `Array`, `ArrayBuffer`, `BigInt64Array`,
    /// `BigUint64Array`, `DataView`, `Date`, `Error`, `Float32Array`, `Float64Array`, `Function`, `Int8Array`,
    /// `Int16Array`, `Int32Array`, `Map`, `WeakMap`, `Set`, `WeakSet`, `Promise`, `RegExp`, `Uint8Array`,
    /// `Uint16Array`, `Uint32Array`, `Uint8ClampedArray`, `SharedArrayBuffer`, `Proxy`, `WeakRef`, `FinalizationRegistry`.
    ///
    /// Disallows the use of `new` for the following builtins: `String`, `Number`, `Boolean`, `Symbol`, `BigInt`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `new` inconsistently can cause confusion. Constructors like `Array` and `RegExp` should always use `new`
    /// to ensure the expected instance type. Meanwhile, `String`, `Number`, `Boolean`, `Symbol`, and `BigInt` should not use `new`,
    /// as they create object wrappers instead of primitive values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = new String('hello world');
    /// const bar = Array(1, 2, 3);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = String('hello world');
    /// const bar = new Array(1, 2, 3);
    /// ```
    NewForBuiltins,
    unicorn,
    pedantic,
    pending
);

impl Rule for NewForBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let Some(builtin_name) = is_expr_global_builtin(&new_expr.callee, ctx) else {
                    return;
                };

                if DISALLOW_NEW_FOR_BUILTINS.contains(&builtin_name) {
                    ctx.diagnostic(disallow(new_expr.span, builtin_name));
                }
            }
            AstKind::CallExpression(call_expr) => {
                let Some(builtin_name) = is_expr_global_builtin(&call_expr.callee, ctx) else {
                    return;
                };

                if ENFORCE_NEW_FOR_BUILTINS.contains(builtin_name) {
                    if builtin_name == "Object" {
                        let parent_kind = ctx.nodes().parent_kind(node.id());
                        if let AstKind::BinaryExpression(bin_expr) = parent_kind
                            && (bin_expr.operator == BinaryOperator::StrictEquality
                                || bin_expr.operator == BinaryOperator::StrictInequality)
                        {
                            return;
                        }
                    }

                    ctx.diagnostic(enforce(call_expr.span, builtin_name));
                }
            }
            _ => {}
        }
    }
}

fn is_expr_global_builtin<'a, 'b>(
    expr: &'b Expression<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b str> {
    let expr = expr.without_parentheses();
    if let Expression::Identifier(ident) = expr {
        let name = ident.name.as_str();
        if !ctx.scoping().root_unresolved_references().contains_key(name) {
            return None;
        }

        Some(name)
    } else {
        let member_expr = expr.as_member_expression()?;

        let Expression::Identifier(ident) = member_expr.object() else {
            return None;
        };

        if !GLOBAL_OBJECT_NAMES.contains(&ident.name.as_str()) {
            return None;
        }

        member_expr.static_property_name()
    }
}

const ENFORCE_NEW_FOR_BUILTINS: phf::Set<&'static str> = phf::phf_set![
    "Array",
    "ArrayBuffer",
    "BigInt64Array",
    "BigUint64Array",
    "DataView",
    "Date",
    "Error",
    "FinalizationRegistry",
    "Float32Array",
    "Float64Array",
    "Function",
    "Int16Array",
    "Int32Array",
    "Int8Array",
    "Map",
    "Object",
    "Promise",
    "Proxy",
    "RegExp",
    "Set",
    "SharedArrayBuffer",
    "Uint16Array",
    "Uint32Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "WeakMap",
    "WeakRef",
    "WeakSet",
];

const DISALLOW_NEW_FOR_BUILTINS: [&str; 5] = ["BigInt", "Boolean", "Number", "Symbol", "String"];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = new Object()",
        "const foo = new Array()",
        "const foo = new ArrayBuffer()",
        "const foo = new BigInt64Array()",
        "const foo = new BigUint64Array()",
        "const foo = new DataView()",
        "const foo = new Error()",
        "const foo = new Float16Array()",
        "const foo = new Float32Array()",
        "const foo = new Float64Array()",
        "const foo = new Function()",
        "const foo = new Int8Array()",
        "const foo = new Int16Array()",
        "const foo = new Int32Array()",
        "const foo = new Map()",
        "const foo = new Map([['foo', 'bar'], ['unicorn', 'rainbow']])",
        "const foo = new WeakMap()",
        "const foo = new Set()",
        "const foo = new WeakSet()",
        "const foo = new Promise()",
        "const foo = new RegExp()",
        "const foo = new UInt8Array()",
        "const foo = new UInt16Array()",
        "const foo = new UInt32Array()",
        "const foo = new Uint8ClampedArray()",
        "const foo = BigInt()",
        "const foo = Boolean()",
        "const foo = Number()",
        "const foo = String()",
        "const foo = Symbol()",
        "
                        import { Map } from 'immutable';
                        const m = Map();
                    ",
        "
                        const {Map} = require('immutable');
                        const foo = Map();
                    ",
        "
                        const {String} = require('guitar');
                        const lowE = new String();
                    ",
        "
                        import {String} from 'guitar';
                        const lowE = new String();
                    ",
        "new Foo();Bar();",
        "Foo();new Bar();",
        "const isObject = v => Object(v) === v;",
        "const isObject = v => globalThis.Object(v) === v;",
        "(x) !== Object(x)",
        // r#"new Symbol("")"#, // {"globals": {"Symbol": "off"}},
        "const foo = new Date();",
    ];

    let fail = vec![
        "const object = (Object)();",
        r#"const symbol = new (Symbol)("");"#,
        r#"const symbol = new /* comment */ Symbol("");"#,
        "const symbol = new Symbol;",
        "() => {
                return new // 1
                    Symbol();
            }",
        "() => {
                return (
                    new // 2
                        Symbol()
                );
            }",
        "() => {
                return new // 3
                    (Symbol);
            }",
        "() => {
                return new // 4
                    Symbol;
            }",
        "() => {
                return (
                    new // 5
                        Symbol
                );
            }",
        "() => {
                return (
                    new // 6
                        (Symbol)
                );
            }",
        "() => {
                throw new // 1
                    Symbol();
            }",
        "() => {
                return new /**/ Symbol;
            }",
        "new globalThis.String()",
        "new global.String()",
        "new self.String()",
        "new window.String()",
        // TODO: Fix.
        // "const {String} = globalThis;
        //     new String();",
        // "const {String: RenamedString} = globalThis;
        //     new RenamedString();",
        // "const RenamedString = globalThis.String;
        //     new RenamedString();",
        "globalThis.Array()",
        "global.Array()",
        "self.Array()",
        "window.Array()",
        // "const {Array: RenamedArray} = globalThis;
        //     RenamedArray();",
        // We do not support configuring globals like this:
        // "globalThis.Array()", // {"globals": {"Array": "off"}},
        // "const {Array} = globalThis;
        //     Array();", // {"globals": {"Symbol": "off"}},
        "const foo = Object()",
        "const foo = Array()",
        "const foo = ArrayBuffer()",
        "const foo = BigInt64Array()",
        "const foo = BigUint64Array()",
        "const foo = DataView()",
        "const foo = Error()",
        "const foo = Error('Foo bar')",
        // "const foo = Float16Array()",
        "const foo = Float32Array()",
        "const foo = Float64Array()",
        "const foo = Function()",
        "const foo = Int8Array()",
        "const foo = Int16Array()",
        "const foo = Int32Array()",
        "const foo = (( Map ))()",
        "const foo = Map([['foo', 'bar'], ['unicorn', 'rainbow']])",
        "const foo = WeakMap()",
        "const foo = Set()",
        "const foo = WeakSet()",
        "const foo = Promise()",
        "const foo = RegExp()",
        "const foo = Uint8Array()",
        "const foo = Uint16Array()",
        "const foo = Uint32Array()",
        "const foo = Uint8ClampedArray()",
        "const foo = new BigInt(123)",
        "const foo = new Boolean()",
        "const foo = new Number()",
        "const foo = new Number('123')",
        "const foo = new String()",
        "const foo = new Symbol()",
        "function varCheck() {
                {
                    var WeakMap = function() {};
                }
                // This should not reported
                return WeakMap()
            }
            function constCheck() {
                {
                    const Array = function() {};
                }
                return Array()
            }
            function letCheck() {
                {
                    let Map = function() {};
                }
                return Map()
            }",
        // "function foo() {
        //         return(globalThis).Map()
        //     }",
        "const foo = Date();",
        "const foo = globalThis.Date();",
        // "function foo() {
        //         return(globalThis).Date();
        //     }",
        "const foo = Date(/*comment*/);",
        "const foo = globalThis/*comment*/.Date();",
        "const foo = Date(bar);",
    ];

    Tester::new(NewForBuiltins::NAME, NewForBuiltins::PLUGIN, pass, fail).test_and_snapshot();
}
