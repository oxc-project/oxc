// Port of internal/rules/no_misused_promises/no_misused_promises.go.

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node, Symbol};
use tsrs_checker::{ContextFlags, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor, options_object,
};
use crate::utils;

fn conditional() -> RuleMessage {
    RuleMessage::new("conditional", "Expected non-Promise value in a boolean conditional.")
}
fn predicate() -> RuleMessage {
    RuleMessage::new("predicate", "Expected a non-Promise value to be returned.")
}
fn spread() -> RuleMessage {
    RuleMessage::new("spread", "Expected a non-Promise value to be spread in an object.")
}
fn void_return_argument() -> RuleMessage {
    RuleMessage::new(
        "voidReturnArgument",
        "Promise returned in function argument where a void return was expected.",
    )
}
fn void_return_attribute() -> RuleMessage {
    RuleMessage::new(
        "voidReturnAttribute",
        "Promise-returning function provided to attribute where a void return was expected.",
    )
}
fn void_return_inherited_method(heritage_type_name: &str) -> RuleMessage {
    RuleMessage::new(
        "voidReturnInheritedMethod",
        format!(
            "Promise-returning method provided where a void return was expected by extended/implemented type '{heritage_type_name}'."
        ),
    )
}
fn void_return_property() -> RuleMessage {
    RuleMessage::new(
        "voidReturnProperty",
        "Promise-returning function provided to property where a void return was expected.",
    )
}
fn void_return_return_value() -> RuleMessage {
    RuleMessage::new(
        "voidReturnReturnValue",
        "Promise-returning function provided to return value where a void return was expected.",
    )
}
fn void_return_variable() -> RuleMessage {
    RuleMessage::new(
        "voidReturnVariable",
        "Promise-returning function provided to variable where a void return was expected.",
    )
}

pub struct NoMisusedPromises {
    checks_conditionals: bool,
    variables: bool,
    listeners: Vec<Listener>,
}

/// Go ChecksVoidReturnOptions.UnmarshalJSON: every field defaults to true when absent or null.
fn void_return_flag(m: &serde_json::Map<String, serde_json::Value>, key: &str) -> bool {
    match m.get(key) {
        None | Some(serde_json::Value::Null) => true,
        Some(serde_json::Value::Bool(b)) => *b,
        _ => false,
    }
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let checks_conditionals = void_return_flag(&m, "checksConditionals");
    let checks_spreads = void_return_flag(&m, "checksSpreads");
    // checksVoidReturn: bool or object (absent / null = true with default nested options).
    let (checks_void_return, vr): (bool, Option<[bool; 6]>) = match m.get("checksVoidReturn") {
        None | Some(serde_json::Value::Null) => (true, Some([true; 6])),
        Some(serde_json::Value::Bool(b)) => (*b, if *b { Some([true; 6]) } else { None }),
        Some(serde_json::Value::Object(o)) => (
            true,
            Some([
                void_return_flag(o, "arguments"),
                void_return_flag(o, "attributes"),
                void_return_flag(o, "inheritedMethods"),
                void_return_flag(o, "properties"),
                void_return_flag(o, "returns"),
                void_return_flag(o, "variables"),
            ]),
        ),
        // Any other JSON value leaves Go's defaults: checksVoidReturn = true, no nested options.
        Some(_) => (true, None),
    };
    let mut listeners = vec![Listener::Enter(Kind::BinaryExpression)];
    if checks_conditionals {
        listeners.extend([
            Listener::Enter(Kind::PropertyAccessExpression),
            Listener::Enter(Kind::ElementAccessExpression),
            Listener::Enter(Kind::PrefixUnaryExpression),
            Listener::Enter(Kind::ConditionalExpression),
            Listener::Enter(Kind::ForStatement),
            Listener::Enter(Kind::DoStatement),
            Listener::Enter(Kind::WhileStatement),
            Listener::Enter(Kind::IfStatement),
        ]);
    }
    let mut variables = false;
    if let (true, Some([arguments, attributes, inherited_methods, properties, returns, vars])) =
        (checks_void_return, vr)
    {
        variables = vars;
        if arguments {
            listeners.push(Listener::Enter(Kind::CallExpression));
            listeners.push(Listener::Enter(Kind::NewExpression));
        }
        if attributes {
            listeners.push(Listener::Enter(Kind::JsxAttribute));
        }
        if inherited_methods {
            listeners.push(Listener::Enter(Kind::ClassDeclaration));
            listeners.push(Listener::Enter(Kind::ClassExpression));
            listeners.push(Listener::Enter(Kind::InterfaceDeclaration));
        }
        if properties {
            listeners.push(Listener::Enter(Kind::PropertyAssignment));
            listeners.push(Listener::Enter(Kind::MethodDeclaration));
            listeners.push(Listener::Enter(Kind::ShorthandPropertyAssignment));
        }
        if returns {
            listeners.push(Listener::Enter(Kind::ReturnStatement));
        }
        if vars {
            listeners.push(Listener::Enter(Kind::VariableDeclaration));
        }
    }
    if checks_spreads {
        listeners.push(Listener::Enter(Kind::SpreadElement));
        listeners.push(Listener::Enter(Kind::SpreadAssignment));
    }
    Ok(Box::new(NoMisusedPromises { checks_conditionals, variables, listeners }))
}

