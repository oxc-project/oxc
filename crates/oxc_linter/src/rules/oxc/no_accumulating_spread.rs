use oxc_ast::{
    ast::{Argument, BindingPatternKind, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum NoAccumulatingSpreadDiagnostic {
    #[error("oxc(no-accumulating-spread): Do not spread accumulators in Array.prototype.reduce()")]
    #[diagnostic(severity(warning), help("It looks like you're spreading an `Array`. Consider using the `Array.push` or `Array.concat` methods to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity."))]
    LikelyArray(#[label("From this spread")] Span, #[label("For this reduce")] Span),
    #[error("oxc(no-accumulating-spread): Do not spread accumulators in Array.prototype.reduce()")]
    #[diagnostic(severity(warning), help("It looks like you're spreading an `Object`. Consider using the `Object.assign` or assignment operators to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity."))]
    LikelyObject(#[label("From this spread")] Span, #[label("For this reduce")] Span),
    #[error("oxc(no-accumulating-spread): Do not spread accumulators in Array.prototype.reduce()")]
    #[diagnostic(severity(warning), help("Consider using `Object.assign()` or `Array.prototype.push()` to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity."))]
    Unknown(#[label("From this spread")] Span, #[label("For this reduce")] Span),
}

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
            params.items.first().and_then(|item| get_identifier_symbol_id(&item.pattern.kind));
        if !first_param_symbol_id.is_some_and(|id| id == referenced_symbol_id) {
            return;
        }

        // invalid number of parameters to reduce callback
        let params_count = params.parameters_count();
        if params_count != 2 {
            return;
        }

        // Check if the declaration resides within a call to reduce()
        for parent in nodes.iter_parents(declaration.id()) {
            if let AstKind::CallExpression(call_expr) = parent.kind() {
                if is_method_call(
                    call_expr,
                    None,
                    Some(&["reduce", "reduceRight"]),
                    Some(1),
                    Some(2),
                ) {
                    ctx.diagnostic(get_diagnostic(call_expr, spread.span));
                }
                return;
            }
        }
    }
}

fn get_diagnostic<'a>(
    call_expr: &'a CallExpression<'a>,
    spread_span: Span,
) -> NoAccumulatingSpreadDiagnostic {
    // unwrap is safe because we already checked that this is a reduce call
    let (reduce_call_span, _) = call_expr_method_callee_info(call_expr).unwrap();

    if let Some(Argument::Expression(second_arg)) = call_expr.arguments.get(1) {
        let second_arg = second_arg.without_parenthesized();
        let second_arg =
            if let Expression::TSAsExpression(as_expr) = second_arg.without_parenthesized() {
                as_expr.expression.without_parenthesized()
            } else {
                second_arg
            };

        if matches!(second_arg, Expression::ObjectExpression(_)) {
            return NoAccumulatingSpreadDiagnostic::LikelyObject(spread_span, reduce_call_span);
        } else if matches!(second_arg, Expression::ArrayExpression(_)) {
            return NoAccumulatingSpreadDiagnostic::LikelyArray(spread_span, reduce_call_span);
        }
    }

    NoAccumulatingSpreadDiagnostic::Unknown(spread_span, reduce_call_span)
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
        // source: https://github.com/biomejs/biome/blob/main/crates/biome_js_analyze/tests/specs/performance/noAccumulatingSpread/valid.jsonc#L3C1-L23C52
        "foo.reduce((acc, bar) => {acc.push(bar); return acc;}, [])",
        "foo.reduceRight((acc, bar) => {acc.push(bar); return acc;}, [])",
        // Array - Allow spreading the item into the accumulator
        "foo.reduce((acc, bar) => {acc.push(...bar); return acc;}, [])",
        "foo.reduceRight((acc, bar) => {acc.push(...bar); return acc;}, [])",
        // Object - Allow setting an attribute on the accumulator
        "foo.reduce((acc, bar) => {acc[bar.key] = bar.value; return acc;}, {})",
        "foo.reduceRight((acc, bar) => {acc[bar.key] = bar.value; return acc;}, {})",
        // Object - Allow spreading the item into the accumulator
        "foo.reduce((acc, bar) => {acc[bar.key] = { ...bar.value }; return acc;}, {})",
        "foo.reduceRight((acc, bar) => {acc[bar.key] = { ...bar.value }; return acc;}, {})",
        // Callbacks with wrong number of parameters
        "foo.reduce((acc,value,index,array,somethingExtra) => [...acc, value], [])",
        "foo.reduce((acc) => [...acc], [])",
        // Wrong number of arguments to known method (reduce can have 1 or 2 args, but not more)
        "foo.reduce((acc, bar) => [...acc, bar], [], 123)",
    ];

    let fail = vec![
        "Object.keys(obj).reduce((acc, key) => ({ ...acc, [key]: obj[key] }), {})",
        // check we get the correct diagnostic for parenthesized expressions + as
        "Object.keys(obj).reduce((acc, key) => ({ ...acc, [key]: obj[key] }), ({} as foo))",
        "Object.keys(obj).reduce((acc, key) => ({ ...acc, [key]: obj[key] }), foo)",
        "arr.reduce((acc, x) => ({ ...acc, [x]: x }), {})",
        "arr.reduce((differentName, x) => ({ ...differentName, [x]: x }), {})",
        "a.b.arr.reduce((acc, x) => ({ ...acc, [x]: x }), {})",
        // check we get the correct diagnostic for parenthesized expressions
        "a.b.arr.reduce((acc, x) => ({ ...acc, [x]: x }), (({} as baz)))",
        "a.b.arr.reduce((acc, x) => ({ ...acc, [x]: x }), (({})))",
        "a.b.c.d.reduce((acc,x) => ([...acc, x]), [])",
        "a.b.c.d.reduce((acc,x) => ([...acc, x]), ([]))",
        "a.b.c.d.reduce((acc,x) => ([...acc, x]), ([] as foo))",
        "a.b.c.d.reduce((acc,x) => ([...acc, x]), (([]) as foo))",
        "get_array().reduce((acc, x) => ({ ...acc, [x]: x }), {})",
        "arr.reduce(function (acc, x) { return { ...acc, [x]: x } }, {})",
        "arr.reduce((acc, x) => {
            let temp = { ...acc, x }
            return temp
        }, {})",
        // source https://github.com/biomejs/biome/blob/main/crates/biome_js_analyze/tests/specs/performance/noAccumulatingSpread/invalid.jsonc#L2-L32
        // Array - Arrow return
        "foo.reduce((acc, bar) => [...acc, bar], [])",
        "foo.reduceRight((acc, bar) => [...acc, bar], [])",
        // Array - Body return
        "foo.reduce((acc, bar) => {return [...acc, bar];}, [])",
        "foo.reduceRight((acc, bar) => {return [...acc, bar];}, [])",
        // Array - Arrow return with item spread
        "foo.reduce((acc, bar) => [...acc, ...bar], [])",
        "foo.reduceRight((acc, bar) => [...acc, ...bar], [])",
        // Array - Body return with item spread
        "foo.reduce((acc, bar) => {return [...acc, ...bar];}, [])",
        "foo.reduceRight((acc, bar) => {return [...acc, ...bar];}, [])",
        // Object - Arrow return
        "foo.reduce((acc, bar) => ({...acc, [bar.key]: bar.value}), {})",
        "foo.reduceRight((acc, bar) => ({...acc, [bar.key]: bar.value}), {})",
        // Object - Body return
        "foo.reduce((acc, bar) => {return {...acc, [bar.key]: bar.value};}, {})",
        "foo.reduceRight((acc, bar) => {return {...acc, [bar.key]: bar.value};}, {})",
        // Object - Arrow return with item spread
        "foo.reduce((acc, bar) => ({...acc, ...bar}), {})",
        "foo.reduceRight((acc, bar) => ({...acc, ...bar}), {})",
        // Object - Body return with item spread
        "foo.reduce((acc, bar) => {return {...acc, ...bar};}, {})",
        "foo.reduceRight((acc, bar) => {return {...acc, ...bar};}, {})",
    ];

    Tester::new_without_config(NoAccumulatingSpread::NAME, pass, fail).test_and_snapshot();
}
