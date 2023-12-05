use oxc_ast::{
    ast::{BindingPatternKind, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("oxc(no-accumulating-spread): Do not spread accumulators in Array.prototype.reduce()")]
#[diagnostic(severity(warning), help("Consider using `Object.assign()` or `Array.prototype.concat()` to mutate the accumulator instead. Using spreads within accumulators leads to `O(n^2)` time complexity."))]
struct NoAccumulatingSpreadDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAccumulatingSpread;

declare_oxc_lint!(
    /// ### What it does
    /// Prevents using object or array spreads on accumulators in `Array.prototype.reduce()`.
    ///
    /// ### Why is this bad?
    /// Object and array spreads create a new object or array on each iteration.
    /// In the worst case, they also cause O(n) copies (both memory and time complexity).
    /// When used on an accumulator, this can lead to `O(n^2)` memory complexity and
    /// `Ω(n)`/`O(n^2)` time complexity.
    ///
    /// For a more in-depth explanation, see this [blog post](https://prateeksurana.me/blog/why-using-object-spread-with-reduce-bad-idea/)
    /// by Prateek Surana.
    ///
    ///
    /// ### Example
    /// Pass
    /// ```javascript
    /// function fn (x) {
    ///   // ...
    /// }
    ///   
    /// arr.reduce((acc, x) => acc.push(fn(x)), [])
    /// Object.keys(obj).reduce((acc, el) => {
    ///   acc[el] = fn(el)
    /// }, {})
    /// // spreading non-accumulators should be avoided if possible, but is not
    /// // banned by this rule
    /// Object.keys(obj).reduce((acc, el) => {
    ///   acc[el] = { ...obj[el] }
    ///   return acc
    /// }, {})
    /// ```
    ///
    /// Fail
    /// ```javascript
    /// arr.reduce((acc, x) => ({ ...acc, [x]: fn(x) }), {})
    /// Object.keys(obj).reduce((acc, el) => ({ ...acc, [el]: fn(el) }), {})
    /// ```
    NoAccumulatingSpread,
    nursery
);

const REDUCE: &str = "reduce";
impl Rule for NoAccumulatingSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // only check spreads on identifiers
        let AstKind::SpreadElement(spread) = node.kind() else { return };
        let Expression::Identifier(ref ident) = spread.argument else { return };

        let nodes = ctx.semantic().nodes();
        let symbols = ctx.semantic().symbols();

        // get the AST node + symbol id of the declaration of the identifier
        let Some(reference_id) = ident.reference_id.get() else { return };
        let reference = symbols.get_reference(reference_id);
        let Some(referenced_symbol_id) = reference.symbol_id() else { return };
        let declaration_id = symbols.get_declaration(referenced_symbol_id);
        let declaration = ctx.semantic().nodes().get_node(declaration_id);
        let AstKind::FormalParameters(params) = declaration.kind() else { return };

        // We're only looking for the first parameter, since that's where acc is.
        // Skip non-parameter or non-first-parameter declarations.
        let first_param_symbol_id =
            params.items.get(0).and_then(|item| get_identifier_symbol_id(&item.pattern.kind));
        if !first_param_symbol_id.is_some_and(|id| id == referenced_symbol_id) {
            return;
        }

        // Check if the declaration resides within a call to reduce()
        for parent in nodes.iter_parents(declaration.id()) {
            if matches!(parent.kind(), AstKind::CallExpression(call_expr) if
            is_method_call(call_expr, None, Some(&[REDUCE]), None, None) )
            {
                ctx.diagnostic(NoAccumulatingSpreadDiagnostic(spread.span));
                return;
            }
        }
    }
}

fn get_identifier_symbol_id(ident: &BindingPatternKind<'_>) -> Option<SymbolId> {
    match ident {
        BindingPatternKind::BindingIdentifier(ident) => ident.symbol_id.get(),
        BindingPatternKind::AssignmentPattern(ident) => get_identifier_symbol_id(&ident.left.kind),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let x = { ...a }",
        "let x = [ ...a ]",
        "[1,2,3].map(n => ({ ...obj, n }))",
        "arr.map(function (x) { return { ...obj, [x]: x } }, {})",
        "[...a, ...b]",
        "[...a, ...b].reduce((acc, x) => { acc['foo'] = x; return acc }, {})",
        "arr.reduce((acc, x) => { acc['foo'] = x; return acc }, { ...a, ...b })",
        "let x = { ...a, ...b }; arr.reduce((x, y) => x + y, 0)",
        "arr.reduce((acc, x) => ({ ...x }), {})",
        "arr.reduce((acc, x) => ({ ...obj }), {})",
        // bad practice, but not a spread on acc
        "arr.reduce((acc, x) => {
                acc[x] = { ...x }
                return acc
            }, {})",
    ];

    let fail = vec![
        "Object.keys(obj).reduce((acc, key) => ({ ...acc, [key]: obj[key] }), {})",
        "arr.reduce((acc, x) => ({ ...acc, [x]: x }), {})",
        "arr.reduce((differentName, x) => ({ ...differentName, [x]: x }), {})",
        "a.b.arr.reduce((acc, x) => ({ ...acc, [x]: x }), {})",
        "get_array().reduce((acc, x) => ({ ...acc, [x]: x }), {})",
        "arr.reduce(function (acc, x) { return { ...acc, [x]: x } }, {})",
        "arr.reduce((acc, x) => {
            let temp = { ...acc, x }
            return temp
        }, {})",
    ];

    Tester::new_without_config(NoAccumulatingSpread::NAME, pass, fail).test_and_snapshot();
}
