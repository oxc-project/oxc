use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectPropertyKind, PropertyKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::LintContext,
    rule::Rule,
    utils::{call_expr_member_expr_property_span, does_expr_match_any_path},
};

fn prefer_object_from_entries_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer 'Object.fromEntries' over manual object construction from entries")
        .with_help("Use 'Object.fromEntries(pairs)' instead of manually building objects with reduce or forEach")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferObjectFromEntries(Box<PreferObjectFromEntriesConfig>);

#[derive(Debug, Clone, Deserialize)]
pub struct PreferObjectFromEntriesConfig {
    functions: Vec<String>,
}

impl Default for PreferObjectFromEntriesConfig {
    fn default() -> Self {
        Self { functions: vec!["_.fromPairs".to_string(), "lodash.fromPairs".to_string()] }
    }
}

impl std::ops::Deref for PreferObjectFromEntries {
    type Target = PreferObjectFromEntriesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Encourages using `Object.fromEntries` when converting an array of key-value pairs
    /// into an object.
    ///
    /// ### Why is this bad?
    ///
    /// Manually constructing objects from key-value pairs using `reduce` or `forEach`
    /// is more verbose, error-prone, and harder to understand. The `Object.fromEntries`
    /// method is clearer, more declarative, and built for exactly this purpose.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const result = pairs.reduce((obj, [key, value]) => {
    ///   obj[key] = value;
    ///   return obj;
    /// }, {});
    ///
    /// const result = {};
    /// pairs.forEach(([key, value]) => {
    ///   result[key] = value;
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const result = Object.fromEntries(pairs);
    /// ```
    PreferObjectFromEntries,
    unicorn,
    style,
    pending
);

impl Rule for PreferObjectFromEntries {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        if call_expr.arguments.len() == 1
            && call_expr.arguments[0].is_expression()
            && does_expr_match_any_path(
                &call_expr.callee,
                self.functions.iter().map(|fun| fun.split('.').collect::<Vec<_>>()),
            )
        {
            ctx.diagnostic(prefer_object_from_entries_diagnostic(call_expr.callee.span()));
            return;
        }

        if !is_method_call(call_expr, None, Some(&["reduce"]), Some(2), Some(2)) {
            return;
        }

        if call_expr.optional || call_expr.callee.to_member_expression().optional() {
            return;
        }

        let Some(accumulator) = call_expr
            .arguments
            .get(1)
            .expect("call expr must have exactly 2 arguments")
            .as_expression()
            .map(oxc_ast::ast::Expression::get_inner_expression)
        else {
            return;
        };

        if !is_empty_object(accumulator) {
            return;
        }

        let Some(Expression::ArrowFunctionExpression(reducer)) = call_expr
            .arguments
            .first()
            .expect("call expr must have exactly 2 arguments")
            .as_expression()
            .map(oxc_ast::ast::Expression::get_inner_expression)
        else {
            return;
        };

