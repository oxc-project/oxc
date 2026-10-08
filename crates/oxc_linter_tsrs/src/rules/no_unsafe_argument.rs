// Port of internal/rules/no_unsafe_argument/no_unsafe_argument.go.

use tsrs_ast::{self as ast, Kind, Node, Symbol};
use tsrs_checker::{Checker, ElementFlags, Type};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_unsafe_argument_message(sender: &str, receiver: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeArgument",
        format!("Unsafe argument of type {sender} assigned to a parameter of type {receiver}."),
    )
}
fn build_unsafe_array_spread_message(sender: &str) -> RuleMessage {
    RuleMessage::new("unsafeArraySpread", format!("Unsafe spread of an {sender} array type."))
}
fn build_unsafe_spread_message(sender: &str) -> RuleMessage {
    RuleMessage::new("unsafeSpread", format!("Unsafe spread of an {sender} type."))
}
fn build_unsafe_tuple_spread_message(sender: &str, receiver: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeTupleSpread",
        format!(
            "Unsafe spread of a tuple type. The argument is {sender} and is assigned to a parameter of type {receiver}."
        ),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RestTypeKind {
    Array,
    Tuple,
    Other,
}

struct RestType {
    index: usize,
    kind: RestTypeKind,
    t: Option<P<Type>>,
    type_arguments: &'static [P<Type>],
}

struct FunctionSignature {
    has_consumed_arguments: bool,
    parameter_type_index: usize,
    node: P<Node>,
    // parameters holds the non-rest parameters; param_types caches their types, resolved lazily so
    // that skipped argument positions never request them.
    parameters: Vec<P<Symbol>>,
    param_types: Vec<Option<P<Type>>>,
    rest_param: Option<P<Symbol>>,
    rest_type: Option<RestType>,
}

impl FunctionSignature {
    fn new(c: &mut Checker, node: P<Node>) -> FunctionSignature {
        let signature = c.get_resolved_signature_exported(node);
        let mut parameters: Vec<P<Symbol>> = signature.parameters.get().to_vec();
        let mut rest_param = None;
        for (i, &param) in parameters.iter().enumerate() {
            let decls = param.declarations();
            if !decls.is_empty() && utils::is_rest_parameter_declaration(decls[0]) {
                // is a rest param
                rest_param = Some(param);
                parameters.truncate(i);
                break;
            }
        }
        let n = parameters.len();
        FunctionSignature {
            has_consumed_arguments: false,
            parameter_type_index: 0,
            node,
            parameters,
            param_types: vec![None; n],
            rest_param,
            rest_type: None,
        }
    }

    fn consume_remaining_arguments(&mut self) {
        self.has_consumed_arguments = true;
    }

    /// Advances past the parameter position of an argument that doesn't need checking, without
    /// resolving the parameter's type.
    fn skip_parameter(&mut self) {
        self.parameter_type_index += 1;
    }

    fn get_rest_type(&mut self, c: &mut Checker) -> &RestType {
        if self.rest_type.is_none() {
            let mut rest_t = RestType {
                index: self.parameters.len(),
                kind: RestTypeKind::Other,
                t: None,
                type_arguments: &[],
            };
            if let Some(rest_param) = self.rest_param {
                let t = c.get_type_of_symbol_at_location(rest_param, Some(self.node)).unwrap();
                if c.is_array_type(t) {
                    rest_t.kind = RestTypeKind::Array;
                    rest_t.t = Some(c.get_type_arguments(t)[0]);
                } else if t.is_tuple_type() {
                    rest_t.kind = RestTypeKind::Tuple;
                    rest_t.type_arguments = c.get_type_arguments(t);
                } else {
                    rest_t.t = Some(t);
                }
            }
            self.rest_type = Some(rest_t);
        }
        self.rest_type.as_ref().unwrap()
    }

    fn get_next_parameter_type(&mut self, c: &mut Checker) -> Option<P<Type>> {
        let index = self.parameter_type_index;
        self.parameter_type_index += 1;

        if index >= self.param_types.len() || self.has_consumed_arguments {
            let has_consumed_arguments = self.has_consumed_arguments;
            let rest_type = self.get_rest_type(c);
            match rest_type.kind {
                RestTypeKind::Tuple => {
                    let type_arguments = rest_type.type_arguments;
                    if type_arguments.is_empty() {
                        return None;
                    }
                    if has_consumed_arguments {
                        // all types consumed by a rest - just assume it's the last type
                        return Some(type_arguments[type_arguments.len() - 1]);
                    }
                    let type_index = index - rest_type.index;
                    if type_index >= type_arguments.len() {
                        return Some(type_arguments[type_arguments.len() - 1]);
                    }
                    return Some(type_arguments[type_index]);
                }
                RestTypeKind::Array | RestTypeKind::Other => return rest_type.t,
            }
        }
        if self.param_types[index].is_none() {
            self.param_types[index] =
                c.get_type_of_symbol_at_location(self.parameters[index], Some(self.node));
        }
        self.param_types[index]
    }
}

/// Whether an argument expression could have a type that IsUnsafeAssignment flags: `any`, or a
/// generic type reference with unsafe type arguments.
fn argument_can_be_unsafe(node: P<Node>) -> bool {
    match node.kind() {
        Kind::StringLiteral
        | Kind::NoSubstitutionTemplateLiteral
        | Kind::TemplateExpression
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword
        | Kind::NullKeyword
        | Kind::RegularExpressionLiteral
        | Kind::ArrowFunction
        | Kind::FunctionExpression => false,
        // Spreading an `any` value makes the whole object literal `any`, so only
        // spread-free literals are safe.
        Kind::ObjectLiteralExpression => node
            .as_object_literal_expression()
            .properties
            .nodes()
            .iter()
            .any(|&p| ast::is_spread_assignment(p)),
        _ => true,
    }
}

pub struct NoUnsafeArgument;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeArgument))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::NewExpression),
    Listener::Enter(Kind::TaggedTemplateExpression),
];

