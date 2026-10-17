// Port of internal/rules/use_unknown_in_catch_callback_variable/use_unknown_in_catch_callback_variable.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};
use crate::utils;

fn build_use_unknown_message_base(method: &str) -> String {
    format!("Prefer the safe `: unknown` for a {method} callback variable.")
}

fn build_add_unknown_rest_type_annotation_suggestion_message() -> RuleMessage {
    RuleMessage::new(
        "addUnknownRestTypeAnnotationSuggestion",
        "Add an explicit `: [unknown]` type annotation to the rejection callback rest variable.",
    )
}
fn build_add_unknown_type_annotation_suggestion_message() -> RuleMessage {
    RuleMessage::new(
        "addUnknownTypeAnnotationSuggestion",
        "Add an explicit `: unknown` type annotation to the rejection callback variable.",
    )
}
fn build_use_unknown_message(method: &str) -> RuleMessage {
    RuleMessage::new("useUnknown", build_use_unknown_message_base(method))
}
fn build_use_unknown_array_destructuring_pattern_message(method: &str) -> RuleMessage {
    RuleMessage::new(
        "useUnknownArrayDestructuringPattern",
        build_use_unknown_message_base(method) + " The thrown error may not be iterable.",
    )
}
fn build_use_unknown_object_destructuring_pattern_message(method: &str) -> RuleMessage {
    RuleMessage::new(
        "useUnknownObjectDestructuringPattern",
        build_use_unknown_message_base(method)
            + " The thrown error may be nullable, or may not have the expected shape.",
    )
}
fn build_wrong_rest_type_annotation_suggestion_message() -> RuleMessage {
    RuleMessage::new(
        "wrongRestTypeAnnotationSuggestion",
        "Change existing type annotation to `: [unknown]`.",
    )
}
fn build_wrong_type_annotation_suggestion_message() -> RuleMessage {
    RuleMessage::new(
        "wrongTypeAnnotationSuggestion",
        "Change existing type annotation to `: unknown`.",
    )
}