impl Rule for NoMisusedPromises {
    fn name(&self) -> &'static str {
        "no-misused-promises"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self, checked_nodes: FxHashSet::default() })
    }
}

struct Visitor {
    o: &'static NoMisusedPromises,
    checked_nodes: FxHashSet<P<Node>>,
}

#[derive(Clone, Copy)]
struct VoidExpectation {
    declaration: P<Node>,
    t: Option<P<Type>>,
}

fn any_signature_is_thenable_type(ctx: &mut Ctx, node: P<Node>, t: P<Type>) -> bool {
    for &sig in utils::get_call_signatures(ctx.checker, t) {
        let rt = ctx.checker.get_return_type_of_signature(sig);
        if utils::is_thenable_type(ctx.checker, node, Some(rt)) {
            return true;
        }
    }
    false
}

fn returns_thenable(ctx: &mut Ctx, node: P<Node>) -> bool {
    let at = ctx.checker.get_type_at_location(node);
    let t = ctx.checker.get_apparent_type(at);
    utils::union_type_parts(t).into_iter().any(|t| any_signature_is_thenable_type(ctx, node, t))
}

fn local_expectation_for_declaration(
    ctx: &Ctx,
    declaration: Option<P<Node>>,
    t: P<Type>,
) -> Option<VoidExpectation> {
    let declaration = declaration?;
    if ast::get_source_file_of_node(declaration) == Some(ctx.file) {
        return Some(VoidExpectation { declaration, t: Some(t) });
    }
    None
}

fn local_expectation_for_symbol(
    ctx: &Ctx,
    symbol: Option<P<Symbol>>,
    t: P<Type>,
) -> Option<VoidExpectation> {
    let symbol = symbol?;
    let mut declarations: Vec<P<Node>> = Vec::with_capacity(symbol.declarations().len() + 1);
    if let Some(v) = symbol.value_declaration() {
        declarations.push(v);
    }
    declarations.extend(symbol.declarations().iter().copied());
    for declaration in declarations {
        if let Some(e) = local_expectation_for_declaration(ctx, Some(declaration), t) {
            return Some(e);
        }
    }
    None
}

fn expectation_range(ctx: &Ctx, expectation: VoidExpectation) -> (i32, i32) {
    let declaration = expectation.declaration;
    if let Some(t) = declaration.type_node() {
        return ctx.trim(t);
    }
    if let Some(n) = declaration.name() {
        return ctx.trim(n);
    }
    ctx.trim(declaration)
}

fn promise_range(ctx: &Ctx, node: P<Node>) -> (i32, i32) {
    let node = ast::skip_parentheses(node);
    if ast::is_function_like(node) {
        if let Some(t) = node.type_node() {
            return ctx.trim(t);
        }
        if ast::is_arrow_function(node) {
            return ctx.trim(node.as_arrow_function().equals_greater_than_token.unwrap());
        }
        return utils::get_function_head_loc(ctx.file, node);
    }
    ctx.trim(node)
}

