// Port of internal/rules/prefer_return_this_type/prefer_return_this_type.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

fn build_use_this_type_message() -> RuleMessage {
    RuleMessage::new("useThisType", "Use `this` type instead.")
}

pub struct PreferReturnThisType;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(PreferReturnThisType))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::PropertyDeclaration), Listener::Enter(Kind::MethodDeclaration)];

impl Rule for PreferReturnThisType {
    fn name(&self) -> &'static str {
        "prefer-return-this-type"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn try_get_name_in_type_node(name: &str, node: P<Node>) -> Option<P<Node>> {
    let node = ast::skip_parentheses(node);
    if ast::is_type_reference_node(node) {
        let n = node.as_type_reference_node().type_name;
        if ast::is_identifier(n) && n.as_identifier().text() == name {
            return Some(node);
        }
    } else if node.kind() == Kind::UnionType {
        for &t in node.as_union_type_node().types().nodes() {
            if let Some(found) = try_get_name_in_type_node(name, t) {
                return Some(found);
            }
        }
    }
    None
}

fn check_function(ctx: &mut Ctx, func: P<Node>, original_class: P<Node>) {
    let (Some(return_type), Some(body)) = (func.type_node(), func.body()) else {
        return;
    };
    let Some(class_name) = original_class.name() else {
        return;
    };
    let Some(node) = try_get_name_in_type_node(class_name.text(), return_type) else {
        return;
    };
    let params = func.parameters();
    if !params.is_empty() {
        let first_arg = params[0].name().unwrap();
        if ast::is_identifier(first_arg) && first_arg.as_identifier().text() == "this" {
            return;
        }
    }
    let class_type_t = ctx.checker.get_type_at_location(original_class);
    let class_type = class_type_t.try_as_interface_type();
    if ast::is_block(body) {
        let mut has_return_this = false;
        let has_return_class_type = ast::for_each_return_statement(body, |stmt| {
            let Some(expr) = stmt.as_return_statement().expression.get() else {
                return false;
            };
            if expr.kind() == Kind::ThisKeyword {
                has_return_this = true;
                return false;
            }
            let Some(class_type) = class_type else {
                return false;
            };
            let t = ctx.checker.get_type_at_location(expr);
            if class_type_t == t {
                return true;
            }
            if class_type.this_type() == Some(t) {
                has_return_this = true;
            }
            false
        });
        if has_return_class_type || !has_return_this {
            return;
        }
    } else {
        let Some(class_type) = class_type else {
            return;
        };
        let t = ctx.checker.get_type_at_location(body);
        if class_type.this_type() != Some(t) {
            return;
        }
    }
    ctx.report_node_with_fixes(node, build_use_this_type_message(), |ctx| {
        vec![ctx.fix_replace(node, "this")]
    });
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let parent = node.parent().unwrap();
        if !ast::is_class_like(parent) {
            return;
        }
        match node.kind() {
            Kind::PropertyDeclaration => {
                let Some(initializer) = node.as_property_declaration().initializer() else {
                    return;
                };
                if ast::is_function_expression(initializer) || ast::is_arrow_function(initializer) {
                    check_function(ctx, initializer, parent);
                }
            }
            Kind::MethodDeclaration => check_function(ctx, node, parent),
            _ => {}
        }
    }
}
