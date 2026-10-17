// Port of internal/rules/strict_void_return/strict_void_return.go.

use tsrs_ast::{self as ast, FunctionFlags, Kind, Node};
use tsrs_checker::{ContextFlags, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils;

fn async_func() -> RuleMessage {
    RuleMessage::new(
        "asyncFunc",
        "Async function used in a context where a void function is expected.",
    )
}
fn non_void_func() -> RuleMessage {
    RuleMessage::new(
        "nonVoidFunc",
        "Value-returning function used in a context where a void function is expected.",
    )
}
fn non_void_return() -> RuleMessage {
    RuleMessage::new(
        "nonVoidReturn",
        "Value returned in a context where a void return is expected.",
    )
}

pub struct StrictVoidReturn {
    allowed_return_type_flags: TypeFlags,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let mut allowed_return_type_flags = TypeFlags::Void | TypeFlags::Never | TypeFlags::Undefined;
    if opt_bool(&m, "allowReturnAny", false) {
        allowed_return_type_flags |= TypeFlags::Any;
    }
    Ok(Box::new(StrictVoidReturn { allowed_return_type_flags }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ArrayLiteralExpression),
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::NewExpression),
    Listener::Enter(Kind::JsxAttribute),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::PropertyDeclaration),
    Listener::Enter(Kind::PropertyAssignment),
    Listener::Enter(Kind::ShorthandPropertyAssignment),
    Listener::Enter(Kind::ReturnStatement),
    Listener::Enter(Kind::VariableDeclaration),
];

impl Rule for StrictVoidReturn {
    fn name(&self) -> &'static str {
        "strict-void-return"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static StrictVoidReturn,
}

fn is_void_returning_function_type(ctx: &mut Ctx, t: P<Type>) -> bool {
    let mut return_types = Vec::new();
    for part in utils::union_type_parts(t) {
        for &signature in utils::get_call_signatures(ctx.checker, part) {
            return_types.push(ctx.checker.get_return_type_of_signature(signature));
        }
    }
    !return_types.is_empty()
        && return_types.iter().all(|&rt| {
            utils::union_type_parts(rt)
                .into_iter()
                .all(|p| utils::is_type_flag_set(p, TypeFlags::Void))
        })
}

fn is_nullish_or_any(t: P<Type>) -> bool {
    utils::is_type_flag_set(
        t,
        TypeFlags::VoidLike
            | TypeFlags::Undefined
            | TypeFlags::Null
            | TypeFlags::Any
            | TypeFlags::Never,
    )
}

fn is_void(t: P<Type>) -> bool {
    utils::is_type_flag_set(t, TypeFlags::Void)
}

impl Visitor {
    fn is_allowed_type(&self, t: P<Type>) -> bool {
        utils::union_type_parts(t)
            .into_iter()
            .all(|p| utils::is_type_flag_set(p, self.o.allowed_return_type_flags))
    }

    fn report_if_non_void_function(&self, ctx: &mut Ctx, func_node: P<Node>) {
        let at = ctx.checker.get_type_at_location(func_node);
        let actual_type = ctx.checker.get_apparent_type(at);
        let mut all_allowed = true;
        for &signature in utils::get_call_signatures(ctx.checker, actual_type) {
            let rt = ctx.checker.get_return_type_of_signature(signature);
            if !self.is_allowed_type(rt) {
                all_allowed = false;
                break;
            }
        }
        if all_allowed {
            return;
        }
        if !ast::is_arrow_function(func_node)
            && !ast::is_function_expression(func_node)
            && !ast::is_method_declaration(func_node)
        {
            ctx.report_node(func_node, non_void_func());
            return;
        }
        let function_flags = ast::get_function_flags(Some(func_node));
        if function_flags.intersects(FunctionFlags::Generator) {
            ctx.report_node(func_node, non_void_func());
            return;
        }
        if function_flags.intersects(FunctionFlags::Async) {
            ctx.report_node(func_node, async_func());
            return;
        }
        if let Some(body) = func_node.body() {
            if !ast::is_block(body) {
                ctx.report_node(body, non_void_return());
                return;
            }
        }
        if let Some(return_type_node) = func_node.type_node() {
            if return_type_node.kind() != Kind::VoidKeyword {
                ctx.report_node(return_type_node, non_void_func());
                return;
            }
        }
        self.visit_returns(ctx, func_node, func_node);
    }