fn build_diagnostic(
    ctx: &mut Ctx,
    node: P<Node>,
    primary: (i32, i32),
    message: RuleMessage,
    expectation: Option<VoidExpectation>,
) -> RuleDiagnostic {
    let t = ctx.checker.get_type_at_location(node);
    let value_description = if ast::is_function_like(ast::skip_parentheses(node))
        || !utils::get_call_signatures(ctx.checker, t).is_empty()
    {
        "This callback"
    } else {
        "This expression"
    };
    let ts = utils::type_to_string(ctx.checker, t);
    let (pp, pe) = promise_range(ctx, node);
    let mut labeled_ranges = vec![LabeledRange {
        label: format!("{value_description} has type `{ts}`."),
        pos: pp,
        end: pe,
    }];
    if let Some(e) = expectation {
        if let Some(et) = e.t {
            let ets = utils::type_to_string(ctx.checker, et);
            let (ep, ee) = expectation_range(ctx, e);
            labeled_ranges.push(LabeledRange {
                label: format!(
                    "This context accepts a `void`-returning callback through type `{ets}`."
                ),
                pos: ep,
                end: ee,
            });
        }
    }
    RuleDiagnostic { pos: primary.0, end: primary.1, message, labeled_ranges }
}

fn report_node(
    ctx: &mut Ctx,
    node: P<Node>,
    message: RuleMessage,
    expectation: Option<VoidExpectation>,
) {
    let r = promise_range(ctx, node);
    let d = build_diagnostic(ctx, node, r, message, expectation);
    ctx.report_diagnostic(d);
}

fn check_array_predicates(ctx: &mut Ctx, node: P<Node>) {
    let Some(parent) = node.parent() else { return };
    if !ast::is_call_expression(parent) {
        return;
    }
    let arguments = parent.arguments();
    if arguments.is_empty() {
        return;
    }
    let callback = arguments[0];
    if utils::is_array_method_call_with_predicate(ctx.checker, parent)
        && returns_thenable(ctx, callback)
    {
        report_node(ctx, callback, predicate(), None);
    }
}

fn is_function_param(ctx: &mut Ctx, param: P<Symbol>, node: P<Node>) -> bool {
    let Some(st) = ctx.checker.get_type_of_symbol_at_location(param, Some(node)) else {
        return false;
    };
    let t = ctx.checker.get_apparent_type(st);
    utils::union_type_parts(t)
        .into_iter()
        .any(|t| !utils::get_call_signatures(ctx.checker, t).is_empty())
}

/// Variation on the thenable check which requires all forms of the type (read: alternates in a
/// union) to be thenable.
fn is_always_thenable(ctx: &mut Ctx, node: P<Node>) -> bool {
    let t = ctx.checker.get_type_at_location(node);
    let apparent = ctx.checker.get_apparent_type(t);
    for sub_type in utils::union_type_parts(apparent) {
        let Some(then_prop) = ctx.checker.get_property_of_type(sub_type, "then") else {
            return false;
        };
        let then_type = ctx.checker.get_type_of_symbol_at_location(then_prop, Some(node)).unwrap();
        let mut has_thenable_signature = false;
        for sub in utils::union_type_parts(then_type) {
            for &signature in utils::get_call_signatures(ctx.checker, sub) {
                let params = signature.parameters.get();
                if !params.is_empty() && is_function_param(ctx, params[0], node) {
                    has_thenable_signature = true;
                }
            }
            if has_thenable_signature {
                break;
            }
        }
        if !has_thenable_signature {
            return false;
        }
    }
    true
}

fn get_member_if_exists(ctx: &mut Ctx, t: P<Type>, member_name: &str) -> Option<P<Symbol>> {
    if let Some(symbol) = t.symbol() {
        if let Some(m) = symbol.members().and_then(|m| m.lookup(member_name)) {
            return Some(m);
        }
    }
    ctx.checker.get_property_of_type(t, member_name)
}

fn is_void_returning_function_type(ctx: &mut Ctx, node: P<Node>, t: P<Type>) -> bool {
    let mut had_void_return = false;
    for t in utils::union_type_parts(t) {
        for &sig in utils::get_call_signatures(ctx.checker, t) {
            let return_type = ctx.checker.get_return_type_of_signature(sig);
            // If a certain positional argument accepts both thenable and void returns, a
            // promise-returning function is valid.
            if utils::is_thenable_type(ctx.checker, node, Some(return_type)) {
                return false;
            }
            had_void_return =
                had_void_return || utils::is_type_flag_set(return_type, TypeFlags::Void);
        }
    }
    had_void_return
}

