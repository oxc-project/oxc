use std::borrow::Cow;

use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_expect_jest_fn_call, ExpectError, PossibleJestNode},
    AstNode,
};

fn valid_expect_diagnostic<S: Into<Cow<'static, str>>>(
    x1: S,
    x2: &'static str,
    span3: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(x1).with_help(x2).with_label(span3)
}

#[derive(Debug, Default, Clone)]
pub struct ValidExpect(Box<ValidExpectConfig>);

#[derive(Debug, Clone)]
pub struct ValidExpectConfig {
    async_matchers: Vec<String>,
    min_args: usize,
    max_args: usize,
    always_await: bool,
}

impl std::ops::Deref for ValidExpect {
    type Target = ValidExpectConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ValidExpectConfig {
    fn default() -> Self {
        Self {
            async_matchers: vec![String::from("toResolve"), String::from("toReject")],
            min_args: 1,
            max_args: 1,
            always_await: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers a warning if `expect()` is called with more than one argument
    /// or without arguments. It would also issue a warning if there is nothing called
    /// on `expect()`, e.g.:
    ///
    /// ### Example
    /// ```javascript
    /// expect();
    /// expect('something');
    /// expect(true).toBeDefined;
    /// expect(Promise.resolve('Hi!')).resolves.toBe('Hi!');
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/valid-expect.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/valid-expect": "error"
    ///   }
    /// }
    /// ```
    ValidExpect,
    correctness
);

impl Rule for ValidExpect {
    fn from_configuration(value: serde_json::Value) -> Self {
        let default_async_matchers = vec![String::from("toResolve"), String::from("toReject")];
        let config = value.get(0);

        let async_matchers = config
            .and_then(|config| config.get("asyncMatchers"))
            .and_then(serde_json::Value::as_array)
            .map_or(default_async_matchers, |v| {
                v.iter().filter_map(serde_json::Value::as_str).map(String::from).collect()
            });
        let min_args = config
            .and_then(|config| config.get("minArgs"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(1, |v| usize::try_from(v).unwrap_or(1));

        let max_args = config
            .and_then(|config| config.get("maxArgs"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(1, |v| usize::try_from(v).unwrap_or(1));

        let always_await = config
            .and_then(|config| config.get("alwaysAwait"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self(Box::new(ValidExpectConfig { async_matchers, min_args, max_args, always_await }))
    }

    fn run_on_jest_node<'a, 'b>(
        &self,
        jest_node: &PossibleJestNode<'a, 'b>,
        ctx: &'b LintContext<'a>,
    ) {
        self.run(jest_node, ctx);
    }
}

impl ValidExpect {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let reporting_span = jest_fn_call.expect_error.map_or(call_expr.span, |_| {
            find_top_most_member_expression(node, ctx).map_or(call_expr.span, GetSpan::span)
        });

        match jest_fn_call.expect_error {
            Some(ExpectError::MatcherNotFound) => {
                let (error, help) = Message::MatcherNotFound.details();
                ctx.diagnostic(valid_expect_diagnostic(error, help, reporting_span));
                return;
            }
            Some(ExpectError::MatcherNotCalled) => {
                let (error, help) = Message::MatcherNotCalled.details();
                ctx.diagnostic(valid_expect_diagnostic(error, help, reporting_span));
                return;
            }
            Some(ExpectError::ModifierUnknown) => {
                let (error, help) = Message::ModifierUnknown.details();
                ctx.diagnostic(valid_expect_diagnostic(error, help, reporting_span));
                return;
            }
            None => {}
        }

        let Some(Expression::CallExpression(call_expr)) = jest_fn_call.head.parent else {
            return;
        };

        if call_expr.arguments.len() > self.max_args {
            let error = format!(
                "Expect takes at most {} argument{} ",
                self.max_args,
                if self.max_args > 1 { "s" } else { "" }
            );
            let help = "Remove the extra arguments.";
            ctx.diagnostic(valid_expect_diagnostic(error, help, call_expr.span));
            return;
        }
        if call_expr.arguments.len() < self.min_args {
            let error = format!(
                "Expect requires at least {} argument{} ",
                self.min_args,
                if self.min_args > 1 { "s" } else { "" }
            );
            let help = "Add the missing arguments.";
            ctx.diagnostic(valid_expect_diagnostic(error, help, call_expr.span));
            return;
        }

        let Some(matcher) = jest_fn_call.matcher() else {
            return;
        };
        let Some(matcher_name) = matcher.name() else {
            return;
        };

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let should_be_awaited =
            jest_fn_call.modifiers().iter().any(|modifier| modifier.is_name_unequal("not"))
                || self.async_matchers.contains(&matcher_name.to_string());

        if ctx.nodes().parent_node(parent.id()).is_none() || !should_be_awaited {
            return;
        }

        // An async assertion can be chained with `then` or `catch` statements.
        // In that case our target CallExpression node is the one with
        // the last `then` or `catch` statement.
        let target_node = get_parent_if_thenable(node, ctx);
        let Some(final_node) = find_promise_call_expression_node(node, ctx, target_node) else {
            return;
        };
        let Some(parent) = ctx.nodes().parent_node(final_node.id()) else {
            return;
        };
        if !is_acceptable_return_node(parent, !self.always_await, ctx) {
            let span;
            let (error, help) = if target_node.id() == final_node.id() {
                let AstKind::CallExpression(call_expr) = target_node.kind() else {
                    return;
                };
                span = call_expr.span;
                Message::AsyncMustBeAwaited.details()
            } else {
                let AstKind::CallExpression(call_expr) = final_node.kind() else {
                    return;
                };
                span = call_expr.span;
                Message::PromisesWithAsyncAssertionsMustBeAwaited.details()
            };
            ctx.diagnostic(valid_expect_diagnostic(error, help, span));
        }
    }
}

fn find_top_most_member_expression<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b MemberExpression<'a>> {
    let mut top_most_member_expression = None;
    let mut node = node;

    loop {
        let parent = ctx.nodes().parent_node(node.id())?;
        match node.kind() {
            AstKind::MemberExpression(member_expr) => {
                top_most_member_expression = Some(member_expr);
            }
            _ => {
                if !matches!(parent.kind(), AstKind::MemberExpression(_)) {
                    break;
                }
            }
        }
        node = parent;
    }

    top_most_member_expression
}

fn is_acceptable_return_node<'a, 'b>(
    node: &'b AstNode<'a>,
    allow_return: bool,
    ctx: &'b LintContext<'a>,
) -> bool {
    let mut node = node;
    loop {
        if allow_return && matches!(node.kind(), AstKind::ReturnStatement(_)) {
            return true;
        }

        match node.kind() {
            AstKind::ConditionalExpression(_)
            | AstKind::Argument(_)
            | AstKind::ExpressionStatement(_)
            | AstKind::FunctionBody(_) => {
                let Some(parent) = ctx.nodes().parent_node(node.id()) else {
                    return false;
                };
                node = parent;
            }
            AstKind::ArrowFunctionExpression(arrow_expr) => return arrow_expr.expression,
            AstKind::AwaitExpression(_) => return true,
            _ => return false,
        }
    }
}

type ParentAndIsFirstItem<'a, 'b> = (&'b AstNode<'a>, bool);

// Returns the parent node of the given node, ignoring some nodes,
// and return whether the first item if parent is an array.
fn get_parent_with_ignore<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<ParentAndIsFirstItem<'a, 'b>> {
    let mut node = node;
    loop {
        let parent = ctx.nodes().parent_node(node.id())?;
        if !matches!(
            parent.kind(),
            AstKind::Argument(_)
                | AstKind::ExpressionArrayElement(_)
                | AstKind::ArrayExpressionElement(_)
        ) {
            // we don't want to report `Promise.all([invalidExpectCall_1, invalidExpectCall_2])` twice.
            // so we need mark whether the node is the first item of an array.
            // if it not the first item, we ignore it in `find_promise_call_expression_node`.
            if let AstKind::ArrayExpressionElement(array_expr_element) = node.kind() {
                if let AstKind::ArrayExpression(array_expr) = parent.kind() {
                    return Some((
                        parent,
                        array_expr.elements.first()?.span() == array_expr_element.span(),
                    ));
                }
            }

            // if parent is not an array, we assume it's the first item
            return Some((parent, true));
        }

        node = parent;
    }
}

fn find_promise_call_expression_node<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
    default_node: &'b AstNode<'a>,
) -> Option<&'b AstNode<'a>> {
    let Some((mut parent, is_first_array_item)) = get_parent_with_ignore(node, ctx) else {
        return Some(default_node);
    };
    if !matches!(parent.kind(), AstKind::CallExpression(_) | AstKind::ArrayExpression(_)) {
        return Some(default_node);
    }
    let Some((grandparent, _)) = get_parent_with_ignore(parent, ctx) else {
        return Some(default_node);
    };
    if matches!(parent.kind(), AstKind::ArrayExpression(_))
        && matches!(grandparent.kind(), AstKind::CallExpression(_))
    {
        parent = grandparent;
    }

    if let AstKind::CallExpression(call_expr) = parent.kind() {
        if let Some(member_expr) = call_expr.callee.as_member_expression() {
            if let Expression::Identifier(ident) = member_expr.object() {
                if matches!(ident.name.as_str(), "Promise")
                    && ctx.nodes().parent_node(parent.id()).is_some()
                {
                    if is_first_array_item {
                        return Some(parent);
                    }
                    return None;
                }
            }
        }
    }

    Some(default_node)
}

fn get_parent_if_thenable<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> &'b AstNode<'a> {
    let grandparent =
        ctx.nodes().parent_node(node.id()).and_then(|node| ctx.nodes().parent_node(node.id()));

    let Some(grandparent) = grandparent else {
        return node;
    };
    let AstKind::CallExpression(call_expr) = grandparent.kind() else {
        return node;
    };
    let Some(member_expr) = call_expr.callee.as_member_expression() else {
        return node;
    };
    let Some(name) = member_expr.static_property_name() else {
        return node;
    };

    if ["then", "catch"].contains(&name) {
        return get_parent_if_thenable(grandparent, ctx);
    }

    node
}

#[derive(Clone, Copy)]
enum Message {
    MatcherNotFound,
    MatcherNotCalled,
    ModifierUnknown,
    AsyncMustBeAwaited,
    PromisesWithAsyncAssertionsMustBeAwaited,
}

impl Message {
    fn details(self) -> (&'static str, &'static str) {
        match self {
            Self::MatcherNotFound => (
                "Expect must have a corresponding matcher call.",
                "Did you forget add a matcher(e.g. `toBe`, `toBeDefined`)",
            ),
            Self::MatcherNotCalled => (
                "Matchers must be called to assert.",
                "You need call your matcher, e.g. `expect(true).toBe(true)`.",
            ),
            Self::ModifierUnknown => {
                ("Expect has an unknown modifier.", "Is it a spelling mistake?")
            }
            Self::AsyncMustBeAwaited => {
                ("Async assertions must be awaited.", "Add `await` to your assertion.")
            }
            Self::PromisesWithAsyncAssertionsMustBeAwaited => (
                "Promises which return async assertions must be awaited.",
                "Add `await` to your assertion.",
            ),
        }
    }
}

#[test]
fn test_1() {
    use crate::tester::Tester;

    let pass = vec![(
        "test('valid-expect', async () => { await Promise.race([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); })",
        None,
    )];
    let fail = vec![];

    Tester::new(ValidExpect::NAME, pass, fail).with_jest_plugin(true).test();
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect('something').toEqual('else');", None),
        ("expect(true).toBeDefined();", None),
        ("expect([1, 2, 3]).toEqual([1, 2, 3]);", None),
        ("expect(undefined).not.toBeDefined();", None),
        ("test('valid-expect', () => { return expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test('valid-expect', () => { return expect(Promise.reject(2)).rejects.toBeDefined(); });", None),
        ("test('valid-expect', () => { return expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test('valid-expect', () => { return expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test('valid-expect', function () { return expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test('valid-expect', function () { return expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test('valid-expect', function () { return Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined()); });", None),
        ("test('valid-expect', function () { return Promise.resolve(expect(Promise.resolve(2)).rejects.not.toBeDefined()); });", None),
        ("test('valid-expect', () => expect(Promise.resolve(2)).resolves.toBeDefined());", Some(serde_json::json!([{ "alwaysAwait": true }]))),
        ("test('valid-expect', () => expect(Promise.resolve(2)).resolves.toBeDefined());", None),
        ("test('valid-expect', () => expect(Promise.reject(2)).rejects.toBeDefined());", None),
        ("test('valid-expect', () => expect(Promise.reject(2)).resolves.not.toBeDefined());", None),
        ("test('valid-expect', () => expect(Promise.reject(2)).rejects.not.toBeDefined());", None),
        ("test('valid-expect', async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined(); });", None),
        ("test('valid-expect', async () => { await expect(Promise.reject(2)).rejects.not.toBeDefined(); });", None),
        ("test('valid-expect', async function () { await expect(Promise.reject(2)).resolves.not.toBeDefined(); });", None),
        ("test('valid-expect', async function () { await expect(Promise.reject(2)).rejects.not.toBeDefined(); });", None),
        ("test('valid-expect', async () => { await Promise.resolve(expect(Promise.reject(2)).rejects.not.toBeDefined()); });", None),
        ("test('valid-expect', async () => { await Promise.reject(expect(Promise.reject(2)).rejects.not.toBeDefined()); });", None),
        ("test('valid-expect', async () => { await Promise.all([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test('valid-expect', async () => { await Promise.race([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test('valid-expect', async () => { await Promise.allSettled([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test('valid-expect', async () => { await Promise.any([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test('valid-expect', async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log('valid-case')); });", None),
        (
            "test('valid-expect', async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log('valid-case')).then(() => console.log('another valid case')); });",
            None,
        ),
        ("test('valid-expect', async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().catch(() => console.log('valid-case')); });", None),
        (
            "test('valid-expect', async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log('valid-case')).catch(() => console.log('another valid case')); });",
            None,
        ),
        ("test('valid-expect', async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => { expect(someMock).toHaveBeenCalledTimes(1); }); });", None),
        ("test('valid-expect', async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log('valid-case')); });", None),
        (
            "test('valid-expect', async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log('valid-case')).then(() => console.log('another valid case')); });",
            None,
        ),
        ("test('valid-expect', async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().catch(() => console.log('valid-case')); });", None),
        (
            "test('valid-expect', async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log('valid-case')).catch(() => console.log('another valid case')); });",
            None,
        ),
        ("test('valid-expect', async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => { expect(someMock).toHaveBeenCalledTimes(1); }); });", None),
        (
            "
                test('valid-expect', () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                        return expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                        await expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => expect(Promise.resolve(2)).resolves.toBe(1));
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                    return this.isNot
                        ? expect(obj).toBe(true)
                        : expect(obj).resolves.not.toThrow();
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                    return this.isNot
                        ? expect(obj).resolves.not.toThrow()
                        : expect(obj).toBe(true);
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                    return this.isNot
                        ? expect(obj).toBe(true)
                        : anotherCondition
                        ? expect(obj).resolves.not.toThrow()
                        : expect(obj).toBe(false)
                    }
                });
            ",
            None,
        ),
        ("expect(1).toBe(2);", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(1, '1 !== 2').toBe(2);", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(1, '1 !== 2').toBe(2);", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 2 }]))),
        ("test('valid-expect', () => { expect(2).not.toBe(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        ("test('valid-expect', () => { expect(Promise.reject(2)).toRejectWith(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }]))),
        ("test('valid-expect', async () => { await expect(Promise.resolve(2)).toResolve(); });", Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }]))),
        ("test('valid-expect', async () => { expect(Promise.resolve(2)).toResolve(); });", Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }]))),
    ];

    let mut fail = vec![
        ("expect().toBe(2);", None),
        ("expect().toBe(true);", None),
        ("expect().toEqual('something');", None),
        ("expect('something', 'else').toEqual('something');", None),
        ("expect('something', 'else', 'entirely').toEqual('something');", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect('something', 'else', 'entirely').toEqual('something');", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 2 }]))),
        ("expect('something', 'else', 'entirely').toEqual('something');", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 1 }]))),
        ("expect('something').toEqual('something');", Some(serde_json::json!([{ "minArgs": 2 }]))),
        ("expect('something', 'else').toEqual('something');", Some(serde_json::json!([{ "maxArgs": 1, "minArgs": 3 }]))),
        ("expect('something');", None),
        ("expect();", None),
        ("expect(true).toBeDefined;", None),
        ("expect(true).not.toBeDefined;", None),
        ("expect(true).nope.toBeDefined;", None),
        ("expect(true).nope.toBeDefined();", None),
        ("expect(true).not.resolves.toBeDefined();", None),
        ("expect(true).not.not.toBeDefined();", None),
        ("expect(true).resolves.not.exactly.toBeDefined();", None),
        ("expect(true).resolves;", None),
        ("expect(true).rejects;", None),
        ("expect(true).not;", None),
        ("expect(Promise.resolve(2)).resolves.toBeDefined();", None),
        ("expect(Promise.resolve(2)).rejects.toBeDefined();", None),
        ("expect(Promise.resolve(2)).resolves.toBeDefined();", Some(serde_json::json!([{ "alwaysAwait": true }]))),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                        ? expect(obj).toBe(true)
                        : expect(obj).resolves.not.toThrow();
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                        ? expect(obj).resolves.not.toThrow()
                        : expect(obj).toBe(true);
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                        ? expect(obj).toBe(true)
                        : anotherCondition
                        ? expect(obj).resolves.not.toThrow()
                        : expect(obj).toBe(false)
                    }
                });
            ",
            None,
        ),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).toResolve(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).toResolve(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).toReject(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).not.toReject(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).rejects.toBeDefined(); });", None),
        ("test('valid-expect', () => { expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test('valid-expect', async () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test('valid-expect', async () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test('valid-expect', () => { expect(Promise.reject(2)).toRejectWith(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        ("test('valid-expect', () => { expect(Promise.reject(2)).rejects.toBe(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        (
            "
                test('valid-expect', async () => {
                expect(Promise.resolve(2)).resolves.not.toBeDefined();
                expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', async () => {
                    await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            "
                test('valid-expect', async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', async () => {
                    await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            "
                test('valid-expect', async () => {
                    await expect(Promise.resolve(2)).toResolve();
                    return expect(Promise.resolve(1)).toReject();
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            "
                test('valid-expect', () => {
                    Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                    Promise.reject(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                    Promise.x(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                    Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            "
                test('valid-expect', () => {
                Promise.all([
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                ]);
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                Promise.x([
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                ]);
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                const assertions = [
                    expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                    expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                ]
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                const assertions = [
                    expect(Promise.resolve(2)).toResolve(),
                    expect(Promise.resolve(3)).toReject(),
                ]
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                const assertions = [
                    expect(Promise.resolve(2)).not.toResolve(),
                    expect(Promise.resolve(3)).resolves.toReject(),
                ]
                });
            ",
            None,
        ),
        ("expect(Promise.resolve(2)).resolves.toBe;", None),
        (
            "
                test('valid-expect', () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                        expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                        await expect(Promise.resolve(2)).resolves.toBe(1);
                        expect(Promise.resolve(4)).resolves.toBe(4);
                    });
                });
            ",
            None,
        ),
        (
            "
                test('valid-expect', async () => {
                    await expect(Promise.resolve(1));
                });
            ",
            None,
        ),
    ];

    let pass_vitest = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect(\"something\").toEqual(\"else\");", None),
        ("expect(true).toBeDefined();", None),
        ("expect([1, 2, 3]).toEqual([1, 2, 3]);", None),
        ("expect(undefined).not.toBeDefined();", None),
        ("test(\"valid-expect\", () => { return expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { return expect(Promise.reject(2)).rejects.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { return expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { return expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", function () { return expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", function () { return expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", function () { return Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", function () { return Promise.resolve(expect(Promise.resolve(2)).rejects.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", () => expect(Promise.resolve(2)).resolves.toBeDefined());", None),
        ("test(\"valid-expect\", () => expect(Promise.reject(2)).rejects.toBeDefined());", None),
        ("test(\"valid-expect\", () => expect(Promise.reject(2)).resolves.not.toBeDefined());", None),
        ("test(\"valid-expect\", () => expect(Promise.reject(2)).rejects.not.toBeDefined());", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async function () { await expect(Promise.reject(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async function () { await expect(Promise.reject(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { await Promise.resolve(expect(Promise.reject(2)).rejects.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", async () => { await Promise.reject(expect(Promise.reject(2)).rejects.not.toBeDefined()); });", None),
        ("test(\"valid-expect\", async () => { await Promise.all([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { await Promise.race([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { await Promise.allSettled([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { await Promise.any([expect(Promise.reject(2)).rejects.not.toBeDefined(), expect(Promise.reject(2)).rejects.not.toBeDefined()]); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).then(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().catch(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).catch(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { return expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => { expect(someMock).toHaveBeenCalledTimes(1); }); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).then(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().catch(() => console.log(\"valid-case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => console.log(\"valid-case\")).catch(() => console.log(\"another valid case\")); });", None),
        ("test(\"valid-expect\", async () => { await expect(Promise.reject(2)).resolves.not.toBeDefined().then(() => { expect(someMock).toHaveBeenCalledTimes(1); }); });", None),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                        return expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
        ",
        None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                        await expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => expect(Promise.resolve(2)).resolves.toBe(1));
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        return this.isNot
                            ? expect(obj).toBe(true)
                            : expect(obj).resolves.not.toThrow();
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        return this.isNot
                            ? expect(obj).resolves.not.toThrow()
                            : expect(obj).toBe(true);
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        return this.isNot
                        ? expect(obj).toBe(true)
                        : anotherCondition
                            ? expect(obj).resolves.not.toThrow()
                            : expect(obj).toBe(false)
                    }
                });
            ",
            None,
        ),
        ("expect(1).toBe(2);", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(1, \"1 !== 2\").toBe(2);", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        (
            "test(\"valid-expect\", () => { expect(2).not.toBe(2); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }])),
        ),
        (
            "test(\"valid-expect\", () => { expect(Promise.reject(2)).toRejectWith(2); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }])),
        ),
        (
            "test(\"valid-expect\", async () => { await expect(Promise.resolve(2)).toResolve(); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }])),
        ),
        (
            "test(\"valid-expect\", async () => { expect(Promise.resolve(2)).toResolve(); });",
            Some(serde_json::json!([{ "asyncMatchers": ["toResolveWith"] }])),
        ),
    ];

    let fail_vitest = vec![
        ("expect().toBe(2);", Some(serde_json::json!([{ "minArgs": "undefined", "maxArgs": "undefined" }]))),
        ("expect().toBe(true);", None),
        ("expect().toEqual(\"something\");", None),
        ("expect(\"something\", \"else\").toEqual(\"something\");", None),
        ("expect(\"something\", \"else\", \"entirely\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 2 }]))),
        ("expect(\"something\", \"else\", \"entirely\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 2 }]))),
        ("expect(\"something\", \"else\", \"entirely\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 2, "minArgs": 1 }]))),
        ("expect(\"something\").toEqual(\"something\");", Some(serde_json::json!([{ "minArgs": 2 }]))),
        ("expect(\"something\", \"else\").toEqual(\"something\");", Some(serde_json::json!([{ "maxArgs": 1, "minArgs": 3 }]))),
        ("expect(\"something\");", None),
        ("expect();", None),
        ("expect(true).toBeDefined;", None),
        ("expect(true).not.toBeDefined;", None),
        ("expect(true).nope.toBeDefined;", None),
        ("expect(true).nope.toBeDefined();", None),
        ("expect(true).not.resolves.toBeDefined();", None),
        ("expect(true).not.not.toBeDefined();", None),
        ("expect(true).resolves.not.exactly.toBeDefined();", None),
        ("expect(true).resolves;", None),
        ("expect(true).rejects;", None),
        ("expect(true).not;", None),
        ("expect(Promise.resolve(2)).resolves.toBeDefined();", None),
        ("expect(Promise.resolve(2)).rejects.toBeDefined();", None),
        ("expect(Promise.resolve(2)).resolves.toBeDefined();", Some(serde_json::json!([{ "alwaysAwait": true }]))),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                            ? expect(obj).toBe(true)
                            : expect(obj).resolves.not.toThrow();
                    }
                });
            ",
            None,
        ),
        (
            "
                expect.extend({
                    toResolve(obj) {
                        this.isNot
                            ? expect(obj).resolves.not.toThrow()
                            : expect(obj).toBe(true);
                    }
                });
            ",
            None,
        ),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).toResolve(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).toResolve(); });", Some(serde_json::json!([{ "asyncMatchers": "undefined" }]))),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).toReject(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).not.toReject(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).rejects.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.resolve(2)).rejects.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { expect(Promise.resolve(2)).resolves.toBeDefined(); });", None),
        ("test(\"valid-expect\", async () => { expect(Promise.resolve(2)).resolves.not.toBeDefined(); });", None),
        ("test(\"valid-expect\", () => { expect(Promise.reject(2)).toRejectWith(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        ("test(\"valid-expect\", () => { expect(Promise.reject(2)).rejects.toBe(2); });", Some(serde_json::json!([{ "asyncMatchers": ["toRejectWith"] }]))),
        (
            "
                test(\"valid-expect\", async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", async () => {
                    await expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        ("
                test(\"valid-expect\", async () => {
                    expect(Promise.resolve(2)).resolves.not.toBeDefined();
                    return expect(Promise.resolve(1)).rejects.toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.x(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.resolve(expect(Promise.resolve(2)).resolves.not.toBeDefined());
                });
            ",
            Some(serde_json::json!([{ "alwaysAwait": true }])),
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.all([
                        expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                        expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                    ]);
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    Promise.x([
                        expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                        expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                    ]);
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    const assertions = [
                        expect(Promise.resolve(2)).resolves.not.toBeDefined(),
                        expect(Promise.resolve(3)).resolves.not.toBeDefined(),
                    ]
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    const assertions = [
                        expect(Promise.resolve(2)).toResolve(),
                        expect(Promise.resolve(3)).toReject(),
                    ]
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    const assertions = [
                        expect(Promise.resolve(2)).not.toResolve(),
                        expect(Promise.resolve(3)).resolves.toReject(),
                    ]
                });
            ",
            None,
        ),
        ("expect(Promise.resolve(2)).resolves.toBe;", None),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(() => {
                        expect(Promise.resolve(2)).resolves.toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", () => {
                    return expect(functionReturningAPromise()).resolves.toEqual(1).then(async () => {
                        await expect(Promise.resolve(2)).resolves.toBe(1);
                        expect(Promise.resolve(4)).resolves.toBe(4);
                    });
                });
            ",
            None,
        ),
        (
            "
                test(\"valid-expect\", async () => {
                    await expect(Promise.resolve(1));
                });
            ",
            None,
        ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(ValidExpect::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
