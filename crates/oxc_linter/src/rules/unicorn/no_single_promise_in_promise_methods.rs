use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn no_single_promise_in_promise_methods_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Wrapping single-element array with `Promise.{method_name}()` is unnecessary."
    ))
    .with_help("Either use the value directly, or switch to `Promise.resolve(…)`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSinglePromiseInPromiseMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow passing single-element arrays to `Promise` methods.
    ///
    /// ### Why is this bad?
    ///
    /// Passing a single-element array to `Promise.all()`, `Promise.any()`, or
    /// `Promise.race()` is likely a mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async function bad() {
    ///   const foo = await Promise.all([promise]);
    ///   const foo = await Promise.any([promise]);
    ///   const foo = await Promise.race([promise]);
    ///   const promise = Promise.all([nonPromise]);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function good() {
    ///   const foo = await promise;
    ///   const promise = Promise.resolve(nonPromise);
    ///   const foo = await Promise.all(promises);
    ///   const foo = await Promise.any([promise, anotherPromise]);
    ///   const [{ value: foo, reason: error }] = await Promise.allSettled([promise]);
    /// }
    /// ```
    NoSinglePromiseInPromiseMethods,
    unicorn,
    correctness,
    conditional_fix
);

impl Rule for NoSinglePromiseInPromiseMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_promise_method_with_single_argument(call_expr) {
            return;
        }
        let Some(first_argument) = call_expr.arguments[0].as_expression() else {
            return;
        };
        let first_argument = first_argument.get_inner_expression();
        let Expression::ArrayExpression(first_argument_array_expr) = first_argument else {
            return;
        };

        if first_argument_array_expr.elements.len() != 1 {
            return;
        }

        let first = &first_argument_array_expr.elements[0];
        if !first.is_expression() {
            return;
        }

        let (span, method_name) = call_expr
            .callee
            .get_member_expr()
            .expect("callee is a member expression")
            .static_property_info()
            .expect("callee is a static property");

        let diagnostic = no_single_promise_in_promise_methods_diagnostic(span, method_name);

        let await_expr_span = ctx
            .nodes()
            // get first non-parenthesis parent node
            .ancestors(node.id())
            .map(AstNode::kind)
            .find(|kind| !is_ignorable_kind(kind))
            // check if it's an `await ...` expression
            .and_then(|kind| {
                if let AstKind::AwaitExpression(await_expr) = kind {
                    Some(await_expr.span)
                } else {
                    None
                }
            });

        if await_expr_span.is_none() && method_name == "all" {
            return ctx.diagnostic(diagnostic);
        }

        if is_fixable(node.id(), ctx) {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                let elem_text = fixer.source_range(first.span());
                let call_span = call_expr.span;

                if let Some(await_span) = await_expr_span {
                    if method_name == "all" {
                        fixer.replace(await_span, format!("[await {elem_text}]"))
                    } else {
                        fixer.replace(call_span, elem_text.to_owned())
                    }
                } else {
                    fixer.replace(call_span, format!("Promise.resolve({elem_text})"))
                }
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

fn is_promise_method_with_single_argument(call_expr: &CallExpression) -> bool {
    if !is_method_call(
        call_expr,
        Some(&["Promise"]),
        Some(&["all", "any", "race"]),
        Some(1),
        Some(1),
    ) || call_expr.optional
    {
        return false;
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    !member_expr.optional() && !member_expr.is_computed()
}

fn is_fixable(call_node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    for parent in ctx.nodes().ancestors(call_node_id) {
        match parent.kind() {
            AstKind::CallExpression(_)
            | AstKind::VariableDeclarator(_)
            | AstKind::AssignmentExpression(_)
            | AstKind::ReturnStatement(_) => return false,
            AstKind::AwaitExpression(_) => {}
            kind if is_ignorable_kind(&kind) => {}
            _ => return true,
        }
    }
    true
}

/// We want to skip:
///
/// - parenthesis
/// - TS type modification expressions (as const, satisfies, non-null
///   assertions, etc)
fn is_ignorable_kind(kind: &AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSInstantiationExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.race([promise, anotherPromise])",
        "Promise.race(notArrayLiteral)",
        "Promise.race([...promises])",
        "Promise.any([promise, anotherPromise])",
        "Promise.race([promise, anotherPromise])",
        "Promise.notListedMethod([promise])",
        "Promise[race]([promise])",
        "Promise.race([,])",
        "NotPromise.race([promise])",
        "Promise?.race([promise])",
        "Promise.race?.([promise])",
        "Promise.race(...[promise])",
        "Promise.race([promise], extraArguments)",
        "Promise.race()",
        "new Promise.race([promise])",
        // We are not checking these cases
        "globalThis.Promise.race([promise])",
        r#"Promise["race"]([promise])"#,
        // This can't be checked
        "Promise.allSettled([promise])",
    ];

    let fail = vec![
        // `await`ed
        "await Promise.race([(0, promise)])",
        "async function * foo() {await Promise.race([yield promise])}",
        "async function * foo() {await Promise.race([yield* promise])}",
        "await Promise.race([() => promise,],)",
        "await Promise.race([a ? b : c,],)",
        "await Promise.race([x ??= y,],)",
        "await Promise.race([x ||= y,],)",
        "await Promise.race([x &&= y,],)",
        "await Promise.race([x |= y,],)",
        "await Promise.race([x ^= y,],)",
        "await Promise.race([x ??= y,],)",
        "await Promise.race([x ||= y,],)",
        "await Promise.race([x &&= y,],)",
        "await Promise.race([x | y,],)",
        "await Promise.race([x ^ y,],)",
        "await Promise.race([x & y,],)",
        "await Promise.race([x !== y,],)",
        "await Promise.race([x == y,],)",
        "await Promise.race([x in y,],)",
        "await Promise.race([x >>> y,],)",
        "await Promise.race([x + y,],)",
        "await Promise.race([x / y,],)",
        "await Promise.race([x ** y,],)",
        "await Promise.race([promise,],)",
        "await Promise.race([getPromise(),],)",
        "await Promise.race([promises[0],],)",
        "await Promise.race([await promise])",
        "await Promise.any([promise])",
        "await Promise.race([promise])",
        "await Promise.race([new Promise(() => {})])",
        "+await Promise.race([+1])",
        // ASI, `Promise.race()` is not really `await`ed
        "
        await Promise.race([(x,y)])
        [0].toString()
        ",
        // Not `await`ed
        "Promise.race([promise,],)",
        "
        foo
        Promise.race([(0, promise),],)
        ",
        "
        foo
        Promise.race([[array][0],],)
        ",
        "Promise.race([promise]).then()",
        "Promise.race([1]).then()",
        "Promise.race([1.]).then()",
        "Promise.race([.1]).then()",
        "Promise.race([(0, promise)]).then()",
        "const _ = () => Promise.race([ a ?? b ,],)",
        "Promise.race([ {a} = 1 ,],)",
        "Promise.race([ function () {} ,],)",
        "Promise.race([ class {} ,],)",
        "Promise.race([ new Foo ,],).then()",
        "Promise.race([ new Foo ,],).toString",
        "foo(Promise.race([promise]))",
        "Promise.race([promise]).foo = 1",
        "Promise.race([promise])[0] ||= 1",
        "Promise.race([undefined]).then()",
        "Promise.race([null]).then()",
        // `Promise.all` specific
        "Promise.all([promise])",
        "await Promise.all([promise])",
        "const foo = () => Promise.all([promise])",
        "const foo = await Promise.all([promise])",
        "foo = await Promise.all([promise])",
        // `Promise.{all, race}()` should not care if the result is used
        "const foo = await Promise.race([promise])",
        "const foo = () => Promise.race([promise])",
        "foo = await Promise.race([promise])",
        "const results = await Promise.any([promise])",
        "const results = await Promise.race([promise])",
        // Fixable, but not provided at this point
        "const [foo] = await Promise.all([promise])",
        // TypeScript-specific
        "Promise.all([x] as const).then()",
        "Promise.all([x] satisfies any[]).then()",
        "Promise.all([x as const]).then()",
        "Promise.all([x!]).then()",
        "Promise.all(['one']).then(something);",
    ];

    let fix = vec![
        ("Promise.race([null]).then()", "Promise.resolve(null).then()", None),
        // Promise.all returns an array, so we preserve the array structure
        // `await Promise.all([x])` -> `[await x]`
        ("await Promise.all([x]);", "[await x];", None),
        ("await Promise.all([x as Promise<number>]);", "[await x as Promise<number>];", None),
        ("while(true) { await Promise.all([x]); }", "while(true) { [await x]; }", None),
        // Promise.any and Promise.race return a single value, not an array
        ("await Promise.any([x]);", "await x;", None),
        ("await Promise.race([x]);", "await x;", None),
        ("const foo = await Promise.all([x])", "const foo = await Promise.all([x])", None),
        ("const [foo] = await Promise.all([x])", "const [foo] = await Promise.all([x])", None),
        ("let foo; foo = await Promise.all([x])", "let foo; foo = await Promise.all([x])", None),
        (
            "function foo () { return Promise.all([x]); }",
            "function foo () { return Promise.all([x]); }",
            None,
        ),
        ("const foo = () => Promise.race([x])", "const foo = () => Promise.resolve(x)", None),
        ("foo = await Promise.race([x])", "foo = await Promise.race([x])", None),
        (
            "Promise.all(['one']).then((result) => result[0]);",
            "Promise.all(['one']).then((result) => result[0]);",
            None,
        ),
    ];

    Tester::new(
        NoSinglePromiseInPromiseMethods::NAME,
        NoSinglePromiseInPromiseMethods::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path_extension("mts")
    .expect_fix(fix)
    .test_and_snapshot();
}
