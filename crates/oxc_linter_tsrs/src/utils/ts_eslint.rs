//! Ports of internal/utils/ts_eslint.go helpers, added as rules need them.

use super::*;
use rustc_hash::{FxHashMap, FxHashSet};

/// ts_eslint.go GetFunctionHeadLoc.
pub fn get_function_head_loc(source_file: P<SourceFile>, node: P<Node>) -> (i32, i32) {
    if ast::is_arrow_function(node) {
        if let Some(arrow) = node.as_arrow_function().equals_greater_than_token {
            return (arrow.pos(), arrow.end());
        }
    }
    let Some(params) = node.parameter_list() else {
        return trim_node_text_range(source_file, node);
    };
    // The parameter list's range starts right after `(`, so `pos() - 1` is the `(`.
    let end = params.pos() - 1;
    // Skip leading decorators; keep modifiers such as `export`, `public`, `async`.
    let mut start = tsrs_scanner::get_token_pos_of_node(node, source_file, false);
    for &m in node.modifier_nodes() {
        if m.kind() != Kind::Decorator {
            start = tsrs_scanner::get_token_pos_of_node(m, source_file, false);
            break;
        }
    }
    if end < start {
        return trim_node_text_range(source_file, node);
    }
    (start, end)
}

const ARRAY_PREDICATE_FUNCTIONS: &[&str] =
    &["every", "filter", "find", "findIndex", "findLast", "findLastIndex", "some"];

/// ts_eslint.go IsArrayMethodCallWithPredicate (`node` is the CallExpression).
pub fn is_array_method_call_with_predicate(c: &mut Checker, node: P<Node>) -> bool {
    let callee = node.expression().unwrap();
    if !ast::is_access_expression(callee) {
        return false;
    }
    let (property_name, ok) = c.get_accessed_property_name(callee);
    if !ok || !ARRAY_PREDICATE_FUNCTIONS.contains(&property_name.as_str()) {
        return false;
    }
    let t = get_constrained_type_at_location(c, callee.expression().unwrap());
    type_recurser(t, &mut |t| c.is_array_or_tuple_type(t))
}

/// ts_eslint.go IsRestParameterDeclaration.
pub fn is_rest_parameter_declaration(decl: P<Node>) -> bool {
    ast::is_parameter_declaration(decl)
        && decl.as_parameter_declaration().dot_dot_dot_token().is_some()
}

/// ts_eslint.go GetForStatementHeadLoc.
pub fn get_for_statement_head_loc(source_file: P<SourceFile>, node: P<Node>) -> (i32, i32) {
    let statement = if ast::is_for_statement(node) {
        node.as_for_statement().iteration_statement_base.statement
    } else {
        node.as_for_in_or_of_statement().statement
    };
    let (pos, _) = trim_node_text_range(source_file, node);
    (pos, statement.pos())
}

/// ts_eslint.go GetDeclaration.
pub fn get_declaration(c: &mut Checker, node: P<Node>) -> Option<P<Node>> {
    let symbol = c.get_symbol_at_location_exported(node)?;
    symbol.declarations().first().copied()
}

/// ts_eslint.go GetParentFunctionNode.
pub fn get_parent_function_node(node: P<Node>) -> Option<P<Node>> {
    let mut current = node.parent();
    while let Some(c) = current {
        if ast::is_function_like_declaration(c) {
            return Some(c);
        }
        current = c.parent();
    }
    None
}

/// ts_eslint.go GetTypeName.
pub fn get_type_name(c: &mut Checker, t: P<Type>) -> String {
    // It handles `string` and string literal types as string.
    if t.flags().intersects(TypeFlags::StringLike) {
        return "string".to_string();
    }
    // If the type is a type parameter which extends primitive string types,
    // but it was not recognized as a string like. So check the constraint
    // type of the type parameter.
    if is_type_parameter(t) {
        if let Some(symbol) = t.symbol() {
            let decls = symbol.declarations();
            if !decls.is_empty() && ast::is_type_parameter_declaration(decls[0]) {
                if let Some(constraint) = decls[0].as_type_parameter_declaration().constraint() {
                    let ct = c.get_type_from_type_node(constraint);
                    return get_type_name(c, ct);
                }
            }
        }
    }
    // If the type is a union and all types in the union are string like,
    // return `string`.
    if is_union_type(t) && union_type_parts(t).into_iter().all(|t| get_type_name(c, t) == "string")
    {
        return "string".to_string();
    }
    // If the type is an intersection and a type in the intersection is string
    // like, return `string`.
    if is_intersection_type(t)
        && intersection_type_parts(t).into_iter().any(|t| get_type_name(c, t) == "string")
    {
        return "string".to_string();
    }
    super::type_to_string(c, t)
}

