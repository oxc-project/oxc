use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, rule::Rule,
    utils::does_expr_match_any_path,
};

fn no_array_method_this_argument_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using 'thisArg' with array iteration methods")
        .with_help("Use arrow functions or lexical scoping instead of passing 'thisArg' as a second argument to array methods like map, filter, etc.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayMethodThisArgument;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of the `thisArg` parameter in array iteration methods such as
    /// `map`, `filter`, `some`, `every`, and similar.
    ///
    /// ### Why is this bad?
    ///
    /// The `thisArg` parameter makes code harder to understand and reason about. Instead,
    /// prefer arrow functions or bind explicitly in a clearer way. Arrow functions inherit
    /// `this` from the lexical scope, which is more intuitive and less error-prone.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// array.map(function(x) { return x + this.y }, this);
    /// array.filter(function(x) { return x !== this.value }, this);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// array.map(x => x + this.y);
    /// array.filter(x => x !== this.value);
    /// const self = this;
    /// array.map(function(x) { return x + self.y });
    /// ```
    NoArrayMethodThisArgument,
    unicorn,
    style,
    pending
);

impl Rule for NoArrayMethodThisArgument {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        check_array_prototype_methods(call_expr, ctx);
        check_array_from(call_expr, ctx);
    }
}

fn check_array_prototype_methods(call_expr: &CallExpression, ctx: &LintContext) {
    if !is_method_call(
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
        Some(2),
        Some(2),
    ) || call_expr
        .arguments
        .first()
        .is_some_and(|arg| arg.as_expression().is_none_or(|expr| is_node_not_function(expr)))
        || call_expr.arguments.get(1).is_some_and(|arg| matches!(arg, Argument::SpreadElement(_)))
        || does_expr_match_any_path(&call_expr.callee, IGNORED)
    {
        return;
    }

    ctx.diagnostic(no_array_method_this_argument_diagnostic(
        call_expr.arguments.first().map_or(call_expr.span, GetSpan::span),
    ));
}

fn check_array_from(call_expr: &CallExpression, ctx: &LintContext) {
    if !is_method_call(call_expr, Some(&["Array"]), Some(&["from", "fromAsync"]), Some(3), Some(3))
        || call_expr.arguments.first().is_some_and(|arg| matches!(arg, Argument::SpreadElement(_)))
        || call_expr.arguments.get(2).is_some_and(|arg| matches!(arg, Argument::SpreadElement(_)))
        || call_expr
            .arguments
            .get(1)
            .is_some_and(|arg| arg.as_expression().is_none_or(|expr| is_node_not_function(expr)))
    {
        return;
    }

    ctx.diagnostic(no_array_method_this_argument_diagnostic(
        call_expr.arguments.get(2).map_or(call_expr.span, GetSpan::span),
    ));
}

fn is_node_not_function(expr: &Expression) -> bool {
    match expr {
        Expression::ArrayExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NullLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::NewExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::ThisExpression(_) => true,
        Expression::Identifier(ident) if ident.name == "undefined" => true,
        Expression::CallExpression(call_expr) => {
            !is_method_call(call_expr, None, Some(&["bind"]), None, None)
        }
        _ => false,
    }
}

