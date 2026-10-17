// Port of internal/rules/no_generated_empty_object_type/no_generated_empty_object_type.go (tsgolint 7.0.2003).

use tsrs_ast::{Kind, Node};
use tsrs_checker::{Checker, ObjectFlags, SignatureKind, Type};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

pub struct NoGeneratedEmptyObjectType;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoGeneratedEmptyObjectType))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::IntersectionType), Listener::Enter(Kind::TypeReference)];

impl Rule for NoGeneratedEmptyObjectType {
    fn name(&self) -> &'static str {
        "no-generated-empty-object-type"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_empty_object_type(c: &mut Checker, t: P<Type>) -> bool {
    utils::is_object_type(t)
        && !t.object_flags().intersects(ObjectFlags::ClassOrInterface)
        && c.get_properties_of_type(t).is_empty()
        && c.get_index_infos_of_type(t).is_empty()
        && c.get_signatures_of_type(t, SignatureKind::Call).is_empty()
        && c.get_signatures_of_type(t, SignatureKind::Construct).is_empty()
        // Unresolved mapped types can have no members and accept number, but reject string. Both primitives
        // must be assignable to {}.
        && c.is_type_assignable_to(c.number_type, t)
        && c.is_type_assignable_to(c.string_type, t)
}

fn check_node(ctx: &mut Ctx, node: P<Node>) {
    let t = ctx.checker.get_type_at_location(node);
    for part in utils::union_type_parts(t) {
        if is_empty_object_type(ctx.checker, part) {
            ctx.report_node(
                node,
                RuleMessage::new(
                    "noGeneratedEmptyObjectType",
                    "This type resolves to `{}`, the empty object type. This was likely not intentional.",
                ),
            );
            return;
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::IntersectionType => check_node(ctx, node),
            Kind::TypeReference => {
                let mut parent = node.parent().unwrap();
                while parent.kind() == Kind::ParenthesizedType {
                    parent = parent.parent().unwrap();
                }
                if node
                    .as_type_reference_node()
                    .node_with_type_arguments_base
                    .type_arguments
                    .is_some()
                    && parent.kind() != Kind::IntersectionType
                {
                    check_node(ctx, node);
                }
            }
            _ => {}
        }
    }
}