        if !reducer.expression || reducer.r#async {
            return;
        }

        let Statement::ExpressionStatement(stmt) = reducer
            .body
            .statements
            .first()
            .expect("arrow function expressions must have at least one body statement")
        else {
            return;
        };
        let stmt = stmt.expression.get_inner_expression();

        let Some(accumulator_ident) =
            reducer.params.items.first().and_then(|arg| arg.pattern.get_binding_identifier())
        else {
            return;
        };

        if ctx.symbol_references(accumulator_ident.symbol_id()).count() != 1 {
            return;
        }

        // `() => Object.assign(object, {key})`
        if let Expression::CallExpression(call_expr) = &stmt {
            if !is_method_call(call_expr, Some(&["Object"]), Some(&["assign"]), Some(2), Some(2)) {
                return;
            }

            let Some(Expression::Identifier(target)) = call_expr
                .arguments
                .first()
                .expect("call expression should have 2 arguments")
                .as_expression()
                .map(oxc_ast::ast::Expression::get_inner_expression)
            else {
                return;
            };

            if ctx.scoping().get_reference(target.reference_id()).symbol_id()
                != Some(accumulator_ident.symbol_id())
            {
                return;
            }

            let Some(Expression::ObjectExpression(source)) = call_expr
                .arguments
                .get(1)
                .expect("call expression should have 2 arguments")
                .as_expression()
                .map(oxc_ast::ast::Expression::get_inner_expression)
            else {
                return;
            };

            if source.properties.len() != 1 {
                return;
            }

            let ObjectPropertyKind::ObjectProperty(k) = &source.properties[0] else { return };
            if k.kind != PropertyKind::Init || k.method {
                return;
            }

            ctx.diagnostic(prefer_object_from_entries_diagnostic(
                call_expr_member_expr_property_span(call_expr),
            ));
        }

        // `() => ({...object, key})`
        if let Expression::ObjectExpression(object_expr) = &stmt
            && object_expr.properties.len() == 2
            && let ObjectPropertyKind::SpreadProperty(spread) = &object_expr.properties[0]
            && let Expression::Identifier(spread_ident) = spread.argument.get_inner_expression()
        {
            let Some(spread_symbol_id) =
                ctx.scoping().get_reference(spread_ident.reference_id()).symbol_id()
            else {
                return;
            };

            if spread_symbol_id != accumulator_ident.symbol_id() {
                return;
            }
            let ObjectPropertyKind::ObjectProperty(object_prop) = &object_expr.properties[1] else {
                return;
            };

            if object_prop.kind != PropertyKind::Init || object_prop.method {
                return;
            }

            ctx.diagnostic(prefer_object_from_entries_diagnostic(
                call_expr_member_expr_property_span(call_expr),
            ));
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let config: PreferObjectFromEntriesConfig = value
            .as_array()
            .and_then(|arr| arr.first())
            .map(|config| {
                serde_json::from_value(config.clone()).expect("Failed to deserialize config")
            })
            .unwrap_or_default();

        Self(Box::new(config))
    }
}

