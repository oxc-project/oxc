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
#[error("eslint-plugin-unicorn(new-for-builtins):")]
#[diagnostic(severity(warning), help(""))]
struct NewForBuiltinsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NewForBuiltins;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NewForBuiltins,
    correctness
);

impl Rule for NewForBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let callee = &new_expr.callee.without_parenthesized();

                let Expression::Identifier(name) = callee else { return };

                // if ENFORCE_NEW_FOR_BUILTINS.contains(name.name.as_str()) {
                //     if new_expr.arguments.len() == 0 {
                //         ctx.diagnostic(NewForBuiltinsDiagnostic(new_expr.span));
                //     }
                // } else
                if DISALLOW_NEW_FOR_BUILTINS.contains(name.name.as_str()) {
                    // if new_expr.arguments.len() == 0 {
                    ctx.diagnostic(NewForBuiltinsDiagnostic(new_expr.span));
                    // }
                }
            }
            AstKind::CallExpression(call_expr) => {
                let callee = call_expr.callee.without_parenthesized();

                let Expression::Identifier(name) = callee else { return };

                if ENFORCE_NEW_FOR_BUILTINS.contains(name.name.as_str()) {
                    if name.name.as_str() == "Object" {
                        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
                            // parent.type === 'BinaryExpression'
                            // && (parent.operator === '===' || parent.operator === '!==')
                            // && (parent.left === node || parent.right === node)

                            if let AstKind::BinaryExpression(bin_expr) = parent.kind() {
                                if bin_expr.operator == BinaryOperator::StrictEquality
                                    || bin_expr.operator == BinaryOperator::StrictInequality
                                {
                                    return;
                                }
                            }
                        }
                    }

                    // if call_expr.arguments.len() == 0 {
                    ctx.diagnostic(NewForBuiltinsDiagnostic(call_expr.span));
                    // }
                } //else
                  //  if DISALLOW_NEW_FOR_BUILTINS.contains(name.name.as_str()) {
                  //     if call_expr.arguments.len() == 0 {
                  //         ctx.diagnostic(NewForBuiltinsDiagnostic(call_expr.span));
                  //     }
                  // }
            }
            _ => {}
        }
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
        // r#"
        // 				import { Map } from 'immutable';
        // 				const m = Map();
        // 			"#,
        // r#"
        // 				const {Map} = require('immutable');
        // 				const foo = Map();
        // 			"#,
        // r#"
        // 				const {String} = require('guitar');
        // 				const lowE = new String();
        // 			"#,
        // r#"
        // 				import {String} from 'guitar';
        // 				const lowE = new String();
        // 			"#,
        r#"new Foo();Bar();"#,
        r#"Foo();new Bar();"#,
        r#"const isObject = v => Object(v) === v;"#,
        r#"const isObject = v => globalThis.Object(v) === v;"#,
        r#"(x) !== Object(x)"#,
        // r#"new Symbol("")"#,
    ];

    let fail = vec![
        r#"const object = (Object)();"#,
        r#"const symbol = new (Symbol)("");"#,
        r#"const symbol = new /* comment */ Symbol("");"#,
        r#"const symbol = new Symbol;"#,
        // r#"new globalThis.String()"#,
        // r#"new global.String()"#,
        // r#"new self.String()"#,
        // r#"new window.String()"#,
        // r#"globalThis.Array()"#,
        // r#"global.Array()"#,
        // r#"self.Array()"#,
        // r#"window.Array()"#,
        // r#"globalThis.Array()"#,
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