/// ts_eslint.go GetContextualType.
pub fn get_contextual_type(c: &mut Checker, node: P<Node>) -> Option<P<Type>> {
    use tsrs_checker::ContextFlags;
    let parent = node.parent()?;
    if ast::is_call_expression(parent) || ast::is_new_expression(parent) {
        if Some(node) == parent.expression() {
            // is the callee, so has no contextual type
            return None;
        }
    } else if ast::is_variable_declaration(parent)
        || ast::is_property_declaration(parent)
        || ast::is_parameter_declaration(parent)
    {
        return parent.type_node().map(|t| c.get_type_from_type_node(t));
    } else if parent.kind() == Kind::JsxExpression {
        return c.get_contextual_type(parent, ContextFlags::None);
    } else if ast::is_identifier(node)
        && (ast::is_property_assignment(parent) || ast::is_shorthand_property_assignment(parent))
    {
        return c.get_contextual_type(node, ContextFlags::None);
    } else if ast::is_binary_expression(parent)
        && parent.as_binary_expression().operator_token.kind() == Kind::EqualsToken
        && parent.as_binary_expression().right.get() == node
    {
        // is RHS of assignment
        return Some(c.get_type_at_location(parent.as_binary_expression().left));
    } else if parent.kind() != Kind::JsxExpression && !ast::is_template_span(parent) {
        // parent is not something we know we can get the contextual type of
        return None;
    }
    c.get_contextual_type(node, ContextFlags::None)
}

/// ts_eslint.go IsUnsafeAssignment: Some((receiver, sender)) if assigning t to receiver_t is unsafe
/// (any leaks into a non-any receiver, possibly through generic type arguments).
pub fn is_unsafe_assignment(
    t: P<Type>,
    receiver_t: P<Type>,
    c: &mut Checker,
    sender_node: Option<P<Node>>,
) -> Option<(P<Type>, P<Type>)> {
    is_unsafe_assignment_worker(t, receiver_t, c, sender_node, &mut FxHashMap::default())
}

fn is_unsafe_assignment_worker(
    t: P<Type>,
    receiver: P<Type>,
    c: &mut Checker,
    sender_node: Option<P<Node>>,
    visited: &mut FxHashMap<P<Type>, FxHashSet<P<Type>>>,
) -> Option<(P<Type>, P<Type>)> {
    if is_type_any_type(t) {
        // Allow assignment of any ==> unknown.
        if is_type_unknown_type(receiver) {
            return None;
        }
        if !is_type_any_type(receiver) {
            return Some((receiver, t));
        }
    }

    if let Some(seen) = visited.get_mut(&t) {
        if seen.contains(&receiver) {
            return None;
        }
        seen.insert(receiver);
    } else {
        let mut s = FxHashSet::default();
        s.insert(receiver);
        visited.insert(t, s);
    }

    if tsrs_checker::is_non_deferred_type_reference(t)
        && tsrs_checker::is_non_deferred_type_reference(receiver)
    {
        // TODO - figure out how to handle cases like this, where the types are assignable, but not
        // the same type (see ts_eslint.go).
        if t.target() != receiver.target() {
            // if the type references are different, assume safe, as we won't know how to compare the
            // two types; the generic positions might not be equivalent for both types
            return None;
        }

        if let Some(sender) = sender_node {
            if ast::is_new_expression(sender) {
                let callee = sender.expression().unwrap();
                if ast::is_identifier(callee)
                    && callee.text() == "Map"
                    && sender.arguments().is_empty()
                    && sender.type_argument_list().is_none()
                {
                    // special case to handle new Map(): Map's default empty constructor is typed to
                    // return Map<any, any> (typescript-eslint#2109)
                    return None;
                }
            }
        }

        let type_arguments = c.get_type_arguments(t);
        let receiver_type_arguments = c.get_type_arguments(receiver);
        for (i, &arg) in type_arguments.iter().enumerate() {
            let receiver_arg = receiver_type_arguments[i];
            if is_unsafe_assignment_worker(arg, receiver_arg, c, sender_node, visited).is_some() {
                return Some((receiver, t));
            }
        }
        return None;
    }

    None
}

/// ts_eslint.go IsTypeAnyArrayType: true if the type is `any[]`.
pub fn is_type_any_array_type(t: P<Type>, c: &mut Checker) -> bool {
    c.is_array_type(t) && is_type_any_type(c.get_type_arguments(t)[0])
}

/// ts_eslint.go IsTypeUnknownArrayType: true if the type is `unknown[]`.
pub fn is_type_unknown_array_type(t: P<Type>, c: &mut Checker) -> bool {
    c.is_array_type(t) && is_type_unknown_type(c.get_type_arguments(t)[0])
}