    fn visit_returns(&self, ctx: &mut Ctx, func_node: P<Node>, node: P<Node>) {
        if node != func_node && ast::is_function_like(node) {
            return;
        }
        if ast::is_return_statement(node) {
            if let Some(expression) = node.expression() {
                let return_type = ctx.checker.get_type_at_location(expression);
                if !self.is_allowed_type(return_type) {
                    ctx.report_node(node, non_void_return());
                }
            }
        }
        for child in node.iter_children() {
            self.visit_returns(ctx, func_node, child);
        }
    }

    fn check_expression_node(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        if let Some(expected_type) = ctx.checker.get_contextual_type(node, ContextFlags::None) {
            if is_void_returning_function_type(ctx, expected_type) {
                self.report_if_non_void_function(ctx, node);
                return true;
            }
        }
        false
    }

    fn check_function_call_node(&self, ctx: &mut Ctx, call_node: P<Node>) {
        let args = call_node.arguments();
        if args.is_empty() {
            return;
        }
        let callee = call_node.expression().unwrap();
        let func_type = ctx.checker.get_type_at_location(callee);
        let mut signatures = Vec::new();
        for part in utils::union_type_parts(func_type) {
            if ast::is_call_expression(call_node) {
                signatures.extend_from_slice(utils::get_call_signatures(ctx.checker, part));
            } else {
                signatures.extend_from_slice(utils::get_construct_signatures(ctx.checker, part));
            }
        }
        for (arg_idx, &arg_node) in args.iter().enumerate() {
            if arg_node.kind() == Kind::SpreadElement {
                continue;
            }
            let mut arg_expected_return_types = Vec::new();
            for &sig in &signatures {
                let parameters = sig.parameters.get();
                if arg_idx >= parameters.len() {
                    continue;
                }
                let param_type = ctx
                    .checker
                    .get_type_of_symbol_at_location(parameters[arg_idx], Some(callee))
                    .unwrap();
                for part in utils::union_type_parts(param_type) {
                    for &ps in utils::get_call_signatures(ctx.checker, part) {
                        arg_expected_return_types
                            .push(ctx.checker.get_return_type_of_signature(ps));
                    }
                }
            }
            let has_single_signature = signatures.len() == 1;
            let all_signatures_return_void = arg_expected_return_types
                .iter()
                .all(|&rt| is_void(rt) || is_nullish_or_any(rt) || utils::is_type_parameter(rt));
            if (has_single_signature || all_signatures_return_void)
                && self.check_expression_node(ctx, arg_node)
            {
                continue;
            }
            if arg_expected_return_types.iter().any(|&t| is_void(t))
                && arg_expected_return_types.iter().all(|&t| is_nullish_or_any(t))
            {
                self.report_if_non_void_function(ctx, arg_node);
            }
        }
    }

    fn get_member_name(ctx: &mut Ctx, name_node: Option<P<Node>>) -> String {
        let Some(name_node) = name_node else {
            return String::new();
        };
        if let Some(symbol) = ctx.checker.get_symbol_at_location_exported(name_node) {
            if !symbol.name().is_empty() {
                return symbol.name().to_string();
            }
        }
        match name_node.kind() {
            Kind::Identifier
            | Kind::PrivateIdentifier
            | Kind::StringLiteral
            | Kind::NumericLiteral
            | Kind::BigIntLiteral => return name_node.text().to_string(),
            Kind::ComputedPropertyName => {
                let expr = name_node.as_computed_property_name().expression;
                if matches!(
                    expr.kind(),
                    Kind::Identifier
                        | Kind::StringLiteral
                        | Kind::NumericLiteral
                        | Kind::BigIntLiteral
                ) {
                    return expr.text().to_string();
                }
            }
            _ => {}
        }
        String::new()
    }

