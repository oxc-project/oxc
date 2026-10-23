// Port of internal/rules/await_thenable/await_thenable.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, Type};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleSuggestion, RuleVisitor,
};
use crate::utils::{self, TypeAwaitable};

fn await_message() -> RuleMessage {
    RuleMessage::with_help(
        "await",
        "Unexpected `await` of a non-Promise (non-\"Thenable\") value.",
        "Remove `await` if the value is synchronous, or change the expression to return a Promise or Thenable before awaiting it.",
    )
}
fn remove_await() -> RuleMessage {
    RuleMessage::new("removeAwait", "Remove unnecessary `await`.")
}
fn for_await_of_non_async_iterable() -> RuleMessage {
    RuleMessage::with_help(
        "forAwaitOfNonAsyncIterable",
        "Unexpected `for await...of` of a value that is not async iterable.",
        "Use `for...of` for synchronous iterables, or change the iterable to implement `Symbol.asyncIterator`.",
    )
}
fn convert_to_ordinary_for() -> RuleMessage {
    RuleMessage::new("convertToOrdinaryFor", "Convert to an ordinary `for...of` loop.")
}
fn await_using_of_non_async_disposable() -> RuleMessage {
    RuleMessage::with_help(
        "awaitUsingOfNonAsyncDisposable",
        "Unexpected `await using` of a value that is not async disposable.",
        "Use plain `using` for synchronous disposables, or change the value to implement `Symbol.asyncDispose`.",
    )
}
fn invalid_promise_aggregator_input() -> RuleMessage {
    RuleMessage::with_help(
        "invalidPromiseAggregatorInput",
        "Unexpected iterable of non-Promise (non-\"Thenable\") values passed to promise aggregator.",
        "Pass an iterable of Promise-like values, or wrap each synchronous value in `Promise.resolve(...)` before calling the aggregator.",
    )
}

fn build_diagnostic_label_text(message_id: &str) -> &'static str {
    match message_id {
        "forAwaitOfNonAsyncIterable" => "This value is not async iterable",
        "awaitUsingOfNonAsyncDisposable" => "This value is not async disposable",
        _ => "This expression is not Promise-like",
    }
}

fn build_await_thenable_diagnostic(
    range: (i32, i32),
    message: RuleMessage,
    label_range: (i32, i32),
    extra_labels: Vec<LabeledRange>,
) -> RuleDiagnostic {
    let mut labeled_ranges = vec![LabeledRange {
        label: build_diagnostic_label_text(message.id).to_string(),
        pos: label_range.0,
        end: label_range.1,
    }];
    labeled_ranges.extend(extra_labels);
    RuleDiagnostic { pos: range.0, end: range.1, message, labeled_ranges }
}

const PROMISE_AGGREGATOR_METHODS: &[&str] = &["all", "allSettled", "any", "race"];

pub struct AwaitThenable;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(AwaitThenable))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::AwaitExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::ForOfStatement),
    Listener::Enter(Kind::VariableDeclarationList),
];

impl Rule for AwaitThenable {
    fn name(&self) -> &'static str {
        "await-thenable"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_promise_aggregator_method(ctx: &mut Ctx, call_expression: P<Node>) -> bool {
    let callee = ast::skip_parentheses(call_expression.expression().unwrap());
    if !ast::is_access_expression(callee) {
        return false;
    }
    let (method_name, ok) = ctx.checker.get_accessed_property_name(callee);
    if !ok || !PROMISE_AGGREGATOR_METHODS.contains(&method_name.as_str()) {
        return false;
    }
    let t = utils::get_constrained_type_at_location(ctx.checker, callee.expression().unwrap());
    utils::is_promise_constructor_like(ctx.program, ctx.checker, t)
}

fn is_invalid_promise_aggregator_input(c: &mut Checker, node: P<Node>, t: P<Type>) -> bool {
    if !is_iterable(c, t) {
        return false;
    }
    for part in utils::union_type_parts(t) {
        for value_type in get_value_types_of_array_like(c, part) {
            if contains_non_awaitable_type(c, node, value_type) {
                return true;
            }
        }
    }
    false
}

fn get_value_types_of_array_like(c: &mut Checker, t: P<Type>) -> Vec<P<Type>> {
    if t.is_tuple_type() {
        return c.get_type_arguments(t).to_vec();
    }
    if let Some(number_index_type) = utils::get_number_index_type(c, t) {
        return vec![number_index_type];
    }
    let type_arguments = c.get_type_arguments(t);
    if !type_arguments.is_empty() {
        return type_arguments[..1].to_vec();
    }
    Vec::new()
}

fn is_always_non_awaitable_type(c: &mut Checker, node: P<Node>, t: P<Type>) -> bool {
    utils::union_type_parts(t)
        .into_iter()
        .all(|part| utils::needs_to_be_awaited(c, node, part) == TypeAwaitable::Never)
}

fn contains_non_awaitable_type(c: &mut Checker, node: P<Node>, t: P<Type>) -> bool {
    utils::union_type_parts(t)
        .into_iter()
        .any(|part| utils::needs_to_be_awaited(c, node, part) == TypeAwaitable::Never)
}

fn is_iterable(c: &mut Checker, t: P<Type>) -> bool {
    utils::union_type_parts(t)
        .into_iter()
        .all(|part| utils::get_well_known_symbol_property_of_type(part, "iterator", c).is_some())
}

impl Visitor {
    fn await_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let await_argument = node.expression().unwrap();
        let t = ctx.checker.get_type_at_location(await_argument);
        let certainty = utils::needs_to_be_awaited(ctx.checker, await_argument, t);
        if certainty == TypeAwaitable::Never {
            let r = tsrs_scanner::get_range_of_token_at_position(ctx.file, node.pos());
            let range = (r.pos(), r.end());
            let d = build_await_thenable_diagnostic(
                range,
                await_message(),
                ctx.trim(await_argument),
                Vec::new(),
            );
            ctx.report_diagnostic_with_suggestions(d, |ctx| {
                vec![RuleSuggestion {
                    message: remove_await(),
                    fixes: vec![ctx.fix_remove_range(range.0, range.1)],
                }]
            });
        }
    }