pub struct UseUnknownInCatchCallbackVariable;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(UseUnknownInCatchCallbackVariable))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for UseUnknownInCatchCallbackVariable {
    fn name(&self) -> &'static str {
        "use-unknown-in-catch-callback-variable"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_flaggable_handler_type(ctx: &mut Ctx, t: P<Type>) -> bool {
    for part in utils::union_type_parts(t) {
        for &call_signature in utils::get_call_signatures(ctx.checker, part) {
            let params = call_signature.parameters.get();
            if params.is_empty() {
                continue;
            }
            let first_param = params[0];
            let mut first_param_type = ctx.checker.get_type_of_symbol(first_param);
            let decl = first_param.value_declaration();
            if let Some(decl) = decl {
                if decl.as_parameter_declaration().dot_dot_dot_token().is_some() {
                    // a rest arg that's not an array or tuple should definitely be flagged.
                    if !ctx.checker.is_array_or_tuple_type(first_param_type) {
                        return true;
                    }
                    first_param_type = ctx.checker.get_type_arguments(first_param_type)[0];
                }
            }
            if !utils::is_type_flag_set(first_param_type, TypeFlags::Unknown) {
                return true;
            }
        }
    }
    false
}

fn collect_flagged_nodes(ctx: &mut Ctx, node: P<Node>) -> Vec<P<Node>> {
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::BinaryExpression => {
            let n = node.as_binary_expression();
            if ast::is_logical_expression(node) {
                let mut v = collect_flagged_nodes(ctx, n.left);
                v.extend(collect_flagged_nodes(ctx, n.right.get()));
                return v;
            }
            if n.operator_token.kind() == Kind::CommaToken {
                return collect_flagged_nodes(ctx, n.right.get());
            }
        }
        Kind::ConditionalExpression => {
            let n = node.as_conditional_expression();
            let mut v = collect_flagged_nodes(ctx, n.when_true);
            v.extend(collect_flagged_nodes(ctx, n.when_false));
            return v;
        }
        Kind::ArrowFunction | Kind::FunctionExpression => {
            let t = ctx.checker.get_type_at_location(node);
            if is_flaggable_handler_type(ctx, t) {
                return vec![node];
            }
        }
        _ => {}
    }
    Vec::new()
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let expr = node.as_call_expression();
        let callee = expr.expression;
        if !ast::is_access_expression(callee) {
            return;
        }
        let (property_name, found) = ctx.checker.get_accessed_property_name(callee);
        if !found {
            return;
        }
        let (method, arg_index_to_check) = match property_name.as_str() {
            "catch" => ("`catch`", 0usize),
            "then" => ("`then` rejection", 1usize),
            _ => return,
        };
        let args = node.arguments();
        if args.len() < arg_index_to_check + 1 {
            return;
        }
        for (i, &arg) in args.iter().enumerate() {
            if ast::is_spread_element(arg) {
                return;
            }
            if i == arg_index_to_check {
                break;
            }
        }
        let callee_expr_type = ctx.checker.get_type_at_location(callee.expression().unwrap());
        if !utils::is_thenable_type(ctx.checker, callee, Some(callee_expr_type)) {
            return;
        }
        for flagged in collect_flagged_nodes(ctx, args[arg_index_to_check]) {
            let catch_param_node = flagged.parameters()[0];
            let catch_param = catch_param_node.as_parameter_declaration();
            let catch_variable = catch_param.name();
            let catch_type_annotation = catch_param.type_();
            if catch_param.dot_dot_dot_token().is_some() {
                match catch_type_annotation {
                    None => {
                        ctx.report_node_with_suggestions(
                            catch_param_node,
                            build_use_unknown_message(method),
                            |ctx| {
                                vec![RuleSuggestion {
                                    message:
                                        build_add_unknown_rest_type_annotation_suggestion_message(),
                                    fixes: vec![
                                        ctx.fix_insert_after(catch_variable, ": [unknown]"),
                                    ],
                                }]
                            },
                        );
                    }
                    Some(annotation) => {
                        ctx.report_node_with_suggestions(
                            catch_param_node,
                            build_use_unknown_message(method),
                            |ctx| {
                                vec![RuleSuggestion {
                                    message: build_wrong_rest_type_annotation_suggestion_message(),
                                    fixes: vec![ctx.fix_replace(annotation, "[unknown]")],
                                }]
                            },
                        );
                    }
                }
                continue;
            }
            match catch_variable.kind() {
                Kind::Identifier => match catch_type_annotation {
                    None => {
                        ctx.report_node_with_suggestions(
                            catch_param_node,
                            build_use_unknown_message(method),
                            |ctx| {
                                let fixes = if utils::is_parenless_arrow_function(flagged) {
                                    vec![
                                        ctx.fix_insert_before(catch_variable, "("),
                                        ctx.fix_insert_after(catch_variable, ": unknown)"),
                                    ]
                                } else {
                                    let insert_after =
                                        catch_param.question_token().unwrap_or(catch_variable);
                                    vec![ctx.fix_insert_after(insert_after, ": unknown")]
                                };
                                vec![RuleSuggestion {
                                    message: build_add_unknown_type_annotation_suggestion_message(),
                                    fixes,
                                }]
                            },
                        );
                    }
                    Some(annotation) => {
                        ctx.report_node_with_suggestions(
                            catch_param_node,
                            build_use_unknown_message(method),
                            |ctx| {
                                vec![RuleSuggestion {
                                    message: build_wrong_type_annotation_suggestion_message(),
                                    fixes: vec![ctx.fix_replace(annotation, "unknown")],
                                }]
                            },
                        );
                    }
                },
                Kind::ArrayBindingPattern => ctx.report_node(
                    catch_param_node,
                    build_use_unknown_array_destructuring_pattern_message(method),
                ),
                Kind::ObjectBindingPattern => ctx.report_node(
                    catch_param_node,
                    build_use_unknown_object_destructuring_pattern_message(method),
                ),
                _ => {}
            }
        }
    }
}
