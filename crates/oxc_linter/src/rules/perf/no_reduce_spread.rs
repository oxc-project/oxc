use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("perf(no-reduce-spread): Do not spread accumulators in Array.prototype.reduce()")]
#[diagnostic(severity(warning), help("Prefer property assignment or Array.prototype.push()"))]
struct NoReduceSpreadDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoReduceSpread;

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
    NoReduceSpread,
    nursery
);

const REDUCE: &str = "reduce";
impl Rule for NoReduceSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SpreadElement(spread) = node.kind() else { return };
        let Expression::Identifier(ref ident) = spread.argument else { return };
        let nodes = ctx.semantic().nodes();
        let symbols = ctx.semantic().symbols();

        let Some(reference_id) = ident.reference_id.get() else { return };
        let reference = symbols.get_reference(reference_id);
        let Some(referenced_symbol_id) = reference.symbol_id() else { return };
        let declaration_id = symbols.get_declaration(referenced_symbol_id);
        let declaration = ctx.semantic().nodes().get_node(declaration_id);
        let AstKind::FormalParameters(params) = declaration.kind() else { return };

        let first_param_symbol_id =
            params.items.get(0).map(|item| get_identifier_symbol_id(&item.pattern.kind)).flatten();
        if !first_param_symbol_id.is_some_and(|id| id == referenced_symbol_id) {
            return;
        }
        for parent in nodes.iter_parents(declaration.id()) {
            if is_call_to_reduce(parent) {
                ctx.diagnostic(NoReduceSpreadDiagnostic(spread.span));
                return;
            }
        }
    }
}

fn get_identifier_symbol_id<'a>(ident: &BindingPatternKind<'a>) -> Option<SymbolId> {
    match ident {
        BindingPatternKind::BindingIdentifier(ident) => ident.symbol_id.get(),
        BindingPatternKind::AssignmentPattern(ident) => get_identifier_symbol_id(&ident.left.kind),
        _ => None,
    }
}
fn is_call_to_reduce(node: &AstNode<'_>) -> bool {
    let AstKind::CallExpression(call) = node.kind() else { return false };
    // only check calls to reduce()
    let Expression::MemberExpression(member_expr) = &call.callee else { return false };
    member_expr.static_property_name() == Some(REDUCE)
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

    Tester::new_without_config(NoReduceSpread::NAME, pass, fail).test_and_snapshot();
}
