use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;
use phf::phf_set;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum NewForBuiltinsDiagnostic {
    #[error("eslint-plugin-unicorn(new-for-builtins): Use `new {1}()` instead of `{1}()`")]
    #[diagnostic(severity(warning))]
    Enforce(#[label] Span, String),
    #[error("eslint-plugin-unicorn(new-for-builtins): Use `{1}()` instead of `new {1}()`")]
    #[diagnostic(severity(warning))]
    Disallow(#[label] Span, String),
}

#[derive(Debug, Default, Clone)]
pub struct NewForBuiltins;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `new` for following builtins: `Object`, `Array`, `ArrayBuffer`, `BigInt64Array`, `BigUint64Array`, `DataView`, `Date`, `Error`, `Float32Array`, `Float64Array`, `Function`, `Int8Array`, `Int16Array`, `Int32Array`, `Map`, `WeakMap`, `Set`, `WeakSet`, `Promise`, `RegExp`, `Uint8Array`, `Uint16Array`, `Uint32Array`, `Uint8ClampedArray`, `SharedArrayBuffer`, `Proxy`, `WeakRef`, `FinalizationRegistry`.
    ///
    /// Disallows the use of `new` for following builtins: `String`, `Number`, `Boolean`, `Symbol`, `BigInt`.
    ///
    /// These should not use `new` as that would create object wrappers for the primitive values, which is not what you want. However, without `new` they can be useful for coercing a value to that type.
    ///
    /// ### Why is this bad?
    ///
    /// They work the same, but `new` should be preferred for consistency with other constructors.
    ///
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// const foo = new String('hello world');
    /// const bar = Array(1, 2, 3);
    ///
    /// // good
    /// const foo = String('hello world');
    /// const bar = new Array(1, 2, 3);
    /// ```
    NewForBuiltins,
    pedantic
);

impl Rule for NewForBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let callee = new_expr.callee.without_parenthesized();

                let Some(builtin_name) = is_expr_global_builtin(callee, ctx) else { return };

                if DISALLOW_NEW_FOR_BUILTINS.contains(builtin_name) {
                    ctx.diagnostic(NewForBuiltinsDiagnostic::Disallow(
                        new_expr.span,
                        builtin_name.to_string(),
                    ));
                }
            }
            AstKind::CallExpression(call_expr) => {
                let Some(builtin_name) =
                    is_expr_global_builtin(call_expr.callee.without_parenthesized(), ctx)
                else {
                    return;
                };

                if ENFORCE_NEW_FOR_BUILTINS.contains(builtin_name) {
                    if builtin_name == "Object" {
                        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
                            if let AstKind::BinaryExpression(bin_expr) = parent.kind() {
                                if bin_expr.operator == BinaryOperator::StrictEquality
                                    || bin_expr.operator == BinaryOperator::StrictInequality
                                {
                                    return;
                                }
                            }
                        }
                    }

                    ctx.diagnostic(NewForBuiltinsDiagnostic::Enforce(
                        call_expr.span,
                        builtin_name.to_string(),
                    ));
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
    match expr {
        Expression::Identifier(ident) => {
            if !ctx.semantic().is_reference_to_global_variable(ident) {
                return None;
            }
            Some(ident.name.as_str())
        }
        Expression::MemberExpression(member_expr) => {
            let Expression::Identifier(ident) = member_expr.object() else { return None };

            if !GLOBAL_OBJECT_NAMES.contains(ident.name.as_str()) {
                return None;
            }

            return member_expr.static_property_name();
        }
        _ => None,
    }
}

const ENFORCE_NEW_FOR_BUILTINS: phf::Set<&'static str> = phf_set! {
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
    "Object",
    "Array",
    "ArrayBuffer",
    "DataView",
    "Date",
    "Error",
    "Function",
    "Map",
    "WeakMap",
    "Set",
    "WeakSet",
    "Promise",
    "RegExp",
    "SharedArrayBuffer",
    "Proxy",
    "WeakRef",
    "FinalizationRegistry",
};

const DISALLOW_NEW_FOR_BUILTINS: phf::Set<&'static str> = phf_set! {
    "BigInt",
    "Boolean",
    "Number",
    "Symbol",
    "String",
};

