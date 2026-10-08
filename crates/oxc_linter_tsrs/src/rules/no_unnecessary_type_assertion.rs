// Port of internal/rules/no_unnecessary_type_assertion (no_unnecessary_type_assertion.go,
// assertion_fixes.go, narrowing_assignment.go, options.go).
//
// typescript-go represents parentheses as AST nodes. Expression and parent lookups skip these nodes to
// match ESTree semantics.

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{
    self as ast, CheckFlags, Kind, ModifierFlags, Node, NodeFlags, Symbol, SymbolFlags,
};
use tsrs_checker::{
    AliasArg, CheckMode, Checker, ContextFlags, ObjectFlags, Signature, Type, TypeFacts, TypeFlags,
    UnionReduction,
};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

const CONTEXTUALLY_UNNECESSARY: &str =
    "This assertion is unnecessary since the receiver accepts the original type of the expression.";
const UNNECESSARY_ASSERTION: &str =
    "This assertion is unnecessary since it does not change the type of the expression.";

fn build_contextually_unnecessary_message(pos: i32, end: i32) -> RuleDiagnostic {
    RuleDiagnostic {
        pos,
        end,
        message: RuleMessage::new("contextuallyUnnecessary", CONTEXTUALLY_UNNECESSARY),
        labeled_ranges: Vec::new(),
    }
}
fn build_unnecessary_assertion_diagnostic(pos: i32, end: i32) -> RuleDiagnostic {
    RuleDiagnostic {
        pos,
        end,
        message: RuleMessage::new("unnecessaryAssertion", UNNECESSARY_ASSERTION),
        labeled_ranges: Vec::new(),
    }
}

pub struct NoUnnecessaryTypeAssertion {
    check_literal_const_assertions: bool,
    types_to_ignore: Vec<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let types_to_ignore = match m.get("typesToIgnore") {
        Some(serde_json::Value::Array(a)) => {
            a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        }
        _ => Vec::new(),
    };
    Ok(Box::new(NoUnnecessaryTypeAssertion {
        check_literal_const_assertions: opt_bool(&m, "checkLiteralConstAssertions", false),
        types_to_ignore,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::AsExpression),
    Listener::Enter(Kind::TypeAssertionExpression),
    Listener::Enter(Kind::NonNullExpression),
];

impl Rule for NoUnnecessaryTypeAssertion {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-assertion"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let options = ctx.program.options();
        Box::new(Visitor {
            rule: self,
            env: Env {
                is_strict_null_checks: utils::is_strict_compiler_option_enabled(
                    &options,
                    options.strict_null_checks,
                ),
                no_unchecked_indexed_access: options.no_unchecked_indexed_access.is_true(),
                exact_optional_property_types: options.exact_optional_property_types.is_true(),
            },
        })
    }
}

#[derive(Clone, Copy)]
struct Env {
    is_strict_null_checks: bool,
    no_unchecked_indexed_access: bool,
    exact_optional_property_types: bool,
}

struct Visitor {
    rule: &'static NoUnnecessaryTypeAssertion,
    env: Env,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::NonNullExpression) => {
                check_non_null_expression(ctx, self.env, node)
            }
            _ => check_type_assertion(ctx, self.rule, self.env, node),
        }
    }
}

fn parent_through_parens(node: P<Node>) -> Option<P<Node>> {
    let mut parent = node.parent();
    while let Some(p) = parent {
        if !ast::is_parenthesized_expression(p) {
            break;
        }
        parent = p.parent();
    }
    parent
}

fn skip_parens_opt(node: Option<P<Node>>) -> Option<P<Node>> {
    node.map(ast::skip_parentheses)
}

/// Returns true if there's a chance the variable has been used before a value has been assigned to it
fn is_possibly_used_before_assigned(c: &mut Checker, env: Env, node: P<Node>) -> bool {
    let Some(declaration) = utils::get_declaration(c, node) else {
        // don't know what the declaration is for some reason, so just assume the worst
        return true;
    };
    // non-strict mode doesn't care about used before assigned errors
    if !env.is_strict_null_checks {
        return false;
    }
    // ignore class properties as they are compile time guarded
    // also ignore function arguments as they can't be used before defined
    if !ast::is_variable_declaration(declaration) {
        return false;
    }
    let decl = declaration.as_variable_declaration();
    let decl_parent = declaration.parent();

    // For var declarations, we need to check whether the node is actually in a descendant of its
    // declaration or not. If not, it may be used before defined.
    if let Some(list) = decl_parent {
        if ast::is_variable_declaration_list(list) && list.flags().is_empty() {
            let declarator_scope = ast::get_enclosing_block_scope_container(declaration);
            let scope = ast::get_enclosing_block_scope_container(node);
            let mut parent_scope = declarator_scope;
            while let Some(p) = parent_scope {
                parent_scope = ast::get_enclosing_block_scope_container(p);
                if parent_scope.is_none() {
                    break;
                }
                if parent_scope == scope {
                    return true;
                }
            }
        }
    }

    if declaration.initializer().is_none() && decl.exclamation_token().is_none() {
        if let Some(type_node) = declaration.type_node() {
            // check if the defined variable type has changed since assignment
            let declaration_type = c.get_type_from_type_node(type_node);
            let t = utils::get_constrained_type_at_location(c, node);
            let is_declare = decl_parent.is_some_and(|list| {
                ast::is_variable_declaration_list(list)
                    && list.parent().is_some_and(|stmt| {
                        ast::is_variable_statement(stmt)
                            && utils::includes_modifier(stmt, Kind::DeclareKeyword)
                    })
            });
            // 'declare's are never narrowed, so never skip them
            if declaration_type == t && !is_declare {
                // possibly used before assigned, so just skip it
                return true;
            }
        }
    }
    false
}

fn is_const_assertion(node: P<Node>) -> bool {
    if !ast::is_type_reference_node(node) {
        return false;
    }
    let type_name = node.as_type_reference_node().type_name;
    ast::is_identifier(type_name) && type_name.text() == "const"
}

fn is_implicitly_narrowed_literal_declaration(node: P<Node>) -> bool {
    let expression = ast::skip_parentheses(node.expression().unwrap());
    let parent = parent_through_parens(node);
    // Even on const variable declarations, template literals with expressions can sometimes be
    // widened without a type assertion (typescript-eslint#8737).
    if ast::is_template_expression(expression) {
        return false;
    }
    let Some(parent) = parent else { return false };
    (ast::is_variable_declaration(parent)
        && parent.parent().is_some_and(|p| {
            ast::is_variable_declaration_list(p) && p.flags().intersects(NodeFlags::Const)
        }))
        || (ast::is_property_declaration(parent)
            && parent.modifier_flags().intersects(ModifierFlags::Readonly))
}

fn get_type_arguments(c: &mut Checker, t: P<Type>) -> &'static [P<Type>] {
    if let Some(alias) = t.alias() {
        if !alias.type_arguments().is_empty() {
            return alias.type_arguments();
        }
    }
    if !t.object_flags().intersects(ObjectFlags::Reference) {
        return &[];
    }
    c.get_type_arguments(t)
}

