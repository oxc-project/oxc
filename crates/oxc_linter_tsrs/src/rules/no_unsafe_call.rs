// Port of internal/rules/no_unsafe_call/no_unsafe_call.go.

use tsrs_ast::{Kind, Node};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_unsafe_call_message(t: &str) -> RuleMessage {
    RuleMessage::new("unsafeCall", format!("Unsafe call of a(n) {t} typed value."))
}
fn build_unsafe_call_this_message(t: &str) -> RuleMessage {
    RuleMessage::with_help(
        "unsafeCallThis",
        format!("Unsafe call of a(n) {t} typed value. `this` is typed as {t}.\n"),
        "You can try to fix this by turning on the `noImplicitThis` compiler option, or adding a `this` parameter to the function.",
    )
}
fn build_unsafe_new_message(t: &str) -> RuleMessage {
    RuleMessage::new("unsafeNew", format!("Unsafe construction of a(n) {t} typed value."))
}
fn build_unsafe_template_tag_message(t: &str) -> RuleMessage {
    RuleMessage::new("unsafeTemplateTag", format!("Unsafe use of a(n) {t} typed template tag."))
}

pub struct NoUnsafeCall;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeCall))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::NewExpression),
    Listener::Enter(Kind::TaggedTemplateExpression),
];

impl Rule for NoUnsafeCall {
    fn name(&self) -> &'static str {
        "no-unsafe-call"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let compiler_options = ctx.program.options();
        let is_no_implicit_this = utils::is_strict_compiler_option_enabled(
            &compiler_options,
            compiler_options.no_implicit_this,
        );
        Box::new(Visitor { is_no_implicit_this })
    }
}

struct Visitor {
    is_no_implicit_this: bool,
}

impl Visitor {
    fn check_call(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        reporting_node: P<Node>,
        mut message_builder: fn(&str) -> RuleMessage,
        new_call: bool,
    ) {
        let t = utils::get_constrained_type_at_location(ctx.checker, node);

        if utils::is_type_any_type(t) {
            if !self.is_no_implicit_this {
                // `this()` or `this.foo()` or `this.foo[bar]()`
                if let Some(this_expression) = utils::get_this_expression(node) {
                    let this_type =
                        utils::get_constrained_type_at_location(ctx.checker, this_expression);
                    if utils::is_type_any_type(this_type) {
                        message_builder = build_unsafe_call_this_message;
                    }
                }
            }
            let is_error_type = utils::is_intrinsic_error_type(t);
            let msg = if is_error_type { "`error` type" } else { "`any`" };
            ctx.report_node(reporting_node, message_builder(msg));
            return;
        }

        if utils::is_builtin_symbol_like(ctx.program, ctx.checker, t, &["Function"]) {
            // this also matches subtypes of `Function`, like `interface Foo extends Function {}`.
            //
            // safe to construct if:
            // - they have at least one call signature _that is not void-returning_,
            // - OR they have at least one construct signature.
            //
            // safe to call (including as template) if:
            // - they have at least one call signature
            // - OR they have at least one construct signature.
            if !utils::get_construct_signatures(ctx.checker, t).is_empty() {
                return;
            }
            let call_signatures = utils::get_call_signatures(ctx.checker, t);
            if new_call {
                for &signature in call_signatures {
                    let return_type = ctx.checker.get_return_type_of_signature_exported(signature);
                    if !utils::is_intrinsic_void_type(return_type) {
                        return;
                    }
                }
            } else if !call_signatures.is_empty() {
                return;
            }
            ctx.report_node(reporting_node, message_builder("`Function`"));
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::CallExpression => {
                let callee = node.expression().unwrap();
                if callee.kind() == Kind::ImportKeyword {
                    return;
                }
                self.check_call(ctx, callee, callee, build_unsafe_call_message, false);
            }
            Kind::NewExpression => {
                let callee = node.expression().unwrap();
                self.check_call(ctx, callee, node, build_unsafe_new_message, true);
            }
            Kind::TaggedTemplateExpression => {
                let tag = node.as_tagged_template_expression().tag;
                self.check_call(ctx, tag, tag, build_unsafe_template_tag_message, false);
            }
            _ => {}
        }
    }
}