fn check_heritage_type_for_member_returning_void(
    ctx: &mut Ctx,
    node_member: P<Node>,
    heritage_type: P<Type>,
    member_name: &str,
) {
    let Some(heritage_member) = get_member_if_exists(ctx, heritage_type, member_name) else {
        return;
    };
    let member_type =
        ctx.checker.get_type_of_symbol_at_location(heritage_member, Some(node_member)).unwrap();
    if !is_void_returning_function_type(ctx, node_member, member_type) {
        return;
    }
    let name = utils::type_to_string(ctx.checker, heritage_type);
    let e = local_expectation_for_symbol(ctx, Some(heritage_member), member_type);
    report_node(ctx, node_member, void_return_inherited_method(&name), e);
}

fn check_jsx_attribute(ctx: &mut Ctx, node: P<Node>) {
    let a = node.as_jsx_attribute();
    let Some(initializer) = a.initializer() else {
        return;
    };
    if initializer.kind() != Kind::JsxExpression {
        return;
    }
    let Some(expression) = initializer.as_jsx_expression().expression() else {
        return;
    };
    let Some(contextual_type) = ctx.checker.get_contextual_type(initializer, ContextFlags::None)
    else {
        return;
    };
    if is_void_returning_function_type(ctx, initializer, contextual_type)
        && returns_thenable(ctx, expression)
    {
        let mut expectation = None;
        if let Some(attributes_type) =
            ctx.checker.get_contextual_type(node.parent().unwrap(), ContextFlags::None)
        {
            let property_symbol =
                ctx.checker.get_property_of_type(attributes_type, node.name().unwrap().text());
            expectation = local_expectation_for_symbol(ctx, property_symbol, contextual_type);
        }
        report_node(ctx, expression, void_return_attribute(), expectation);
    }
}

fn check_spread(ctx: &mut Ctx, node: P<Node>) {
    let expression = node.expression().unwrap();
    if utils::is_thenable_type(ctx.checker, expression, None) {
        let (pos, _) = ctx.trim(node);
        let (end, _) = ctx.trim(expression);
        let d = build_diagnostic(ctx, expression, (pos, end), spread(), None);
        ctx.report_diagnostic(d);
    }
}

fn is_thenable_returning_function_type(ctx: &mut Ctx, node: P<Node>, t: P<Type>) -> bool {
    utils::union_type_parts(t).into_iter().any(|t| any_signature_is_thenable_type(ctx, node, t))
}

struct ArgumentState {
    thenable_return_indices: Vec<usize>,
    void_return_indices: Vec<usize>,
    void_return_expectations: FxHashMap<usize, VoidExpectation>,
}

fn check_thenable_or_void_argument(
    ctx: &mut Ctx,
    node: P<Node>,
    t: P<Type>,
    index: usize,
    state: &mut ArgumentState,
    parameter_declaration: Option<P<Node>>,
    from_contextual_type: bool,
) {
    let callee = node.expression().unwrap();
    if is_thenable_returning_function_type(ctx, callee, t) {
        state.thenable_return_indices.push(index);
    } else if is_void_returning_function_type(ctx, callee, t)
        && !state.thenable_return_indices.contains(&index)
    {
        state.void_return_indices.push(index);
        if let Some(expectation) = local_expectation_for_declaration(ctx, parameter_declaration, t)
        {
            if !state.void_return_expectations.contains_key(&index) || !from_contextual_type {
                state.void_return_expectations.insert(index, expectation);
            }
        }
    }
    let contextual_type = ctx.checker.get_contextual_type_for_argument_at_index(node, index as i32);
    if let Some(contextual_type) = contextual_type {
        if contextual_type != t {
            check_thenable_or_void_argument(
                ctx,
                node,
                contextual_type,
                index,
                state,
                parameter_declaration,
                true,
            );
        }
    }
}