fn type_contains(
    c: &mut Checker,
    t: Option<P<Type>>,
    predicate: &dyn Fn(P<Type>) -> bool,
    seen_types: &mut FxHashSet<P<Type>>,
    active_signatures: &mut FxHashSet<P<Signature>>,
) -> bool {
    let Some(t) = t else { return false };
    if !seen_types.insert(t) {
        return false;
    }
    if predicate(t) {
        return true;
    }
    if utils::is_union_type(t) || utils::is_intersection_type(t) {
        return t
            .types()
            .iter()
            .any(|&part| type_contains(c, Some(part), predicate, seen_types, active_signatures));
    }
    for &type_argument in get_type_arguments(c, t) {
        if type_contains(c, Some(type_argument), predicate, seen_types, active_signatures) {
            return true;
        }
    }
    for &sig in utils::get_call_signatures(c, t) {
        // Generic signature instantiations can produce fresh recursive return types. If their shared
        // original target is already active, conservatively assume the predicate could occur in the
        // unseen cycle instead of making an unsafe autofix.
        let mut signature_identity = sig;
        while let Some(target) = signature_identity.target.get() {
            signature_identity = target;
        }
        if !active_signatures.insert(signature_identity) {
            return true;
        }
        for &param in sig.parameters.get() {
            let pt = c.get_type_of_symbol(param);
            if type_contains(c, Some(pt), predicate, seen_types, active_signatures) {
                return true;
            }
        }
        let rt = c.get_return_type_of_signature(sig);
        if type_contains(c, Some(rt), predicate, seen_types, active_signatures) {
            return true;
        }
        active_signatures.remove(&signature_identity);
    }
    false
}

fn contains_any(c: &mut Checker, t: P<Type>) -> bool {
    type_contains(
        c,
        Some(t),
        &|part| utils::is_type_flag_set(part, TypeFlags::Any),
        &mut FxHashSet::default(),
        &mut FxHashSet::default(),
    )
}

fn contains_type_variable(c: &mut Checker, t: P<Type>) -> bool {
    type_contains(
        c,
        Some(t),
        &|part| utils::is_type_flag_set(part, TypeFlags::TypeVariable | TypeFlags::Index),
        &mut FxHashSet::default(),
        &mut FxHashSet::default(),
    )
}

fn has_index_signature(c: &mut Checker, t: P<Type>) -> bool {
    utils::union_type_parts(t).into_iter().any(|part| !c.get_index_infos_of_type(part).is_empty())
}

fn has_same_properties(c: &mut Checker, uncast: P<Type>, cast: P<Type>) -> bool {
    let uncast_props = c.get_properties_of_type(uncast);
    let cast_props = c.get_properties_of_type(cast);
    if uncast_props.len() != cast_props.len() {
        return false;
    }
    let mut cast_props_by_name: FxHashMap<&str, P<Symbol>> = FxHashMap::default();
    for &prop in cast_props {
        cast_props_by_name.insert(prop.name(), prop);
    }
    for &prop in uncast_props {
        let Some(&cast_prop) = cast_props_by_name.get(prop.name()) else {
            return false;
        };
        if c.is_readonly_symbol(prop) != c.is_readonly_symbol(cast_prop) {
            return false;
        }
    }
    true
}

fn have_same_type_arguments(c: &mut Checker, uncast: P<Type>, cast: P<Type>) -> bool {
    let uncast_args = get_type_arguments(c, uncast);
    let cast_args = get_type_arguments(c, cast);
    uncast_args == cast_args
}

fn are_mutually_assignable(c: &mut Checker, a: P<Type>, b: P<Type>) -> bool {
    c.is_type_assignable_to(a, b) && c.is_type_assignable_to(b, a)
}

fn are_union_parts_equivalent_ignoring_undefined(uncast: P<Type>, cast: P<Type>) -> bool {
    let mut uncast_parts: FxHashSet<P<Type>> = FxHashSet::default();
    for part in utils::union_type_parts(uncast) {
        if !utils::is_type_flag_set(part, TypeFlags::Undefined) {
            uncast_parts.insert(part);
        }
    }
    let mut cast_parts_count = 0;
    for part in utils::union_type_parts(cast) {
        if utils::is_type_flag_set(part, TypeFlags::Undefined) {
            continue;
        }
        if !uncast_parts.contains(&part) {
            return false;
        }
        cast_parts_count += 1;
    }
    uncast_parts.len() == cast_parts_count
}

fn is_empty_object_type(c: &mut Checker, t: P<Type>) -> bool {
    utils::is_type_flag_set(t, TypeFlags::NonPrimitive)
        || (c.get_properties_of_type(t).is_empty()
            && utils::get_call_signatures(c, t).is_empty()
            && utils::get_construct_signatures(c, t).is_empty()
            && c.get_index_infos_of_type(t).is_empty())
}

fn has_phantom_type_arguments(c: &mut Checker, t: P<Type>) -> bool {
    is_empty_object_type(c, t) && !get_type_arguments(c, t).is_empty()
}

fn is_conceptually_literal(node: P<Node>) -> bool {
    let node = ast::skip_parentheses(node);
    ast::is_array_literal_expression(node)
        || ast::is_object_literal_expression(node)
        || ast::is_class_expression(node)
        || ast::is_function_expression(node)
        || ast::is_arrow_function(node)
        || ast::is_jsx_element(node)
        || ast::is_jsx_self_closing_element(node)
        || ast::is_jsx_fragment(node)
        || ast::is_string_literal(node)
        || matches!(
            node.kind(),
            Kind::NumericLiteral
                | Kind::BigIntLiteral
                | Kind::RegularExpressionLiteral
                | Kind::NoSubstitutionTemplateLiteral
                | Kind::TrueKeyword
                | Kind::FalseKeyword
                | Kind::NullKeyword
        )
        || ast::is_template_expression(node)
}

fn is_type_literal(t: P<Type>) -> bool {
    utils::is_type_flag_set(
        t,
        TypeFlags::StringLiteral
            | TypeFlags::NumberLiteral
            | TypeFlags::BigIntLiteral
            | TypeFlags::BooleanLiteral,
    )
}

fn skip_paren_parents(mut current: Option<P<Node>>) -> Option<P<Node>> {
    while let Some(c) = current {
        if !ast::is_parenthesized_expression(c) {
            break;
        }
        current = c.parent();
    }
    current
}

fn is_receiver_of_write_access(node: P<Node>) -> bool {
    let mut current = skip_paren_parents(node.parent());
    while let Some(c) = current {
        if !ast::is_access_expression(c) {
            break;
        }
        if ast::is_write_access(c) {
            return true;
        }
        current = skip_paren_parents(c.parent());
    }
    false
}

fn is_element_access_argument(node: P<Node>) -> bool {
    let mut current = node;
    let mut parent = current.parent();
    while let Some(p) = parent {
        if !ast::is_parenthesized_expression(p) {
            break;
        }
        current = p;
        parent = p.parent();
    }
    parent.is_some_and(|p| {
        ast::is_element_access_expression(p)
            && p.as_element_access_expression().argument_expression == current
    })
}

fn has_enum_type(t: P<Type>) -> bool {
    if utils::is_type_flag_set(t, TypeFlags::EnumLike) {
        return true;
    }
    (utils::is_union_type(t) || utils::is_intersection_type(t))
        && t.types().iter().any(|&p| has_enum_type(p))
}

