use oxc_ast::{
    ast::{Argument, CallExpression, Expression, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_prototype_property,
    AstNode,
};

fn no_array_reduce_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Don't use `Array#reduce()` and `Array#reduceRight()`, use `for` loops instead.",
    )
    .with_help("Refactor your code to use `for` loops instead.")
    .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoArrayReduce {
    pub allow_simple_operations: bool,
}

impl Default for NoArrayReduce {
    fn default() -> Self {
        Self { allow_simple_operations: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `Array#reduce()` and `Array#reduceRight()`.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#reduce()` and `Array#reduceRight()` usually result in [hard-to-read](https://twitter.com/jaffathecake/status/1213077702300852224) and [less performant](https://www.richsnapp.com/article/2019/06-09-reduce-spread-anti-pattern) code. In almost every case, it can be replaced by `.map`, `.filter`, or a `for-of` loop.
    ///
    /// It's only somewhat useful in the rare case of summing up numbers, which is allowed by default.
    ///
    /// ### Example
    /// ```javascript
    /// array.reduce(reducer, initialValue);
    /// array.reduceRight(reducer, initialValue);
    /// ```
    NoArrayReduce,
    unicorn,
    restriction
);

impl Rule for NoArrayReduce {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_simple_operations = value
            .as_object()
            .and_then(|v| v.get("allowSimpleOperations"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        Self { allow_simple_operations }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let Some((span, _)) = member_expr.static_property_info() else {
            return;
        };

        if is_method_call(call_expr, None, Some(&["reduce", "reduceRight"]), Some(1), Some(2))
            && !matches!(call_expr.arguments.first(), Some(Argument::SpreadElement(_)))
            && !call_expr.optional
            && !member_expr.is_computed()
        {
            if self.allow_simple_operations && is_simple_operation(call_expr) {
                return;
            }
            ctx.diagnostic(no_array_reduce_diagnostic(span));
        }

        if let Some(member_expr_obj) = member_expr.object().as_member_expression() {
            if is_method_call(call_expr, None, Some(&["call", "apply"]), None, None)
                && !member_expr.optional()
                && !member_expr.is_computed()
                && !call_expr.optional
                && !member_expr_obj.is_computed()
                && (is_prototype_property(member_expr_obj, "reduce", Some("Array"))
                    || is_prototype_property(member_expr_obj, "reduceRight", Some("Array")))
            {
                ctx.diagnostic(no_array_reduce_diagnostic(span));
            }
        }
    }
}

fn is_simple_operation(node: &CallExpression) -> bool {
    let Some(callback_arg) = node.arguments.first() else {
        return false;
    };
    let function_body = match callback_arg {
        // `array.reduce((accumulator, element) => accumulator + element)`
        Argument::ArrowFunctionExpression(callback) => &callback.body,
        Argument::FunctionExpression(callback) => {
            let Some(body) = &callback.body else {
                return false;
            };
            body
        }
        _ => return false,
    };

    if function_body.statements.len() != 1 {
        return false;
    }

    match &function_body.statements[0] {
        Statement::ExpressionStatement(expr) => {
            matches!(expr.expression, Expression::BinaryExpression(_))
        }
        Statement::ReturnStatement(ret) => {
            matches!(&ret.argument, Some(Expression::BinaryExpression(_)))
        }
        Statement::BlockStatement(block) => {
            if block.body.len() != 1 {
                return false;
            }

            match &block.body[0] {
                Statement::ReturnStatement(ret) => {
                    matches!(&ret.argument, Some(Expression::BinaryExpression(_)))
                }
                _ => false,
            }
        }
        _ => false,
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r"a[b.reduce]()", None),
        (r"a(b.reduce)", None),
        (r"a.reduce()", None),
        (r"a.reduce(1, 2, 3)", None),
        (r"a.reduce(b, c, d)", None),
        (r"[][reduce].call()", None),
        (r"[1, 2].reduce.call(() => {}, 34)", None),
        // Test `.reduce`
        // Not `CallExpression`
        (r"new foo.reduce(fn);", None),
        // Not `MemberExpression`
        (r"reduce(fn);", None),
        // `callee.property` is not a `Identifier`
        (r#"foo["reduce"](fn);"#, None),
        // Computed
        (r"foo[reduce](fn);", None),
        // Not listed method or property
        (r"foo.notListed(fn);", None),
        // More or less argument(s)
        (r"foo.reduce();", None),
        (r"foo.reduce(fn, extraArgument1, extraArgument2);", None),
        (r"foo.reduce(...argumentsArray)", None),
        // Test `[].reduce.{call,apply}`
        // Not `CallExpression`
        (r"new [].reduce.call(foo, fn);", None),
        // Not `MemberExpression`
        (r"call(foo, fn);", None),
        (r"reduce.call(foo, fn);", None),
        // `callee.property` is not a `Identifier`
        (r#"[].reduce["call"](foo, fn);"#, None),
        (r#"[]["reduce"].call(foo, fn);"#, None),
        // Computed
        (r"[].reduce[call](foo, fn);", None),
        (r"[][reduce].call(foo, fn);", None),
        // Not listed method or property
        (r"[].reduce.notListed(foo, fn);", None),
        (r"[].notListed.call(foo, fn);", None),
        // Not empty
        (r"[1].reduce.call(foo, fn)", None),
        // Not ArrayExpression
        (r#""".reduce.call(foo, fn)"#, None),
        // More or less argument(s)
        // We are not checking arguments length

        // Test `Array.prototype.{call,apply}`
        // Not `CallExpression`
        (r"new Array.prototype.reduce.call(foo, fn);", None),
        // Not `MemberExpression`
        (r"call(foo, fn);", None),
        (r"reduce.call(foo, fn);", None),
        // `callee.property` is not a `Identifier`
        (r#"Array.prototype.reduce["call"](foo, fn);"#, None),
        (r#"Array.prototype[",educe"].call(foo, fn);"#, None),
        (r#""Array".prototype.reduce.call(foo, fn);"#, None),
        // Computed
        (r"Array.prototype.reduce[call](foo, fn);", None),
        (r"Array.prototype[reduce].call(foo, fn);", None),
        (r"Array[prototype].reduce.call(foo, fn);", None),
        // Not listed method
        (r"Array.prototype.reduce.notListed(foo, fn);", None),
        (r"Array.prototype.notListed.call(foo, fn);", None),
        (r"Array.notListed.reduce.call(foo, fn);", None),
        // Not `Array`
        (r"NotArray.prototype.reduce.call(foo, fn);", None),
        // More or less argument(s)
        // We are not checking arguments length

        // `reduce-like`
        (r"array.reducex(foo)", None),
        (r"array.xreduce(foo)", None),
        (r"[].reducex.call(array, foo)", None),
        (r"[].xreduce.call(array, foo)", None),
        (r"Array.prototype.reducex.call(array, foo)", None),
        (r"Array.prototype.xreduce.call(array, foo)", None),
        // Option: allowSimpleOperations
        (r"array.reduce((total, item) => total + item)", None),
        (r"array.reduce((total, item) => { return total - item })", None),
        (r"array.reduce(function (total, item) { return total * item })", None),
        (r"array.reduce((total, item) => total + item, 0)", None),
        (r"array.reduce((total, item) => { return total - item }, 0 )", None),
        (r"array.reduce(function (total, item) { return total * item }, 0)", None),
        (
            r"
        array.reduce((total, item) => {
            return (total / item) * 100;
        }, 0);
        ",
            None,
        ),
        (r"array.reduce((total, item) => { return total + item }, 0)", None),
        (r"a[b.reduceRight]()", None),
        (r"a(b.reduceRight)", None),
        (r"a.reduceRight()", None),
        (r"a.reduceRight(1, 2, 3)", None),
        (r"a.reduceRight(b, c, d)", None),
        (r"[][reduceRight].call()", None),
        (r"[1, 2].reduceRight.call(() => {}, 34)", None),
        // Test `.reduceRight`
        // Not `CallExpression`
        (r"new foo.reduceRight(fn);", None),
        // Not `MemberExpression`
        (r"reduce(fn);", None),
        // `callee.property` is not a `Identifier`
        (r#"foo["reduce"](fn);"#, None),
        // Computed
        (r"foo[reduceRight](fn);", None),
        // Not listed method or property
        (r"foo.notListed(fn);", None),
        // More or less argument(s)
        (r"foo.reduceRight();", None),
        (r"foo.reduceRight(fn, extraArgument1, extraArgument2);", None),
        (r"foo.reduceRight(...argumentsArray)", None),
        // Test `[].reduceRight.{call,apply}`
        // Not `CallExpression`
        (r"new [].reduceRight.call(foo, fn);", None),
        // Not `MemberExpression`
        (r"call(foo, fn);", None),
        (r"reduce.call(foo, fn);", None),
        // `callee.property` is not a `Identifier`
        (r#"[].reduceRight["call"](foo, fn);"#, None),
        (r#"[]["reduce"].call(foo, fn);"#, None),
        // Computed
        (r"[].reduceRight[call](foo, fn);", None),
        (r"[][reduceRight].call(foo, fn);", None),
        // Not listed method or property
        (r"[].reduceRight.notListed(foo, fn);", None),
        (r"[].notListed.call(foo, fn);", None),
        // Not empty
        (r"[1].reduceRight.call(foo, fn)", None),
        // Not ArrayExpression
        (r#""".reduceRight.call(foo, fn)"#, None),
        // More or less argument(s)
        // We are not checking arguments length

        // Test `Array.prototype.{call,apply}`
        // Not `CallExpression`
        (r"new Array.prototype.reduceRight.call(foo, fn);", None),
        // Not `MemberExpression`
        (r"call(foo, fn);", None),
        (r"reduce.call(foo, fn);", None),
        // `callee.property` is not a `Identifier`
        (r#"Array.prototype.reduceRight["call"](foo, fn);"#, None),
        (r#"Array.prototype["reeduce"].call(foo, fn);"#, None),
        (r#""Array".prototype.reduceRight.call(foo, fn);"#, None),
        // Computed
        (r"Array.prototype.reduceRight[call](foo, fn);", None),
        (r"Array.prototype[reduceRight].call(foo, fn);", None),
        (r"Array[prototype].reduceRight.call(foo, fn);", None),
        // Not listed method
        (r"Array.prototype.reduceRight.notListed(foo, fn);", None),
        (r"Array.prototype.notListed.call(foo, fn);", None),
        (r"Array.notListed.reduceRight.call(foo, fn);", None),
        // Not `Array`
        (r"NotArray.prototype.reduceRight.call(foo, fn);", None),
        // More or less argument(s)
        // We are not checking arguments length

        // `reduceRight-like`
        (r"array.reduceRightx(foo)", None),
        (r"array.xreduceRight(foo)", None),
        (r"[].reduceRightx.call(array, foo)", None),
        (r"[].xreduceRight.call(array, foo)", None),
        (r"Array.prototype.reduceRightx.call(array, foo)", None),
        (r"Array.prototype.xreduceRight.call(array, foo)", None),
        // Option: allowSimpleOperations
        (r"array.reduceRight((total, item) => total + item)", None),
        (r"array.reduceRight((total, item) => { return total - item })", None),
        (r"array.reduceRight(function (total, item) { return total * item })", None),
        (r"array.reduceRight((total, item) => total + item, 0)", None),
        (r"array.reduceRight((total, item) => { return total - item }, 0 )", None),
        (r"array.reduceRight(function (total, item) { return total * item }, 0)", None),
        (
            r"
        array.reduceRight((total, item) => {
            return (total / item) * 100;
        }, 0);
        ",
            None,
        ),
        (r"array.reduceRight((total, item) => { return total + item }, 0)", None),
    ];

    let fail = vec![
        (r#"array.reduce((str, item) => str += item, "")"#, None),
        (
            r"
			array.reduce((obj, item) => {
				obj[item] = null;
				return obj;
			}, {})
			",
            None,
        ),
        (r"array.reduce((obj, item) => ({ [item]: null }), {})", None),
        (
            r#"
			const hyphenate = (str, char) => `${str}-${char}`;
			["a", "b", "c"].reduce(hyphenate);
			"#,
            None,
        ),
        (r"[].reduce.call(array, (s, i) => s + i)", None),
        (r"[].reduce.call(array, sum);", None),
        (r"[].reduce.call(sum);", None),
        (r"Array.prototype.reduce.call(array, (s, i) => s + i)", None),
        (r"Array.prototype.reduce.call(array, sum);", None),
        (r"[].reduce.apply(array, [(s, i) => s + i])", None),
        (r"[].reduce.apply(array, [sum]);", None),
        (r"Array.prototype.reduce.apply(array, [(s, i) => s + i])", None),
        (r"Array.prototype.reduce.apply(array, [sum]);", None),
        (
            r"
			array.reduce((total, item) => {
				return total + doComplicatedThings(item);
				function doComplicatedThings(item) {
					return item + 1;
				}
			}, 0);
			",
            None,
        ),
        // Option: allowSimpleOperations
        (
            r"array.reduce((total, item) => total + item)",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduce((total, item) => { return total - item })",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduce(function (total, item) { return total * item })",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduce((total, item) => total + item, 0)",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduce((total, item) => { return total - item }, 0 )",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduce(function (total, item) { return total * item }, 0)",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"
				array.reduce((total, item) => {
					return (total / item) * 100;
				}, 0);
		",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (r#"array.reduceRight((str, item) => str += item, "")"#, None),
        (
            r"
			array.reduceRight((obj, item) => {
				obj[item] = null;
				return obj;
			}, {})
			",
            None,
        ),
        (r"array.reduceRight((obj, item) => ({ [item]: null }), {})", None),
        (
            r#"
			const hyphenate = (str, char) => `${str}-${char}`;
			["a", "b", "c"].reduceRight(hyphenate);
			"#,
            None,
        ),
        (r"[].reduceRight.call(array, (s, i) => s + i)", None),
        (r"[].reduceRight.call(array, sum);", None),
        (r"[].reduceRight.call(sum);", None),
        (r"Array.prototype.reduceRight.call(array, (s, i) => s + i)", None),
        (r"Array.prototype.reduceRight.call(array, sum);", None),
        (r"[].reduceRight.apply(array, [(s, i) => s + i])", None),
        (r"[].reduceRight.apply(array, [sum]);", None),
        (r"Array.prototype.reduceRight.apply(array, [(s, i) => s + i])", None),
        (r"Array.prototype.reduceRight.apply(array, [sum]);", None),
        (
            r"
			array.reduceRight((total, item) => {
				return total + doComplicatedThings(item);
				function doComplicatedThings(item) {
					return item + 1;
				}
			}, 0);
			",
            None,
        ),
        // Option: allowSimpleOperations
        (
            r"array.reduceRight((total, item) => total + item)",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduceRight((total, item) => { return total - item })",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduceRight(function (total, item) { return total * item })",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduceRight((total, item) => total + item, 0)",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduceRight((total, item) => { return total - item }, 0 )",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"array.reduceRight(function (total, item) { return total * item }, 0)",
            Some(json!({ "allowSimpleOperations": false})),
        ),
        (
            r"
				array.reduceRight((total, item) => {
					return (total / item) * 100;
				}, 0);
		",
            Some(json!({ "allowSimpleOperations": false})),
        ),
    ];
    Tester::new(NoArrayReduce::NAME, NoArrayReduce::PLUGIN, pass, fail).test_and_snapshot();
}
