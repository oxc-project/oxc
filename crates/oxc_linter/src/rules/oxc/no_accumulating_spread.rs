use oxc_ast::{
    ast::{
        Argument, BindingPatternKind, CallExpression, Expression, ForInStatement, ForOfStatement,
        ForStatement, VariableDeclarationKind,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
    AstNode,
};

fn reduce_likely_array_spread_diagnostic(spread_span: Span, reduce_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in Array.prototype.reduce()")
        .with_help("It looks like you're spreading an `Array`. Consider using the `Array.push` or `Array.concat` methods to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            spread_span.label("From this spread"),
            reduce_span.label("For this reduce")
        ])
}

fn reduce_likely_object_spread_diagnostic(spread_span: Span, reduce_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in Array.prototype.reduce()")
        .with_help("It looks like you're spreading an `Object`. Consider using the `Object.assign` or assignment operators to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            spread_span.label("From this spread"),
            reduce_span.label("For this reduce")
        ])
}

fn reduce_unknown(spread_span: Span, reduce_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in Array.prototype.reduce()")
        .with_help("Consider using `Object.assign()` or `Array.prototype.push()` to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            spread_span.label("From this spread"),
            reduce_span.label("For this reduce")
        ])
}

fn loop_spread_likely_object_diagnostic(
    accumulator_decl_span: Span,
    spread_span: Span,
    loop_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in loops")
        .with_help("Consider using `Object.assign()` to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            accumulator_decl_span.label("From this accumulator"),
            spread_span.label("From this spread"),
            loop_span.label("For this loop")
        ])
}
fn loop_spread_likely_array_diagnostic(
    accumulator_decl_span: Span,
    spread_span: Span,
    loop_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in loops")
        .with_help("Consider using `Array.prototype.push()` to mutate the accumulator instead.\nUsing spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            accumulator_decl_span.label("From this accumulator"),
            spread_span.label("From this spread"),
            loop_span.label("For this loop")
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoAccumulatingSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents using object or array spreads on accumulators in `Array.prototype.reduce()` and in loops.
    ///
    /// ### Why is this bad?
    ///
    /// Object and array spreads create a new object or array on each iteration.
    /// In the worst case, they also cause O(n) copies (both memory and time complexity).
    /// When used on an accumulator, this can lead to `O(n^2)` memory complexity and
    /// `O(n^2)` time complexity.
    ///
    /// For a more in-depth explanation, see this [blog post](https://prateeksurana.me/blog/why-using-object-spread-with-reduce-bad-idea/)
    /// by Prateek Surana.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// arr.reduce((acc, x) => ({ ...acc, [x]: fn(x) }), {})
    /// Object.keys(obj).reduce((acc, el) => ({ ...acc, [el]: fn(el) }), {})
    ///
    /// let foo = []; for (let i = 0; i < 10; i++) { foo = [...foo, i]; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
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
    ///
    /// let foo = []; for (let i = 0; i < 10; i++) { foo.push(i); }
    /// ```
    NoAccumulatingSpread,
    oxc,
    perf,
);

impl Rule for NoAccumulatingSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // only check spreads on identifiers
        let AstKind::SpreadElement(spread) = node.kind() else {
            return;
        };
        let Expression::Identifier(ident) = &spread.argument else {
            return;
        };

        let symbols = ctx.semantic().symbols();

        // get the AST node + symbol id of the declaration of the identifier
        let reference = symbols.get_reference(ident.reference_id());
        let Some(referenced_symbol_id) = reference.symbol_id() else {
            return;
        };
        let declaration_id = symbols.get_declaration(referenced_symbol_id);
        let Some(declaration) = ctx.semantic().nodes().parent_node(declaration_id) else {
            return;
        };

        check_reduce_usage(declaration, referenced_symbol_id, spread.span, ctx);
        check_loop_usage(
            declaration,
            ctx.semantic().nodes().get_node(declaration_id),
            referenced_symbol_id,
            node.id(),
            spread.span,
            ctx,
        );
    }
}