fn is_type_unchanged(
    c: &mut Checker,
    env: Env,
    node: P<Node>,
    expression: P<Node>,
    uncast: P<Type>,
    cast: P<Type>,
) -> bool {
    let expression = ast::skip_parentheses(expression);
    if uncast == cast {
        return true;
    }
    // Numeric enums and number are mutually assignable, but assertions between them still change the
    // type, including within unions and intersections.
    if (has_enum_type(uncast) || has_enum_type(cast)) && !c.is_type_identical_to(uncast, cast) {
        return false;
    }
    if utils::is_type_parameter(uncast) && is_receiver_of_write_access(node) {
        // Local safeguard: widening a generic write receiver can make the write legal.
        return false;
    }
    if env.no_unchecked_indexed_access && is_element_access_argument(node) {
        // Local safeguard: the asserted key type can change indexed-access nullability.
        return false;
    }

    let type_node = ast::skip_type_parentheses(node.type_node().unwrap());
    if ast::is_intersection_type_node(type_node) && contains_type_variable(c, cast) {
        return false;
    }

    if env.exact_optional_property_types
        && utils::is_type_flag_set(uncast, TypeFlags::Undefined)
        && utils::is_type_flag_set(cast, TypeFlags::Undefined)
    {
        return are_union_parts_equivalent_ignoring_undefined(uncast, cast);
    }

    if (utils::is_type_flag_set(uncast, TypeFlags::NonPrimitive)
        && !utils::is_type_flag_set(cast, TypeFlags::NonPrimitive))
        || (has_index_signature(c, uncast) != has_index_signature(c, cast))
        || contains_any(c, uncast)
        || contains_any(c, cast)
        || (contains_type_variable(c, cast) && !contains_type_variable(c, uncast))
    {
        return false;
    }

    if is_conceptually_literal(expression)
        && (!ast::is_object_literal_expression(expression)
            || expression.properties().is_empty()
            || c.get_properties_of_type(cast).iter().any(|&prop| {
                let t = c.get_type_of_symbol(prop);
                is_type_literal(t)
            }))
    {
        return false;
    }

    if utils::is_intersection_type(cast) && !utils::is_intersection_type(uncast) {
        let cast_parts = cast.types();
        let other_part = cast_parts.iter().copied().find(|&part| part != uncast);
        if utils::is_type_parameter(uncast) && cast_parts.len() == 2 && cast_parts.contains(&uncast)
        {
            if let Some(other_part) = other_part {
                if is_empty_object_type(c, other_part) && !contains_type_variable(c, other_part) {
                    if let Some(constraint) = c.get_base_constraint_of_type(uncast) {
                        if !utils::is_nullable_type(c, constraint) {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }

    if !has_same_properties(c, uncast, cast) || !have_same_type_arguments(c, uncast, cast) {
        return false;
    }

    are_mutually_assignable(c, uncast, cast)
}

fn is_type_any(t: P<Type>) -> bool {
    utils::is_type_flag_set(t, TypeFlags::Any)
}

fn is_type_unknown(t: P<Type>) -> bool {
    utils::is_type_flag_set(t, TypeFlags::Unknown)
}

fn is_nullable_for_non_null_assertion(c: &mut Checker, t: P<Type>) -> bool {
    if utils::is_nullable_type(c, t) {
        return true;
    }
    utils::union_type_parts(t).into_iter().any(|part| {
        utils::is_type_flag_set(part, TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Void)
    })
}

fn is_iife(expression: P<Node>) -> bool {
    let expression = ast::skip_parentheses(expression);
    if !ast::is_call_expression(expression) {
        return false;
    }
    let callee = ast::skip_parentheses(expression.expression().unwrap());
    ast::is_arrow_function(callee) || ast::is_function_expression(callee)
}

fn is_context_sensitive_call_like_expression(expression: P<Node>) -> bool {
    if ast::is_call_expression(expression)
        || ast::is_new_expression(expression)
        || ast::is_tagged_template_expression(expression)
    {
        return true;
    }
    if ast::is_await_expression(expression) {
        return is_context_sensitive_call_like_expression(ast::skip_parentheses(
            expression.expression().unwrap(),
        ));
    }
    false
}

fn get_uncast_type(c: &mut Checker, node: P<Node>) -> P<Type> {
    let expression = ast::skip_parentheses(node.expression().unwrap());

    if is_iife(expression) {
        let callee = ast::skip_parentheses(expression.expression().unwrap());
        let function_type = c.get_type_at_location(callee);
        let signatures = c.get_call_signatures(function_type);
        if !signatures.is_empty() {
            let return_type = c.get_return_type_of_signature(signatures[0]);
            if callee.type_node().is_none()
                && utils::is_type_flag_set(return_type, TypeFlags::Undefined)
            {
                return c.get_void_type();
            }
            return return_type;
        }
    }

    // For call-like expressions, use the context-free expression type so contextual typing from the
    // assertion itself doesn't leak into generic inference for the original expression.
    if is_context_sensitive_call_like_expression(expression) {
        return c.get_context_free_type_of_expression(expression);
    }

    c.get_type_at_location(expression)
}

fn get_original_expression(node: P<Node>) -> P<Node> {
    let mut current = ast::skip_parentheses(node.expression().unwrap());
    while ast::is_as_expression(current) || ast::is_type_assertion(current) {
        current = ast::skip_parentheses(current.expression().unwrap());
    }
    current
}

fn is_argument_to_parent_call_or_new(node: P<Node>) -> Option<usize> {
    let parent = parent_through_parens(node)?;
    if !ast::is_call_expression(parent) && !ast::is_new_expression(parent) {
        return None;
    }
    parent
        .arguments()
        .iter()
        .position(|&argument| argument == node || ast::skip_parentheses(argument) == node)
}

fn has_type_params(sig: P<Signature>) -> bool {
    !sig.type_parameters.get().is_empty()
}

fn has_generic_call_signature(c: &mut Checker, t: P<Type>) -> bool {
    utils::get_call_signatures(c, t).iter().any(|&s| has_type_params(s))
}

fn is_rest_parameter_symbol(param: P<Symbol>) -> bool {
    param.value_declaration().is_some_and(|vd| {
        vd.kind() == Kind::Parameter && vd.as_parameter_declaration().dot_dot_dot_token().is_some()
    })
}

fn has_generic_inference_parameter_at_argument(
    c: &mut Checker,
    call_or_new: P<Node>,
    arg_index: usize,
    element_path: &[usize],
) -> bool {
    let mut signature = c.get_resolved_signature(call_or_new, None, CheckMode::Normal);
    while let Some(target) = signature.target.get() {
        signature = target;
    }
    if signature.type_parameters.get().is_empty() {
        return false;
    }
    let params = signature.parameters.get();
    if params.is_empty() {
        return false;
    }
    let param_index = arg_index.min(params.len() - 1);
    let param = params[param_index];
    let mut param_type = c.get_type_of_symbol(param);
    if is_rest_parameter_symbol(param) {
        let type_arguments = get_type_arguments(c, param_type);
        if !type_arguments.is_empty() {
            let mut type_argument_index = 0;
            if type_arguments.len() > 1 {
                type_argument_index = (arg_index - param_index).min(type_arguments.len() - 1);
            }
            param_type = type_arguments[type_argument_index];
        }
    }
    for &element_index in element_path {
        let element_type = if tsrs_checker::is_tuple_type_exported(param_type) {
            let type_arguments = c.get_type_arguments(param_type);
            if type_arguments.is_empty() {
                None
            } else {
                Some(type_arguments[element_index.min(type_arguments.len() - 1)])
            }
        } else {
            utils::get_number_index_type(c, param_type)
        };
        match element_type {
            Some(e) => param_type = e,
            None => break,
        }
    }
    contains_type_variable(c, param_type)
}

fn generics_mismatch(c: &mut Checker, uncast: P<Type>, contextual: P<Type>) -> bool {
    for &prop in c.get_properties_of_type(contextual) {
        let prop_type = c.get_type_of_symbol(prop);
        let contextual_sigs =
            c.get_signatures_of_type(prop_type, tsrs_checker::SignatureKind::Call);
        if !contextual_sigs.iter().any(|&s| has_type_params(s)) {
            continue;
        }
        let Some(uncast_prop) = c.get_property_of_type(uncast, prop.name()) else {
            return true;
        };
        let uncast_prop_type = c.get_type_of_symbol(uncast_prop);
        let uncast_sigs =
            c.get_signatures_of_type(uncast_prop_type, tsrs_checker::SignatureKind::Call);
        if !uncast_sigs.iter().any(|&s| has_type_params(s)) {
            return true;
        }
    }
    false
}

fn is_argument_to_overloaded_function(c: &mut Checker, node: P<Node>) -> bool {
    let Some(arg_index) = is_argument_to_parent_call_or_new(node) else {
        return false;
    };
    let parent = parent_through_parens(node).unwrap();
    let t = c.get_type_at_location(parent.expression().unwrap());
    let callee_type = c.get_non_nullable_type(t);
    let signatures = c.get_call_signatures(callee_type);
    if signatures.len() <= 1 {
        return false;
    }
    let mut param_types: Vec<P<Type>> = Vec::with_capacity(signatures.len());
    for &sig in signatures {
        let params = sig.parameters.get();
        if arg_index >= params.len() {
            return true;
        }
        let mut param_type = c.get_type_of_symbol(params[arg_index]);
        if is_rest_parameter_symbol(params[arg_index]) {
            let type_arguments = get_type_arguments(c, param_type);
            if !type_arguments.is_empty() {
                param_type = type_arguments[0];
            }
        }
        param_types.push(param_type);
    }
    let first_param_type = param_types[0];
    if param_types.iter().any(|&p| p != first_param_type) {
        let uncast_type = c.get_type_at_location(node.expression().unwrap());
        return param_types.iter().any(|&p| !c.is_type_assignable_to(uncast_type, p));
    }
    false
}

fn is_in_destructuring_declaration(node: P<Node>) -> bool {
    let Some(parent) = parent_through_parens(node) else {
        return false;
    };
    ast::is_variable_declaration(parent)
        && skip_parens_opt(parent.initializer()) == Some(node)
        && parent.name().is_some_and(ast::is_binding_pattern)
}

fn is_property_in_problematic_context(c: &mut Checker, node: P<Node>) -> bool {
    let Some(parent) = parent_through_parens(node) else {
        return false;
    };
    if !ast::is_property_assignment(parent) || skip_parens_opt(parent.initializer()) != Some(node) {
        return false;
    }
    let Some(object_expr) = parent.parent() else {
        return false;
    };
    if !ast::is_object_literal_expression(object_expr) {
        return false;
    }
    if let Some(object_contextual_type) = c.get_contextual_type(object_expr, ContextFlags::None) {
        if utils::is_union_type(object_contextual_type) {
            let Some(prop_contextual_type) = c.get_contextual_type(node, ContextFlags::None) else {
                return true;
            };
            let non_nullable_contextual_type = c.get_non_nullable_type(prop_contextual_type);
            if utils::is_union_type(non_nullable_contextual_type) {
                return true;
            }
            let uncast_type = c.get_type_at_location(node.expression().unwrap());
            return !c.is_type_assignable_to(uncast_type, non_nullable_contextual_type);
        }
    }
    let object_parent = parent_through_parens(object_expr);
    // Also preserve casts whose property context comes from another assertion. typescript-go uses that
    // context during inference; removing the inner cast can change the inferred property type.
    object_parent.is_some_and(|op| {
        ast::is_as_expression(op)
            || ast::is_type_assertion(op)
            || ast::is_satisfies_expression(op)
            || (ast::is_call_expression(op)
                && op.parent().is_some_and(ast::is_satisfies_expression))
    })
}

fn is_assignment_in_non_statement_context(node: P<Node>) -> bool {
    let Some(parent) = parent_through_parens(node) else {
        return false;
    };
    ast::is_assignment_expression(parent, false)
        && ast::skip_parentheses(parent.as_binary_expression().right.get()) == node
        && parent_through_parens(parent).is_none_or(|p| p.kind() != Kind::ExpressionStatement)
}

fn is_right_hand_side_of_logical_assignment(node: P<Node>) -> bool {
    let Some(parent) = parent_through_parens(node) else {
        return false;
    };
    ast::is_binary_expression(parent)
        && ast::skip_parentheses(parent.as_binary_expression().right.get()) == node
        && ast::is_logical_or_coalescing_assignment_operator(
            parent.as_binary_expression().operator_token.kind(),
        )
}

fn is_nested_in_array_literal_argument_to_generic_call(c: &mut Checker, node: P<Node>) -> bool {
    // Local safeguard: contextual acceptance alone does not preserve inference for a generic
    // parameter inferred from an array element.
    let mut element_path: Vec<usize> = Vec::new();
    let mut next_child = node;
    let mut next = node.parent();
    while let Some(current) = next {
        let child = next_child;
        next_child = current;
        next = current.parent();

        if ast::is_function_expression(current) || ast::is_arrow_function(current) {
            return false;
        }
        if !ast::is_array_literal_expression(current) {
            continue;
        }
        let Some(element_index) = current.elements().iter().position(|&element| {
            element == child || ast::skip_parentheses(element) == ast::skip_parentheses(child)
        }) else {
            continue;
        };
        element_path.insert(0, element_index);

        let mut call_argument = current;
        let mut parent = parent_through_parens(call_argument);
        let mut spread_offset = 0;
        if let Some(p) = parent {
            if ast::is_spread_element(p) {
                spread_offset = element_index;
                call_argument = p;
                parent = parent_through_parens(call_argument);
            }
        }
        let Some(parent) = parent else { continue };
        if !ast::is_call_expression(parent) && !ast::is_new_expression(parent) {
            continue;
        }
        if parent.type_argument_list().is_some() {
            return false;
        }

        let Some(arg_index) = parent.arguments().iter().position(|&candidate| {
            candidate == call_argument || ast::skip_parentheses(candidate) == call_argument
        }) else {
            continue;
        };

        let mut parameter_element_path = &element_path[..];
        if ast::is_spread_element(call_argument) {
            parameter_element_path = &parameter_element_path[1..];
        }
        return has_generic_inference_parameter_at_argument(
            c,
            parent,
            arg_index + spread_offset,
            parameter_element_path,
        );
    }
    false
}

fn is_in_generic_context(c: &mut Checker, node: P<Node>) -> bool {
    let mut seen_function = false;
    let mut next = node.parent();
    while let Some(current) = next {
        next = current.parent();
        if current.kind() == Kind::FunctionDeclaration {
            return false;
        }
        if ast::is_function_expression(current) || ast::is_arrow_function(current) {
            if current.body().is_some_and(|b| b.kind() == Kind::Block) {
                return false;
            }
            if seen_function {
                return false;
            }
            seen_function = true;
        }
        if ast::is_call_expression(current) || ast::is_new_expression(current) {
            if current.type_argument_list().is_some() {
                continue;
            }
            if ast::is_call_expression(current)
                && ast::is_access_expression(current.expression().unwrap())
                && current
                    .arguments()
                    .iter()
                    .any(|&argument| ast::skip_parentheses(argument) == node)
            {
                continue;
            }
            let callee_type = c.get_type_at_location(current.expression().unwrap());
            if has_generic_call_signature(c, callee_type) {
                return true;
            }
        }
    }
    false
}

fn is_property_in_inferred_callback_return(c: &mut Checker, node: P<Node>) -> bool {
    // Local safeguard for the same inference dependency in callback returns.
    let Some(parent) = parent_through_parens(node) else {
        return false;
    };
    if !ast::is_property_assignment(parent) || skip_parens_opt(parent.initializer()) != Some(node) {
        return false;
    }
    let Some(object_expr) = parent.parent() else {
        return false;
    };
    if !ast::is_object_literal_expression(object_expr) {
        return false;
    }
    let callback = parent_through_parens(object_expr);
    callback.is_some_and(|cb| {
        ast::is_arrow_function(cb) && skip_parens_opt(cb.body()) == Some(object_expr)
    }) && is_in_generic_context(c, node)
}

fn is_skip_parent_type(node: P<Node>) -> bool {
    parent_through_parens(node).is_some_and(|parent| {
        ast::is_as_expression(parent)
            || ast::is_type_assertion(parent)
            || parent.kind() == Kind::SpreadElement
            || parent.kind() == Kind::SpreadAssignment
            || ast::is_satisfies_expression(parent)
    })
}

fn should_skip_contextual_type_fallback(
    c: &mut Checker,
    env: Env,
    node: P<Node>,
    cast_is_any: bool,
    uncast_type: P<Type>,
    cast_type: P<Type>,
) -> bool {
    let parent = parent_through_parens(node);
    // An assignment can narrow the receiver for subsequent statements. Accepting the original type
    // does not make that narrowing unnecessary.
    if is_in_narrowing_assignment(c, env, node, uncast_type, cast_type) {
        return true;
    }
    if cast_is_any {
        return parent.is_some_and(ast::is_logical_expression)
            || is_in_generic_context(c, node)
            || is_property_in_problematic_context(c, node);
    }

    // Interpolated templates can widen to string even when the context accepts them
    // (typescript-eslint#12276).
    if ast::is_template_expression(ast::skip_parentheses(node.expression().unwrap())) {
        return true;
    }

    if is_skip_parent_type(node)
        || ast::is_array_literal_expression(ast::skip_parentheses(node.expression().unwrap()))
        || is_nested_in_array_literal_argument_to_generic_call(c, node)
        || is_in_destructuring_declaration(node)
        || is_property_in_problematic_context(c, node)
        || is_property_in_inferred_callback_return(c, node)
        || is_assignment_in_non_statement_context(node)
        || is_right_hand_side_of_logical_assignment(node)
        || is_argument_to_overloaded_function(c, node)
    {
        return true;
    }

    if is_in_generic_context(c, node) {
        let original_expr = get_original_expression(node);
        return !is_conceptually_literal(original_expr)
            && parent.is_none_or(|p| !ast::is_property_assignment(p));
    }

    false
}

fn has_phantom_type_argument_mismatch(
    c: &mut Checker,
    node: P<Node>,
    uncast_type: P<Type>,
    contextual_type: P<Type>,
) -> bool {
    is_in_generic_context(c, node)
        && (has_phantom_type_arguments(c, uncast_type)
            || has_phantom_type_arguments(c, contextual_type))
        && !have_same_type_arguments(c, uncast_type, contextual_type)
}

fn is_nullish_literal_to_union(node: P<Node>, cast_type: P<Type>) -> bool {
    let expression = ast::skip_parentheses(node.expression().unwrap());
    utils::is_union_type(cast_type)
        && (expression.kind() == Kind::NullKeyword
            || (ast::is_identifier(expression) && expression.text() == "undefined"))
}

fn is_constrained_to(
    c: &mut Checker,
    source: Option<P<Type>>,
    target: P<Type>,
    seen: &mut FxHashSet<P<Type>>,
) -> bool {
    if source == Some(target) {
        return true;
    }
    let Some(source) = source else { return false };
    if !seen.insert(source) {
        return false;
    }
    if utils::is_type_parameter(source) {
        let constraint = c.get_constraint_of_type_parameter(source);
        return is_constrained_to(c, constraint, target, seen);
    }
    if utils::is_intersection_type(source) {
        return source.types().iter().any(|&part| is_constrained_to(c, Some(part), target, seen));
    }
    false
}

fn is_double_assertion_unnecessary(
    c: &mut Checker,
    env: Env,
    node: P<Node>,
    contextual_type: Option<P<Type>>,
) -> Option<&'static str> {
    let inner_expression = ast::skip_parentheses(node.expression().unwrap());
    if !ast::is_as_expression(inner_expression) && !ast::is_type_assertion(inner_expression) {
        return None;
    }

    let original_expr = get_original_expression(node);
    let original_type = c.get_type_at_location(original_expr);
    let cast_type = c.get_type_at_location(node);
    let different_unrelated_type_parameters = original_type != cast_type
        && utils::is_type_parameter(original_type)
        && utils::is_type_parameter(cast_type)
        && !is_constrained_to(c, Some(original_type), cast_type, &mut FxHashSet::default());

    if is_type_unchanged(c, env, node, inner_expression, original_type, cast_type)
        && !is_type_any(cast_type)
    {
        return Some("unnecessaryAssertion");
    }
    if let Some(contextual_type) = contextual_type {
        if !different_unrelated_type_parameters {
            // Keep bridges between unrelated type parameters: typescript-go can accept their
            // contextual constraints without accepting the direct cast.
            let intermediate_type = c.get_type_at_location(inner_expression);
            if (is_type_any(intermediate_type) || is_type_unknown(intermediate_type))
                && c.is_type_assignable_to(original_type, contextual_type)
            {
                return Some("contextuallyUnnecessary");
            }
        }
    }
    None
}

fn report_double_assertion_if_unnecessary(
    ctx: &mut Ctx,
    env: Env,
    node: P<Node>,
    contextual_type: Option<P<Type>>,
) {
    let Some(message_id) = is_double_assertion_unnecessary(ctx.checker, env, node, contextual_type)
    else {
        return;
    };
    let description = if message_id == "unnecessaryAssertion" {
        UNNECESSARY_ASSERTION
    } else {
        CONTEXTUALLY_UNNECESSARY
    };
    let (pos, end) = assertion_range(ctx, node);
    ctx.report_diagnostic_with_fixes(
        RuleDiagnostic {
            pos,
            end,
            message: RuleMessage::new(message_id, description),
            labeled_ranges: Vec::new(),
        },
        |ctx| {
            let original_expr = get_original_expression(node);
            let (p, e) = ctx.trim(original_expr);
            let mut text = ctx.text()[p as usize..e as usize].to_string();
            if ast::is_object_literal_expression(original_expr)
                && node.parent().is_some_and(|parent| {
                    ast::is_arrow_function(parent) && parent.body() == Some(node)
                })
            {
                text = format!("({text})");
            }
            vec![ctx.fix_replace(node, text)]
        },
    );
}

fn check_type_assertion(ctx: &mut Ctx, opts: &NoUnnecessaryTypeAssertion, env: Env, node: P<Node>) {
    let type_node = ast::skip_type_parentheses(node.type_node().unwrap());
    let (tp, te) = ctx.trim(type_node);
    let type_text = &ctx.text()[tp as usize..te as usize];
    if opts.types_to_ignore.iter().any(|t| t == type_text) {
        return;
    }

    let c = &mut *ctx.checker;
    let cast_type = c.get_type_at_location(node);
    let cast_type_is_literal = is_type_literal(cast_type);
    let type_annotation_is_const_assertion = is_const_assertion(type_node);

    if !opts.check_literal_const_assertions
        && cast_type_is_literal
        && type_annotation_is_const_assertion
    {
        return;
    }

    let expression = node.expression().unwrap();
    let mut uncast_type = get_uncast_type(c, node);

    let expression_for_type = ast::skip_parentheses(expression);
    if uncast_type == cast_type && ast::is_identifier(expression_for_type) {
        // typescript-go may resolve a conditional type at the assertion site. Retain its declared form
        // when deciding whether the cast changed it.
        if let Some(symbol) = c.get_symbol_at_location_exported(expression_for_type) {
            let symbol_type = c.get_type_of_symbol(symbol);
            if symbol_type.flags().intersects(TypeFlags::Conditional) {
                uncast_type = symbol_type;
            }
        }
    }

    let type_is_unchanged = is_type_unchanged(c, env, node, expression, uncast_type, cast_type);

    let would_same_type_be_inferred = if cast_type_is_literal {
        is_implicitly_narrowed_literal_declaration(node)
    } else {
        !type_annotation_is_const_assertion
    };

    if type_is_unchanged && would_same_type_be_inferred {
        let (pos, end) = assertion_range(ctx, node);
        ctx.report_diagnostic_with_fixes(build_unnecessary_assertion_diagnostic(pos, end), |ctx| {
            create_assertion_fixer(ctx, node)
        });
        return;
    }

    let cast_is_any = is_type_any(cast_type) && !is_skip_parent_type(node);
    let mut contextual_type = None;
    if !should_skip_contextual_type_fallback(c, env, node, cast_is_any, uncast_type, cast_type) {
        contextual_type = c.get_contextual_type(node, ContextFlags::None);
    }

    if let Some(contextual_type) = contextual_type {
        let contextual_type_is_any = is_type_any(contextual_type);
        let is_call_argument = is_argument_to_parent_call_or_new(node).is_some();
        let any_involved_in_contextual_check = (!contextual_type_is_any
            && !contains_any(c, contextual_type))
            || (contextual_type_is_any && is_call_argument && !contains_any(c, cast_type));

        let is_contextually_unnecessary = !type_annotation_is_const_assertion
            && !contains_any(c, uncast_type)
            && any_involved_in_contextual_check
            && !has_phantom_type_argument_mismatch(c, node, uncast_type, contextual_type)
            && (cast_is_any || !generics_mismatch(c, uncast_type, contextual_type))
            && (contextual_type_is_any || c.is_type_assignable_to(uncast_type, contextual_type))
            && !is_nullish_literal_to_union(node, cast_type);

        if is_contextually_unnecessary {
            let (pos, end) = assertion_range(ctx, node);
            ctx.report_diagnostic_with_fixes(
                build_contextually_unnecessary_message(pos, end),
                |ctx| create_assertion_fixer(ctx, node),
            );
            return;
        }
    }

    report_double_assertion_if_unnecessary(ctx, env, node, contextual_type);
}

fn exclamation_token_range(ctx: &Ctx, node: P<Node>) -> (i32, i32) {
    let s = tsrs_scanner::get_scanner_for_source_file(ctx.file, node.expression().unwrap().end());
    let r = s.token_range();
    (r.pos(), r.end())
}

fn check_non_null_expression(ctx: &mut Ctx, env: Env, node: P<Node>) {
    let expression = ast::skip_parentheses(node.expression().unwrap());

    let parent = parent_through_parens(node);
    if let Some(parent) = parent {
        if ast::is_assignment_expression(parent, true) {
            if ast::skip_parentheses(parent.as_binary_expression().left) == node {
                let (ep, ee) = exclamation_token_range(ctx, node);
                let (pos, end) = assertion_range(ctx, node);
                ctx.report_diagnostic_with_fixes(
                    build_contextually_unnecessary_message(pos, end),
                    |ctx| vec![ctx.fix_remove_range(ep, ee)],
                );
            }
            // for all other = assignments we ignore non-null checks; non-null assertions can change
            // the type-flow of the code, so whilst they might be unnecessary for the assignment - they
            // are necessary for following code
            return;
        }
    }

    let c = &mut *ctx.checker;
    let constrained_type = utils::get_constrained_type_at_location(c, expression);
    let actual_type = c.get_type_at_location(expression);

    let constrained_type_is_nullable = is_nullable_for_non_null_assertion(c, constrained_type);
    let actual_type_is_nullable = is_nullable_for_non_null_assertion(c, actual_type);

    if !constrained_type_is_nullable && !actual_type_is_nullable {
        if ast::is_identifier(expression) && is_possibly_used_before_assigned(c, env, expression) {
            return;
        }
        let (ep, ee) = exclamation_token_range(ctx, node);
        let (pos, end) = assertion_range(ctx, node);
        ctx.report_diagnostic_with_fixes(build_unnecessary_assertion_diagnostic(pos, end), |ctx| {
            vec![ctx.fix_remove_range(ep, ee)]
        });
    } else {
        // we know it's a nullable type, so figure out if the variable is used in a place that accepts
        // nullable types
        if constrained_type != actual_type {
            return;
        }

        let mut t_flags = TypeFlags::empty();
        for part in utils::union_type_parts(constrained_type) {
            t_flags |= part.flags();
        }

        let Some(contextual_type) = utils::get_contextual_type(c, node) else {
            return;
        };
        let mut contextual_flags = TypeFlags::empty();
        for part in utils::union_type_parts(contextual_type) {
            contextual_flags |= part.flags();
        }

        if t_flags.intersects(TypeFlags::Unknown)
            && !contextual_flags.intersects(TypeFlags::Unknown)
        {
            return;
        }

        // in strict mode you can't assign null to undefined, so we have to make sure that the two
        // types share a nullable type
        let type_includes_undefined = t_flags.intersects(TypeFlags::Undefined);
        let type_includes_null = t_flags.intersects(TypeFlags::Null);
        let type_includes_void = t_flags.intersects(TypeFlags::Void);

        let contextual_type_includes_undefined = contextual_flags.intersects(TypeFlags::Undefined);
        let contextual_type_includes_null = contextual_flags.intersects(TypeFlags::Null);
        let contextual_type_includes_void = contextual_flags.intersects(TypeFlags::Void);

        // make sure that the parent accepts the same types, i.e. assigning
        // string | null | undefined to string | undefined is invalid
        let is_valid_undefined = !type_includes_undefined || contextual_type_includes_undefined;
        let is_valid_null = !type_includes_null || contextual_type_includes_null;
        let is_valid_void = !type_includes_void || contextual_type_includes_void;

        if is_valid_undefined && is_valid_null && is_valid_void {
            let (ep, ee) = exclamation_token_range(ctx, node);
            let (pos, end) = assertion_range(ctx, node);
            ctx.report_diagnostic_with_fixes(
                build_contextually_unnecessary_message(pos, end),
                |ctx| vec![ctx.fix_remove_range(ep, ee)],
            );
        }
    }
}

// ---- assertion_fixes.go ----

/// typescript-go represents JSDoc casts as assertions. Locate the cast's comment so it can be reported
/// and removed.
fn js_doc_assertion_range(ctx: &Ctx, node: P<Node>) -> Option<(i32, i32)> {
    let type_node = node.type_node()?;
    let expression = node.expression().unwrap();
    if !ast::is_in_js_file(node) || type_node.pos() >= expression.pos() {
        return None;
    }
    let mut search_start = node.pos();
    if let Some(parent) = node.parent() {
        if ast::is_parenthesized_expression(parent) {
            search_start = parent.pos();
        }
    }
    let before_expression = &ctx.text()[search_start as usize..expression.pos() as usize];
    let start = before_expression.rfind("/**")?;
    let end = before_expression.rfind("*/")?;
    if end < start {
        return None;
    }
    Some((search_start + start as i32, search_start + end as i32 + "*/".len() as i32))
}

fn assertion_range(ctx: &Ctx, node: P<Node>) -> (i32, i32) {
    if let Some(r) = js_doc_assertion_range(ctx, node) {
        return r;
    }
    // Report the whole assertion, excluding surrounding trivia.
    ctx.trim(node)
}

/// Remove the assertion while preserving valid expression syntax.
fn create_assertion_fixer(ctx: &mut Ctx, node: P<Node>) -> Vec<RuleFix> {
    let expression = node.expression().unwrap();
    if let Some((pos, mut end)) = js_doc_assertion_range(ctx, node) {
        let text = ctx.text().as_bytes();
        while end < expression.pos() && utils::is_str_white_space(text[end as usize] as char) {
            end += 1;
        }
        return vec![ctx.fix_remove_range(pos, end)];
    }
    let type_node = node.type_node().unwrap();
    if ast::is_type_assertion(node) {
        let mut s = tsrs_scanner::get_scanner_for_source_file(ctx.file, node.pos());
        let opening_angle_bracket = s.token_range();
        s.reset_pos(type_node.end());
        s.scan();
        let closing_angle_bracket = s.token_range();
        s.scan();
        let first_operand_token = s.token();
        let mut statement_start_token = first_operand_token;
        // At the start of a statement, async function needs parentheses to remain an expression after
        // the assertion is removed.
        if first_operand_token == Kind::AsyncKeyword
            && s.scan() == Kind::FunctionKeyword
            && !s.has_preceding_line_break()
        {
            statement_start_token = Kind::FunctionKeyword;
        }
        let needs_parens =
            is_start_of_expression_statement_needing_parentheses(ctx, node, statement_start_token)
                || is_start_of_arrow_function_body_needing_parentheses(
                    ctx,
                    node,
                    first_operand_token,
                );

        let mut fixes = Vec::new();
        if needs_parens {
            fixes.push(ctx.fix_insert_before(node, "("));
        }
        fixes.push(ctx.fix_remove_range(opening_angle_bracket.pos(), closing_angle_bracket.end()));
        if needs_parens {
            fixes.push(ctx.fix_insert_after(node, ")"));
        }
        return fixes;
    }

    // Preserve the token or comment before 'as' when removing the assertion.
    let s = tsrs_scanner::get_scanner_for_source_file(ctx.file, expression.end());
    let as_token = s.token_range();
    let mut token_before_as_end = expression.end();
    for comment in utils::get_comments_in_range(
        ctx.file,
        tsrs_core::TextRange::new(expression.end(), as_token.pos()),
    ) {
        token_before_as_end = token_before_as_end.max(comment.end());
    }
    vec![ctx.fix_remove_range(token_before_as_end, node.end())]
}

/// Matching start positions limits traversal to the beginning of a statement and stops at existing
/// parentheses.
fn is_start_of_expression_statement_needing_parentheses(
    ctx: &Ctx,
    node: P<Node>,
    first_token: Kind,
) -> bool {
    if first_token != Kind::OpenBraceToken
        && first_token != Kind::ClassKeyword
        && first_token != Kind::FunctionKeyword
    {
        return false;
    }
    let start = ctx.trim(node).0;
    let mut ancestor = node.parent();
    while let Some(a) = ancestor {
        if ctx.trim(a).0 != start {
            break;
        }
        if ast::is_expression_statement(a) {
            return true;
        }
        ancestor = a.parent();
    }
    false
}

fn is_start_of_arrow_function_body_needing_parentheses(
    ctx: &Ctx,
    node: P<Node>,
    first_token: Kind,
) -> bool {
    if first_token != Kind::OpenBraceToken {
        return false;
    }
    let mut current = node;
    while let Some(parent) = current.parent() {
        if ast::is_parenthesized_expression(parent) {
            return false;
        }
        if ast::is_arrow_function(parent) && parent.body() == Some(current) {
            return true;
        }
        if ctx.trim(parent).0 != ctx.trim(current).0 {
            return false;
        }
        current = parent;
    }
    false
}

// ---- narrowing_assignment.go ----
//
// These checks supplement typescript-eslint's contextual fallback for issue #1122. A receiver
// accepting the original type does not imply that removing the assertion preserves the receiver's
// type in later statements. Keep this analysis separate from the upstream rule and use
// typescript-go's assignment reduction semantics.

fn atoi(s: &str) -> Option<i64> {
    s.parse::<i64>().ok()
}

fn get_destructuring_default(mut target: P<Node>, property_path: &[String]) -> Option<P<Node>> {
    for i in (0..property_path.len()).rev() {
        target = ast::skip_parentheses(target);
        let mut element: Option<P<Node>> = None;
        if ast::is_array_literal_expression(target) {
            let elements = target.elements();
            match atoi(&property_path[i]) {
                Some(index) if index >= 0 && (index as usize) < elements.len() => {
                    element = Some(elements[index as usize]);
                }
                _ => return None,
            }
        } else if ast::is_object_literal_expression(target) {
            for &property in target.properties() {
                if ast::is_spread_assignment(property) {
                    continue;
                }
                let Some(name_node) = property.name() else {
                    continue;
                };
                let name = ast::try_get_text_of_property_name(name_node)?;
                if name == property_path[i] {
                    // Every target reading this property must preserve the narrowing. A default on
                    // only one of several targets is not sufficient.
                    if element.is_some() {
                        return None;
                    }
                    element = Some(property);
                }
            }
        }
        let mut element = element?;
        if i == 0 {
            if ast::is_shorthand_property_assignment(element) {
                return element
                    .as_shorthand_property_assignment()
                    .object_assignment_initializer
                    .get();
            }
            if ast::is_property_assignment(element) {
                element = element.initializer()?;
            }
            if ast::is_assignment_expression(element, true) {
                return Some(element.as_binary_expression().right.get());
            }
            return None;
        }
        target = ast::get_target_of_binding_or_assignment_element(element)?;
    }
    None
}

fn get_logical_result_type(
    c: &mut Checker,
    env: Env,
    operator: Kind,
    left: P<Type>,
    right: P<Type>,
) -> P<Type> {
    match operator {
        Kind::AmpersandAmpersandToken if c.has_type_facts(left, TypeFacts::Truthy) => {
            let mut falsy_source = left;
            if !env.is_strict_null_checks {
                falsy_source = c.get_base_type_of_literal_type(right);
            }
            let falsy = c.extract_definitely_falsy_types(falsy_source);
            c.get_union_type_ex(&[falsy, right], UnionReduction::Literal, AliasArg::None, None)
        }
        Kind::BarBarToken if c.has_type_facts(left, TypeFacts::Falsy) => {
            let removed = c.remove_definitely_falsy_types(left);
            let truthy = c.get_non_nullable_type(removed);
            c.get_union_type_ex(&[truthy, right], UnionReduction::Subtype, AliasArg::None, None)
        }
        Kind::QuestionQuestionToken if c.has_type_facts(left, TypeFacts::EQUndefinedOrNull) => {
            let non_nullable = c.get_non_nullable_type(left);
            c.get_union_type_ex(
                &[non_nullable, right],
                UnionReduction::Subtype,
                AliasArg::None,
                None,
            )
        }
        _ => left,
    }
}

fn is_in_narrowing_assignment(
    c: &mut Checker,
    env: Env,
    node: P<Node>,
    mut uncast_type: P<Type>,
    mut cast_type: P<Type>,
) -> bool {
    let mut in_literal = false;
    let mut property_path: Vec<String> = Vec::new();
    let mut path_known = true;
    let mut current = node;
    while let Some(parent) = current.parent() {
        if !in_literal
            && ast::is_logical_or_coalescing_binary_expression(parent)
            && parent.as_binary_expression().right.get() == current
        {
            let binary = parent.as_binary_expression();
            // The left operand can also supply the value assigned to the receiver.
            let left_type = c.get_type_at_location(binary.left);
            let op = binary.operator_token.kind();
            uncast_type = get_logical_result_type(c, env, op, left_type, uncast_type);
            cast_type = get_logical_result_type(c, env, op, left_type, cast_type);
        }
        if path_known
            && ast::is_conditional_expression(parent)
            && parent.as_conditional_expression().condition != current
        {
            let conditional = parent.as_conditional_expression();
            let mut other = conditional.when_true;
            if other == current {
                other = conditional.when_false;
            }
            // Either branch can supply the assigned value. Preserve the other branch's types when
            // comparing narrowing with and without the cast.
            let mut other_type = Some(c.get_type_at_location(other));
            for name in property_path.iter().rev() {
                let Some(other_ty) = other_type else { break };
                let mut property_type = c.get_type_of_property_of_type(other_ty, name);
                if property_type.is_none() && atoi(name).is_some() {
                    property_type = utils::get_number_index_type(c, other_ty);
                }
                other_type = property_type;
            }
            if let Some(other_type) = other_type {
                uncast_type = c.get_union_type_ex(
                    &[uncast_type, other_type],
                    UnionReduction::Subtype,
                    AliasArg::None,
                    None,
                );
                cast_type = c.get_union_type_ex(
                    &[cast_type, other_type],
                    UnionReduction::Subtype,
                    AliasArg::None,
                    None,
                );
            }
        }
        if !in_literal
            && ast::is_binary_expression(parent)
            && parent.as_binary_expression().left == current
            && matches!(
                parent.as_binary_expression().operator_token.kind(),
                Kind::BarBarToken | Kind::QuestionQuestionToken
            )
        {
            // These operators already discard nullish values from their left operand, so a purely
            // non-nullable assertion cannot change narrowing.
            let uncast_non_nullable = c.get_non_nullable_type(uncast_type);
            let cast_non_nullable = c.get_non_nullable_type(cast_type);
            if c.is_type_identical_to(uncast_non_nullable, cast_non_nullable) {
                return false;
            }
        }
        if ast::is_parenthesized_expression(parent)
            || ast::is_logical_or_coalescing_binary_expression(parent)
            || (ast::is_conditional_expression(parent)
                && parent.as_conditional_expression().condition != current)
            || (ast::is_binary_expression(parent)
                && parent.as_binary_expression().operator_token.kind() == Kind::CommaToken
                && parent.as_binary_expression().right.get() == current)
        {
            current = parent;
            continue;
        }
        if ast::is_array_literal_expression(parent)
            || ast::is_object_literal_expression(parent)
            || ast::is_spread_assignment(parent)
            || (ast::is_property_assignment(parent) && parent.initializer() == Some(current))
        {
            in_literal = true;
            if ast::is_property_assignment(parent) {
                let name = ast::try_get_text_of_property_name(parent.name().unwrap());
                path_known = path_known && name.is_some();
                property_path.push(name.unwrap_or_default());
            } else if ast::is_array_literal_expression(parent) {
                let elements = parent.elements();
                let index = elements.iter().position(|&e| e == current);
                // A spread can shift the runtime index away from the syntax index.
                path_known = path_known
                    && index
                        .is_some_and(|i| !elements[..i].iter().any(|&e| ast::is_spread_element(e)));
                property_path.push(match index {
                    Some(i) => i.to_string(),
                    None => "-1".to_string(),
                });
            } else if ast::is_object_literal_expression(parent)
                && path_known
                && !property_path.is_empty()
            {
                // A later required property replaces this value, so its assertion cannot narrow the
                // target. Match the checker's spread-property rules.
                let properties = parent.properties();
                let start = properties.iter().position(|&p| p == current).map_or(0, |i| i + 1);
                let last = property_path.last().unwrap().clone();
                for &property in &properties[start..] {
                    if ast::is_spread_assignment(property) {
                        let spread_type = c.get_type_at_location(property.expression().unwrap());
                        if let Some(overwriting) = c.get_property_of_type(spread_type, &last) {
                            if !overwriting.flags().intersects(SymbolFlags::Optional)
                                && !overwriting.check_flags().intersects(CheckFlags::Partial)
                                && !tsrs_checker::get_declaration_modifier_flags_from_symbol_exported(
                                    overwriting,
                                )
                                .intersects(ModifierFlags::Private | ModifierFlags::Protected)
                                && c.is_spreadable_property(overwriting)
                            {
                                return false;
                            }
                        }
                        continue;
                    }
                    let Some(name_node) = property.name() else {
                        continue;
                    };
                    if ast::try_get_text_of_property_name(name_node).as_deref() == Some(&last) {
                        return false;
                    }
                }
            }
            current = parent;
            continue;
        }
        if !ast::is_assignment_expression(parent, true)
            || parent.as_binary_expression().right.get() != current
        {
            return false;
        }

        let left = ast::skip_parentheses(parent.as_binary_expression().left);
        let receiver_type = if in_literal {
            if !ast::is_assignment_pattern(left) {
                return false;
            }
            // Destructuring context supplies the type of the corresponding target.
            let receiver_type = c.get_contextual_type(node, ContextFlags::None);
            if path_known {
                if let Some(initializer) = get_destructuring_default(left, &property_path) {
                    uncast_type = c.get_type_with_default(uncast_type, Some(initializer));
                    cast_type = c.get_type_with_default(cast_type, Some(initializer));
                }
            }
            receiver_type
        } else {
            Some(c.get_type_at_location(left))
        };
        let Some(mut receiver_type) = receiver_type else {
            return false;
        };
        if let Some(constraint) = c.get_base_constraint_of_type(receiver_type) {
            receiver_type = constraint;
        }
        if !utils::is_union_type(receiver_type) {
            return false;
        }

        // Use the checker's assignment reduction so assertions that leave the same union members
        // reachable can still be reported as unnecessary.
        let uncast_reduced = c.get_assignment_reduced_type(receiver_type, uncast_type);
        let cast_reduced = c.get_assignment_reduced_type(receiver_type, cast_type);
        return !c.is_type_identical_to(uncast_reduced, cast_reduced);
    }
    false
}
