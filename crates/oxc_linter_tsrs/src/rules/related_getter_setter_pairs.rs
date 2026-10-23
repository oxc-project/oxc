// Port of internal/rules/related_getter_setter_pairs/related_getter_setter_pairs.go.

use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_mismatch_message() -> RuleMessage {
    RuleMessage::new(
        "mismatch",
        "`get()` type should be assignable to its equivalent `set()` type.",
    )
}

pub struct RelatedGetterSetterPairs;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(RelatedGetterSetterPairs))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ClassDeclaration),
    Listener::Enter(Kind::ClassExpression),
    Listener::Enter(Kind::InterfaceDeclaration),
    Listener::Enter(Kind::TypeLiteral),
];

impl Rule for RelatedGetterSetterPairs {
    fn name(&self) -> &'static str {
        "related-getter-setter-pairs"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn check_accessors_pair(ctx: &mut Ctx, getter: P<Node>, setter: P<Node>) {
    let get_type = ctx.checker.get_type_at_location(getter);
    let set_type = ctx.checker.get_type_at_location(setter.parameters()[0]);
    if !ctx.checker.is_type_assignable_to(get_type, set_type) {
        ctx.report_node(getter.type_node().unwrap(), build_mismatch_message());
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.member_list().is_none() {
            return;
        }
        let mut get_accessors: FxHashMap<String, P<Node>> = FxHashMap::default();
        let mut set_accessors: FxHashMap<String, P<Node>> = FxHashMap::default();
        for &member in node.members() {
            if ast::is_get_accessor_declaration(member) {
                if member.type_node().is_none() {
                    continue;
                }
                let (name, _) = utils::get_name_from_member(ctx.file, member.name().unwrap());
                if let Some(&set_accessor) = set_accessors.get(&name) {
                    check_accessors_pair(ctx, member, set_accessor);
                } else {
                    get_accessors.insert(name, member);
                }
            } else if ast::is_set_accessor_declaration(member) {
                if member.parameters().len() != 1 {
                    continue;
                }
                let (name, _) = utils::get_name_from_member(ctx.file, member.name().unwrap());
                if let Some(&get_accessor) = get_accessors.get(&name) {
                    check_accessors_pair(ctx, get_accessor, member);
                } else {
                    set_accessors.insert(name, member);
                }
            }
        }
    }
}
