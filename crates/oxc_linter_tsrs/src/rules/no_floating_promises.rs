// Port of internal/rules/no_floating_promises/no_floating_promises.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Signature, Type};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleSuggestion,
    RuleVisitor, opt_bool, options_object,
};
use crate::utils;

const MESSAGE_BASE: &str = "Promises must be awaited, add await operator.";
const MESSAGE_BASE_HELP: &str = "The promise must end with a call to .catch, or end with a call to .then with a rejection handler.";
const MESSAGE_BASE_VOID: &str = "Promises must be awaited, add void operator to ignore.";
const MESSAGE_BASE_VOID_HELP: &str = "The promise must end with a call to .catch, or end with a call to .then with a rejection handler, or be explicitly marked as ignored with the `void` operator.";
const MESSAGE_REJECTION_HANDLER: &str =
    "A rejection handler that is not a function will be ignored.";

fn floating() -> RuleMessage {
    RuleMessage::with_help("floating", MESSAGE_BASE, MESSAGE_BASE_HELP)
}
fn floating_fix_await() -> RuleMessage {
    RuleMessage::new("floatingFixAwait", "Add await operator.")
}
fn floating_fix_void() -> RuleMessage {
    RuleMessage::new("floatingFixVoid", "Add void operator to ignore.")
}
fn floating_promise_array() -> RuleMessage {
    RuleMessage::with_help(
        "floatingPromiseArray",
        "An array of Promises may be unintentional.",
        "Consider handling the promises' fulfillment or rejection with Promise.all or similar.",
    )
}
fn floating_promise_array_void() -> RuleMessage {
    RuleMessage::with_help(
        "floatingPromiseArrayVoid",
        "An array of Promises may be unintentional.",
        "Consider handling the promises' fulfillment or rejection with Promise.all or similar, or explicitly marking the expression as ignored with the `void` operator.",
    )
}
fn floating_useless_rejection_handler() -> RuleMessage {
    RuleMessage::with_help(
        "floatingUselessRejectionHandler",
        MESSAGE_BASE,
        format!("{MESSAGE_BASE_HELP} {MESSAGE_REJECTION_HANDLER}"),
    )
}
fn floating_useless_rejection_handler_void() -> RuleMessage {
    RuleMessage::with_help(
        "floatingUselessRejectionHandlerVoid",
        MESSAGE_BASE_VOID,
        format!("{MESSAGE_BASE_VOID_HELP} {MESSAGE_REJECTION_HANDLER}"),
    )
}
fn floating_void() -> RuleMessage {
    RuleMessage::with_help("floatingVoid", MESSAGE_BASE_VOID, MESSAGE_BASE_VOID_HELP)
}