    fn call_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let args = node.arguments();
        if args.is_empty() {
            return;
        }
        if !is_promise_aggregator_method(ctx, node) {
            return;
        }
        let argument = args[0];
        if ast::is_array_literal_expression(argument) {
            for &element in argument.as_array_literal_expression().elements.nodes() {
                if ast::is_omitted_expression(element) {
                    continue;
                }
                let t = utils::get_constrained_type_at_location(ctx.checker, element);
                if is_always_non_awaitable_type(ctx.checker, element, t) {
                    let argument_range = ctx.trim(argument);
                    let element_range = ctx.trim(element);
                    let d = build_await_thenable_diagnostic(
                        element_range,
                        invalid_promise_aggregator_input(),
                        element_range,
                        vec![LabeledRange {
                            label: "Promise aggregator input".to_string(),
                            pos: argument_range.0,
                            end: argument_range.1,
                        }],
                    );
                    ctx.report_diagnostic(d);
                }
            }
            return;
        }
        let t = utils::get_constrained_type_at_location(ctx.checker, argument);
        if is_invalid_promise_aggregator_input(ctx.checker, argument, t) {
            let arg_range = ctx.trim(argument);
            let d = build_await_thenable_diagnostic(
                arg_range,
                invalid_promise_aggregator_input(),
                arg_range,
                Vec::new(),
            );
            ctx.report_diagnostic(d);
        }
    }

    fn for_of_statement(&self, ctx: &mut Ctx, node: P<Node>) {
        let stmt = node.as_for_in_or_of_statement();
        let Some(await_modifier) = stmt.await_modifier else {
            return;
        };
        let expr_type = ctx.checker.get_type_at_location(stmt.expression);
        if utils::is_type_any_type(expr_type) {
            return;
        }
        for part in utils::union_type_parts(expr_type) {
            if utils::get_well_known_symbol_property_of_type(part, "asyncIterator", ctx.checker)
                .is_some()
            {
                return;
            }
        }
        let d = build_await_thenable_diagnostic(
            utils::get_for_statement_head_loc(ctx.file, node),
            for_await_of_non_async_iterable(),
            ctx.trim(stmt.expression),
            Vec::new(),
        );
        ctx.report_diagnostic_with_suggestions(d, |ctx| {
            // Note that this suggestion causes broken code for sync iterables of promises, since
            // the loop variable is not awaited.
            vec![RuleSuggestion {
                message: convert_to_ordinary_for(),
                fixes: vec![ctx.fix_replace(await_modifier, "")],
            }]
        });
    }

    fn variable_declaration_list(&self, ctx: &mut Ctx, node: P<Node>) {
        if !ast::is_var_await_using(node) {
            return;
        }
        let declarations = node.as_variable_declaration_list().declarations.nodes();
        'declarators: for &declarator in declarations {
            let Some(init) = declarator.initializer() else {
                continue;
            };
            let init_type = ctx.checker.get_type_at_location(init);
            if utils::is_type_any_type(init_type) {
                continue;
            }
            for part in utils::union_type_parts(init_type) {
                if utils::get_well_known_symbol_property_of_type(part, "asyncDispose", ctx.checker)
                    .is_some()
                {
                    continue 'declarators;
                }
            }
            let mut suggestions = Vec::new();
            // let the user figure out what to do if there's
            // await using a = b, c = d, e = f;
            // it's rare and not worth the complexity to handle.
            if declarations.len() == 1 {
                let r = tsrs_scanner::get_range_of_token_at_position(ctx.file, node.pos());
                suggestions.push(RuleSuggestion {
                    message: remove_await(),
                    fixes: vec![ctx.fix_remove_range(r.pos(), r.end())],
                });
            }
            let init_range = ctx.trim(init);
            let d = build_await_thenable_diagnostic(
                init_range,
                await_using_of_non_async_disposable(),
                init_range,
                Vec::new(),
            );
            ctx.report_diagnostic_with_suggestions(d, |_| suggestions);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::AwaitExpression => self.await_expression(ctx, node),
            Kind::CallExpression => self.call_expression(ctx, node),
            Kind::ForOfStatement => self.for_of_statement(ctx, node),
            Kind::VariableDeclarationList => self.variable_declaration_list(ctx, node),
            _ => {}
        }
    }
}