impl Rule for NoUnsafeArgument {
    fn name(&self) -> &'static str {
        "no-unsafe-argument"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn describe_type(c: &mut Checker, t: P<Type>) -> String {
    if utils::is_intrinsic_error_type(t) {
        return "error typed".to_string();
    }
    utils::type_to_string(c, t)
}

fn describe_type_for_spread(c: &mut Checker, t: P<Type>) -> String {
    if c.is_array_type(t) && utils::is_intrinsic_error_type(c.get_type_arguments(t)[0]) {
        return "error".to_string();
    }
    describe_type(c, t)
}

fn describe_type_for_tuple(c: &mut Checker, t: P<Type>) -> String {
    if utils::is_intrinsic_error_type(t) {
        return "error typed".to_string();
    }
    format!("of type {}", utils::type_to_string(c, t))
}

fn check_unsafe_arguments(ctx: &mut Ctx, args: &[P<Node>], callee: P<Node>, node: P<Node>) {
    // A report requires at least one argument whose type could be unsafe; skip the callee/signature
    // type queries when none qualifies.
    if !args.iter().any(|&a| argument_can_be_unsafe(a)) {
        return;
    }

    // ignore any-typed calls as these are caught by no-unsafe-call
    let callee_type = ctx.checker.get_type_at_location(callee);
    if utils::is_type_any_type(callee_type) {
        return;
    }

    let mut signature = FunctionSignature::new(ctx.checker, node);

    if ast::is_tagged_template_expression(node) {
        // Consumes the first parameter (TemplateStringsArray) of the function called with
        // TaggedTemplateExpression.
        signature.skip_parameter();
    }

    for &argument in args {
        match argument.kind() {
            // spreads consume
            Kind::SpreadElement => {
                let spread_arg_type =
                    ctx.checker.get_type_at_location(argument.expression().unwrap());
                if utils::is_type_any_type(spread_arg_type) {
                    // foo(...any)
                    let d = describe_type(ctx.checker, spread_arg_type);
                    ctx.report_node(argument, build_unsafe_spread_message(&d));
                } else if utils::is_type_any_array_type(spread_arg_type, ctx.checker) {
                    // foo(...any[])
                    // TODO - we could break down the spread and compare the array type against each argument
                    let d = describe_type_for_spread(ctx.checker, spread_arg_type);
                    ctx.report_node(argument, build_unsafe_array_spread_message(&d));
                } else if spread_arg_type.is_tuple_type() {
                    // foo(...[tuple1, tuple2])
                    let spread_type_arguments = ctx.checker.get_type_arguments(spread_arg_type);
                    for &tuple_type in spread_type_arguments {
                        let Some(parameter_type) = signature.get_next_parameter_type(ctx.checker)
                        else {
                            continue;
                        };
                        // we can't pass the individual tuple members in here as this will most
                        // likely be a spread variable not a spread array
                        if utils::is_unsafe_assignment(
                            tuple_type,
                            parameter_type,
                            ctx.checker,
                            None,
                        )
                        .is_some()
                        {
                            let s = describe_type_for_tuple(ctx.checker, tuple_type);
                            let r = describe_type(ctx.checker, parameter_type);
                            ctx.report_node(argument, build_unsafe_tuple_spread_message(&s, &r));
                        }
                    }
                    if spread_arg_type
                        .target()
                        .unwrap()
                        .as_tuple_type()
                        .combined_flags
                        .get()
                        .intersects(ElementFlags::Variable)
                    {
                        // the last element was a rest - so all remaining defined arguments can be
                        // considered "consumed"; all remaining arguments should be compared against
                        // the rest type (if one exists)
                        signature.consume_remaining_arguments();
                    }
                } else {
                    // something that's iterable
                    // handling this will be pretty complex - so we ignore it for now
                    // TODO - handle generic iterable case
                }
            }
            _ => {
                if !argument_can_be_unsafe(argument) {
                    signature.skip_parameter();
                    continue;
                }
                let Some(parameter_type) = signature.get_next_parameter_type(ctx.checker) else {
                    continue;
                };
                let argument_type = ctx.checker.get_type_at_location(argument);
                if utils::is_unsafe_assignment(
                    argument_type,
                    parameter_type,
                    ctx.checker,
                    Some(argument),
                )
                .is_some()
                {
                    let s = describe_type(ctx.checker, argument_type);
                    let r = describe_type(ctx.checker, parameter_type);
                    ctx.report_node(argument, build_unsafe_argument_message(&s, &r));
                }
            }
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::CallExpression | Kind::NewExpression => {
                check_unsafe_arguments(ctx, node.arguments(), node.expression().unwrap(), node);
            }
            Kind::TaggedTemplateExpression => {
                let expr = node.as_tagged_template_expression();
                let template = expr.template;
                if ast::is_template_expression(template) {
                    let args: Vec<P<Node>> = template
                        .as_template_expression()
                        .template_spans()
                        .nodes()
                        .iter()
                        .map(|span| span.expression().unwrap())
                        .collect();
                    check_unsafe_arguments(ctx, &args, expr.tag, node);
                }
            }
            _ => {}
        }
    }
}