fn check_reduce_usage<'a>(
    declaration: &AstNode<'a>,
    referenced_symbol_id: SymbolId,
    spread_span: Span,
    ctx: &LintContext<'a>,
) {
    let AstKind::FormalParameters(params) = declaration.kind() else {
        return;
    };

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
    for parent in ctx.nodes().ancestors(declaration.id()) {
        if let AstKind::CallExpression(call_expr) = parent.kind() {
            if is_method_call(call_expr, None, Some(&["reduce", "reduceRight"]), Some(1), Some(2)) {
                ctx.diagnostic(get_reduce_diagnostic(call_expr, spread_span));
            }
            return;
        }
    }
}

fn check_loop_usage<'a>(
    declaration_node: &AstNode<'a>,
    declarator: &AstNode<'a>,
    referenced_symbol_id: SymbolId,
    spread_node_id: NodeId,
    spread_span: Span,
    ctx: &LintContext<'a>,
) {
    let AstKind::VariableDeclaration(declaration) = declaration_node.kind() else {
        return;
    };
    // if the accumulator's declaration is not a `let`, then we know it's never
    // reassigned, hence cannot be a violation of the rule
    if !matches!(declaration.kind, VariableDeclarationKind::Let) {
        return;
    }

    let AstKind::VariableDeclarator(declarator) = declarator.kind() else {
        return;
    };

    let Some(write_reference) =
        ctx.semantic().symbol_references(referenced_symbol_id).find(|r| r.is_write())
    else {
        return;
    };

    let Some(assignment_target) = ctx.nodes().parent_node(write_reference.node_id()) else {
        return;
    };

    let AstKind::SimpleAssignmentTarget(_) = assignment_target.kind() else { return };

    let Some(assignment_expr) = ctx.nodes().parent_node(assignment_target.id()) else { return };
    if !matches!(assignment_expr.kind(), AstKind::AssignmentTarget(_)) {
        return;
    }
    let Some(assignment) = ctx.nodes().parent_node(assignment_expr.id()) else { return };
    let AstKind::AssignmentExpression(assignment_expression) = assignment.kind() else {
        return;
    };

    let assignment_expression_right_inner_expr = assignment_expression.right.get_inner_expression();
    match assignment_expression_right_inner_expr {
        Expression::ArrayExpression(array_expr)
            if array_expr.span.contains_inclusive(spread_span) => {}
        Expression::ObjectExpression(object_expr)
            if object_expr.span.contains_inclusive(spread_span) => {}
        _ => return,
    }

    for parent in ctx.nodes().ancestors(spread_node_id) {
        if let Some(loop_span) = get_loop_span(parent.kind()) {
            if !parent.kind().span().contains_inclusive(declaration.span)
                && parent.kind().span().contains_inclusive(spread_span)
            {
                match assignment_expression_right_inner_expr {
                    Expression::ArrayExpression(_) => {
                        ctx.diagnostic(loop_spread_likely_array_diagnostic(
                            declarator.id.span(),
                            spread_span,
                            loop_span,
                        ));
                    }
                    Expression::ObjectExpression(_) => {
                        ctx.diagnostic(loop_spread_likely_object_diagnostic(
                            declarator.id.span(),
                            spread_span,
                            loop_span,
                        ));
                    }
                    // we check above that the expression is either an array or object expression
                    _ => unreachable!(),
                }
            }
        }
    }
}

fn get_loop_span(ast_kind: AstKind) -> Option<Span> {
    match ast_kind {
        AstKind::ForStatement(ForStatement { span, .. })
        | AstKind::ForOfStatement(ForOfStatement { span, .. })
        | AstKind::ForInStatement(ForInStatement { span, .. }) => Some(Span::sized(span.start, 3)),
        AstKind::WhileStatement(while_stmt) => Some(Span::sized(while_stmt.span.start, 5)),
        AstKind::DoWhileStatement(do_stmt) => Some(Span::sized(do_stmt.span.start, 2)),
        _ => None,
    }
}

