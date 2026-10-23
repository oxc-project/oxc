// Port of internal/rules/consistent_return/consistent_return.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, FunctionFlags, Kind, Node, NodeFlags};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils;

fn build_missing_return_value_message(function_name_with_kind: &str) -> RuleMessage {
    RuleMessage::new(
        "missingReturnValue",
        format!("{function_name_with_kind} expected a return value."),
    )
}

fn build_unexpected_return_value_message(function_name_with_kind: &str) -> RuleMessage {
    RuleMessage::new(
        "unexpectedReturnValue",
        format!("{function_name_with_kind} expected no return value."),
    )
}

pub struct ConsistentReturn {
    treat_undefined_as_unspecified: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(ConsistentReturn {
        treat_undefined_as_unspecified: opt_bool(&m, "treatUndefinedAsUnspecified", false),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Exit(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Exit(Kind::FunctionExpression),
    Listener::Enter(Kind::ArrowFunction),
    Listener::Exit(Kind::ArrowFunction),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Exit(Kind::MethodDeclaration),
    Listener::Enter(Kind::Constructor),
    Listener::Exit(Kind::Constructor),
    Listener::Enter(Kind::GetAccessor),
    Listener::Exit(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
    Listener::Exit(Kind::SetAccessor),
    Listener::Enter(Kind::ReturnStatement),
];

impl Rule for ConsistentReturn {
    fn name(&self) -> &'static str {
        "consistent-return"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self, stack: Vec::new() })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MessageId {
    None,
    MissingReturnValue,
    UnexpectedReturnValue,
}

struct FunctionState {
    node: P<Node>,
    has_return: bool,
    has_return_value: bool,
    has_mismatch: bool,
    allows_void_return: Option<bool>,
    message_id: MessageId,
}

struct Visitor {
    rule: &'static ConsistentReturn,
    stack: Vec<FunctionState>,
}

fn get_function_name_with_kind(ctx: &Ctx, node: P<Node>) -> String {
    let kind_prefix = if ast::get_function_flags(Some(node)).intersects(FunctionFlags::Async) {
        "Async function"
    } else {
        "Function"
    };
    let Some(name_node) = ast::get_name_of_declaration(node) else {
        return kind_prefix.to_string();
    };
    let name_text = if ast::is_identifier(name_node) {
        name_node.as_identifier().text().to_string()
    } else {
        let (pos, end) = ctx.trim(name_node);
        ctx.text()[pos as usize..end as usize].trim().to_string()
    };
    if name_text.is_empty() {
        return kind_prefix.to_string();
    }
    format!("{kind_prefix} '{name_text}'")
}

fn is_thenable_type_with_void_value(
    c: &mut Checker,
    node: P<Node>,
    t: Option<P<Type>>,
    visited: &mut FxHashSet<P<Type>>,
) -> bool {
    let Some(t) = t else { return false };
    if !visited.insert(t) {
        return false;
    }
    if utils::is_intrinsic_void_type(t) {
        return true;
    }
    if utils::is_union_type(t) || utils::is_intersection_type(t) {
        return t
            .types()
            .iter()
            .any(|&part| is_thenable_type_with_void_value(c, node, Some(part), visited));
    }
    if !utils::is_thenable_type(c, node, Some(t)) {
        return false;
    }
    let awaited_type = c.get_awaited_type(t);
    match awaited_type {
        None => false,
        Some(a) if a == t => false,
        Some(a) => is_thenable_type_with_void_value(c, node, Some(a), visited),
    }
}

fn is_return_void_or_thenable_void(ctx: &mut Ctx, function_node: P<Node>) -> bool {
    let function_type = ctx.checker.get_type_at_location(function_node);
    let call_signatures = utils::get_call_signatures(ctx.checker, function_type);
    if call_signatures.is_empty() {
        return false;
    }
    let is_async_function =
        ast::get_function_flags(Some(function_node)).intersects(FunctionFlags::Async);
    call_signatures.iter().any(|&signature| {
        let return_type = ctx.checker.get_return_type_of_signature(signature);
        if is_async_function {
            return is_thenable_type_with_void_value(
                ctx.checker,
                function_node,
                Some(return_type),
                &mut FxHashSet::default(),
            );
        }
        utils::union_type_parts(return_type).into_iter().any(utils::is_intrinsic_void_type)
    })
}

fn get_has_return_value(
    ctx: &mut Ctx,
    node: P<Node>,
    treat_undefined_as_unspecified: bool,
) -> bool {
    let Some(expression) = node.as_return_statement().expression.get() else {
        return false;
    };
    if !treat_undefined_as_unspecified {
        return true;
    }
    let return_value_type = ctx.checker.get_type_at_location(expression);
    if return_value_type.flags() == TypeFlags::Undefined {
        return false;
    }
    !utils::is_undefined_literal(Some(expression))
}

impl Visitor {
    fn allows_void_return(&mut self, ctx: &mut Ctx) -> bool {
        let f = self.stack.last_mut().unwrap();
        if let Some(v) = f.allows_void_return {
            return v;
        }
        let v = is_return_void_or_thenable_void(ctx, f.node);
        f.allows_void_return = Some(v);
        v
    }

    fn exit_function(&mut self, ctx: &mut Ctx) {
        let Some(f) = self.stack.last() else { return };
        if !f.has_mismatch
            && f.has_return
            && f.has_return_value
            && f.node.flags().intersects(NodeFlags::HasImplicitReturn)
            && !self.allows_void_return(ctx)
        {
            let node = self.stack.last().unwrap().node;
            let name = get_function_name_with_kind(ctx, node);
            ctx.report_node(node, build_missing_return_value_message(&name));
        }
        self.stack.pop();
    }

    fn on_return_statement(&mut self, ctx: &mut Ctx, node: P<Node>) {
        if self.stack.is_empty() {
            return;
        }
        if node.as_return_statement().expression.get().is_none() && self.allows_void_return(ctx) {
            return;
        }
        let has_return_value =
            get_has_return_value(ctx, node, self.rule.treat_undefined_as_unspecified);
        let f = self.stack.last_mut().unwrap();
        if !f.has_return {
            f.has_return = true;
            f.has_return_value = has_return_value;
            f.message_id = if has_return_value {
                MessageId::MissingReturnValue
            } else {
                MessageId::UnexpectedReturnValue
            };
            return;
        }
        if f.has_return_value != has_return_value {
            f.has_mismatch = true;
            let message_id = f.message_id;
            let fn_node = f.node;
            match message_id {
                MessageId::MissingReturnValue => {
                    let name = get_function_name_with_kind(ctx, fn_node);
                    ctx.report_node(node, build_missing_return_value_message(&name));
                }
                MessageId::UnexpectedReturnValue => {
                    let name = get_function_name_with_kind(ctx, fn_node);
                    ctx.report_node(node, build_unexpected_return_value_message(&name));
                }
                MessageId::None => {}
            }
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::ReturnStatement) => self.on_return_statement(ctx, node),
            Listener::Enter(_) => self.stack.push(FunctionState {
                node,
                has_return: false,
                has_return_value: false,
                has_mismatch: false,
                allows_void_return: None,
                message_id: MessageId::None,
            }),
            Listener::Exit(_) => self.exit_function(ctx),
            _ => {}
        }
    }
}
