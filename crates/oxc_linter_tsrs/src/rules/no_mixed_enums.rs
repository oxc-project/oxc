// Port of internal/rules/no_mixed_enums/no_mixed_enums.go.

use tsrs_ast::{Kind, Node};
use tsrs_checker::{Checker, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_mixed_message() -> RuleMessage {
    RuleMessage::new("mixed", "Mixing number and string enums can be confusing.")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AllowedType {
    Number,
    String,
    Unknown,
}

pub struct NoMixedEnums;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoMixedEnums))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::EnumDeclaration)];

impl Rule for NoMixedEnums {
    fn name(&self) -> &'static str {
        "no-mixed-enums"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn get_member_type(c: &mut Checker, node: P<Node>) -> AllowedType {
    let Some(initializer) = node.initializer() else {
        return AllowedType::Number;
    };
    match initializer.kind() {
        Kind::NumericLiteral => AllowedType::Number,
        Kind::StringLiteral => AllowedType::String,
        _ => {
            let t = c.get_type_at_location(initializer);
            if utils::is_type_flag_set(t, TypeFlags::StringLike) {
                return AllowedType::String;
            }
            if utils::is_type_flag_set(t, TypeFlags::NumberLike) {
                return AllowedType::Number;
            }
            AllowedType::Unknown
        }
    }
}

fn get_desired_type_for_definition(c: &mut Checker, node: P<Node>) -> AllowedType {
    let symbol = c.get_symbol_at_location_exported(node.name().unwrap()).unwrap();
    let declaration = symbol.declarations()[0];
    get_member_type(c, declaration.members()[0])
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _listener: Listener, node: P<Node>) {
        let members = node.members();
        if members.is_empty() {
            return;
        }
        let desired_type = get_desired_type_for_definition(ctx.checker, node);
        if desired_type == AllowedType::Unknown {
            return;
        }
        for &member in members {
            let current_type = get_member_type(ctx.checker, member);
            if current_type == AllowedType::Unknown {
                return;
            }
            if current_type != desired_type {
                let init = member.initializer().unwrap_or(member);
                ctx.report_node(init, build_mixed_message());
                return;
            }
        }
    }
}
