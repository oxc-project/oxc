use oxc_ast::{
    AstKind,
    ast::{Expression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn no_array_callback_reference_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid passing a function reference directly to iterator methods")
        .with_help(
            "Wrap the function in an arrow function to explicitly pass only the element argument",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayCallbackReference;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents passing a function reference directly to iterator methods
    ///
    /// ### Why is this bad?
    ///
    /// Passing functions to iterator methods can cause issues when the function is changed
    /// without realizing that the iterator passes 2 more parameters to it (index and array).
    /// This can lead to unexpected behavior when the function signature changes.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = array.map(callback);
    /// array.forEach(callback);
    /// const result = array.filter(lib.method);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = array.map(element => callback(element));
    /// array.forEach(element => { callback(element); });
    /// const result = array.filter(element => lib.method(element));
    ///
    /// // Built-in functions are allowed
    /// const foo = array.map(String);
    /// const bar = array.filter(Boolean);
    /// ```
    NoArrayCallbackReference,
    unicorn,
    pedantic,
    pending
);

impl Rule for NoArrayCallbackReference {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let is_relevant_method = is_method_call(
            call_expr,
            None,
            Some(&[
                "every",
                "filter",
                "find",
                "findLast",
                "findIndex",
                "findLastIndex",
                "flatMap",
                "forEach",
                "map",
                "some",
            ]),
            Some(1),
            Some(2),
        ) || is_method_call(
            call_expr,
            None,
            Some(&["reduce", "reduceRight"]),
            Some(1),
            Some(2),
        );

        if !is_relevant_method {
            return;
        }

        if let Some(member_expr) = call_expr.callee.get_member_expr() {
            if member_expr.is_computed() {
                return;
            }

            let object = member_expr.object();
            if is_ignored_object(object) {
                return;
            }

            // Skip if object is a member expression (e.g., types.map, oidc.Client.find)
            // These are likely not array methods but methods on namespaces/classes
            if object.as_member_expression().is_some() {
                return;
            }
        }

        let Some(first_arg) = call_expr.arguments.first() else { return };

        let Some(callback_expr) = first_arg.as_expression() else { return };

        if !should_wrap_callback(callback_expr) {
            return;
        }

        ctx.diagnostic(no_array_callback_reference_diagnostic(callback_expr.span()));
    }
}

fn should_wrap_callback(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(ident) if is_allowed_builtin(&ident.name) => false,
        Expression::ConditionalExpression(cond_expr) => {
            should_wrap_callback(&cond_expr.consequent)
                || should_wrap_callback(&cond_expr.alternate)
        }
        Expression::CallExpression(call_expr) => {
            if let Some(member_expr) = call_expr.callee.get_member_expr()
                && let Some(prop_name) = member_expr.static_property_name()
                && prop_name == "bind"
            {
                return false;
            }

            true
        }
        Expression::SequenceExpression(seq_expr) => {
            seq_expr.expressions.last().is_none_or(|e| should_wrap_callback(e))
        }
        Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_)
        | Expression::Identifier(_)
        | Expression::YieldExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::NewExpression(_) => true,

        // These can't be callbacks, don't need to wrap
        _ => false,
    }
}

fn is_allowed_builtin(name: &str) -> bool {
    matches!(
        name,
        "String"
            | "Number"
            | "Boolean"
            | "Symbol"
            | "BigInt"
            | "RegExp"
            | "Date"
            | "Array"
            | "Object"
            | "Map"
            | "Set"
            | "WeakMap"
            | "WeakSet"
            | "Promise"
            | "Error"
            | "AggregateError"
            | "EvalError"
            | "RangeError"
            | "ReferenceError"
            | "SyntaxError"
            | "TypeError"
            | "URIError"
            | "Int8Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Int16Array"
            | "Uint16Array"
            | "Int32Array"
            | "Uint32Array"
            | "Float32Array"
            | "Float64Array"
            | "BigInt64Array"
            | "BigUint64Array"
            | "DataView"
            | "ArrayBuffer"
            | "SharedArrayBuffer"
    )
}

