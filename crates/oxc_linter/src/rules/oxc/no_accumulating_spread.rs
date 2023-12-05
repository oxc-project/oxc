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
    /// `O(n^2)` time complexity.
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
    perf,
);

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
            if let AstKind::CallExpression(call_expr) = parent.kind() {
                if is_method_call(call_expr, None, Some(&["reduce"]), Some(1), Some(2)) {
                    ctx.diagnostic(NoAccumulatingSpreadDiagnostic(spread.span));
                }
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
        // Source: https://github.com/microsoft/vscode/blob/3481f35b91afff6c93d4888a528318d4f9f01a16/src/vs/workbench/contrib/extensions/browser/extensionEditor.ts#L1299-L1303 (MIT license)
        // testing: view `result` (accumulator) is not spread`
        r"
        const views = Object.keys(contrib).reduce((result, location) => {
			const viewsForLocation: IView[] = contrib[location];
			result.push(...viewsForLocation.map(view => ({ ...view, location })));
			return result;
		}, [] as Array<{ id: string; name: string; location: string }>);
        ",
        // Source https://github.com/microsoft/vscode/blob/4bb9f7f4f8bb39a8b07aefe5fe87f09cae10f533/src/vs/platform/policy/common/policy.ts#L51-L53 (MIT license)
        // testing: incorrect number of args to `reduce`
        r"
        export abstract class AbstractPolicyService
            extends Disposable
            implements IPolicyService
        {
            serialize(): IStringDictionary<{
                definition: PolicyDefinition;
                value: PolicyValue;
            }> {
                return Iterable.reduce<
                    [PolicyName, PolicyDefinition],
                    IStringDictionary<{ definition: PolicyDefinition; value: PolicyValue }>
                >(
                    Object.entries(this.policyDefinitions),
                    (r, [name, definition]) => ({
                        ...r,
                        [name]: { definition, value: this.policies.get(name)! },
                    }),
                    {}
                );
            }
        }
        ",
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