/// ts_eslint.go GetThisExpression.
pub fn get_this_expression(mut node: P<Node>) -> Option<P<Node>> {
    loop {
        node = ast::skip_parentheses(node);
        if ast::is_call_expression(node) {
            node = node.expression().unwrap();
        } else if node.kind() == Kind::ThisKeyword {
            return Some(node);
        } else if ast::is_access_expression(node) {
            node = node.expression().unwrap();
        } else {
            break;
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiscriminatedAnyType {
    Any,
    PromiseAny,
    AnyArray,
    Safe,
}

/// ts_eslint.go DiscriminateAnyType: Any if the type is `any`, AnyArray if it is `any[]` or
/// `readonly any[]`, PromiseAny if it is `Promise<any>`, otherwise Safe.
pub fn discriminate_any_type(
    t: P<Type>,
    c: &mut Checker,
    program: &Program,
    node: P<Node>,
) -> DiscriminatedAnyType {
    let mut visited = rustc_hash::FxHashSet::default();
    discriminate_any_type_worker(t, c, program, node, &mut visited)
}

#[expect(
    clippy::only_used_in_recursion,
    reason = "Go signature: discriminateAnyTypeWorker passes program through the recursion"
)]
fn discriminate_any_type_worker(
    t: P<Type>,
    c: &mut Checker,
    program: &Program,
    node: P<Node>,
    // TODO(port): do we really need visited here?
    visited: &mut rustc_hash::FxHashSet<P<Type>>,
) -> DiscriminatedAnyType {
    if visited.contains(&t) {
        return DiscriminatedAnyType::Safe;
    }
    visited.insert(t);
    if is_type_any_type(t) {
        return DiscriminatedAnyType::Any;
    }
    if is_type_any_array_type(t, c) {
        return DiscriminatedAnyType::AnyArray;
    }
    let found_promise_any = type_recurser(t, &mut |t| {
        if !is_thenable_type(c, node, Some(t)) {
            return false;
        }
        let Some(awaited_type) = c.get_awaited_type(t) else {
            return false;
        };
        discriminate_any_type_worker(awaited_type, c, program, node, visited)
            == DiscriminatedAnyType::Any
    });
    if found_promise_any {
        return DiscriminatedAnyType::PromiseAny;
    }
    DiscriminatedAnyType::Safe
}

/// ts_eslint.go getBaseEnumType: if passed an enum member, returns the type of the parent;
/// otherwise returns itself.
fn get_base_enum_type(c: &mut Checker, t: P<Type>) -> P<Type> {
    let symbol = t.symbol();
    if !is_symbol_flag_set(symbol, SymbolFlags::EnumMember) {
        return t;
    }
    let parent = symbol.unwrap().value_declaration().unwrap().parent().unwrap();
    c.get_type_at_location(parent)
}

/// ts_eslint.go GetEnumLiterals: only the enum literals of a type.
pub fn get_enum_literals(t: P<Type>) -> Vec<P<Type>> {
    union_type_parts(t)
        .into_iter()
        .filter(|&sub_type| is_type_flag_set(sub_type, TypeFlags::EnumLiteral))
        .collect()
}

/// ts_eslint.go GetEnumTypes: the enum types of a type (0 or more).
pub fn get_enum_types(c: &mut Checker, t: P<Type>) -> Vec<P<Type>> {
    get_enum_literals(t).into_iter().map(|t| get_base_enum_type(c, t)).collect()
}

/// ts_eslint.go IsParenlessArrowFunction.
pub fn is_parenless_arrow_function(node: P<Node>) -> bool {
    if !ast::is_arrow_function(node) {
        return false;
    }
    let n = node.as_arrow_function();
    node.parameter_list().unwrap().end() == n.equals_greater_than_token.unwrap().pos()
}

/// ts_eslint.go MemberNameType.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemberNameType {
    Private,
    Quoted,
    Normal,
    Expression,
}

/// ts_eslint.go GetNameFromMember: a string name representation of a member name, with handling for
/// computed property names.
pub fn get_name_from_member(
    source_file: P<SourceFile>,
    member: P<Node>,
) -> (String, MemberNameType) {
    match member.kind() {
        Kind::Identifier => {
            return (member.as_identifier().text().to_string(), MemberNameType::Normal);
        }
        Kind::PrivateIdentifier => {
            return (member.as_private_identifier().text().to_string(), MemberNameType::Private);
        }
        Kind::ComputedPropertyName => {
            let expr = member.as_computed_property_name().expression;
            if ast::is_literal_expression(expr) {
                let text = expr.text();
                if !tsrs_scanner::is_valid_identifier(text) {
                    return (format!("\"{text}\""), MemberNameType::Quoted);
                }
                return (text.to_string(), MemberNameType::Normal);
            }
        }
        _ => {}
    }
    let (pos, end) = trim_node_text_range(source_file, member);
    (source_file.text()[pos as usize..end as usize].to_string(), MemberNameType::Expression)
}
