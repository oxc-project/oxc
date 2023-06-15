mod return_checker;

use oxc_ast::{
    ast::{ChainElement, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use phf::phf_set;
use serde_json::Value;

use self::return_checker::{check_function_body, StatementReturnStatus};
use crate::{
    ast_util::{get_enclosing_function, is_nth_argument, outermost_paren},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum ArrayCallbackReturnDiagnostic {
    #[error("eslint(array-callback-return): Missing return on some path for array method {0:?}")]
    #[diagnostic(
        severity(warning),
        help("Array method {0:?} needs to have valid return on all code paths")
    )]
    ExpectReturn(Atom, #[label] Span),

    #[error("eslint(array-callback-return): Unexpected return for array method {0}")]
    #[diagnostic(
        severity(warning),
        help("Array method {0} expects no useless return from the function")
    )]
    ExpectNoReturn(Atom, #[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct ArrayCallbackReturn {
    /// When set to true, rule will also report forEach callbacks that return a value.
    check_for_each: bool,
    /// When set to true, allows callbacks of methods that require a return value to
    /// implicitly return undefined with a return statement containing no expression.
    allow_implicit_return: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce return statements in callbacks of array methods
    ///
    /// ### Why is this bad?
    /// Array has several methods for filtering, mapping, and folding.
    /// If we forget to write return statement in a callback of those, it’s probably a mistake.
    /// If you don’t want to use a return or don’t need the returned results,
    /// consider using .forEach instead.
    ///
    /// ### Example
    /// ```javascript
    /// let foo = [1, 2, 3, 4];
    /// foo.map((a) => {
    ///   console.log(a)
    /// });
    /// ```
    ArrayCallbackReturn,
    correctness
);

impl Rule for ArrayCallbackReturn {
    fn from_configuration(value: Value) -> Self {
        let (check_for_each, allow_implicit_return) =
            value.get(0).map_or((false, false), |config| {
                (
                    config.get("checkForEach").and_then(Value::as_bool).unwrap_or_default(),
                    config.get("allowImplicit").and_then(Value::as_bool).unwrap_or_default(),
                )
            });

        Self { check_for_each, allow_implicit_return }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (function_body, always_explicit_return) = match node.get().kind() {
            // Async, generator, and single expression arrow functions
            // always have explicit return value
            AstKind::ArrowExpression(arrow) => {
                (&arrow.body, arrow.r#async || arrow.generator || arrow.is_single_expression())
            }
            AstKind::Function(function) => {
                if let Some(body) = &function.body {
                    (body, function.r#async || function.generator)
                } else {
                    return;
                }
            }
            _ => return,
        };

        // Filter on target methods on Arrays
        if let Some(array_method) = get_array_method_name(node, ctx) {
            let return_status = if always_explicit_return {
                StatementReturnStatus::AlwaysExplicit
            } else {
                check_function_body(function_body)
            };

            match (array_method, self.check_for_each, self.allow_implicit_return) {
                ("forEach", false, _) => (),
                ("forEach", true, _) => {
                    if return_status.may_return_explicit() {
                        ctx.diagnostic(ArrayCallbackReturnDiagnostic::ExpectNoReturn(
                            full_array_method_name(array_method),
                            function_body.span,
                        ));
                    }
                }
                (_, _, true) => {
                    if !return_status.must_return() {
                        ctx.diagnostic(ArrayCallbackReturnDiagnostic::ExpectReturn(
                            full_array_method_name(array_method),
                            function_body.span,
                        ));
                    }
                }
                (_, _, false) => {
                    if !return_status.must_return() || return_status.may_return_implicit() {
                        ctx.diagnostic(ArrayCallbackReturnDiagnostic::ExpectReturn(
                            full_array_method_name(array_method),
                            function_body.span,
                        ));
                    }
                }
            }
        }
    }
}

/// Code ported from [eslint](https://github.com/eslint/eslint/blob/main/lib/rules/array-callback-return.js)
/// We're currently on a `Function` or `ArrowExpression`, findout if it is an argument
/// to the target array methods we're interested in.
pub fn get_array_method_name<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'static str> {
    let mut current_node = node;
    while !matches!(current_node.get().kind(), AstKind::Root) {
        let parent = ctx.parent_node(current_node).unwrap();

        match parent.get().kind() {
            // foo.every(nativeFoo || function foo() { ... })
            AstKind::LogicalExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::Argument(_)
            | AstKind::ParenthesizedExpression(_) => {
                current_node = parent;
            }

            // If the function is instantly invoked function expression (IIFE)
            // E.g.
            // foo.every(function() {
            //  return function() {}
            // }())
            AstKind::ReturnStatement(_) => {
                let func_node = get_enclosing_function(parent, ctx).unwrap();
                let func_node = outermost_paren(func_node, ctx);

                // the node that calls func_node
                let func_parent = ctx.parent_node(func_node).unwrap();

                if let AstKind::CallExpression(call) = func_parent.get().kind() {
                    let expected_callee = &call.callee;
                    if expected_callee.span() == func_node.get().kind().span() {
                        current_node = func_parent;
                        continue;
                    }
                }

                return None;
            }

            AstKind::CallExpression(call) => {
                let AstKind::Argument(current_node_arg) = current_node.get().kind() else {
                  return None;
                };

                let callee = call.callee.get_inner_expression();
                let callee = match callee {
                    Expression::MemberExpression(member) => member,
                    Expression::ChainExpression(chain) => {
                        if let ChainElement::MemberExpression(member) = &chain.expression {
                            member
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                };

                // Array.from
                if callee.is_specific_member_access("Array", "from") {
                    // Check that current node is parent's second argument
                    if call.arguments.len() == 2 && is_nth_argument(call, current_node_arg, 1) {
                        return Some("from");
                    }
                }

                // "methods",
                let Some(method) = callee.static_property_name() else { return None; };
                if let Some(&array_method) = TARGET_METHODS.get_key(method) {
                    // Check that current node is parent's first argument
                    if call.arguments.len() == 1 && is_nth_argument(call, current_node_arg, 0) {
                        return Some(array_method);
                    }
                }

                return None;
            }

            _ => return None,
        }
    }

    None
}

const TARGET_METHODS: phf::Set<&'static str> = phf_set! {
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flatMap",
    "forEach",
    "map",
    "reduce",
    "reduceRight",
    "some",
    "sort",
    "toSorted",
};

fn full_array_method_name(array_method: &'static str) -> Atom {
    match array_method {
        "from" => Atom::from("Array.from"),
        s => Atom::from(format!("Array.prototype.{s}")),
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo.every(function(){ return 1; })", None),
        ("foo.every(function(){}())", None),
        ("foo.every(function(){ return function() { return true; }; }())", None),
        ("foo.every(function(){ return function() { return; }; })", None),
        ("foo.forEach(bar || function(x) { var a=0; })", None),
        ("foo.forEach(bar || function(x) { return a; })", None),
        ("foo.forEach(function() {return function() { var a = 0;}}())", None),
        ("foo.forEach(function(x) { var a=0; })", None),
        ("foo.forEach(function(x) { return a;})", None),
        ("foo.forEach(function(x) { return; })", None),
        ("foo.forEach(function(x) { if (a === b) { return;} var a=0; })", None),
        ("foo.forEach(function(x) { if (a === b) { return x;} var a=0; })", None),
        ("foo.bar().forEach(function(x) { return; })", None),
        ("[\"foo\",\"bar\",\"baz\"].forEach(function(x) { return x; })", None),
        ("foo.forEach(x => { var a=0; })", None),
        ("foo.forEach(x => { if (a === b) { return;} var a=0; })", None),
        ("foo.forEach(x => x)", None),
        ("foo.forEach(val => y += val)", None),
        ("foo.map(async function(){})", None),
        ("foo.map(async () => {})", None),
        ("foo.map(function* () {})", None),
        (
            "Array.from(x, function() { return true; })",
            Some(serde_json::json!([{ "allowImplicit": false }])),
        ),
        // (
        //     "Int32Array.from(x, function() { return true; })",
        //     Some(serde_json::json!([{ "allowImplicit": false }])),
        // ),
        ("foo.every(function() { return true; })", None),
        ("foo.filter(function() { return true; })", None),
        ("foo.find(function() { return true; })", None),
        ("foo.findIndex(function() { return true; })", None),
        ("foo.findLast(function() { return true; })", None),
        ("foo.findLastIndex(function() { return true; })", None),
        ("foo.flatMap(function() { return true; })", None),
        ("foo.forEach(function() { return; })", None),
        ("foo.map(function() { return true; })", None),
        ("foo.reduce(function() { return true; })", None),
        ("foo.reduceRight(function() { return true; })", None),
        ("foo.some(function() { return true; })", None),
        ("foo.sort(function() { return 0; })", None),
        ("foo.toSorted(function() { return 0; })", None),
        ("foo.every(() => { return true; })", None),
        ("foo.every(function() { if (a) return true; else return false; })", None),
        ("foo.every(function() { switch (a) { case 0: bar(); default: return true; } })", None),
        (
            "foo.every(function() { try { bar(); return true; } catch (err) { return false; } })",
            None,
        ),
        ("foo.every(function() { try { bar(); } finally { return true; } })", None),
        (
            "Array.from(x, function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        // (
        //     "Int32Array.from(x, function() { return; })",
        //     Some(serde_json::json!([{"allowImplicit": true}])),
        // ),
        ("foo.every(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.filter(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.find(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.findIndex(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.findLast(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.findLastIndex(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.flatMap(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.forEach(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.map(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.reduce(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.reduceRight(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.some(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.sort(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.toSorted(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.every(() => { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.every(function() { if (a) return; else return a; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.every(function() { switch (a) { case 0: bar(); default: return; } })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.every(function() { try { bar(); return; } catch (err) { return; } })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.every(function() { try { bar(); } finally { return; } })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.forEach(function(x) { return; })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(function(x) { var a=0; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a === b) { return;} var a=0; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function() {return function() { if (a == b) { return; }}}())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach(x => { var a=0; })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(x => { if (a === b) { return;} var a=0; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach(x => { x })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(bar || function(x) { return; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "Array.from(x, function() { return true; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        // (
        //     "Int32Array.from(x, function() { return true; })",
        //     Some(serde_json::json!([{"checkForEach": true}])),
        // ),
        ("foo.every(() => { return true; })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.every(function() { if (a) return 1; else return a; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { switch (a) { case 0: return bar(); default: return a; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { try { bar(); return 1; } catch (err) { return err; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { try { bar(); } finally { return 1; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        ("Arrow.from(x, function() {})", None),
        ("foo.abc(function() {})", None),
        ("every(function() {})", None),
        ("foo[every](function() {})", None),
        ("var every = function() {}", None),
        ("foo[`${every}`](function() {})", None),
        ("foo.every(() => true)", None),
    ];

    let fail = vec![
        ("Array.from(x, function() {})", None),
        ("Array.from(x, function foo() {})", None),
        // ("Int32Array.from(x, function() {})", None),
        // ("Int32Array.from(x, function foo() {})", None),
        ("foo.every(function() {})", None),
        ("foo.every(function foo() {})", None),
        ("foo.filter(function() {})", None),
        ("foo.filter(function foo() {})", None),
        ("foo.find(function() {})", None),
        ("foo.find(function foo() {})", None),
        ("foo.findLast(function() {})", None),
        ("foo.findLast(function foo() {})", None),
        ("foo.findIndex(function() {})", None),
        ("foo.findIndex(function foo() {})", None),
        ("foo.findLastIndex(function() {})", None),
        ("foo.findLastIndex(function foo() {})", None),
        ("foo.flatMap(function() {})", None),
        ("foo.flatMap(function foo() {})", None),
        ("foo.map(function() {})", None),
        ("foo.map(function foo() {})", None),
        ("foo.reduce(function() {})", None),
        ("foo.reduce(function foo() {})", None),
        ("foo.reduceRight(function() {})", None),
        ("foo.reduceRight(function foo() {})", None),
        ("foo.some(function() {})", None),
        ("foo.some(function foo() {})", None),
        ("foo.sort(function() {})", None),
        ("foo.sort(function foo() {})", None),
        ("foo.toSorted(function() {})", None),
        ("foo.toSorted(function foo() {})", None),
        ("foo.bar.baz.every(function() {})", None),
        ("foo.bar.baz.every(function foo() {})", None),
        ("foo[\"every\"](function() {})", None),
        ("foo[\"every\"](function foo() {})", None),
        ("foo[`every`](function() {})", None),
        ("foo[`every`](function foo() {})", None),
        ("foo.every(() => {})", None),
        ("foo.every(function() { if (a) return true; })", None),
        ("foo.every(function cb() { if (a) return true; })", None),
        ("foo.every(function() { switch (a) { case 0: break; default: return true; } })", None),
        ("foo.every(function foo() { switch (a) { case 0: break; default: return true; } })", None),
        ("foo.every(function() { try { bar(); } catch (err) { return true; } })", None),
        ("foo.every(function foo() { try { bar(); } catch (err) { return true; } })", None),
        ("foo.every(function() { return; })", None),
        ("foo.every(function foo() { return; })", None),
        ("foo.every(function() { if (a) return; })", None),
        ("foo.every(function foo() { if (a) return; })", None),
        ("foo.every(function() { if (a) return; else return; })", None),
        ("foo.every(function foo() { if (a) return; else return; })", None),
        ("foo.every(cb || function() {})", None),
        ("foo.every(cb || function foo() {})", None),
        ("foo.every(a ? function() {} : function() {})", None),
        ("foo.every(a ? function foo() {} : function bar() {})", None),
        ("foo.every(function(){ return function() {}; }())", None),
        ("foo.every(function(){ return function foo() {}; }())", None),
        ("foo.every(() => {})", Some(serde_json::json!([{ "allowImplicit": false }]))),
        ("foo.every(() => {})", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Array.from(x, function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.every(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.filter(function foo() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.find(function foo() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.map(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.reduce(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.reduceRight(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.bar.baz.every(function foo() {})",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.every(cb || function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "[\"foo\",\"bar\"].sort(function foo() {})",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "[\"foo\",\"bar\"].toSorted(function foo() {})",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.forEach(x => x)",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a == b) {return x;}})",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        ("foo.forEach(x => x)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.forEach(val => y += val)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("[\"foo\",\"bar\"].forEach(x => ++x)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.bar().forEach(x => x === y)", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(function() {return function() { if (a == b) { return a; }}}())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a == b) {return x;}})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a == b) {return undefined;}})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.bar().forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "[\"foo\",\"bar\"].forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach((x) => { return x;})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("Array.from(x, function() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.every(function() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.filter(function foo() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.filter(function foo() { return; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.every(cb || function() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.filter(bar => { baz(); } )", None),
        ("foo.filter(\n() => {} )", None),
        ("foo.filter(bar || ((baz) => {}) )", None),
        ("foo.filter(bar => { return; })", None),
        ("Array.from(foo, bar => { bar })", None),
        ("foo.forEach(bar => bar)", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach((function () { return (bar) => bar; })())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach((() => {\n return bar => bar; })())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach((bar) => { if (bar) { return; } else { return bar ; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.filter(function(){})", None),
        ("foo.filter(function (){})", None),
        ("foo.filter(function\n(){})", None),
        ("foo.filter(function bar(){})", None),
        ("foo.filter(function bar  (){})", None),
        ("foo.filter(function\n bar() {})", None),
        ("Array.from(foo, function bar(){})", None),
        ("Array.from(foo, bar ? function (){} : baz)", None),
        ("foo.filter(function bar() { return \n })", None),
        (
            "foo.forEach(function () { \nif (baz) return bar\nelse return\n })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo?.filter(() => { console.log('hello') })", None),
        ("(foo?.filter)(() => { console.log('hello') })", None),
        ("Array?.from([], () => { console.log('hello') })", None),
        ("(Array?.from)([], () => { console.log('hello') })", None),
        ("foo?.filter((function() { return () => { console.log('hello') } })?.())", None),
    ];

    Tester::new(ArrayCallbackReturn::NAME, pass, fail).test_and_snapshot();
}
