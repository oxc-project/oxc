// Port of internal/rules/no_unsafe_return/no_unsafe_return.go.

use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor};
use crate::utils::{self, DiscriminatedAnyType};

type Range = (i32, i32);

fn build_unsafe_return_message(t: &str) -> RuleMessage {
    RuleMessage::new("unsafeReturn", format!("Unsafe return of a value of type {t}."))
}
fn build_unsafe_return_assignment_message(sender: &str, receiver: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeReturnAssignment",
        format!("Unsafe return of type `{sender}` from function with return type `{receiver}`."),
    )
}
fn build_unsafe_return_this_message(t: &str) -> RuleMessage {
    RuleMessage::with_help(
        "unsafeReturnThis",
        format!("Unsafe return of a value of type `{t}`. `this` is typed as `any`."),
        "You can try to fix this by turning on the `noImplicitThis` compiler option, or adding a `this` parameter to the function.",
    )
}

fn build_unsafe_return_diagnostic(
    message: RuleMessage,
    primary_range: Range,
    returned_range: Range,
    returned_type: &str,
    expected_range: Option<Range>,
    expected_type: &str,
) -> RuleDiagnostic {
    let mut diagnostic = RuleDiagnostic {
        pos: primary_range.0,
        end: primary_range.1,
        message,
        labeled_ranges: vec![LabeledRange {
            label: format!("Returned expression has type `{returned_type}`."),
            pos: returned_range.0,
            end: returned_range.1,
        }],
    };
    if let Some(r) = expected_range {
        diagnostic.labeled_ranges.push(LabeledRange {
            label: format!("Function expects return type `{expected_type}`."),
            pos: r.0,
            end: r.1,
        });
    }
    diagnostic
}

fn render_return_type(c: &mut Checker, t: P<Type>) -> String {
    if utils::is_intrinsic_error_type(t) {
        return "error".to_string();
    }
    crate::utils::type_to_string(c, t)
}

pub struct NoUnsafeReturn;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeReturn))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::ArrowFunction), Listener::Enter(Kind::ReturnStatement)];