const GLOBAL_OBJECT_NAMES: phf::Set<&'static str> = phf_set! {
    "global",
    "globalThis",
    "self",
    "window",
};

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"const foo = new Object()"#,
        r#"const foo = new Array()"#,
        r#"const foo = new ArrayBuffer()"#,
        r#"const foo = new BigInt64Array()"#,
        r#"const foo = new BigUint64Array()"#,
        r#"const foo = new DataView()"#,
        r#"const foo = new Date()"#,
        r#"const foo = new Error()"#,
        r#"const foo = new Float32Array()"#,
        r#"const foo = new Float64Array()"#,
        r#"const foo = new Function()"#,
        r#"const foo = new Int8Array()"#,
        r#"const foo = new Int16Array()"#,
        r#"const foo = new Int32Array()"#,
        r#"const foo = new Map()"#,
        r#"const foo = new Map([['foo', 'bar'], ['unicorn', 'rainbow']])"#,
        r#"const foo = new WeakMap()"#,
        r#"const foo = new Set()"#,
        r#"const foo = new WeakSet()"#,
        r#"const foo = new Promise()"#,
        r#"const foo = new RegExp()"#,
        r#"const foo = new UInt8Array()"#,
        r#"const foo = new UInt16Array()"#,
        r#"const foo = new UInt32Array()"#,
        r#"const foo = new Uint8ClampedArray()"#,
        r#"const foo = BigInt()"#,
        r#"const foo = Boolean()"#,
        r#"const foo = Number()"#,
        r#"const foo = String()"#,
        r#"const foo = Symbol()"#,
        r#"
            import { Map } from 'immutable';
            const m = Map();
        "#,
        r#"
        	const {Map} = require('immutable');
        	const foo = Map();
        "#,
        r#"
        	const {String} = require('guitar');
        	const lowE = new String();
        "#,
        r#"
        	import {String} from 'guitar';
        	const lowE = new String();
        "#,
        r#"new Foo();Bar();"#,
        r#"Foo();new Bar();"#,
        r#"const isObject = v => Object(v) === v;"#,
        r#"const isObject = v => globalThis.Object(v) === v;"#,
        r#"(x) !== Object(x)"#,
    ];

    let fail = vec![
        r#"const object = (Object)();"#,
        r#"const symbol = new (Symbol)("");"#,
        r#"const symbol = new /* comment */ Symbol("");"#,
        r#"const symbol = new Symbol;"#,
        r#"new globalThis.String()"#,
        r#"new global.String()"#,
        r#"new self.String()"#,
        r#"new window.String()"#,
        r#"globalThis.Array()"#,
        r#"global.Array()"#,
        r#"self.Array()"#,
        r#"window.Array()"#,
        r#"globalThis.Array()"#,
        r#"const foo = Object()"#,
        r#"const foo = Array()"#,
        r#"const foo = ArrayBuffer()"#,
        r#"const foo = BigInt64Array()"#,
        r#"const foo = BigUint64Array()"#,
        r#"const foo = DataView()"#,
        r#"const foo = Date()"#,
        r#"const foo = Error()"#,
        r#"const foo = Error('Foo bar')"#,
        r#"const foo = Float32Array()"#,
        r#"const foo = Float64Array()"#,
        r#"const foo = Function()"#,
        r#"const foo = Int8Array()"#,
        r#"const foo = Int16Array()"#,
        r#"const foo = Int32Array()"#,
        r#"const foo = (( Map ))()"#,
        r#"const foo = Map([['foo', 'bar'], ['unicorn', 'rainbow']])"#,
        r#"const foo = WeakMap()"#,
        r#"const foo = Set()"#,
        r#"const foo = WeakSet()"#,
        r#"const foo = Promise()"#,
        r#"const foo = RegExp()"#,
        r#"const foo = Uint8Array()"#,
        r#"const foo = Uint16Array()"#,
        r#"const foo = Uint32Array()"#,
        r#"const foo = Uint8ClampedArray()"#,
        r#"const foo = new BigInt(123)"#,
        r#"const foo = new Boolean()"#,
        r#"const foo = new Number()"#,
        r#"const foo = new Number('123')"#,
        r#"const foo = new String()"#,
        r#"const foo = new Symbol()"#,
        r#"
			function varCheck() {
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
			}
        "#,
    ];

    Tester::new_without_config(NewForBuiltins::NAME, pass, fail).test_and_snapshot();
}