/// The positions of arguments which are void functions (and not also thenable functions).
fn void_function_arguments(
    ctx: &mut Ctx,
    node: P<Node>,
) -> (Vec<usize>, FxHashMap<usize, VoidExpectation>) {
    // 'new' can be used without any arguments, as in 'let b = new Object;'.
    if node.argument_list().is_none() {
        return (Vec::new(), FxHashMap::default());
    }
    let num_args = node.arguments().len();
    let mut state = ArgumentState {
        thenable_return_indices: Vec::new(),
        void_return_indices: Vec::new(),
        void_return_expectations: FxHashMap::default(),
    };
    let callee = node.expression().unwrap();
    let t = ctx.checker.get_type_at_location(callee);
    for sub_type in utils::union_type_parts(t) {
        let signatures = if ast::is_call_expression(node) {
            utils::get_call_signatures(ctx.checker, sub_type)
        } else {
            utils::get_construct_signatures(ctx.checker, sub_type)
        };
        for &signature in signatures {
            for (index, &parameter) in signature.parameters.get().iter().enumerate() {
                let parameter_declaration = parameter.value_declaration();
                let mut t =
                    ctx.checker.get_type_of_symbol_at_location(parameter, Some(callee)).unwrap();
                if parameter_declaration.is_some_and(utils::is_rest_parameter_declaration) {
                    if ctx.checker.is_array_type(t) {
                        t = ctx.checker.get_type_arguments(t)[0];
                        for i in index..num_args {
                            check_thenable_or_void_argument(
                                ctx,
                                node,
                                t,
                                i,
                                &mut state,
                                parameter_declaration,
                                false,
                            );
                        }
                    } else if t.is_tuple_type() {
                        let type_args = ctx.checker.get_type_arguments(t);
                        let mut i = index;
                        while i < num_args && i - index < type_args.len() {
                            check_thenable_or_void_argument(
                                ctx,
                                node,
                                type_args[i - index],
                                i,
                                &mut state,
                                parameter_declaration,
                                false,
                            );
                            i += 1;
                        }
                    }
                } else {
                    check_thenable_or_void_argument(
                        ctx,
                        node,
                        t,
                        index,
                        &mut state,
                        parameter_declaration,
                        false,
                    );
                }
            }
        }
    }
    let ArgumentState {
        thenable_return_indices,
        mut void_return_indices,
        mut void_return_expectations,
    } = state;
    for index in thenable_return_indices {
        if let Some(at) = void_return_indices.iter().position(|&i| i == index) {
            void_return_indices.remove(at);
            void_return_expectations.remove(&index);
        }
    }
    (void_return_indices, void_return_expectations)
}

fn check_arguments(ctx: &mut Ctx, node: P<Node>) {
    let (void_args, expectations) = void_function_arguments(ctx, node);
    if void_args.is_empty() {
        return;
    }
    for (index, &argument) in node.arguments().iter().enumerate() {
        if !void_args.contains(&index) {
            continue;
        }
        if returns_thenable(ctx, argument) {
            report_node(ctx, argument, void_return_argument(), expectations.get(&index).copied());
        }
    }
}

fn check_class_like_or_interface_node(ctx: &mut Ctx, node: P<Node>) {
    let Some(heritage_clauses) = utils::get_heritage_clauses(node) else {
        return;
    };
    if heritage_clauses.nodes().is_empty() {
        return;
    }
    let mut heritage_types: Option<Vec<P<Type>>> = None;
    for &node_member in node.members() {
        let Some(name) = node_member.name() else {
            continue;
        };
        if !(ast::is_identifier(name)
            || ast::is_private_identifier(name)
            || ast::is_string_literal(name)
            || ast::is_numeric_literal(name)
            || ast::is_big_int_literal(name))
        {
            continue;
        }
        let member_name = name.text();
        if ast::is_static(node_member) {
            continue;
        }
        if !returns_thenable(ctx, node_member) {
            continue;
        }
        if heritage_types.is_none() {
            let mut v = Vec::new();
            for &h in heritage_clauses.nodes() {
                for &n in h.as_heritage_clause().types.get().nodes() {
                    v.push(ctx.checker.get_type_at_location(n));
                }
            }
            heritage_types = Some(v);
        }
        for &heritage_type in heritage_types.as_ref().unwrap() {
            check_heritage_type_for_member_returning_void(
                ctx,
                node_member,
                heritage_type,
                member_name,
            );
        }
    }
}

fn local_property_expectation(
    ctx: &mut Ctx,
    property_node: P<Node>,
    t: P<Type>,
) -> Option<VoidExpectation> {
    let name = property_node.name()?;
    let parent = property_node.parent()?;
    if !ast::is_object_literal_expression(parent) {
        return None;
    }
    let obj_type = ctx.checker.get_contextual_type(parent, ContextFlags::None)?;
    let property_symbol = ctx.checker.get_property_of_type(obj_type, name.text());
    local_expectation_for_symbol(ctx, property_symbol, t)
}

