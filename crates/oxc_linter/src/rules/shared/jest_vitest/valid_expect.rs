use std::borrow::Cow;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;

use crate::{
    AstNode,
    context::LintContext,
    utils::{
        ExpectError, PossibleJestNode, collect_possible_jest_call_node, parse_expect_jest_fn_call,
    },
};

fn valid_expect_diagnostic<S: Into<Cow<'static, str>>>(
    x1: S,
    x2: &'static str,
    span3: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(x1).with_help(x2).with_label(span3)
}

pub const DOCUMENTATION: &str = r"### What it does

Checks that `expect()` is called correctly.

### Why is this bad?

`expect()` is a function that is used to assert values in tests.
It should be called with a single argument, which is the value to be tested.
If you call `expect()` with no arguments, or with more than one argument, it will not work as expected.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect();
expect('something');
expect(true).toBeDefined;
expect(Promise.resolve('Hi!')).resolves.toBe('Hi!');
```

Examples of **correct** code for this rule:
```javascript
expect('something').toEqual('something');
expect(true).toBeDefined();
expect(Promise.resolve('Hi!')).resolves.toBe('Hi!');
```
";

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ValidExpectConfig {
    /// List of matchers that are considered async and therefore require awaiting (e.g. `toResolve`, `toReject`).
    async_matchers: Vec<String>,
    /// Minimum number of arguments `expect` should be called with.
    min_args: usize,
    /// Maximum number of arguments `expect` should be called with.
    max_args: usize,
    /// When `true`, allow a string or template literal second argument as a custom message.
    #[serde(skip)]
    allow_string_message_arg: bool,
    /// When `true`, async assertions must be awaited in all contexts (not just return statements).
    always_await: bool,
}

impl Default for ValidExpectConfig {
    fn default() -> Self {
        Self {
            async_matchers: vec![String::from("toResolve"), String::from("toReject")],
            min_args: 1,
            max_args: 1,
            allow_string_message_arg: false,
            always_await: false,
        }
    }
}

impl ValidExpectConfig {
    pub fn from_configuration(value: &serde_json::Value) -> Self {
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

        Self { async_matchers, min_args, max_args, allow_string_message_arg: false, always_await }
    }

    pub fn allow_string_message_arg(mut self) -> Self {
        self.allow_string_message_arg = true;
        self
    }

    pub fn run_once(&self, ctx: &LintContext) {
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|node| node.node.id());
        let mut fixed_function_expression: FxHashSet<ScopeId> = FxHashSet::default();

        for jest_node in possible_jest_nodes {
            self.run(&jest_node, &mut fixed_function_expression, ctx);
        }
    }

    fn run<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        fixed_function_expression: &mut FxHashSet<ScopeId>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let reporting_span = jest_fn_call.expect_error.map_or(call_expr.span, |_| {
            find_top_most_member_expression(node, ctx)
                .map_or(call_expr.span, |top_most_member_expr| top_most_member_expr.span())
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

        let allow_message_arg = self.allow_string_message_arg
            && call_expr.arguments.len() == 2
            && call_expr.arguments.get(1).and_then(|arg| arg.as_expression()).is_some_and(|expr| {
                matches!(expr, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
            });

        if call_expr.arguments.len() > self.max_args && !allow_message_arg {
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

        let parent = ctx.nodes().parent_node(node.id());

        let should_be_awaited =
            jest_fn_call.modifiers().iter().any(|modifier| modifier.is_name_unequal("not"))
                || self.async_matchers.contains(&matcher_name.to_string());

        if matches!(parent.kind(), AstKind::Program(_)) || !should_be_awaited {
            return;
        }

        // An async assertion can be chained with `then` or `catch` statements.
        // In that case our target CallExpression node is the one with
        // the last `then` or `catch` statement.
        let target_node = get_parent_if_thenable(node, ctx);
        let Some(final_node) = find_promise_call_expression_node(node, ctx, target_node) else {
            return;
        };
        let parent = ctx.nodes().parent_node(final_node.id());
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
            ctx.diagnostic_with_suggestion(valid_expect_diagnostic(error, help, span), |fixer| {
                let Some(function_scope_node) =
                    ctx.nodes().ancestors(node.id()).find(|node| node.kind().is_function_like())
                else {
                    return fixer.noop();
                };

                let function_scope_id = function_scope_node.scope_id();

                let multifixer = fixer.for_multifix();

                let is_async_function = match function_scope_node.kind() {
                    AstKind::ArrowFunctionExpression(fn_kind) => fn_kind.r#async,
                    AstKind::Function(fn_kind) => fn_kind.r#async,
                    _ => return fixer.noop(),
                };

                let needs_async =
                    !fixed_function_expression.contains(&function_scope_id) && !is_async_function;
                let capacity = if needs_async { 2 } else { 1 };

                let mut fixes = multifixer.new_fix_with_capacity(capacity);

                if needs_async {
                    fixed_function_expression.insert(function_scope_id);

                    let context_function = ctx.nodes().parent_node(function_scope_node.id());

                    /* Difference between ESTree and Oxc in the following scenario
                     *
                     * expect.extend({
                     *               toResolve(obj) {
                     *                 this.isNot
                     *                   ? expect(obj).toBe(true)
                     *                   : expect(obj).resolves.not.toThrow();
                     *               }
                     *             })
                     *
                     * ESLint's span returns toResolve(obj) {...}, but Oxc only returns (obj){...}.
                     * This difference produces an invalid fix by adding `async` between the function name and arguments,
                     * writing toResolveasync (obj) instead of async toResolve(obj).
                     *
                     */
                    let span_to_insert_before =
                        if matches!(context_function.kind(), AstKind::ObjectProperty(_)) {
                            context_function.span()
                        } else {
                            function_scope_node.span()
                        };

                    fixes.push(fixer.insert_text_before_range(span_to_insert_before, "async "));
                }

                let is_parent_return_statement =
                    matches!(parent.kind(), AstKind::ReturnStatement(_));

                if self.always_await && is_parent_return_statement {
                    let return_source_code_text = ctx.source_range(parent.span());

                    fixes.push(fixer.replace(
                        parent.span(),
                        // The alternative was casting the value from &str -> cow string -> String
                        #[expect(clippy::disallowed_methods)]
                        return_source_code_text.replace("return", "await"),
                    ));
                } else {
                    fixes.push(fixer.insert_text_before_range(final_node.span(), "await "));
                }

                fixes.with_message(if needs_async {
                    "Add `await` and make the enclosing function `async`."
                } else {
                    "Add `await`."
                })
            });
        }
    }
}

fn find_top_most_member_expression<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<AstKind<'a>> {
    let mut top_most_member_expression = None;
    let mut node = node;

    loop {
        let parent = ctx.nodes().parent_node(node.id());
        match node.kind() {
            member_expr if member_expr.is_member_expression_kind() => {
                top_most_member_expression = Some(member_expr);
            }
            _ => {
                if !parent.kind().is_member_expression_kind() {
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
            | AstKind::ExpressionStatement(_)
            | AstKind::FunctionBody(_) => {
                node = ctx.nodes().parent_node(node.id());
            }
            AstKind::ArrowFunctionExpression(arrow_expr) => return arrow_expr.expression,
            AstKind::AwaitExpression(_) => return true,
            _ => return false,
        }
    }
}

type ParentAndIsFirstItem<'a, 'b> = (&'b AstNode<'a>, bool);

/// Checks if a node should be skipped during parent traversal
fn should_skip_parent_node(node: &AstNode, parent: &AstNode) -> bool {
    match parent.kind() {
        AstKind::CallExpression(call) => {
            // Don't skip arguments to Promise methods - they're semantically important for await detection
            if let Some(member_expr) = call.callee.as_member_expression()
                && let Expression::Identifier(ident) = member_expr.object()
                && ident.name == "Promise"
            {
                return false; // Never skip Promise method arguments
            }

            // For other call expressions, skip if this node is one of the arguments
            call.arguments.iter().any(|arg| arg.span() == node.span())
        }
        AstKind::NewExpression(new_expr) => {
            // Skip if this node is one of the new expression arguments
            new_expr.arguments.iter().any(|arg| arg.span() == node.span())
        }
        _ => false,
    }
}

// Returns the parent node of the given node, ignoring some nodes,
// and return whether the first item if parent is an array.
fn get_parent_with_ignore<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<ParentAndIsFirstItem<'a, 'b>> {
    let mut node = node;
    loop {
        let parent = ctx.nodes().parent_node(node.id());
        if !should_skip_parent_node(node, parent) {
            // we don't want to report `Promise.all([invalidExpectCall_1, invalidExpectCall_2])` twice.
            // so we need mark whether the node is the first item of an array.
            // if it not the first item, we ignore it in `find_promise_call_expression_node`.
            let is_first_item = if let AstKind::ArrayExpression(array_expr) = parent.kind() {
                array_expr.elements.first()?.span() == node.span()
            } else {
                // if parent is not an array, we assume it's the first item
                true
            };

            return Some((parent, is_first_item));
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

    if let AstKind::CallExpression(call_expr) = parent.kind()
        && let Some(member_expr) = call_expr.callee.as_member_expression()
        && let Expression::Identifier(ident) = member_expr.object()
        && matches!(ident.name.as_str(), "Promise")
        && !matches!(parent.kind(), AstKind::Program(_))
    {
        if is_first_array_item {
            return Some(parent);
        }
        return None;
    }

    Some(default_node)
}

fn get_parent_if_thenable<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> &'b AstNode<'a> {
    let grandparent = ctx.nodes().parent_node(ctx.nodes().parent_id(node.id()));
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
                "Did you forget to add a matcher, e.g. `toBe`, `toBeDefined`",
            ),
            Self::MatcherNotCalled => (
                "Matchers must be called to assert.",
                "You need to call your matcher, e.g. `expect(true).toBe(true)`.",
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
