// Port of internal/rules/prefer_promise_reject_errors/prefer_promise_reject_errors.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils::{self, TypeOrValueSpecifier};

fn reject_an_error() -> RuleMessage {
    RuleMessage::new("rejectAnError", "Expected the Promise rejection reason to be an Error.")
}

pub struct PreferPromiseRejectErrors {
    allow: Vec<TypeOrValueSpecifier>,
    allow_empty_reject: bool,
    allow_throwing_any: bool,
    allow_throwing_unknown: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(PreferPromiseRejectErrors {
        allow: utils::unmarshal_type_or_value_specifiers(m.get("allow")).map_err(|e| {
            format!("prefer-promise-reject-errors: failed to unmarshal options: {e}")
        })?,
        allow_empty_reject: opt_bool(&m, "allowEmptyReject", false),
        allow_throwing_any: opt_bool(&m, "allowThrowingAny", false),
        allow_throwing_unknown: opt_bool(&m, "allowThrowingUnknown", false),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for PreferPromiseRejectErrors {
    fn name(&self) -> &'static str {
        "prefer-promise-reject-errors"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static PreferPromiseRejectErrors,
}

impl Visitor {
    fn check_reject_call(&self, ctx: &mut Ctx, call_expression: P<Node>) {
        let args = call_expression.arguments();
        if !args.is_empty() {
            let argument = args[0];
            let t = ctx.checker.get_type_at_location(argument);
            if utils::type_matches_some_specifier(t, &self.o.allow, ctx.program) {
                return;
            }
            if self.o.allow_throwing_any && utils::is_type_any_type(t) {
                return;
            }
            if self.o.allow_throwing_unknown && utils::is_type_unknown_type(t) {
                return;
            }
            if utils::is_error_like(ctx.program, ctx.checker, t)
                || utils::is_readonly_error_like(ctx.program, ctx.checker, t)
            {
                return;
            }
        } else if self.o.allow_empty_reject {
            return;
        }
        ctx.report_node(call_expression, reject_an_error());
    }
}

fn type_at_location_is_like_promise(ctx: &mut Ctx, node: P<Node>) -> bool {
    let t = ctx.checker.get_type_at_location(node);
    utils::is_promise_constructor_like(ctx.program, ctx.checker, t)
        || utils::is_promise_like(ctx.program, ctx.checker, t)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let callee = ast::skip_parentheses(node.expression().unwrap());
        if ast::is_access_expression(callee) {
            // Promise.reject(...)
            let (method_name, _) = ctx.checker.get_accessed_property_name(callee);
            if method_name == "reject"
                && type_at_location_is_like_promise(ctx, callee.expression().unwrap())
            {
                self.check_reject_call(ctx, node);
            }
        } else if ast::is_identifier(callee) {
            // reject(...)
            let Some(symbol) = ctx.checker.get_symbol_at_location_exported(callee) else {
                return;
            };
            let Some(param) = symbol.value_declaration() else {
                return;
            };
            if !ast::is_parameter_declaration(param) {
                return;
            }
            let Some(mut parent_node) = param.parent() else {
                return;
            };
            if !ast::is_function_expression(parent_node) && !ast::is_arrow_function(parent_node) {
                return;
            }
            let params = parent_node.parameters();
            if params.len() < 2 || params[1] != param {
                return;
            }
            loop {
                parent_node = parent_node.parent().unwrap();
                if ast::is_parenthesized_expression(parent_node) {
                    continue;
                }
                if !ast::is_new_expression(parent_node) {
                    return;
                }
                break;
            }
            let t = ctx.checker.get_type_at_location(parent_node.expression().unwrap());
            if !utils::is_promise_constructor_like(ctx.program, ctx.checker, t) {
                return;
            }
            self.check_reject_call(ctx, node);
        }
    }
}