fn check_property(ctx: &mut Ctx, node: P<Node>) {
    if ast::is_property_assignment(node) {
        let initializer = node.as_property_assignment().initializer();
        let Some(contextual_type) =
            ctx.checker.get_contextual_type(initializer, ContextFlags::None)
        else {
            return;
        };
        if is_void_returning_function_type(ctx, initializer, contextual_type)
            && returns_thenable(ctx, initializer)
        {
            let e = local_property_expectation(ctx, node, contextual_type);
            report_node(ctx, initializer, void_return_property(), e);
        }
    } else if ast::is_shorthand_property_assignment(node) {
        let name = node.name().unwrap();
        let Some(contextual_type) = ctx.checker.get_contextual_type(name, ContextFlags::None)
        else {
            return;
        };
        if is_void_returning_function_type(ctx, name, contextual_type)
            && returns_thenable(ctx, name)
        {
            let e = local_property_expectation(ctx, node, contextual_type);
            report_node(ctx, name, void_return_property(), e);
        }
    } else if ast::is_method_declaration(node) {
        let name = node.name().unwrap();
        if ast::is_computed_property_name(name) {
            return;
        }
        let obj = node.parent().unwrap();
        if !ast::is_object_literal_expression(obj) {
            return;
        }
        if !returns_thenable(ctx, node) {
            return;
        }
        let Some(obj_type) = ctx.checker.get_contextual_type(obj, ContextFlags::None) else {
            return;
        };
        let Some(property_symbol) = ctx.checker.get_property_of_type(obj_type, name.text()) else {
            return;
        };
        let contextual_type =
            ctx.checker.get_type_of_symbol_at_location(property_symbol, Some(name)).unwrap();
        if is_void_returning_function_type(ctx, name, contextual_type) {
            let e = local_expectation_for_symbol(ctx, Some(property_symbol), contextual_type);
            report_node(ctx, node, void_return_property(), e);
        }
    }
}

/// A syntactic check to see if an annotated type is maybe a function type.
fn is_possibly_function_type(node: P<Node>) -> bool {
    match node.kind() {
        Kind::ConditionalType
        | Kind::ConstructorType
        | Kind::FunctionType
        | Kind::ImportType
        | Kind::IndexedAccessType
        | Kind::InferType
        | Kind::IntersectionType
        | Kind::QualifiedName
        | Kind::ThisType
        | Kind::TypeOperator
        | Kind::TypeQuery
        | Kind::TypeReference
        | Kind::UnionType => true,
        Kind::TypeLiteral => node
            .members()
            .iter()
            .any(|m| m.kind() == Kind::CallSignature || m.kind() == Kind::ConstructSignature),
        _ => false,
    }
}

fn check_return_statement(ctx: &mut Ctx, node: P<Node>) {
    let Some(expression) = node.expression() else {
        return;
    };
    let function_node = {
        let mut current = node.parent();
        while let Some(c) = current {
            if ast::is_function_like(c) {
                break;
            }
            current = c.parent();
        }
        current
    };
    if let Some(f) = function_node {
        if let Some(t) = f.type_node() {
            if !is_possibly_function_type(t) {
                return;
            }
        }
    }
    let Some(contextual_type) = ctx.checker.get_contextual_type(expression, ContextFlags::None)
    else {
        return;
    };
    if is_void_returning_function_type(ctx, expression, contextual_type)
        && returns_thenable(ctx, expression)
    {
        let mut expectation = None;
        if let Some(f) = function_node {
            if f.type_node().is_some() {
                expectation = Some(VoidExpectation { declaration: f, t: Some(contextual_type) });
            } else if let Some(function_type) =
                ctx.checker.get_contextual_type(f, ContextFlags::None)
            {
                // The enclosing function may have no contextual type of its own, e.g. a getter
                // whose type comes from its paired setter's parameter.
                for &signature in utils::get_call_signatures(ctx.checker, function_type) {
                    expectation = local_expectation_for_declaration(
                        ctx,
                        signature.declaration.get(),
                        contextual_type,
                    );
                    if expectation.is_some() {
                        break;
                    }
                }
            }
        }
        report_node(ctx, expression, void_return_return_value(), expectation);
    }
}