fn get_reduce_diagnostic<'a>(
    call_expr: &'a CallExpression<'a>,
    spread_span: Span,
) -> OxcDiagnostic {
    // unwrap is safe because we already checked that this is a reduce call
    let (reduce_call_span, _) = call_expr_method_callee_info(call_expr).unwrap();

    if let Some(second_arg) = call_expr.arguments.get(1).and_then(Argument::as_expression) {
        let second_arg = second_arg.get_inner_expression();
        if matches!(second_arg, Expression::ObjectExpression(_)) {
            return reduce_likely_object_spread_diagnostic(spread_span, reduce_call_span);
        } else if matches!(second_arg, Expression::ArrayExpression(_)) {
            return reduce_likely_array_spread_diagnostic(spread_span, reduce_call_span);
        }
    }

    reduce_unknown(spread_span, reduce_call_span)
}

fn get_identifier_symbol_id(ident: &BindingPatternKind<'_>) -> Option<SymbolId> {
    match ident {
        BindingPatternKind::BindingIdentifier(ident) => Some(ident.symbol_id()),
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
        // source: https://github.com/biomejs/biome/blob/cli/v1.9.4/crates/biome_js_analyze/tests/specs/performance/noAccumulatingSpread/valid.jsonc#L3C1-L23C52
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
        // loops, array case
        "let foo = []; for (let i = 0; i < 10; i++) { foo.push(i); }",
        "let foo = []; for (const i = 0; i < 10; i++) { foo.push(i); }",
        "let foo = []; for (let i in [1,2,3]) { foo.push(i); }",
        "let foo = []; for (const i in [1,2,3]) { foo.push(i); }",
        "let foo = []; for (let i of [1,2,3]) { foo.push(i); }",
        "let foo = []; while (foo.length < 10) { foo.push(foo.length); }",
        // loops, object case
        "let foo = {}; for (let i = 0; i < 10; i++) { foo[i] = i; }",
        "let foo = {}; for (const i = 0; i < 10; i++) { foo[i] = i; }",
        "let foo = {}; for (let i in [1,2,3]) { foo[i] = i; }",
        "let foo = {}; for (const i in [1,2,3]) { foo[i] = i; }",
        "let foo = {}; for (let i of [1,2,3]) { foo[i] = i; }",
        "let foo = {}; for (const i of [1,2,3]) { foo[i] = i; }",
        "let foo = {}; while (Object.keys(foo).length < 10) { foo[Object.keys(foo).length] = Object.keys(foo).length; }",
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
        // source https://github.com/biomejs/biome/blob/cli/v1.9.4/crates/biome_js_analyze/tests/specs/performance/noAccumulatingSpread/invalid.jsonc#L2-L32
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
        // loops, array case
        "let foo = []; for (let i = 0; i < 10; i++) { foo = [...foo, i]; }",
        "let foo = []; for (const i = 0; i < 10; i++) { foo = [...foo, i]; }",
        "let foo = []; for (let i in [1,2,3]) { foo = [...foo, i]; }",
        "let foo = []; for (const i in [1,2,3]) { foo = [...foo, i]; }",
        "let foo = []; for (let i of [1,2,3]) { foo = [...foo, i]; }",
        "let foo = []; for (const i of [1,2,3]) { foo = [...foo, i]; }",
        "let foo = []; while (foo.length < 10) { foo = [...foo, foo.length]; }",
        // loops, object case
        "let foo = {}; for (let i = 0; i < 10; i++) { foo = { ...foo, [i]: i }; }",
        "let foo = {}; for (const i = 0; i < 10; i++) { foo = { ...foo, [i]: i }; }",
        "let foo = {}; for (let i in [1,2,3]) { foo = { ...foo, [i]: i }; }",
        "let foo = {}; for (const i in [1,2,3]) { foo = { ...foo, [i]: i }; }",
        "let foo = {}; for (let i of [1,2,3]) { foo = { ...foo, [i]: i }; }",
        "let foo = {}; for (const i of [1,2,3]) { foo = { ...foo, [i]: i }; }",
        "let foo = {}; while (Object.keys(foo).length < 10) { foo = { ...foo, [Object.keys(foo).length]: Object.keys(foo).length }; }",
    ];

    Tester::new(NoAccumulatingSpread::NAME, NoAccumulatingSpread::PLUGIN, pass, fail)
        .test_and_snapshot();
}
