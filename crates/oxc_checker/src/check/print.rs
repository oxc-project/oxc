//! Type → string for diagnostics, tsc-flavored.
//!
//! References print their declared name (`Type 'boolean' is not assignable to
//! type 'ID'.`), so callers should print the *unexpanded* ids they related.

use std::fmt::Write;

use crate::ir::{RefTarget, Type, TypeId};

use super::TypeView;

const MAX_DEPTH: u32 = 3;

/// Render a type id for use in a diagnostic message.
pub fn type_to_string(view: &TypeView<'_>, id: TypeId) -> String {
    let mut out = String::new();
    write_type(view, id, 0, &mut out);
    out
}

fn write_type(view: &TypeView<'_>, id: TypeId, depth: u32, out: &mut String) {
    if depth > MAX_DEPTH {
        out.push_str("...");
        return;
    }
    match view.get(id) {
        // Freshness is invisible in messages; named local aliases print their
        // declared name, as tsc does.
        Type::Fresh(inner) => write_type(view, *inner, depth, out),
        Type::Named { name, .. } | Type::TypeParam { name, .. } => out.push_str(name),
        Type::Readonly(inner) => {
            out.push_str("readonly ");
            write_type(view, *inner, depth, out);
        }
        Type::EnumMember { symbol, index } => {
            let data = view.env.symbol(*symbol);
            out.push_str(&data.name);
            if let crate::ir::SymbolKind::Enum { members } = &data.kind
                && let Some(member) = members.get(*index as usize)
            {
                out.push('.');
                out.push_str(&member.name);
            }
        }
        Type::OptionalElem(inner) => {
            write_type(view, *inner, depth, out);
            out.push('?');
        }
        Type::RestElem(inner) => {
            out.push_str("...");
            write_type(view, *inner, depth, out);
        }
        Type::EnumValue(symbol) | Type::ClassValue(symbol) => {
            out.push_str("typeof ");
            out.push_str(&view.env.symbol(*symbol).name);
        }
        // Unmodeled constructs would have related as `Unknown`; when one
        // appears nested in a printed type, `any` is the least-misleading name.
        Type::Any | Type::Unsupported => out.push_str("any"),
        Type::Unknown => out.push_str("unknown"),
        Type::Never => out.push_str("never"),
        Type::Void => out.push_str("void"),
        Type::Undefined => out.push_str("undefined"),
        Type::Null => out.push_str("null"),
        Type::String => out.push_str("string"),
        Type::Number => out.push_str("number"),
        Type::Boolean => out.push_str("boolean"),
        Type::BigInt => out.push_str("bigint"),
        Type::Symbol => out.push_str("symbol"),
        Type::ObjectKeyword => out.push_str("object"),
        Type::StringLiteral(s) => {
            let _ = write!(out, "\"{s}\"");
        }
        Type::NumberLiteral(n) => {
            let _ = write!(out, "{n}");
        }
        Type::BooleanLiteral(b) => {
            let _ = write!(out, "{b}");
        }
        Type::BigIntLiteral(b) => {
            let _ = write!(out, "{b}n");
        }
        Type::Union(members) => {
            // tsgo's display order (verified empirically against the oracle):
            // primitives by intrinsic rank, then string literals by value,
            // then number literals by value, then other literals, then named
            // types (aliases/interfaces/enums) alphabetically; anything else
            // keeps its position at the end. The sort is stable.
            #[derive(PartialEq, Eq, PartialOrd, Ord)]
            enum Key {
                Primitive(u32),
                StringLit(Box<str>),
                NumberLit(u64),
                OtherLit(u8),
                NamedType(Box<str>),
                Rest,
            }
            let mut ordered: Vec<TypeId> = members.to_vec();
            ordered.sort_by_key(|m| match view.get(*m) {
                Type::String | Type::Number | Type::Boolean | Type::BigInt | Type::Symbol => {
                    Key::Primitive(m.0)
                }
                Type::StringLiteral(value) => Key::StringLit(value.clone()),
                Type::NumberLiteral(value) => Key::NumberLit(value.to_bits()),
                Type::BooleanLiteral(b) => Key::OtherLit(u8::from(*b)),
                Type::BigIntLiteral(_) => Key::OtherLit(2),
                Type::Named { name, .. } => Key::NamedType(name.clone()),
                Type::Ref { target: RefTarget::Symbol(symbol), .. }
                | Type::EnumMember { symbol, .. } => {
                    Key::NamedType(view.env.symbol(*symbol).name.clone())
                }
                _ => Key::Rest,
            });
            for (i, member) in ordered.iter().enumerate() {
                if i > 0 {
                    out.push_str(" | ");
                }
                write_type(view, *member, depth + 1, out);
            }
        }
        Type::Intersection(members) => {
            for (i, member) in members.iter().enumerate() {
                if i > 0 {
                    out.push_str(" & ");
                }
                write_type(view, *member, depth + 1, out);
            }
        }
        Type::Array(elem) => {
            let needs_parens = matches!(
                view.get(*elem),
                Type::Union(_) | Type::Function(_) | Type::Intersection(_)
            );
            if needs_parens {
                out.push('(');
            }
            write_type(view, *elem, depth + 1, out);
            if needs_parens {
                out.push(')');
            }
            out.push_str("[]");
        }
        Type::Tuple(members) => {
            out.push('[');
            for (i, member) in members.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_type(view, *member, depth + 1, out);
            }
            out.push(']');
        }
        Type::Object(shape) => {
            // tsc format: `{ x: number; y?: string; }` — every member ends
            // with a semicolon, including the last.
            if shape.members.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{ ");
            for member in &shape.members {
                out.push_str(&member.name);
                if member.optional {
                    out.push('?');
                }
                out.push_str(": ");
                write_type(view, member.ty, depth + 1, out);
                out.push_str("; ");
            }
            out.push('}');
        }
        Type::Function(shape) => {
            out.push('(');
            for (i, param) in shape.params.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&param.name);
                if param.optional {
                    out.push('?');
                }
                out.push_str(": ");
                write_type(view, param.ty, depth + 1, out);
            }
            out.push_str(") => ");
            write_type(view, shape.ret, depth + 1, out);
        }
        Type::Ref { target, args } => {
            match target {
                RefTarget::Symbol(symbol) => out.push_str(&view.env.symbol(*symbol).name),
                RefTarget::Local(_)
                | RefTarget::PendingLocal(_)
                | RefTarget::PendingImport { .. }
                | RefTarget::Unresolved => out.push_str("any"),
            }
            if !args.is_empty() {
                out.push('<');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write_type(view, *arg, depth + 1, out);
                }
                out.push('>');
            }
        }
    }
}