fn check_assignment(ctx: &mut Ctx, node: P<Node>) {
    let b = node.as_binary_expression();
    let left = b.left;
    let right = b.right.get();
    let var_type = ctx.checker.get_type_at_location(left);
    if !is_void_returning_function_type(ctx, left, var_type) {
        return;
    }
    if returns_thenable(ctx, right) {
        let mut symbol = ctx.checker.get_symbol_at_location_exported(left);
        if symbol.is_none() {
            if let Some(name) = left.name() {
                symbol = ctx.checker.get_symbol_at_location_exported(name);
            }
        }
        let e = local_expectation_for_symbol(ctx, symbol, var_type);
        report_node(ctx, right, void_return_variable(), e);
    }
}

fn check_variable_declaration(ctx: &mut Ctx, node: P<Node>) {
    let Some(initializer) = node.initializer() else {
        return;
    };
    let Some(type_node) = node.type_node() else {
        return;
    };
    if !is_possibly_function_type(type_node) {
        return;
    }
    let var_type = ctx.checker.get_type_at_location(node.name().unwrap());
    if !is_void_returning_function_type(ctx, initializer, var_type) {
        return;
    }
    if returns_thenable(ctx, initializer) {
        report_node(
            ctx,
            initializer,
            void_return_variable(),
            Some(VoidExpectation { declaration: node, t: Some(var_type) }),
        );
    }
}

impl Visitor {
    fn check_conditional(&mut self, ctx: &mut Ctx, node: Option<P<Node>>, is_test_expr: bool) {
        let Some(node) = node else { return };
        if ast::is_assignment_expression(node, false) {
            return;
        }
        // prevent checking the same node multiple times
        if !self.checked_nodes.insert(node) {
            return;
        }
        let node = ast::skip_parentheses(node);
        if ast::is_binary_expression(node) && ast::is_logical_expression(node) {
            let e = node.as_binary_expression();
            // ignore the left operand for nullish coalescing expressions not in a context of a
            // test expression
            if e.operator_token.kind() != Kind::QuestionQuestionToken || is_test_expr {
                self.check_conditional(ctx, Some(e.left), is_test_expr);
            }
            // we ignore the right operand when not in a context of a test expression
            if is_test_expr {
                self.check_conditional(ctx, Some(e.right.get()), is_test_expr);
            }
            return;
        }
        if is_always_thenable(ctx, node) {
            report_node(ctx, node, conditional(), None);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &self.o.listeners
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::BinaryExpression => {
                if self.o.checks_conditionals {
                    self.check_conditional(ctx, Some(node), false);
                }
                if self.o.variables && ast::is_assignment_expression(node, false) {
                    check_assignment(ctx, node);
                }
            }
            Kind::PropertyAccessExpression | Kind::ElementAccessExpression => {
                check_array_predicates(ctx, node)
            }
            Kind::PrefixUnaryExpression => {
                let e = node.as_prefix_unary_expression();
                if e.operator == Kind::ExclamationToken {
                    self.check_conditional(ctx, Some(e.operand), true);
                }
            }
            Kind::ConditionalExpression => {
                let c = node.as_conditional_expression().condition;
                self.check_conditional(ctx, Some(c), true)
            }
            Kind::ForStatement => {
                let c = node.as_for_statement().condition();
                self.check_conditional(ctx, c, true)
            }
            Kind::DoStatement | Kind::WhileStatement | Kind::IfStatement => {
                self.check_conditional(ctx, node.expression(), true)
            }
            Kind::CallExpression | Kind::NewExpression => check_arguments(ctx, node),
            Kind::JsxAttribute => check_jsx_attribute(ctx, node),
            Kind::ClassDeclaration | Kind::ClassExpression | Kind::InterfaceDeclaration => {
                check_class_like_or_interface_node(ctx, node)
            }
            Kind::PropertyAssignment
            | Kind::MethodDeclaration
            | Kind::ShorthandPropertyAssignment => check_property(ctx, node),
            Kind::ReturnStatement => check_return_statement(ctx, node),
            Kind::VariableDeclaration => check_variable_declaration(ctx, node),
            Kind::SpreadElement | Kind::SpreadAssignment => check_spread(ctx, node),
            _ => {}
        }
    }
}
