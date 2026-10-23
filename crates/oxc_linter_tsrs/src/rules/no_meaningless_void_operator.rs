// Port of internal/rules/no_meaningless_void_operator/no_meaningless_void_operator.go.

use tsrs_ast::{Kind, Node};
use tsrs_checker::TypeFlags;
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor, opt_bool, options_object,
};
use crate::utils;

fn meaningless_void_operator(t: &str) -> RuleMessage {
    RuleMessage::new(
        "meaninglessVoidOperator",
        format!(
            "void operator shouldn't be used on {t}; it should convey that a return value is being ignored"
        ),
    )
}
fn remove_void() -> RuleMessage {
    RuleMessage::new("removeVoid", "Remove 'void'")
}

pub struct NoMeaninglessVoidOperator {
    check_never: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(NoMeaninglessVoidOperator { check_never: opt_bool(&m, "checkNever", false) }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VoidExpression)];

impl Rule for NoMeaninglessVoidOperator {
    fn name(&self) -> &'static str {
        "no-meaningless-void-operator"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static NoMeaninglessVoidOperator,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let arg = node.expression().unwrap();
        let arg_type = ctx.checker.get_type_at_location(arg);
        let union_parts = utils::union_type_parts(arg_type);
        let is_always_void_like =
            union_parts.iter().all(|&t| utils::is_type_flag_set(t, TypeFlags::VoidLike));
        let is_always_void_like_or_never = union_parts
            .iter()
            .all(|&t| utils::is_type_flag_set(t, TypeFlags::VoidLike | TypeFlags::Never));
        let fix_remove_void_keyword = |ctx: &Ctx| {
            let (pos, _) = ctx.trim(node);
            ctx.fix_remove_range(pos, arg.pos())
        };
        if is_always_void_like {
            let msg = meaningless_void_operator(&utils::type_to_string(ctx.checker, arg_type));
            ctx.report_node_with_fixes(node, msg, |ctx| vec![fix_remove_void_keyword(ctx)]);
        } else if self.o.check_never && is_always_void_like_or_never {
            let msg = meaningless_void_operator(&utils::type_to_string(ctx.checker, arg_type));
            ctx.report_node_with_suggestions(node, msg, |ctx| {
                vec![RuleSuggestion {
                    message: remove_void(),
                    fixes: vec![fix_remove_void_keyword(ctx)],
                }]
            });
        }
    }
}
