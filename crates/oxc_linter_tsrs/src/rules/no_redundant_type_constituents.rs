// Port of internal/rules/no_redundant_type_constituents/no_redundant_type_constituents.go.

use std::fmt::Write;

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_error_type_overrides_message(type_name: &str, container: &str) -> RuleMessage {
    RuleMessage::new(
        "errorTypeOverrides",
        format!(
            "'{type_name}' is an 'error' type that acts as 'any' and overrides all other types in this {container} type."
        ),
    )
}
fn build_literal_overridden_message(literal: &str, primitive: &str) -> RuleMessage {
    RuleMessage::new(
        "literalOverridden",
        format!("{literal} is overridden by {primitive} in this union type."),
    )
}
fn build_overridden_message(type_name: &str, container: &str) -> RuleMessage {
    RuleMessage::new(
        "overridden",
        format!("'{type_name}' is overridden by other types in this {container} type."),
    )
}
fn build_overrides_message(type_name: &str, container: &str) -> RuleMessage {
    RuleMessage::new(
        "overrides",
        format!("'{type_name}' overrides all other types in this {container} type."),
    )
}
fn build_primitive_overridden_message(literal: &str, primitive: &str) -> RuleMessage {
    RuleMessage::new(
        "primitiveOverridden",
        format!("{primitive} is overridden by the {literal} in this intersection type."),
    )
}

fn is_node_inside_return_type(node: P<Node>) -> bool {
    node.parent().is_some_and(ast::is_function_like)
}

