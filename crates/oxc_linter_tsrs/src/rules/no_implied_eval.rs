// Port of internal/rules/no_implied_eval/no_implied_eval.go.

use tsrs_ast::{self as ast, Kind, Node, SymbolFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn no_function_constructor() -> RuleMessage {
    RuleMessage::new(
        "noFunctionConstructor",
        "Implied eval. Do not use the Function constructor to create functions.",
    )
}
fn no_implied_eval_error() -> RuleMessage {
    RuleMessage::with_help("noImpliedEvalError", "Implied eval.", "Consider passing a function.")
}

const GLOBAL_CANDIDATES: &[&str] = &["global", "globalThis", "window"];
const EVAL_LIKE_FUNCTIONS: &[&str] = &["execScript", "setImmediate", "setInterval", "setTimeout"];
const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::CallExpression), Listener::Enter(Kind::NewExpression)];

pub struct NoImpliedEval;

impl Rule for NoImpliedEval {
    fn name(&self) -> &'static str {
        "no-implied-eval"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn get_callee_name(node: P<Node>) -> String {
    if ast::is_identifier(node) {
        return node.text().to_string();
    }
    if ast::is_access_expression(node) {
        if let Some(e) = node.expression() {
            if ast::is_identifier(e) && GLOBAL_CANDIDATES.contains(&e.text()) {
                if ast::is_property_access_expression(node) {
                    if let Some(name) = node.name() {
                        if ast::is_identifier(name) {
                            return name.text().to_string();
                        }
                    }
                } else if ast::is_element_access_expression(node) {
                    let arg = node.as_element_access_expression().argument_expression;
                    if ast::is_string_literal(arg) {
                        return arg.text().to_string();
                    }
                }
            }
        }
    }
    String::new()
}

fn is_function_type(ctx: &mut Ctx, node: P<Node>) -> bool {
    let t = ctx.checker.get_type_at_location(node);
    if utils::is_symbol_flag_set(t.symbol(), SymbolFlags::Function | SymbolFlags::Method) {
        return true;
    }
    if utils::is_builtin_symbol_like(ctx.program, ctx.checker, t, &["Function"]) {
        return true;
    }
    !utils::get_call_signatures(ctx.checker, t).is_empty()
}

fn is_bind(mut node: P<Node>) -> bool {
    if ast::is_property_access_expression(node) {
        node = node.name().unwrap();
    }
    ast::is_identifier(node) && node.text() == "bind"
}

fn is_function(ctx: &mut Ctx, node: P<Node>) -> bool {
    if ast::is_function_like(node) {
        return true;
    }
    if ast::is_literal_expression(node) {
        return false;
    }
    if ast::is_call_expression(node) {
        return is_bind(node.expression().unwrap()) || is_function_type(ctx, node);
    }
    is_function_type(ctx, node)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let expression = node.expression().unwrap();
        let callee_name = get_callee_name(expression);
        if callee_name.is_empty() {
            return;
        }
        if callee_name == "Function" {
            let t = ctx.checker.get_type_at_location(expression);
            if t.symbol().is_some() {
                if utils::is_builtin_symbol_like(
                    ctx.program,
                    ctx.checker,
                    t,
                    &["FunctionConstructor"],
                ) {
                    ctx.report_node(node, no_function_constructor());
                    return;
                }
            } else {
                ctx.report_node(node, no_function_constructor());
            }
        }
        let args = node.arguments();
        if args.is_empty() {
            return;
        }
        let handler = args[0];
        if EVAL_LIKE_FUNCTIONS.contains(&callee_name.as_str()) && !is_function(ctx, handler) {
            let symbol = ctx.checker.get_symbol_at_location_exported(expression);
            let declared_here = symbol.is_some_and(|s| {
                s.declarations().iter().any(|&d| ast::get_source_file_of_node(d) == Some(ctx.file))
            });
            if !declared_here {
                ctx.report_node(handler, no_implied_eval_error());
            }
        }
    }
}