    fn get_base_member_types(ctx: &mut Ctx, member_node: P<Node>) -> Vec<P<Type>> {
        let Some(class_like_node) = member_node.parent() else {
            return Vec::new();
        };
        let Some(heritage_clauses) = utils::get_heritage_clauses(class_like_node) else {
            return Vec::new();
        };
        let Some(member_name_node) = member_node.name() else {
            return Vec::new();
        };
        let Some(member_symbol) = ctx.checker.get_symbol_at_location_exported(member_name_node)
        else {
            return Vec::new();
        };
        let mut base_member_types = Vec::new();
        for &heritage_clause in heritage_clauses.nodes() {
            for &heritage_type_node in heritage_clause.as_heritage_clause().types.get().nodes() {
                let heritage_type = ctx.checker.get_type_at_location(heritage_type_node);
                let Some(heritage_member) =
                    ctx.checker.get_property_of_type(heritage_type, member_symbol.name())
                else {
                    continue;
                };
                base_member_types.push(
                    ctx.checker
                        .get_type_of_symbol_at_location(heritage_member, Some(member_node))
                        .unwrap(),
                );
            }
        }
        base_member_types
    }

    fn check_object_method_node(&self, ctx: &mut Ctx, method_node: P<Node>) {
        if method_node.name().is_some_and(ast::is_computed_property_name) {
            return;
        }
        let Some(obj_type) =
            ctx.checker.get_contextual_type(method_node.parent().unwrap(), ContextFlags::None)
        else {
            return;
        };
        let member_name = Self::get_member_name(ctx, method_node.name());
        if member_name.is_empty() {
            return;
        }
        let Some(property_symbol) = ctx.checker.get_property_of_type(obj_type, &member_name) else {
            return;
        };
        let expected_type =
            ctx.checker.get_type_of_symbol_at_location(property_symbol, Some(method_node)).unwrap();
        if is_void_returning_function_type(ctx, expected_type) {
            self.report_if_non_void_function(ctx, method_node);
        }
    }

    fn check_class_method_node(&self, ctx: &mut Ctx, method_node: P<Node>) {
        if method_node.body().is_none() {
            return;
        }
        let base_member_types = Self::get_base_member_types(ctx, method_node);
        if base_member_types.into_iter().any(|t| is_void_returning_function_type(ctx, t)) {
            self.report_if_non_void_function(ctx, method_node);
        }
    }

    fn check_class_property_node(&self, ctx: &mut Ctx, property_node: P<Node>) {
        let initializer = property_node.initializer();
        for base_member_type in Self::get_base_member_types(ctx, property_node) {
            if is_void_returning_function_type(ctx, base_member_type) {
                if let Some(initializer) = initializer {
                    self.report_if_non_void_function(ctx, initializer);
                    return;
                }
            }
        }
        if let Some(initializer) = initializer {
            self.check_expression_node(ctx, initializer);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::ArrayLiteralExpression => {
                for &elem in node.as_array_literal_expression().elements.nodes() {
                    if elem.kind() != Kind::SpreadElement {
                        self.check_expression_node(ctx, elem);
                    }
                }
            }
            Kind::ArrowFunction => {
                if let Some(body) = node.body() {
                    if !ast::is_block(body) {
                        self.check_expression_node(ctx, body);
                    }
                }
            }
            Kind::BinaryExpression if ast::is_assignment_expression(node, false) => {
                self.check_expression_node(ctx, node.as_binary_expression().right.get());
            }
            Kind::CallExpression | Kind::NewExpression => self.check_function_call_node(ctx, node),
            Kind::JsxAttribute => {
                let Some(initializer) = node.as_jsx_attribute().initializer() else {
                    return;
                };
                if initializer.kind() != Kind::JsxExpression {
                    return;
                }
                if let Some(expression) = initializer.as_jsx_expression().expression() {
                    if !ast::is_omitted_expression(expression) {
                        self.check_expression_node(ctx, expression);
                    }
                }
            }
            Kind::MethodDeclaration => {
                if node.parent().is_some_and(ast::is_object_literal_expression) {
                    self.check_object_method_node(ctx, node);
                    return;
                }
                self.check_class_method_node(ctx, node);
            }
            Kind::PropertyDeclaration => self.check_class_property_node(ctx, node),
            Kind::PropertyAssignment => {
                self.check_expression_node(ctx, node.initializer().unwrap());
            }
            Kind::ShorthandPropertyAssignment => {
                self.check_expression_node(ctx, node.name().unwrap());
            }
            Kind::ReturnStatement | Kind::VariableDeclaration => {
                let e = if node.kind() == Kind::ReturnStatement {
                    node.expression()
                } else {
                    node.initializer()
                };
                if let Some(e) = e {
                    self.check_expression_node(ctx, e);
                }
            }
            _ => {}
        }
    }
}