/// Go fmt's %q (strconv.Quote).
fn go_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c if c.is_control() => {
                if (c as u32) < 0x10000 {
                    let _ = write!(out, "\\u{:04x}", c as u32);
                } else {
                    let _ = write!(out, "\\U{:08x}", c as u32);
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[derive(Clone, Copy)]
struct TypeFlagsWithNodeOrType {
    flags: TypeFlags,
    // either node or t must be set
    node: Option<P<Node>>,
    t: Option<P<Type>>,
}

impl TypeFlagsWithNodeOrType {
    fn to_display(self, c: &mut Checker) -> String {
        if let Some(node) = self.node {
            match node.kind() {
                Kind::AnyKeyword => return "any".to_string(),
                Kind::BooleanKeyword => return "boolean".to_string(),
                Kind::NeverKeyword => return "never".to_string(),
                Kind::NumberKeyword => return "number".to_string(),
                Kind::StringKeyword => return "string".to_string(),
                Kind::UnknownKeyword => return "unknown".to_string(),
                Kind::LiteralType => {
                    let literal = node.as_literal_type_node().literal;
                    match literal.kind() {
                        Kind::TemplateLiteralType | Kind::NoSubstitutionTemplateLiteral => {
                            return "template literal type".to_string();
                        }
                        Kind::StringLiteral | Kind::NumericLiteral | Kind::BigIntLiteral => {
                            return literal.text().to_string();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            return "literal type".to_string();
        }
        let t = self.t.unwrap();
        if utils::is_type_flag_set(t, TypeFlags::StringLiteral) {
            return go_quote(&utils::type_to_string(c, t));
        }
        utils::type_to_string(c, t)
    }
}

fn join_parts(c: &mut Checker, parts: &[TypeFlagsWithNodeOrType]) -> String {
    parts.iter().map(|p| p.to_display(c)).collect::<Vec<_>>().join(" | ")
}

fn get_type_node_type_part_flags(ctx: &mut Ctx, node: P<Node>) -> Vec<TypeFlagsWithNodeOrType> {
    let node = ast::skip_parentheses(node);
    let mut flags = match node.kind() {
        Kind::AnyKeyword => TypeFlags::Any,
        Kind::BigIntKeyword => TypeFlags::BigInt,
        Kind::BooleanKeyword => TypeFlags::Boolean,
        Kind::NeverKeyword => TypeFlags::Never,
        Kind::NumberKeyword => TypeFlags::Number,
        Kind::StringKeyword => TypeFlags::String,
        Kind::UnknownKeyword => TypeFlags::Unknown,
        _ => TypeFlags::empty(),
    };
    if !flags.is_empty() {
        return vec![TypeFlagsWithNodeOrType { flags, node: Some(node), t: None }];
    }
    if ast::is_literal_type_node(node) {
        flags = match node.as_literal_type_node().literal.kind() {
            Kind::BigIntLiteral => TypeFlags::BigIntLiteral,
            Kind::TrueKeyword | Kind::FalseKeyword => TypeFlags::BooleanLiteral,
            Kind::NumericLiteral => TypeFlags::NumberLiteral,
            Kind::StringLiteral => TypeFlags::StringLiteral,
            _ => TypeFlags::empty(),
        };
        if !flags.is_empty() {
            return vec![TypeFlagsWithNodeOrType { flags, node: Some(node), t: None }];
        }
    }
    if node.kind() == Kind::UnionType {
        let mut result = Vec::new();
        for &sub in node.as_union_type_node().types().nodes() {
            result.extend(get_type_node_type_part_flags(ctx, sub));
        }
        return result;
    }
    let t = ctx.checker.get_type_at_location(node);
    let boolean_type = ctx.checker.get_boolean_type();
    let type_parts = if t == boolean_type { vec![t] } else { utils::union_type_parts(t) };
    type_parts
        .into_iter()
        .map(|part| TypeFlagsWithNodeOrType { flags: part.flags(), node: None, t: Some(part) })
        .collect()
}

fn check_intersection_bottom_and_top_types(
    ctx: &mut Ctx,
    type_part: &TypeFlagsWithNodeOrType,
    type_node: P<Node>,
) -> bool {
    let message = if type_part.flags == TypeFlags::Any {
        let type_name = type_part.to_display(ctx.checker);
        if type_name == "any" {
            build_overrides_message(&type_name, "intersection")
        } else {
            build_error_type_overrides_message(&type_name, "intersection")
        }
    } else if type_part.flags == TypeFlags::Never {
        build_overrides_message(&type_part.to_display(ctx.checker), "intersection")
    } else if type_part.flags == TypeFlags::Unknown {
        build_overridden_message(&type_part.to_display(ctx.checker), "intersection")
    } else {
        return false;
    };
    ctx.report_node(type_node, message);
    true
}

pub struct NoRedundantTypeConstituents;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoRedundantTypeConstituents))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::IntersectionType), Listener::Enter(Kind::UnionType)];

impl Rule for NoRedundantTypeConstituents {
    fn name(&self) -> &'static str {
        "no-redundant-type-constituents"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_string_literal_like_flags(flags: TypeFlags) -> bool {
    flags == TypeFlags::StringLiteral || flags == TypeFlags::TemplateLiteral
}

fn check_intersection(ctx: &mut Ctx, node: P<Node>) {
    let mut seen_big_int_literal_types = Vec::new();
    let mut seen_boolean_literal_types = Vec::new();
    let mut seen_number_literal_types = Vec::new();
    let mut seen_string_literal_types = Vec::new();

    let mut seen_big_int_primitive_types: Vec<P<Node>> = Vec::new();
    let mut seen_boolean_primitive_types: Vec<P<Node>> = Vec::new();
    let mut seen_number_primitive_types: Vec<P<Node>> = Vec::new();
    let mut seen_string_primitive_types: Vec<P<Node>> = Vec::new();

    let mut seen_union_types: Vec<(Vec<TypeFlagsWithNodeOrType>, P<Node>)> = Vec::new();

    for &type_node in node.as_intersection_type_node().types().nodes() {
        let type_part_flags = get_type_node_type_part_flags(ctx, type_node);

        // if any typeNode is TSTypeReference and typePartFlags have more than 1 element, then the
        // referenced type is definitely a union.
        if type_part_flags.len() >= 2 {
            seen_union_types.push((type_part_flags.clone(), type_node));
        }

        for type_part in &type_part_flags {
            if check_intersection_bottom_and_top_types(ctx, type_part, type_node) {
                continue;
            }

            // unions assignability check doesn't require seen*LiteralTypes, so avoid computing them
            if seen_union_types.is_empty() {
                let f = type_part.flags;
                if f == TypeFlags::BigIntLiteral {
                    seen_big_int_literal_types.push(*type_part);
                } else if f == TypeFlags::BooleanLiteral {
                    seen_boolean_literal_types.push(*type_part);
                } else if f == TypeFlags::NumberLiteral {
                    seen_number_literal_types.push(*type_part);
                } else if is_string_literal_like_flags(f) {
                    seen_string_literal_types.push(*type_part);
                }
            }

            let f = type_part.flags;
            if f == TypeFlags::BigInt {
                seen_big_int_primitive_types.push(type_node);
            } else if f == TypeFlags::Boolean {
                seen_boolean_primitive_types.push(type_node);
            } else if f == TypeFlags::Number {
                seen_number_primitive_types.push(type_node);
            } else if f == TypeFlags::String {
                seen_string_primitive_types.push(type_node);
            }
        }
    }

    if !seen_union_types.is_empty()
        && (!seen_big_int_primitive_types.is_empty()
            || !seen_boolean_primitive_types.is_empty()
            || !seen_number_primitive_types.is_empty()
            || !seen_string_primitive_types.is_empty())
    {
        let mut type_values_literal = String::new();
        for (flags, type_node) in &seen_union_types {
            let mut primitive_name = "";
            for type_value in flags {
                let f = type_value.flags;
                primitive_name = if f == TypeFlags::BigIntLiteral
                    && !seen_big_int_primitive_types.is_empty()
                {
                    "bigint"
                } else if f == TypeFlags::BooleanLiteral && !seen_boolean_primitive_types.is_empty()
                {
                    "boolean"
                } else if f == TypeFlags::NumberLiteral && !seen_number_primitive_types.is_empty() {
                    "number"
                } else if is_string_literal_like_flags(f) && !seen_string_primitive_types.is_empty()
                {
                    "string"
                } else {
                    ""
                };
                if primitive_name.is_empty() {
                    break;
                }
            }
            if primitive_name.is_empty() {
                continue;
            }
            if type_values_literal.is_empty() {
                type_values_literal = join_parts(ctx.checker, flags);
            }
            ctx.report_node(
                *type_node,
                build_primitive_overridden_message(primitive_name, &type_values_literal),
            );
        }
    }
    if !seen_union_types.is_empty() {
        return;
    }

    let mut check_literal_type_overrides_primitive =
        |literal_types: &[TypeFlagsWithNodeOrType], primitive_types: &[P<Node>], primitive_name| {
            if literal_types.is_empty() {
                return;
            }
            let type_values_literal = join_parts(ctx.checker, literal_types);
            for &type_node in primitive_types {
                ctx.report_node(
                    type_node,
                    build_primitive_overridden_message(&type_values_literal, primitive_name),
                );
            }
        };

    check_literal_type_overrides_primitive(
        &seen_big_int_literal_types,
        &seen_big_int_primitive_types,
        "bigint",
    );
    check_literal_type_overrides_primitive(
        &seen_boolean_literal_types,
        &seen_boolean_primitive_types,
        "boolean",
    );
    check_literal_type_overrides_primitive(
        &seen_number_literal_types,
        &seen_number_primitive_types,
        "number",
    );
    check_literal_type_overrides_primitive(
        &seen_string_literal_types,
        &seen_string_primitive_types,
        "string",
    );
}

type OverriddenNodes = Vec<(P<Node>, Vec<TypeFlagsWithNodeOrType>)>;

fn upsert(map: &mut OverriddenNodes, node: P<Node>, part: TypeFlagsWithNodeOrType) {
    if let Some(entry) = map.iter_mut().find(|(n, _)| *n == node) {
        entry.1.push(part);
    } else {
        map.push((node, vec![part]));
    }
}

fn check_union(ctx: &mut Ctx, node: P<Node>) {
    // Go keys these by node in maps (random iteration order); source order here.
    let mut overridden_big_int_type_nodes: OverriddenNodes = Vec::new();
    let mut overridden_boolean_type_nodes: OverriddenNodes = Vec::new();
    let mut overridden_number_type_nodes: OverriddenNodes = Vec::new();
    let mut overridden_string_type_nodes: OverriddenNodes = Vec::new();

    let mut seen_primitive_type_flags = TypeFlags::empty();

    let check_union_bottom_and_top_types =
        |ctx: &mut Ctx, type_part: &TypeFlagsWithNodeOrType, type_node: P<Node>| -> bool {
            let message = if type_part.flags == TypeFlags::Any {
                let type_name = type_part.to_display(ctx.checker);
                if type_name == "any" {
                    build_overrides_message(&type_name, "union")
                } else {
                    build_error_type_overrides_message(&type_name, "union")
                }
            } else if type_part.flags == TypeFlags::Unknown {
                build_overrides_message(&type_part.to_display(ctx.checker), "union")
            } else if type_part.flags == TypeFlags::Never {
                if is_node_inside_return_type(node) {
                    return false;
                }
                build_overridden_message("never", "union")
            } else {
                return false;
            };
            ctx.report_node(type_node, message);
            true
        };

    for &type_node in node.as_union_type_node().types().nodes() {
        let type_part_flags = get_type_node_type_part_flags(ctx, type_node);
        for type_part in type_part_flags {
            if check_union_bottom_and_top_types(ctx, &type_part, type_node) {
                continue;
            }
            let f = type_part.flags;
            if f == TypeFlags::BigIntLiteral {
                upsert(&mut overridden_big_int_type_nodes, type_node, type_part);
            } else if f == TypeFlags::BooleanLiteral {
                upsert(&mut overridden_boolean_type_nodes, type_node, type_part);
            } else if f == TypeFlags::NumberLiteral {
                upsert(&mut overridden_number_type_nodes, type_node, type_part);
            } else if is_string_literal_like_flags(f) {
                upsert(&mut overridden_string_type_nodes, type_node, type_part);
            }
            seen_primitive_type_flags |= f
                & (TypeFlags::BigInt | TypeFlags::Boolean | TypeFlags::Number | TypeFlags::String);
        }
    }

    let mut check_overridden_types = |primitive_flag: TypeFlags,
                                      overridden_nodes: &OverriddenNodes,
                                      primitive_name: &str| {
        if !seen_primitive_type_flags.intersects(primitive_flag) {
            return;
        }
        for (type_node, type_flags) in overridden_nodes {
            let literal = join_parts(ctx.checker, type_flags);
            ctx.report_node(*type_node, build_literal_overridden_message(&literal, primitive_name));
        }
    };

    check_overridden_types(TypeFlags::BigInt, &overridden_big_int_type_nodes, "bigint");
    check_overridden_types(TypeFlags::Boolean, &overridden_boolean_type_nodes, "boolean");
    check_overridden_types(TypeFlags::Number, &overridden_number_type_nodes, "number");
    check_overridden_types(TypeFlags::String, &overridden_string_type_nodes, "string");
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::IntersectionType => check_intersection(ctx, node),
            Kind::UnionType => check_union(ctx, node),
            _ => {}
        }
    }
}