pub struct NoFloatingPromises {
    allow_for_known_safe_calls: Vec<utils::TypeOrValueSpecifier>,
    allow_for_known_safe_promises: Vec<utils::TypeOrValueSpecifier>,
    check_thenables: bool,
    ignore_iife: bool,
    ignore_void: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let specifiers = |key: &str| {
        utils::unmarshal_type_or_value_specifiers(m.get(key))
            .map_err(|e| format!("no-floating-promises: failed to unmarshal options: {e}"))
    };
    Ok(Box::new(NoFloatingPromises {
        allow_for_known_safe_calls: specifiers("allowForKnownSafeCalls")?,
        allow_for_known_safe_promises: specifiers("allowForKnownSafePromises")?,
        check_thenables: opt_bool(&m, "checkThenables", false),
        ignore_iife: opt_bool(&m, "ignoreIIFE", false),
        ignore_void: opt_bool(&m, "ignoreVoid", true),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::ExpressionStatement)];

impl Rule for NoFloatingPromises {
    fn name(&self) -> &'static str {
        "no-floating-promises"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static NoFloatingPromises,
}

struct Unhandled {
    node: P<Node>,
    t: P<Type>,
    promise_array: bool,
    non_function_handler: Option<P<Node>>,
}

fn is_higher_precedence_than_unary(node: P<Node>) -> bool {
    let operator = if ast::is_binary_expression(node) {
        node.as_binary_expression().operator_token.kind()
    } else {
        Kind::Unknown
    };
    let p = ast::get_operator_precedence(node.kind(), operator, ast::OperatorPrecedenceFlags::None);
    (p as i32) > (ast::OperatorPrecedence::Unary as i32)
}

impl Visitor {
    fn add_await(&self, ctx: &Ctx, expression: P<Node>, statement: P<Node>) -> Vec<RuleFix> {
        if ast::is_void_expression(expression) {
            let r = tsrs_scanner::get_range_of_token_at_position(ctx.file, expression.pos());
            return vec![ctx.fix_replace_range(r.pos(), r.end(), "await")];
        }
        if is_higher_precedence_than_unary(statement.expression().unwrap()) {
            return vec![ctx.fix_insert_before(statement, "await ")];
        }
        vec![ctx.fix_insert_before(statement, "await ("), ctx.fix_insert_after(expression, ")")]
    }

    fn has_matching_signature(
        ctx: &mut Ctx,
        t: P<Type>,
        matcher: &mut dyn FnMut(&mut Ctx, P<Signature>) -> bool,
    ) -> bool {
        for part in utils::union_type_parts(t) {
            for &sig in utils::get_call_signatures(ctx.checker, part) {
                if matcher(ctx, sig) {
                    return true;
                }
            }
        }
        false
    }

    fn is_function_param(ctx: &mut Ctx, param: P<tsrs_ast::Symbol>, node: P<Node>) -> bool {
        let at = ctx.checker.get_type_of_symbol_at_location(param, Some(node)).unwrap();
        let t = ctx.checker.get_apparent_type(at);
        utils::union_type_parts(t)
            .into_iter()
            .any(|part| !utils::get_call_signatures(ctx.checker, part).is_empty())
    }

    fn is_promise_like(&self, ctx: &mut Ctx, node: P<Node>, t: Option<P<Type>>) -> bool {
        let t = t.unwrap_or_else(|| ctx.checker.get_type_at_location(node));
        // The highest priority is to allow anything allowlisted
        if utils::type_matches_some_specifier(t, &self.o.allow_for_known_safe_promises, ctx.program)
        {
            return false;
        }
        let apparent = ctx.checker.get_apparent_type(t);
        let parts = utils::union_type_parts(apparent);
        for &part in &parts {
            if utils::is_promise_like(ctx.program, ctx.checker, part) {
                return true;
            }
        }
        if !self.o.check_thenables {
            return false;
        }
        for &part in &parts {
            let Some(then) = ctx.checker.get_property_of_type(part, "then") else {
                continue;
            };
            let then_type = ctx.checker.get_type_of_symbol_at_location(then, Some(node)).unwrap();
            if Self::has_matching_signature(ctx, then_type, &mut |ctx, sig| {
                let params = sig.parameters.get();
                params.len() >= 2
                    && Self::is_function_param(ctx, params[0], node)
                    && Self::is_function_param(ctx, params[1], node)
            }) {
                return true;
            }
        }
        false
    }

    fn is_known_safe_promise_return(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        if self.o.allow_for_known_safe_calls.is_empty() {
            return false;
        }
        if !ast::is_call_expression(node) {
            return false;
        }
        let callee = node.expression().unwrap();
        let t = ctx.checker.get_type_at_location(callee);
        if utils::value_matches_some_specifier(
            callee,
            &self.o.allow_for_known_safe_calls,
            ctx.program,
            Some(t),
        ) {
            return true;
        }
        utils::type_matches_some_specifier(t, &self.o.allow_for_known_safe_calls, ctx.program)
    }

    fn is_promise_array(&self, ctx: &mut Ctx, node: P<Node>, t: P<Type>) -> bool {
        for part in utils::union_type_parts(t) {
            let apparent = ctx.checker.get_apparent_type(part);
            if ctx.checker.is_array_type(apparent) {
                let element = ctx.checker.get_type_arguments(apparent)[0];
                if self.is_promise_like(ctx, node, Some(element)) {
                    return true;
                }
            }
            if apparent.is_tuple_type() {
                for &element in ctx.checker.get_type_arguments(apparent) {
                    if self.is_promise_like(ctx, node, Some(element)) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_valid_rejection_handler(ctx: &mut Ctx, handler: P<Node>) -> bool {
        let t = ctx.checker.get_type_at_location(handler);
        !utils::get_call_signatures(ctx.checker, t).is_empty()
    }

    fn is_known_argument_at(args: &[P<Node>], index: usize) -> bool {
        if args.len() <= index {
            return false;
        }
        !args[..=index].iter().any(|&a| ast::is_spread_element(a))
    }

    fn is_unhandled_promise(&self, ctx: &mut Ctx, node: P<Node>) -> Option<Unhandled> {
        if ast::is_assignment_expression(node, false) {
            return None;
        }
        if ast::is_comma_expression(node) {
            let e = node.as_binary_expression();
            if let Some(r) = self.is_unhandled_promise(ctx, e.left) {
                return Some(r);
            }
            return self.is_unhandled_promise(ctx, e.right.get());
        }
        if !self.o.ignore_void && ast::is_void_expression(node) {
            return self.is_unhandled_promise(ctx, node.expression().unwrap());
        }
        let t = ctx.checker.get_type_at_location(node);
        if self.is_promise_array(ctx, node, t) {
            return Some(Unhandled { node, t, promise_array: true, non_function_handler: None });
        }
        if ast::is_await_expression(node) {
            return None;
        }
        if !self.is_promise_like(ctx, node, Some(t)) {
            return None;
        }
        if ast::is_call_expression(node) {
            let callee = node.expression().unwrap();
            let args = node.arguments();
            if ast::is_access_expression(callee) {
                let (method_name, _) = ctx.checker.get_accessed_property_name(callee);
                if method_name == "catch" && !args.is_empty() {
                    if !Self::is_known_argument_at(args, 0) {
                        return Some(Unhandled {
                            node,
                            t,
                            promise_array: false,
                            non_function_handler: None,
                        });
                    }
                    if Self::is_valid_rejection_handler(ctx, args[0]) {
                        return None;
                    }
                    return Some(Unhandled {
                        node,
                        t,
                        promise_array: false,
                        non_function_handler: Some(args[0]),
                    });
                }
                if method_name == "then" && args.len() >= 2 {
                    if !Self::is_known_argument_at(args, 1) {
                        return Some(Unhandled {
                            node,
                            t,
                            promise_array: false,
                            non_function_handler: None,
                        });
                    }
                    if Self::is_valid_rejection_handler(ctx, args[1]) {
                        return None;
                    }
                    return Some(Unhandled {
                        node,
                        t,
                        promise_array: false,
                        non_function_handler: Some(args[1]),
                    });
                }
                if method_name == "finally" {
                    let mut r = self.is_unhandled_promise(ctx, callee.expression().unwrap())?;
                    r.node = node;
                    r.t = t;
                    return Some(r);
                }
            }
            return Some(Unhandled { node, t, promise_array: false, non_function_handler: None });
        }
        if node.kind() == Kind::ConditionalExpression {
            let e = node.as_conditional_expression();
            if let Some(r) = self.is_unhandled_promise(ctx, e.when_false) {
                return Some(r);
            }
            return self.is_unhandled_promise(ctx, e.when_true);
        }
        if ast::is_logical_or_coalescing_binary_expression(node) {
            let e = node.as_binary_expression();
            if let Some(r) = self.is_unhandled_promise(ctx, e.left) {
                return Some(r);
            }
            return self.is_unhandled_promise(ctx, e.right.get());
        }
        Some(Unhandled { node, t, promise_array: false, non_function_handler: None })
    }

    fn build_diagnostic(ctx: &mut Ctx, result: &Unhandled, message: RuleMessage) -> RuleDiagnostic {
        let (pos, end) = ctx.trim(result.node);
        let description = if result.promise_array {
            "This array contains Promises and"
        } else {
            "This unhandled promise-like value"
        };
        let type_string = utils::type_to_string(ctx.checker, result.t);
        let mut labels = vec![LabeledRange {
            label: format!("{description} has type `{type_string}`."),
            pos,
            end,
        }];
        if let Some(h) = result.non_function_handler {
            let ht = ctx.checker.get_type_at_location(h);
            let hs = utils::type_to_string(ctx.checker, ht);
            let (hp, he) = ctx.trim(h);
            labels.push(LabeledRange {
                label: format!("This rejection handler has type `{hs}`, which is not callable."),
                pos: hp,
                end: he,
            });
        }
        RuleDiagnostic { pos, end, message, labeled_ranges: labels }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let statement_expression = node.expression().unwrap();
        if self.o.ignore_iife && ast::is_call_expression(statement_expression) {
            let callee = ast::skip_parentheses(statement_expression.expression().unwrap());
            if ast::is_arrow_function(callee) || ast::is_function_expression(callee) {
                return;
            }
        }
        let expression = ast::skip_parentheses(statement_expression);
        if self.is_known_safe_promise_return(ctx, expression) {
            return;
        }
        let Some(result) = self.is_unhandled_promise(ctx, expression) else {
            return;
        };
        if result.promise_array {
            let msg = if self.o.ignore_void {
                floating_promise_array_void()
            } else {
                floating_promise_array()
            };
            let d = Self::build_diagnostic(ctx, &result, msg);
            ctx.report_diagnostic(d);
        } else if self.o.ignore_void {
            let msg = if result.non_function_handler.is_some() {
                floating_useless_rejection_handler_void()
            } else {
                floating_void()
            };
            let d = Self::build_diagnostic(ctx, &result, msg);
            ctx.report_diagnostic_with_suggestions(d, |ctx| {
                let void_fixes = if is_higher_precedence_than_unary(statement_expression) {
                    vec![ctx.fix_insert_before(node, "void ")]
                } else {
                    vec![
                        ctx.fix_insert_before(node, "void ("),
                        ctx.fix_insert_after(expression, ")"),
                    ]
                };
                vec![
                    RuleSuggestion { message: floating_fix_void(), fixes: void_fixes },
                    RuleSuggestion {
                        message: floating_fix_await(),
                        fixes: self.add_await(ctx, expression, node),
                    },
                ]
            });
        } else {
            let msg = if result.non_function_handler.is_some() {
                floating_useless_rejection_handler()
            } else {
                floating()
            };
            let d = Self::build_diagnostic(ctx, &result, msg);
            ctx.report_diagnostic_with_suggestions(d, |ctx| {
                vec![RuleSuggestion {
                    message: floating_fix_await(),
                    fixes: self.add_await(ctx, expression, node),
                }]
            });
        }
    }
}
