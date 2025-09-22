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
    pedantic
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
        r"const foo = new Object()",
        r"const foo = new Array()",
        r"const foo = new ArrayBuffer()",
        r"const foo = new BigInt64Array()",
        r"const foo = new BigUint64Array()",
        r"const foo = new DataView()",
        r"const foo = new Date()",
        r"const foo = new Error()",
        r"const foo = new Float32Array()",
        r"const foo = new Float64Array()",
        r"const foo = new Function()",
        r"const foo = new Int8Array()",
        r"const foo = new Int16Array()",
        r"const foo = new Int32Array()",
        r"const foo = new Map()",
        r"const foo = new Map([['foo', 'bar'], ['unicorn', 'rainbow']])",
        r"const foo = new WeakMap()",
        r"const foo = new Set()",
        r"const foo = new WeakSet()",
        r"const foo = new Promise()",
        r"const foo = new RegExp()",
        r"const foo = new UInt8Array()",
        r"const foo = new UInt16Array()",
        r"const foo = new UInt32Array()",
        r"const foo = new Uint8ClampedArray()",
        r"const foo = BigInt()",
        r"const foo = Boolean()",
        r"const foo = Number()",
        r"const foo = String()",
        r"const foo = Symbol()",
        r"
            import { Map } from 'immutable';
            const m = Map();
        ",
        r"
        	const {Map} = require('immutable');
        	const foo = Map();
        ",
        r"
        	const {String} = require('guitar');
        	const lowE = new String();
        ",
        r"
        	import {String} from 'guitar';
        	const lowE = new String();
        ",
        r"new Foo();Bar();",
        r"Foo();new Bar();",
        r"const isObject = v => Object(v) === v;",
        r"const isObject = v => globalThis.Object(v) === v;",
        r"(x) !== Object(x)",
    ];

    let fail = vec![
        r"const object = (Object)();",
        r#"const symbol = new (Symbol)("");"#,
        r#"const symbol = new /* comment */ Symbol("");"#,
        r"const symbol = new Symbol;",
        r"new globalThis.String()",
        r"new global.String()",
        r"new self.String()",
        r"new window.String()",
        r"globalThis.Array()",
        r"global.Array()",
        r"self.Array()",
        r"window.Array()",
        r"globalThis.Array()",
        r"const foo = Object()",
        r"const foo = Array()",
        r"const foo = ArrayBuffer()",
        r"const foo = BigInt64Array()",
        r"const foo = BigUint64Array()",
        r"const foo = DataView()",
        r"const foo = Date()",
        r"const foo = Error()",
        r"const foo = Error('Foo bar')",
        r"const foo = Float32Array()",
        r"const foo = Float64Array()",
        r"const foo = Function()",
        r"const foo = Int8Array()",
        r"const foo = Int16Array()",
        r"const foo = Int32Array()",
        r"const foo = (( Map ))()",
        r"const foo = Map([['foo', 'bar'], ['unicorn', 'rainbow']])",
        r"const foo = WeakMap()",
        r"const foo = Set()",
        r"const foo = WeakSet()",
        r"const foo = Promise()",
        r"const foo = RegExp()",
        r"const foo = Uint8Array()",
        r"const foo = Uint16Array()",
        r"const foo = Uint32Array()",
        r"const foo = Uint8ClampedArray()",
        r"const foo = new BigInt(123)",
        r"const foo = new Boolean()",
        r"const foo = new Number()",
        r"const foo = new Number('123')",
        r"const foo = new String()",
        r"const foo = new Symbol()",
        r"
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
        ",
    ];

    Tester::new(NewForBuiltins::NAME, NewForBuiltins::PLUGIN, pass, fail).test_and_snapshot();
}
