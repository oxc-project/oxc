use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentExpression, AssignmentTarget, BindingPattern, CallExpression,
        Expression, ForInStatement, ForOfStatement, ForStatement, IdentifierReference,
        SimpleAssignmentTarget, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
};

fn reduce_likely_array_spread_diagnostic(spread_span: Span, reduce_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in Array.prototype.reduce()")
        .with_help("It looks like you're spreading an `Array`. Consider using the `Array.push` or `Array.concat` methods to mutate the accumulator instead.")
        .with_note("Using spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            spread_span.label("From this spread"),
            reduce_span.label("For this reduce")
        ])
}

fn reduce_likely_object_spread_diagnostic(spread_span: Span, reduce_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in Array.prototype.reduce()")
        .with_help("It looks like you're spreading an `Object`. Consider using the `Object.assign` or assignment operators to mutate the accumulator instead.")
        .with_note("Using spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            spread_span.label("From this spread"),
            reduce_span.label("For this reduce")
        ])
}

fn reduce_unknown(spread_span: Span, reduce_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in Array.prototype.reduce()")
        .with_help("Consider using `Object.assign()` or `Array.prototype.push()` to mutate the accumulator instead.")
        .with_note("Using spreads within accumulators leads to `O(n^2)` time complexity.")
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
        .with_help("Consider using `Object.assign()` to mutate the accumulator instead.")
        .with_note("Using spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            accumulator_decl_span.label("From this accumulator"),
            spread_span.label("From this spread"),
            loop_span.primary_label("For this loop"),
        ])
}
fn loop_spread_likely_array_diagnostic(
    accumulator_decl_span: Span,
    spread_span: Span,
    loop_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not spread accumulators in loops")
        .with_help("Consider using `Array.prototype.push()` to mutate the accumulator instead.")
        .with_note("Using spreads within accumulators leads to `O(n^2)` time complexity.")
        .with_labels([
            accumulator_decl_span.label("From this accumulator"),
            spread_span.label("From this spread"),
            loop_span.primary_label("For this loop"),
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
    version = "0.0.19",
    short_description = "Prevents using object or array spreads on accumulators in `Array.prototype.reduce()` and in loops.",
);

impl Rule for NoAccumulatingSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SpreadElement(spread) = node.kind() else {
            return;
        };
        // Accepts both direct accumulator spreads (`...acc`) and spreads of a member access
        // rooted at the accumulator (`...acc[key]`, `...acc.foo`), since both still copy an
        // ever-growing structure derived from the accumulator on every iteration.
        let Some(ident) = get_root_identifier(&spread.argument) else {
            return;
        };

        let symbols = ctx.scoping();

        // get the AST node + symbol id of the declaration of the identifier
        let reference = symbols.get_reference(ident.reference_id());
        let Some(referenced_symbol_id) = reference.symbol_id() else {
            return;
        };
        let declaration_id = symbols.symbol_declaration(referenced_symbol_id);
        let declaration = ctx.nodes().parent_node(declaration_id);

        check_reduce_usage(declaration, referenced_symbol_id, spread.span, node.id(), ctx);
        check_loop_usage(
            declaration,
            ctx.nodes().get_node(declaration_id),
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
    spread_node_id: NodeId,
    ctx: &LintContext<'a>,
) {
    let AstKind::FormalParameters(params) = declaration.kind() else {
        return;
    };

    // We're only looking for the first parameter, since that's where acc is. This also matches
    // when the accumulator itself is destructured, e.g. `({ a, b }, item) => ({ ...a, ...item })`,
    // since `a`/`b` are still parts of the accumulator.
    let Some(first_param) = params.items.first() else {
        return;
    };
    if !binding_pattern_contains_symbol(&first_param.pattern, referenced_symbol_id) {
        return;
    }

    // Note: extra unused params (beyond accumulator, currentValue, currentIndex, array) are
    // still valid JS and don't change reduce's behavior, so we don't restrict the param count
    // here. The checks above (first param is the accumulator) and below (call is reduce/
    // reduceRight with 1-2 args) are sufficient to confirm this is a real reduce callback.

    // The callback must be a direct argument to reduce()/reduceRight() - stop at the nearest
    // enclosing call, since anything further up (e.g. an unrelated outer .reduce() that merely
    // contains this callback lexically, as in `arr.reduce((r, x) => { ...; other.map(cb) })`)
    // does not make `cb` a reduce callback.
    let Some(call_expr_node) = ctx
        .nodes()
        .ancestors(declaration.id())
        .find(|n| matches!(n.kind(), AstKind::CallExpression(_)))
    else {
        return;
    };
    let AstKind::CallExpression(call_expr) = call_expr_node.kind() else { unreachable!() };

    if is_method_call(call_expr, None, Some(&["reduce", "reduceRight"]), Some(1), Some(2))
        && ctx
            .nodes()
            .ancestors(spread_node_id)
            .take_while(|n| !n.kind().span().contains_inclusive(declaration.span()))
            .all(|n| {
                !matches!(n.kind(), AstKind::ArrowFunctionExpression(_) | AstKind::Function(_))
            })
    {
        ctx.diagnostic(get_reduce_diagnostic(call_expr, spread_span));
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
    if !matches!(declaration.kind, VariableDeclarationKind::Let) {
        return;
    }
    let AstKind::VariableDeclarator(declarator) = declarator.kind() else {
        return;
    };

    let Some(assignment_expr) =
        find_assignment_expression(spread_node_id, referenced_symbol_id, ctx)
    else {
        return;
    };

    let Some(expression_type) =
        get_spread_containing_expression_type(&assignment_expr.right, spread_span)
    else {
        return;
    };

    emit_loop_diagnostic_if_in_loop(
        spread_node_id,
        declarator.id.span(),
        spread_span,
        declaration.span,
        expression_type,
        ctx,
    );
}

/// Find the nearest enclosing assignment expression whose target resolves back to
/// `referenced_symbol_id`, either directly (`foo = ...`) or via a member access rooted at it
/// (`foo.list = ...`, `foo[key] = ...`).
fn find_assignment_expression<'a>(
    spread_node_id: NodeId,
    referenced_symbol_id: SymbolId,
    ctx: &LintContext<'a>,
) -> Option<&'a AssignmentExpression<'a>> {
    for parent in ctx.nodes().ancestors(spread_node_id) {
        if let AstKind::AssignmentExpression(expr) = parent.kind()
            && is_assignment_to_symbol(&expr.left, referenced_symbol_id, ctx)
        {
            return Some(expr);
        }
    }
    None
}

/// Check if the assignment target is our referenced symbol, either directly or via a member
/// access rooted at it (e.g. `foo.list = ...` when `foo` is the referenced symbol).
fn is_assignment_to_symbol(
    assignment_target: &AssignmentTarget,
    referenced_symbol_id: SymbolId,
    ctx: &LintContext,
) -> bool {
    let scoping = ctx.semantic().scoping();
    let root_ident = match assignment_target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => ident,
        _ => {
            let Some(member_expr) = assignment_target
                .as_simple_assignment_target()
                .and_then(SimpleAssignmentTarget::as_member_expression)
            else {
                return false;
            };
            let Some(ident) = get_root_identifier(member_expr.object()) else {
                return false;
            };
            ident
        }
    };
    let reference = scoping.get_reference(root_ident.reference_id());
    reference.symbol_id() == Some(referenced_symbol_id)
}

#[derive(Debug, Clone, Copy)]
enum SpreadExpressionType {
    Array,
    Object,
}

/// Determine if the expression contains the spread and return its type
fn get_spread_containing_expression_type(
    expr: &Expression,
    spread_span: Span,
) -> Option<SpreadExpressionType> {
    let inner_expr = expr.get_inner_expression();
    match inner_expr {
        Expression::ArrayExpression(array_expr)
            if array_expr.span.contains_inclusive(spread_span) =>
        {
            Some(SpreadExpressionType::Array)
        }
        Expression::ObjectExpression(object_expr)
            if object_expr.span.contains_inclusive(spread_span) =>
        {
            Some(SpreadExpressionType::Object)
        }
        _ => None,
    }
}

/// Emit appropriate diagnostic if the spread is within a loop
fn emit_loop_diagnostic_if_in_loop(
    spread_node_id: NodeId,
    declarator_span: Span,
    spread_span: Span,
    declaration_span: Span,
    expression_type: SpreadExpressionType,
    ctx: &LintContext,
) {
    for parent in ctx.nodes().ancestors(spread_node_id) {
        if let Some(loop_span) = get_loop_span(parent.kind()) {
            let parent_span = parent.kind().span();
            if !parent_span.contains_inclusive(declaration_span)
                && parent_span.contains_inclusive(spread_span)
            {
                match expression_type {
                    SpreadExpressionType::Array => {
                        ctx.diagnostic(loop_spread_likely_array_diagnostic(
                            declarator_span,
                            spread_span,
                            loop_span,
                        ));
                    }
                    SpreadExpressionType::Object => {
                        ctx.diagnostic(loop_spread_likely_object_diagnostic(
                            declarator_span,
                            spread_span,
                            loop_span,
                        ));
                    }
                }
                return;
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

/// Returns whether any identifier bound directly within `pattern` is `symbol_id`, e.g. `a` and
/// `b` in `({ a, b }) => ...` when destructuring the accumulator.
fn binding_pattern_contains_symbol(pattern: &BindingPattern<'_>, symbol_id: SymbolId) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => ident.symbol_id() == symbol_id,
        BindingPattern::AssignmentPattern(assignment) => {
            binding_pattern_contains_symbol(&assignment.left, symbol_id)
        }
        BindingPattern::ObjectPattern(object) => {
            object
                .properties
                .iter()
                .any(|prop| binding_pattern_contains_symbol(&prop.value, symbol_id))
                || object
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binding_pattern_contains_symbol(&rest.argument, symbol_id))
        }
        BindingPattern::ArrayPattern(array) => {
            array
                .elements
                .iter()
                .flatten()
                .any(|elem| binding_pattern_contains_symbol(elem, symbol_id))
                || array
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binding_pattern_contains_symbol(&rest.argument, symbol_id))
        }
    }
}

/// Walks through a chain of member expressions (and optional chains, e.g. `acc?.[key]`) to find
/// the root identifier, e.g. `acc` in `acc[key].prop`. Returns `None` if the root isn't an
/// identifier (e.g. `foo().bar`).
fn get_root_identifier<'a, 'b>(expr: &'b Expression<'a>) -> Option<&'b IdentifierReference<'a>> {
    let mut expr = expr.get_inner_expression();
    loop {
        match expr {
            Expression::Identifier(ident) => return Some(ident),
            Expression::ChainExpression(chain) => {
                let member_expr = chain.expression.member_expression()?;
                expr = member_expr.object().get_inner_expression();
            }
            _ => {
                let member_expr = expr.as_member_expression()?;
                expr = member_expr.object().get_inner_expression();
            }
        }
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
        "function doSomething(list) { return list.reduce((acc, each) => { return each.subList.flatMap((subEach) => { return [...acc, subEach.subList] }) }, []) }",
        // Destructured accumulator: spreading an unrelated (non-accumulator) binding is fine
        "arr.reduce(({ a }, x) => ({ ...x }), { a: 1 })",
        // Optional-chained member access on an unrelated (non-accumulator) variable is fine
        "foo.reduce((acc, bar) => ({ ...obj?.[bar.key] }), {})",
        // Loop: mutating an unrelated variable's property isn't an accumulating spread on `foo`
        "let foo = {}; let bar = []; for (let i = 0; i < 10; i++) { foo.list = [...bar, i]; }",
        // A spread inside a nested callback's own param (e.g. a `.map()` callback within a
        // reduce callback's body) isn't an accumulating spread just because it's lexically
        // inside a reduce call - `view` here belongs to `.map()`, not `.reduce()`.
        r"
        const views = Object.keys(contrib).reduce((result, location) => {
            const viewsForLocation: IView[] = contrib[location];
            result.push(...viewsForLocation.map(view => ({ ...view, location })));
            return result;
        }, [] as Array<{ id: string; name: string; location: string }>);
        ",
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
        // Callbacks using the currentIndex / array params should still be caught
        "foo.reduce((acc, bar, index) => [...acc, bar], [])",
        "foo.reduce((acc, bar, index, array) => [...acc, bar], [])",
        // Destructured non-accumulator params should still be caught
        "collaboratorInfo.reduce((acc, { formId, userAccountId }) => { return { ...acc, [formId]: userAccountId }; }, {})",
        "foo.reduce((acc, bar, index) => ({...acc, [index]: bar}), {})",
        // Extra unused params don't prevent this from being a real reduce callback
        "foo.reduce((acc,value,index,array,somethingExtra) => [...acc, value], [])",
        "foo.reduce((acc) => [...acc], [])",
        // Spreading a member access rooted at the accumulator is still an accumulating spread
        "foo.reduce((acc, bar) => { acc[bar.key] = [...acc[bar.key], bar.value]; return acc; }, {})",
        "foo.reduce((acc, bar) => { acc[bar.key] = { ...acc[bar.key], ...bar.value }; return acc; }, {})",
        "foo.reduce((acc, bar) => { acc.list = [...acc.list, bar]; return acc; }, { list: [] })",
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
        // Destructured accumulator: spreading a part of the destructured accumulator is still
        // an accumulating spread
        "arr.reduce(({ list }, x) => ({ list: [...list, x] }), { list: [] })",
        "arr.reduce(({ a, ...rest }, x) => ({ ...rest, x }), { a: 1 })",
        "arr.reduce(([first, ...others], x) => [...others, x], [])",
        // Optional chaining on the spread target is still an accumulating spread
        "foo.reduce((acc, bar) => ({ ...acc?.[bar.key] }), {})",
        "foo.reduce((acc, bar) => ({ ...acc?.nested }), { nested: {} })",
        // Loop: mutating the accumulator through a member expression is still an
        // accumulating spread
        "let foo = { list: [] }; for (let i = 0; i < 10; i++) { foo.list = [...foo.list, i]; }",
        "let foo = { list: [] }; for (let i = 0; i < 10; i++) { foo['list'] = [...foo['list'], i]; }",
    ];

    Tester::new(NoAccumulatingSpread::NAME, NoAccumulatingSpread::PLUGIN, pass, fail)
        .test_and_snapshot();
}