fn is_empty_object(expr: &Expression) -> bool {
    match expr {
        Expression::ObjectExpression(o) if o.properties.is_empty() => true,
        Expression::CallExpression(call_expr)
            if is_method_call(
                call_expr,
                Some(&["Object"]),
                Some(&["create"]),
                Some(1),
                Some(1),
            ) && call_expr
                .arguments
                .first()
                .expect("call expression must have 1 argument")
                .as_expression()
                .map(oxc_ast::ast::Expression::get_inner_expression)
                .is_some_and(|expr| matches!(expr, Expression::NullLiteral(_))) =>
        {
            true
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("pairs.reduce(object => ({...object, key}));", None),
        ("pairs.reduce(object => ({...object, key}), {}, extraArgument);", None),
        ("pairs.reduce({}, object => ({...object, key}));", None),
        ("reduce(object => ({...object, key}), {});", None),
        ("new reduce(object => ({...object, key}), {});", None),
        ("pairs.reduce?.(object => ({...object, key}), {});", None),
        ("pairs?.reduce(object => ({...object, key}), {});", None),
        ("pairs.notReduce(object => ({...object, key}), {});", None),
        ("pairs.reduce(object => ({...object, key}), {notEmpty});", None),
        ("pairs.reduce(object => ({...object, key}), []);", None),
        ("pairs.reduce(object => ({...object, key}), {}, extraArgument);", None),
        ("pairs.reduce(...[(object => ({...object, key}))], {});", None),
        ("pairs.reduce(object => ({...object, key}), ...[{}]);", None),
        ("pairs.reduce(object => ({...object, key}), Object.create());", None),
        ("pairs.reduce(object => ({...object, key}), Object.create(null, extraArgument));", None),
        ("pairs.reduce(object => ({...object, key}), Object.create?.(null));", None),
        ("pairs.reduce(object => ({...object, key}), Object?.create(null));", None),
        ("pairs.reduce(object => ({...object, key}), window.Object.create(null));", None),
        ("pairs.reduce(object => ({...object, key}), Object.notCreate(null));", None),
        ("pairs.reduce(object => ({...object, key}), NotObject.create(null));", None),
        ("pairs.reduce(object => ({...object, key}), object.create(null));", None),
        ("pairs.reduce(object => ({...object, key}), object.CREATE(null));", None),
        (r#"pairs.reduce(object => ({...object, key}), Object.create("null"));"#, None),
        ("pairs.reduce(callback, {})", None),
        ("pairs.reduce(callback, Object.create(null))", None),
        ("pairs.reduce(async function * () {}, {})", None),
        ("pairs.reduce()", None),
        ("pairs.reduce(callback, {}, extraArgument)", None),
        ("pairs.reduce?.(callback, {})", None),
        ("pairs?.reduce(callback, {})", None),
        ("pairs.notReduce(callback, {})", None),
        ("pairs[reduce](callback, {})", None),
        ("pairs.reduce(...callback, {})", None),
        ("pairs.reduce(function(object) {Object.assign(object, {key})}, {});", None),
        ("pairs.reduce(object => ({...object, key} + 1), {});", None),
        //("pairs.reduce((object = {}) => ({...object, key}), {});", None),
        ("pairs.reduce((object) => ({...NotSameObject, key}), {});", None),
        ("pairs.reduce(object => ({...object, key, anotherKey}), {});", None),
        ("pairs.reduce(object => ({}), {});", None),
        ("pairs.reduce(object => ({keyFirst, ...object}), {});", None),
        ("pairs.reduce(async object => ({...object, key}), {});", None),
        ("pairs.reduce(async object => await {...object, key}, {});", None),
        ("pairs.reduce((...object) => ({...object, key}), {});", None),
        ("pairs.reduce(({object}) => ({...object, key}), {});", None),
        ("pairs.reduce(object => ({...object, ...key}), {});", None),
        ("pairs.reduce(object => Object.assign(NotSameObject, {key}), {});", None),
        ("pairs.reduce(object => Object.assign(object, {}), {});", None),
        ("pairs.reduce(object => Object.assign(object, {...key}), {});", None),
        ("pairs.reduce(object => Object.assign?.(object, {key}), {});", None),
        ("pairs.reduce(object => Object?.assign(object, {key}), {});", None),
        ("pairs.reduce(object => Object.notAssign(object, {key}), {});", None),
        ("pairs.reduce(object => NotObject.assign(object, {key}), {});", None),
        ("pairs.reduce(object => ({...object, object}), {});", None),
        ("pairs.reduce(object => ({...object, key: Object.keys(object)}), {});", None),
        ("pairs.reduce((object, [key, value = object]) => ({...object, [key]: value}), {});", None),
        ("pairs.reduce((object) => Object.assign(object, {object}), {});", None),
        ("pairs.reduce(object => ({...object, key: function () { return object; }}), {});", None),
        ("pairs.reduce(object => ({...object, method() {}}), {});", None),
        ("pairs.reduce(object => Object.assign(object, {async * method() {}}), {});", None),
        ("pairs.reduce(object => ({...object, async method() {}}), {});", None),
        ("pairs.reduce(object => ({...object, * method() {}}), {});", None),
        ("pairs.reduce(object => ({...object, async * method() {}}), {});", None),
        ("pairs.reduce(object => ({...object, get key() {}}), {});", None),
        ("pairs.reduce(object => ({...object, set key(v) {}}), {});", None),
        (
            "const flattened = arrayOfObjects.reduce((flattened, next) => Object.assign(flattened, next), {});",
            None,
        ),
        ("underscore.fromPairs(pairs)", None),
        ("_.fromPairs", None),
        ("_.fromPairs()", None),
        ("new _.fromPairs(pairs)", None),
        ("_.fromPairs(...[pairs])", None),
        ("_.foo(pairs)", Some(serde_json::json!([{"functions": ["foo"]}]))),
        ("foo(pairs)", Some(serde_json::json!([{"functions": ["utils.object.foo"]}]))),
        ("object.foo(pairs)", Some(serde_json::json!([{"functions": ["utils.object.foo"]}]))),
    ];

    let fail = vec![
        ("pairs.reduce(object => ({...object, key}), {});", None),
        ("pairs.reduce(object => ({...object, key}), {},);", None),
        ("pairs.reduce(object => ({...object, key,}), {});", None),
        ("pairs.reduce(object => ({...object, key}), Object.create(null));", None),
        ("pairs.reduce(object => ({...object, key}), Object.create(null),);", None),
        ("pairs.reduce(object => ({...object, key}), (( {} )));", None),
        ("pairs.reduce(object => ({...object, key}), (( Object.create(null) )),);", None),
        ("pairs.reduce( (( object => ({...object, key}) )) , {});", None),
        ("pairs.reduce( (( (object) => ({...object, key}) )) , {});", None),
        ("pairs.reduce( (( (object,) => ({...object, key}) )) , {});", None),
        ("pairs.reduce(object => ({...object, [((key))] : ((value))}), {});", None),
        (
            "((
				(( pairs ))
				.reduce(
					((
						(object,) => ((
							((
								Object
							)).assign(
								((
									object
								)),
								(({
									[ ((key)) ] : ((value)),
								}))
							)
						))
					)),
					Object.create(((null)),)
				)
			));",
            None,
        ),
        ("pairs.reduce(object => ({...object, 0: value}), {});", None),
        ("pairs.reduce(object => ({...object, true: value}), {});", None),
        ("pairs.reduce(object => ({...object, 0n: value}), {});", None),
        ("pairs.reduce(object => ({...object, undefined: value}), {});", None),
        ("pairs.reduce(object => ({...object, null: value}), {});", None),
        ("pairs.reduce(object => ({...object, var: value}), {});", None),
        ("pairs.reduce(object => ({...object, for: value}), {});", None),
        ("pairs.reduce(object => ({...object, default: value}), {});", None),
        ("pairs.reduce(object => ({...object, string: value}), {});", None),
        (r#"pairs.reduce(object => ({...object, "string": value}), {});"#, None),
        ("pairs.reduce(object => ({...object, [0]: value}), {});", None),
        ("pairs.reduce(object => ({...object, [true]: value}), {});", None),
        ("pairs.reduce(object => ({...object, [0n]: value}), {});", None),
        ("pairs.reduce(object => ({...object, [undefined]: value}), {});", None),
        ("pairs.reduce(object => ({...object, [null]: value}), {});", None),
        (r#"pairs.reduce(object => ({...object, ["for"]: value}), {});"#, None),
        ("pairs.reduce(object => ({...object, [string]: value}), {});", None),
        (r#"pairs.reduce(object => ({...object, ["string"]: value}), {});"#, None),
        ("pairs.reduce(object => Object.assign(object, {key}), {});", None),
        ("pairs.reduce(object => Object.assign(object, {key,}), {});", None),
        ("pairs.reduce(object => Object.assign(object, {[key]: value,}), {});", None),
        ("pairs.reduce((object, element, index, array) => ({...object, key}), {});", None),
        (
            "pairs.reduce((object, [key, value], index, array,) => ({...object, [key]: value + index + array.length}), {});",
            None,
        ),
        (
            "pairs.reduce(object => ({...object, key: function (object) { return object; }}), {});",
            None,
        ),
        ("pairs.reduce(object => ({...object, method: async () => {}}), {});", None),
        ("pairs.reduce(object => ({...object, method: async function * (){}}), {});", None),
        ("_.fromPairs(pairs)", None),
        ("lodash.fromPairs(pairs)", None),
        (
            "myFromPairsFunction(pairs)",
            Some(serde_json::json!([{"functions": ["myFromPairsFunction"]}])),
        ),
        ("utils.object.foo(pairs)", Some(serde_json::json!([{"functions": ["utils.object.foo"]}]))),
    ];

    Tester::new(PreferObjectFromEntries::NAME, PreferObjectFromEntries::PLUGIN, pass, fail)
        .test_and_snapshot();
}