fn is_ignored_object(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            matches!(
                ident.name.as_str(),
                "Promise"
                    | "lodash"
                    | "underscore"
                    | "_"
                    | "React"
                    | "Vue"
                    | "Async"
                    | "async"
                    | "$"
                    | "jQuery"
                    | "Children"
                    | "types" // MobX State Tree and similar type libraries
            )
        }
        // Check for call expressions like $(this) or jQuery(...)
        Expression::CallExpression(call_expr) => {
            if let Expression::Identifier(ident) = call_expr.callee.without_parentheses() {
                matches!(ident.name.as_str(), "$" | "jQuery")
            } else {
                false
            }
        }
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            if let Expression::Identifier(obj_ident) = member_expr.object()
                && obj_ident.name == "React"
                && let Some(prop_name) = member_expr.static_property_name()
                && prop_name == "Children"
            {
                return true;
            }

            false
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.find(Boolean)",
        "foo.map(String)",
        "foo.map(Number)",
        "foo.map(BigInt)",
        "foo.map(Boolean)",
        "foo.map(Symbol)",
        "new foo.map(fn);",
        "map(fn);",
        "foo['map'](fn);",
        "foo[map](fn);",
        "foo.notListedMethod(fn);",
        "foo.map();",
        "foo.map(fn, extraArgument1, extraArgument2);",
        "foo.map(...argumentsArray)",
        "Promise.map(fn)",
        "Promise.forEach(fn)",
        "lodash.map(fn)",
        "underscore.map(fn)",
        "_.map(fn)",
        "Async.map(list, fn)",
        "async.map(list, fn)",
        "React.Children.forEach(children, fn)",
        "Children.forEach(children, fn)",
        "Vue.filter(name, fn)",
        "$(this).find(tooltip)",
        "$.map(realArray, function(value, index) {});",
        "$(this).filter(tooltip)",
        "jQuery(this).find(tooltip)",
        "jQuery.map(realArray, function(value, index) {});",
        "jQuery(this).filter(tooltip)",
        "foo.map(() => {})",
        "foo.map(function() {})",
        "foo.map(function bar() {})",
        "foo.map(function (a) {}.bind(bar))",
        "async function foo() {
				const clientId = 20
				const client = await oidc.Client.find(clientId)
			}",
        "const results = collection
				.find({
					$and: [cursorQuery, params.query]
				}, {
					projection: params.projection
				})
				.sort($sort)
				.limit(params.limit + 1)
				.toArray()",
        "const EventsStore = types.model('EventsStore', {
				events: types.optional(types.map(Event), {}),
			})",
        "foo.map(_ ? () => {} : _ ? () => {} : () => {})",
        "foo.reduce(_ ? () => {} : _ ? () => {} : () => {})",
        "foo.every(_ ? Boolean : _ ? Boolean : Boolean)",
        "foo.map(_ ? String : _ ? Number : Boolean)",
    ];

    let fail = vec![
        "bar.map(fn)",
        "bar.reduce(fn)",
        "foo.map(lib.fn)",
        "foo.reduce(lib.fn)",
        "foo.map(
				_
					? String // This one should be ignored
					: callback
			);",
        "foo.forEach(
				_
					? callbackA
					: _
							? callbackB
							: callbackC
			);",
        "async function * foo () {
				foo.map((0, bar));
				foo.map(yield bar);
				foo.map(yield* bar);
				foo.map(() => bar);
				foo.map(bar &&= baz);
				foo.map(bar || baz);
				foo.map(bar + bar);
				foo.map(+ bar);
				foo.map(++ bar);
				foo.map(new Function(''));
			}",
    ];

    Tester::new(NoArrayCallbackReference::NAME, NoArrayCallbackReference::PLUGIN, pass, fail)
        .test_and_snapshot();
}