impl Rule for NoUnsafeReturn {
    fn name(&self) -> &'static str {
        "no-unsafe-return"
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
    fn check_return(&self, ctx: &mut Ctx, return_node: P<Node>, primary_range: Range) {
        let t = ctx.checker.get_type_at_location(return_node);

        let any_type = utils::discriminate_any_type(t, ctx.checker, ctx.program, return_node);
        let Some(function_node) = utils::get_parent_function_node(return_node) else {
            return;
        };

        // function has an explicit return type, so ensure it's a safe return
        let return_node_type = utils::get_constrained_type_at_location(ctx.checker, return_node);

        // function expressions will not have their return type modified based on receiver typing
        // so we have to use the contextual typing in these cases, i.e.
        // const foo1: () => Set<string> = () => new Set<any>();
        // the return type of the arrow function is Set<any> even though the variable is typed as Set<string>
        let mut function_type = None;
        let mut uses_contextual_type = false;
        if ast::is_function_expression(function_node) || ast::is_arrow_function(function_node) {
            function_type = utils::get_contextual_type(ctx.checker, function_node);
            uses_contextual_type = function_type.is_some();
        }
        let function_type =
            function_type.unwrap_or_else(|| ctx.checker.get_type_at_location(function_node));
        let call_signatures = utils::collect_all_call_signatures(ctx.checker, function_type);
        let mut expected_range: Option<Range> = None;
        let mut expected_type = String::new();
        if let Some(return_type_node) = function_node.type_node() {
            expected_range = Some(ctx.trim(return_type_node));
            let rt = ctx.checker.get_type_at_location(return_type_node);
            expected_type = render_return_type(ctx.checker, rt);
        } else if uses_contextual_type {
            for &signature in &call_signatures {
                let Some(declaration) = signature.declaration.get() else {
                    continue;
                };
                let Some(declaration_type) = declaration.type_node() else {
                    continue;
                };
                if ast::get_source_file_of_node(declaration) != Some(ctx.file) {
                    continue;
                }
                expected_range = Some(ctx.trim(declaration_type));
                let rt = ctx.checker.get_return_type_of_signature_exported(signature);
                expected_type = render_return_type(ctx.checker, rt);
                break;
            }
        }
        let report = |ctx: &mut Ctx, message: RuleMessage| {
            let returned_range = ctx.trim(return_node);
            let returned_type = render_return_type(ctx.checker, return_node_type);
            ctx.report_diagnostic(build_unsafe_return_diagnostic(
                message,
                primary_range,
                returned_range,
                &returned_type,
                expected_range,
                &expected_type,
            ));
        };

        // If there is an explicit type annotation *and* that type matches the actual function
        // return type, we shouldn't complain (it's intentional, even if unsafe)
        if function_node.type_node().is_some() {
            for &signature in &call_signatures {
                let signature_return_type =
                    ctx.checker.get_return_type_of_signature_exported(signature);
                if return_node_type == signature_return_type
                    || utils::is_type_flag_set(
                        signature_return_type,
                        TypeFlags::Any | TypeFlags::Unknown,
                    )
                {
                    return;
                }
                if ast::has_syntactic_modifier(function_node, ModifierFlags::Async) {
                    let awaited_signature_return_type =
                        ctx.checker.get_awaited_type(signature_return_type);
                    let awaited_return_node_type = ctx.checker.get_awaited_type(return_node_type);
                    if awaited_signature_return_type == awaited_return_node_type
                        || awaited_signature_return_type.is_some_and(|t| {
                            utils::is_type_flag_set(t, TypeFlags::Any | TypeFlags::Unknown)
                        })
                    {
                        return;
                    }
                }
            }
        }

        if any_type != DiscriminatedAnyType::Safe {
            // Allow cases when the declared return type of the function is either unknown or
            // unknown[] and the function is returning any or any[].
            for &signature in &call_signatures {
                let function_return_type =
                    ctx.checker.get_return_type_of_signature_exported(signature);
                if any_type == DiscriminatedAnyType::Any
                    && utils::is_type_unknown_type(function_return_type)
                {
                    return;
                }
                if any_type == DiscriminatedAnyType::AnyArray
                    && utils::is_type_unknown_array_type(function_return_type, ctx.checker)
                {
                    return;
                }
                let awaited_type = ctx.checker.get_awaited_type(function_return_type);
                if let Some(awaited_type) = awaited_type {
                    if any_type == DiscriminatedAnyType::PromiseAny
                        && utils::is_type_unknown_type(awaited_type)
                    {
                        return;
                    }
                }
            }

            if any_type == DiscriminatedAnyType::PromiseAny
                && !ast::has_syntactic_modifier(function_node, ModifierFlags::Async)
            {
                return;
            }

            let type_string = if utils::is_intrinsic_error_type(return_node_type) {
                "error"
            } else if any_type == DiscriminatedAnyType::Any {
                "`any`"
            } else if any_type == DiscriminatedAnyType::PromiseAny {
                "`Promise<any>`"
            } else {
                "`any[]`"
            };

            if !self.is_no_implicit_this {
                // `return this`
                if let Some(this_expression) = utils::get_this_expression(return_node) {
                    let this_type =
                        utils::get_constrained_type_at_location(ctx.checker, this_expression);
                    if utils::is_type_any_type(this_type) {
                        report(ctx, build_unsafe_return_this_message(type_string));
                        return;
                    }
                }
            }
            // If the function return type was not unknown/unknown[], mark usage as unsafeReturn.
            report(ctx, build_unsafe_return_message(type_string));
            return;
        }

        let Some(&signature) = call_signatures.first() else {
            return;
        };
        let function_return_type = ctx.checker.get_return_type_of_signature_exported(signature);

        let Some((receiver, sender)) = utils::is_unsafe_assignment(
            return_node_type,
            function_return_type,
            ctx.checker,
            Some(return_node),
        ) else {
            return;
        };

        let s = crate::utils::type_to_string(ctx.checker, sender);
        let r = crate::utils::type_to_string(ctx.checker, receiver);
        report(ctx, build_unsafe_return_assignment_message(&s, &r));
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::ArrowFunction => {
                let body = node.body().unwrap();
                if !ast::is_block(body) {
                    let token = node.as_arrow_function().equals_greater_than_token().unwrap();
                    let range = ctx.trim(token);
                    self.check_return(ctx, body, range);
                }
            }
            Kind::ReturnStatement => {
                let Some(argument) = node.expression() else {
                    return;
                };
                let r = tsrs_scanner::get_range_of_token_at_position(ctx.file, node.pos());
                self.check_return(ctx, argument, (r.pos(), r.end()));
            }
            _ => {}
        }
    }
}
