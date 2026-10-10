// Port of internal/rules/no_unsafe_member_access/no_unsafe_member_access.go.

use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::Type;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils;

fn is_optional_chain(node: P<Node>) -> bool {
    if ast::is_property_access_expression(node) {
        return node.as_property_access_expression().question_dot_token().is_some();
    }
    if ast::is_element_access_expression(node) {
        return node.as_element_access_expression().question_dot_token().is_some();
    }
    false
}

fn build_unsafe_computed_member_access_message(property: &str, t: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeComputedMemberAccess",
        format!("Computed name {property} resolves to an {t} value."),
    )
}
fn build_unsafe_member_expression_message(property: &str, t: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeMemberExpression",
        format!("Unsafe member access {property} on an {t} value."),
    )
}
fn build_unsafe_this_member_expression_message(property: &str) -> RuleMessage {
    RuleMessage::with_help(
        "unsafeThisMemberExpression",
        format!("Unsafe member access {property} on an `any` value. `this` is typed as `any`."),
        "You can try to fix this by turning on the `noImplicitThis` compiler option, or adding a `this` parameter to the function.",
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Unsafe,
    Safe,
    Chained,
}

fn create_data_type(t: P<Type>) -> &'static str {
    if utils::is_intrinsic_error_type(t) {
        return "`error` typed";
    }
    "`any`"
}

pub struct NoUnsafeMemberAccess {
    allow_optional_chaining: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(NoUnsafeMemberAccess {
        allow_optional_chaining: opt_bool(&m, "allowOptionalChaining", false),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::PropertyAccessExpression),
    Listener::Enter(Kind::ElementAccessExpression),
];

impl Rule for NoUnsafeMemberAccess {
    fn name(&self) -> &'static str {
        "no-unsafe-member-access"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let compiler_options = ctx.program.options();
        let is_no_implicit_this = utils::is_strict_compiler_option_enabled(
            &compiler_options,
            compiler_options.no_implicit_this,
        );
        Box::new(Visitor {
            allow_optional_chaining: self.allow_optional_chaining,
            is_no_implicit_this,
            state_cache: FxHashMap::default(),
        })
    }
}

struct Visitor {
    allow_optional_chaining: bool,
    is_no_implicit_this: bool,
    state_cache: FxHashMap<P<Node>, State>,
}

impl Visitor {
    fn check_member_expression(&mut self, ctx: &mut Ctx, node: P<Node>) -> State {
        if self.allow_optional_chaining && is_optional_chain(node) {
            self.state_cache.insert(node, State::Chained);
            return State::Chained;
        }

        if let Some(&cached_state) = self.state_cache.get(&node) {
            return cached_state;
        }

        let mut parent = node.parent().unwrap();
        while !ast::is_source_file(parent) {
            // ignore MemberExpressions with ancestors of type `TSClassImplements` or `TSInterfaceHeritage`
            if ast::is_heritage_clause(parent) {
                return State::Safe;
            }
            parent = parent.parent().unwrap();
        }

        let expression = node.expression().unwrap();
        if ast::is_access_expression(expression) {
            let object_state = self.check_member_expression(ctx, expression);
            if object_state == State::Unsafe {
                // if the object is unsafe, we know this will be unsafe as well; we don't need to
                // report, as we have already reported on the inner member expr
                self.state_cache.insert(node, object_state);
                return object_state;
            }
        }

        let t = ctx.checker.get_type_at_location(expression);
        let state = if utils::is_type_any_type(t) { State::Unsafe } else { State::Safe };
        self.state_cache.insert(node, state);

        if state == State::Unsafe {
            let (property, property_name) = if ast::is_property_access_expression(node) {
                let property = node.name().unwrap();
                let (pos, end) = ctx.trim(property);
                (property, format!(".{}", &ctx.text()[pos as usize..end as usize]))
            } else {
                let property = node.as_element_access_expression().argument_expression;
                let (pos, end) = ctx.trim(property);
                (property, format!("[{}]", &ctx.text()[pos as usize..end as usize]))
            };

            if !self.is_no_implicit_this {
                // `this.foo` or `this.foo[bar]`
                if let Some(this_expression) = utils::get_this_expression(node) {
                    let this_type =
                        utils::get_constrained_type_at_location(ctx.checker, this_expression);
                    if utils::is_type_any_type(this_type) {
                        ctx.report_node(
                            property,
                            build_unsafe_this_member_expression_message(&property_name),
                        );
                        return state;
                    }
                }
            }

            ctx.report_node(
                property,
                build_unsafe_member_expression_message(&property_name, create_data_type(t)),
            );
        }

        state
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::PropertyAccessExpression => {
                self.check_member_expression(ctx, node);
            }
            Kind::ElementAccessExpression => {
                self.check_member_expression(ctx, node);

                // Skip computed property check if allowOptionalChaining and this is an optional chain
                if self.allow_optional_chaining && is_optional_chain(node) {
                    return;
                }

                let arg = node.as_element_access_expression().argument_expression;
                // x[1]
                if ast::is_literal_expression(arg) {
                    // perf optimizations - literals can obviously never be `any`
                    return;
                }

                // x[1++] x[++x] etc
                // FUN FACT - **all** update expressions return type number, regardless of the
                // argument's type, because JS engines return NaN if the argument is not a number.
                let unary_operator_kind = if ast::is_prefix_unary_expression(arg) {
                    arg.as_prefix_unary_expression().operator
                } else if arg.kind() == Kind::PostfixUnaryExpression {
                    arg.as_postfix_unary_expression().operator
                } else {
                    Kind::Unknown
                };
                if unary_operator_kind == Kind::PlusPlusToken
                    || unary_operator_kind == Kind::MinusMinusToken
                {
                    return;
                }

                let t = ctx.checker.get_type_at_location(arg);
                if utils::is_type_any_type(t) {
                    let (pos, end) = ctx.trim(arg);
                    let property_name = format!("[{}]", &ctx.text()[pos as usize..end as usize]);
                    ctx.report_node(
                        arg,
                        build_unsafe_computed_member_access_message(
                            &property_name,
                            create_data_type(t),
                        ),
                    );
                }
            }
            _ => {}
        }
    }
}