const IGNORED: &[&[&str]] = &[
    &["lodash", "every"],
    &["_", "every"],
    &["underscore", "every"],
    &["lodash", "filter"],
    &["_", "filter"],
    &["underscore", "filter"],
    &["Vue", "filter"],
    &["R", "filter"],
    &["lodash", "find"],
    &["_", "find"],
    &["underscore", "find"],
    &["R", "find"],
    &["lodash", "findLast"],
    &["_", "findLast"],
    &["underscore", "findLast"],
    &["R", "findLast"],
    &["lodash", "findIndex"],
    &["_", "findIndex"],
    &["underscore", "findIndex"],
    &["R", "findIndex"],
    &["lodash", "findLastIndex"],
    &["_", "findLastIndex"],
    &["underscore", "findLastIndex"],
    &["R", "findLastIndex"],
    &["lodash", "flatMap"],
    &["_", "flatMap"],
    &["lodash", "forEach"],
    &["_", "forEach"],
    &["React", "Children", "forEach"],
    &["Children", "forEach"],
    &["R", "forEach"],
    &["lodash", "map"],
    &["_", "map"],
    &["underscore", "map"],
    &["React", "Children", "map"],
    &["Children", "map"],
    &["jQuery", "map"],
    &["$", "map"],
    &["R", "map"],
    &["lodash", "some"],
    &["_", "some"],
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.unknownMethod(() => {}, thisArgument)",
        "new array.map(() => {}, thisArgument)",
        "Array.unknownMethod(iterableOrArrayLike, () => {}, thisArgument)",
        "new Array.from(iterableOrArrayLike, () => {}, thisArgument)",
        "NotArray.from(iterableOrArrayLike, () => {}, thisArgument)",
        "new Array.fromAsync(iterableOrArrayLike, () => {}, thisArgument)",
        "NotArray.fromAsync(iterableOrArrayLike, () => {}, thisArgument)",
        "array.map()",
        "array.map(() => {},)",
        "array.map(() => {}, ...thisArgument)",
        "array.map(...() => {}, thisArgument)",
        "array.map(() => {}, thisArgument, extraArgument)",
        "Array.from()",
        "Array.from(iterableOrArrayLike)",
        "Array.from(iterableOrArrayLike, () => {},)",
        "Array.from(iterableOrArrayLike, () => {}, ...thisArgument)",
        "Array.from(iterableOrArrayLike, ...() => {}, thisArgument)",
        "Array.from(...iterableOrArrayLike, () => {}, thisArgument)",
        "Array.from(iterableOrArrayLike, () => {}, thisArgument, extraArgument)",
        "Array.fromAsync()",
        "Array.fromAsync(iterableOrArrayLike)",
        "Array.fromAsync(iterableOrArrayLike, () => {},)",
        "Array.fromAsync(iterableOrArrayLike, () => {}, ...thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, ...() => {}, thisArgument)",
        "Array.fromAsync(...iterableOrArrayLike, () => {}, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, () => {}, thisArgument, extraArgument)",
        "lodash.every(array, () => {})",
        "lodash.find(array, () => {})",
        "jQuery.map(array, () => {})",
        "$.map(array, () => {})",
        "React.Children.map(children, () => {})",
        "Children.map(children, () => {})",
        "React.Children.forEach(children, () => {})",
        "Children.forEach(children, () => {})",
        r#"Vue.filter("capitalize", () => {})"#,
        "R.filter(() => {}, [])",
        "R.find(() => {}, [])",
        "R.findIndex(() => {}, [])",
        "R.forEach(() => {}, [])",
        "R.map(() => {}, [])",
        r#"$( "li" ).filter( ":nth-child(2n)" ).css( "background-color", "red" );"#,
        r#"$( "li.item-ii" ).find( "li" ).css( "background-color", "red" );"#,
        "array.map(new Callback, thisArgument)",
        "array.map(1, thisArgument)",
        "async () => array.map(await callback, thisArgument)",
        "Array.from(iterableOrArrayLike, new Callback, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, new Callback, thisArgument)",
        "Array.from(iterableOrArrayLike, 1, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, 1, thisArgument)",
        "Array.from(iterableOrArrayLike, await callback, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, await callback, thisArgument)",
    ];

    let fail = vec![
        "array.every(() => {}, thisArgument)",
        "array.filter(() => {}, thisArgument)",
        "array.find(() => {}, thisArgument)",
        "array.findIndex(() => {}, thisArgument)",
        "array.findLast(() => {}, thisArgument)",
        "array.findLastIndex(() => {}, thisArgument)",
        "array.flatMap(() => {}, thisArgument)",
        "array.forEach(() => {}, thisArgument)",
        "array.map(() => {}, thisArgument)",
        "Array.from(iterableOrArrayLike, () => {}, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, () => {}, thisArgument)",
        "array.map(() => {}, thisArgument,)",
        "array.map(() => {}, (0, thisArgument),)",
        "Array.from(iterableOrArrayLike, () => {}, thisArgument,)",
        "Array.fromAsync(iterableOrArrayLike, () => {}, thisArgument,)",
        "array.map(() => {}, thisArgumentHasSideEffect())",
        "Array.from(iterableOrArrayLike, () => {}, thisArgumentHasSideEffect())",
        "Array.fromAsync(iterableOrArrayLike, () => {}, thisArgumentHasSideEffect())",
        "array.map(callback, thisArgument)",
        "Array.from(iterableOrArrayLike, callback, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, callback, thisArgument)",
        "array.map(callback, (0, thisArgument))",
        "Array.from(iterableOrArrayLike, callback, (0, thisArgument))",
        "Array.fromAsync(iterableOrArrayLike, callback, (0, thisArgument))",
        "array.map(function () {}, thisArgument)",
        "Array.from(iterableOrArrayLike, function () {}, thisArgument)",
        "array.map(function callback () {}, thisArgument)",
        "Array.from(iterableOrArrayLike, function callback () {}, thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, function callback () {}, thisArgument)",
        "array.map( foo as bar, (( thisArgument )),)", // {"parser": parsers.typescript},
        "Array.from(iterableOrArrayLike, foo as bar, (( thisArgument )),)", // {"parser": parsers.typescript},
        "Array.fromAsync(iterableOrArrayLike, foo as bar, (( thisArgument )),)", // {"parser": parsers.typescript},
        "array.map( (( foo as bar )), (( thisArgument )),)", // {"parser": parsers.typescript},
        "Array.from(iterableOrArrayLike, (( foo as bar )), (( thisArgument )),)", // {"parser": parsers.typescript},
        "Array.fromAsync(iterableOrArrayLike, (( foo as bar )), (( thisArgument )),)", // {"parser": parsers.typescript},
        "array.map( (( 0, callback )), (( thisArgument )),)",
        "Array.from(iterableOrArrayLike, (( 0, callback )), (( thisArgument )),)",
        "Array.fromAsync(iterableOrArrayLike, (( 0, callback )), (( thisArgument )),)",
        "array.map((0, () => {}), thisArgument)",
        "Array.from(iterableOrArrayLike, (0, () => {}), thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, (0, () => {}), thisArgument)",
        "array.map(callback.bind(foo), thisArgument)",
        "Array.from(iterableOrArrayLike, callback.bind(foo), thisArgument)",
        "Array.fromAsync(iterableOrArrayLike, callback.bind(foo), thisArgument)",
    ];

    Tester::new(NoArrayMethodThisArgument::NAME, NoArrayMethodThisArgument::PLUGIN, pass, fail)
        .test_and_snapshot();
}
