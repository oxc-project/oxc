// Port of internal/rules/only_throw_error/only_throw_error.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::TypeFlags;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils::{self, TypeOrValueSpecifier};

fn object() -> RuleMessage {
    RuleMessage::new("object", "Expected an error object to be thrown.")
}
fn undef() -> RuleMessage {
    RuleMessage::new("undef", "Do not throw undefined.")
}

/// Whether the node is a rethrown caught error:
/// 1. try { } catch (e) { throw e; }
/// 2. promise.catch(e => { throw e; })
/// 3. promise.then(onFulfilled, e => { throw e; })
fn is_rethrown_error(ctx: &mut Ctx, node: P<Node>) -> bool {
    if !ast::is_identifier(node) {
        return false;
    }
    let Some(decl) = utils::get_declaration(ctx.checker, node) else {
        return false;
    };
    let Some(func_node) = decl.parent() else {
        return false;
    };
    // Case 1: try { } catch (e) { throw e; }
    if ast::is_catch_clause(func_node) {
        return true;
    }
    // Case 2 & 3: the declaration must be the first, non-rest parameter of an arrow function that
    // is a direct argument of a .catch() / .then() call.
    if !ast::is_parameter_declaration(decl) {
        return false;
    }
    if decl.as_parameter_declaration().dot_dot_dot_token().is_some() {
        return false;
    }
    if !ast::is_arrow_function(func_node) {
        return false;
    }
    let params = func_node.parameters();
    if params.is_empty() || params[0] != decl {
        return false;
    }
    let Some(call) = func_node.parent() else {
        return false;
    };
    if !ast::is_call_expression(call) {
        return false;
    }
    let callee = call.expression().unwrap();
    if !ast::is_property_access_expression(callee) {
        return false;
    }
    let method_name = callee.name().unwrap().text();
    let args = call.arguments();
    let is_rejection_handler = match method_name {
        // .catch(onRejected)
        "catch" => !args.is_empty() && args[0] == func_node,
        // .then(onFulfilled, onRejected); onFulfilled must not be a spread element.
        "then" => args.len() >= 2 && args[1] == func_node && !ast::is_spread_element(args[0]),
        _ => false,
    };
    if !is_rejection_handler {
        return false;
    }
    // Verify that the object is actually a thenable (Promise)
    let object_node = callee.expression().unwrap();
    let object_type = ctx.checker.get_type_at_location(object_node);
    utils::is_thenable_type(ctx.checker, object_node, Some(object_type))
}

pub struct OnlyThrowError {
    allow: Vec<TypeOrValueSpecifier>,
    allow_rethrowing: bool,
    allow_throwing_any: bool,
    allow_throwing_unknown: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(OnlyThrowError {
        allow: utils::unmarshal_type_or_value_specifiers(m.get("allow"))
            .map_err(|e| format!("only-throw-error: failed to unmarshal options: {e}"))?,
        allow_rethrowing: opt_bool(&m, "allowRethrowing", true),
        allow_throwing_any: opt_bool(&m, "allowThrowingAny", true),
        allow_throwing_unknown: opt_bool(&m, "allowThrowingUnknown", true),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::ThrowStatement)];

impl Rule for OnlyThrowError {
    fn name(&self) -> &'static str {
        "only-throw-error"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static OnlyThrowError,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let expr = node.expression().unwrap();
        let t = ctx.checker.get_type_at_location(expr);
        if utils::type_matches_some_specifier(t, &self.o.allow, ctx.program) {
            return;
        }
        if utils::is_type_flag_set(t, TypeFlags::Undefined) {
            ctx.report_node(node, undef());
            return;
        }
        if self.o.allow_throwing_any && utils::is_type_any_type(t) {
            return;
        }
        if self.o.allow_throwing_unknown && utils::is_type_unknown_type(t) {
            return;
        }
        if self.o.allow_rethrowing && is_rethrown_error(ctx, expr) {
            return;
        }
        if utils::is_error_like(ctx.program, ctx.checker, t) {
            return;
        }
        ctx.report_node(expr, object());
    }
}
