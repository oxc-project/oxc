//! Tri-state assignability.
//!
//! `relate` answers "is `src` assignable to `tgt`" with `True` / `False` /
//! `Unknown`. Diagnostics are emitted **only on `False`**: any construct v0
//! does not model resolves to `Unknown` and stays silent, so the checker is
//! sound-for-what-it-reports by construction.

use crate::ir::{ObjectShape, RefTarget, SymbolId, SymbolKind, Type, TypeId};

use super::TypeView;

/// Result of a relation query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// Definitely assignable.
    True,
    /// Definitely not assignable → diagnostic.
    False,
    /// Not determinable with v0's model → silent.
    Unknown,
}

impl Relation {
    fn and(self, other: Relation) -> Relation {
        match (self, other) {
            (Relation::False, _) | (_, Relation::False) => Relation::False,
            (Relation::Unknown, _) | (_, Relation::Unknown) => Relation::Unknown,
            (Relation::True, Relation::True) => Relation::True,
        }
    }
}

/// What a type id resolves to after chasing aliases and interface refs.
pub(super) enum Resolved {
    /// A structural type, possibly the input itself.
    Concrete(TypeId),
    /// A reference to an enum type — nominal.
    Enum(SymbolId),
    /// Anything v0 cannot see through (uninstantiated generics, namespaces,
    /// unresolved names, unsupported constructs).
    Opaque,
}

const MAX_DEPTH: u32 = 32;

/// Is `src` assignable to `tgt`?
pub fn relate(view: &TypeView<'_>, src: TypeId, tgt: TypeId) -> Relation {
    relate_inner(view, src, tgt, 0)
}

pub(super) fn resolve(view: &TypeView<'_>, mut id: TypeId) -> Resolved {
    for _ in 0..MAX_DEPTH {
        match view.get(id) {
            // Freshness and local-alias names are transparent to relations.
            Type::Fresh(inner) | Type::Named { ty: inner, .. } => id = *inner,
            Type::Ref { target, args } => {
                let RefTarget::Symbol(symbol) = target else { return Resolved::Opaque };
                match &view.env.symbol(*symbol).kind {
                    SymbolKind::TypeAlias { ty, params } | SymbolKind::Interface { ty, params }
                        if args.is_empty() && params.is_empty() =>
                    {
                        id = *ty;
                    }
                    // Classes are structural in TS: follow to the instance
                    // shape (private members already made the shape inexact).
                    SymbolKind::Class { instance } if args.is_empty() => id = *instance,
                    SymbolKind::Enum { .. } => return Resolved::Enum(*symbol),
                    // Uninstantiated generics, namespaces, values in type
                    // position — unmodeled.
                    _ => return Resolved::Opaque,
                }
            }
            Type::Unsupported | Type::Intersection(_) | Type::TypeParam { .. } => {
                return Resolved::Opaque;
            }
            _ => return Resolved::Concrete(id),
        }
    }
    // Alias cycle (`type A = B; type B = A`) or pathological depth.
    Resolved::Opaque
}

// The match below is organized rule-by-rule mirroring TS's assignability
// rules; several distinct rules share a body on purpose, and flattening the
// or-patterns would obscure the pairs.
#[expect(clippy::match_same_arms, clippy::unnested_or_patterns)]
fn relate_inner(view: &TypeView<'_>, src: TypeId, tgt: TypeId, depth: u32) -> Relation {
    if depth > MAX_DEPTH {
        return Relation::Unknown;
    }
    if src == tgt {
        return Relation::True;
    }

    let (src, tgt) = match (resolve(view, src), resolve(view, tgt)) {
        (Resolved::Opaque, _) | (_, Resolved::Opaque) => return Relation::Unknown,
        // Enums are nominal.
        (Resolved::Enum(a), Resolved::Enum(b)) => {
            return if a == b { Relation::True } else { Relation::False };
        }
        (Resolved::Enum(a), Resolved::Concrete(t)) => {
            return relate_enum_to_concrete(view, a, src, t, depth);
        }
        (Resolved::Concrete(s), Resolved::Enum(b)) => {
            return relate_concrete_to_enum(view, s, b, tgt, depth);
        }
        (Resolved::Concrete(s), Resolved::Concrete(t)) => (s, t),
    };
    if src == tgt {
        return Relation::True;
    }

    let sty = view.get(src);
    let tty = view.get(tgt);

    // Top/bottom rules. `never` accepts only `never` — including from `any`.
    if matches!(tty, Type::Never) {
        return if matches!(sty, Type::Never) { Relation::True } else { Relation::False };
    }
    if matches!(tty, Type::Any | Type::Unknown) || matches!(sty, Type::Any | Type::Never) {
        return Relation::True;
    }
    if matches!(sty, Type::Unknown) {
        return Relation::False; // only assignable to any/unknown, handled above
    }

    // Loose null/undefined.
    if !view.env.strict_null_checks && matches!(sty, Type::Null | Type::Undefined) {
        return Relation::True;
    }

    // Unions: a union source needs every member assignable; a union target
    // needs some member to accept the source.
    if let Type::Union(members) = sty {
        let mut result = Relation::True;
        for member in members {
            match relate_inner(view, *member, tgt, depth + 1) {
                Relation::False => return Relation::False,
                relation => result = result.and(relation),
            }
        }
        return result;
    }
    if let Type::Union(members) = tty {
        let mut all_false = true;
        for member in members {
            match relate_inner(view, src, *member, depth + 1) {
                Relation::True => return Relation::True,
                Relation::Unknown => all_false = false,
                Relation::False => {}
            }
        }
        return if all_false { Relation::False } else { Relation::Unknown };
    }

    // Readonly arrays/tuples: a mutable (or readonly) source satisfies a
    // readonly target element-wise; a readonly source never satisfies a
    // mutable array/tuple target (TS4104).
    match (sty, tty) {
        (Type::Readonly(s_inner), Type::Readonly(t_inner)) => {
            return relate_inner(view, *s_inner, *t_inner, depth + 1);
        }
        (_, Type::Readonly(t_inner)) => {
            return relate_inner(view, src, *t_inner, depth + 1);
        }
        (Type::Readonly(_), Type::Array(_) | Type::Tuple(_)) => return Relation::False,
        (Type::Readonly(s_inner), _) => {
            return relate_inner(view, *s_inner, tgt, depth + 1);
        }
        _ => {}
    }

    match (sty, tty) {
        // Literal → its base primitive, literal → identical literal.
        (Type::StringLiteral(a), Type::StringLiteral(b)) => bool_relation(a == b),
        (Type::StringLiteral(_), Type::String) => Relation::True,
        #[expect(clippy::float_cmp)]
        (Type::NumberLiteral(a), Type::NumberLiteral(b)) => bool_relation(a == b),
        (Type::NumberLiteral(_), Type::Number) => Relation::True,
        (Type::BooleanLiteral(a), Type::BooleanLiteral(b)) => bool_relation(a == b),
        (Type::BooleanLiteral(_), Type::Boolean) => Relation::True,
        (Type::BigIntLiteral(a), Type::BigIntLiteral(b)) => bool_relation(a == b),
        (Type::BigIntLiteral(_), Type::BigInt) => Relation::True,

        // Identical primitives (distinct ids can both be e.g. `string` only
        // via the fixed intrinsic ids, but stay defensive).
        (Type::String, Type::String)
        | (Type::Number, Type::Number)
        | (Type::Boolean, Type::Boolean)
        | (Type::BigInt, Type::BigInt)
        | (Type::Symbol, Type::Symbol)
        | (Type::Null, Type::Null)
        | (Type::Undefined, Type::Undefined)
        | (Type::Void, Type::Void)
        | (Type::Undefined, Type::Void) => Relation::True,

        // Enum members: nominal identity; assignable to their base primitive
        // by value kind.
        (
            Type::EnumMember { symbol: e1, index: i1 },
            Type::EnumMember { symbol: e2, index: i2 },
        ) => bool_relation(e1 == e2 && i1 == i2),
        (Type::EnumMember { symbol, index }, Type::Number) => {
            bool_relation(!enum_member_is_string(view, *symbol, *index))
        }
        (Type::EnumMember { symbol, index }, Type::String) => {
            bool_relation(enum_member_is_string(view, *symbol, *index))
        }
        // Member values are not tracked: literal comparisons stay silent.
        (Type::EnumMember { .. }, Type::NumberLiteral(_) | Type::StringLiteral(_))
        | (Type::NumberLiteral(_) | Type::StringLiteral(_), Type::EnumMember { .. }) => {
            Relation::Unknown
        }
        (Type::EnumMember { .. }, _) | (_, Type::EnumMember { .. }) => Relation::False,
        // Enum/class value sides (the `typeof X` objects) are unmodeled
        // beyond definitely not being primitives.
        (
            Type::EnumValue(_) | Type::ClassValue(_),
            Type::String
            | Type::Number
            | Type::Boolean
            | Type::BigInt
            | Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_)
            | Type::Null
            | Type::Undefined
            | Type::Void,
        ) => Relation::False,
        (Type::EnumValue(_) | Type::ClassValue(_), _)
        | (_, Type::EnumValue(_) | Type::ClassValue(_)) => Relation::Unknown,

        // `object` accepts exactly the non-primitives.
        (
            Type::Object(_) | Type::Array(_) | Type::Tuple(_) | Type::Function(_),
            Type::ObjectKeyword,
        ) => Relation::True,
        (_, Type::ObjectKeyword) => Relation::False,

        // Arrays/tuples (covariant, as in TS).
        (Type::Array(a), Type::Array(b)) => relate_inner(view, *a, *b, depth + 1),
        (Type::Tuple(members), Type::Array(elem)) => {
            let mut result = Relation::True;
            for member in members {
                match relate_inner(view, *member, *elem, depth + 1) {
                    Relation::False => return Relation::False,
                    relation => result = result.and(relation),
                }
            }
            result
        }
        (Type::Tuple(src_members), Type::Tuple(tgt_members)) => {
            // Sources with modifiers (declared tuples) are unmodeled.
            if src_members
                .iter()
                .any(|m| matches!(view.get(*m), Type::OptionalElem(_) | Type::RestElem(_)))
            {
                return Relation::Unknown;
            }
            let fixed = tgt_members
                .iter()
                .take_while(|m| !matches!(view.get(**m), Type::OptionalElem(_) | Type::RestElem(_)))
                .count();
            let optionals = tgt_members[fixed..]
                .iter()
                .take_while(|m| matches!(view.get(**m), Type::OptionalElem(_)))
                .count();
            let rest = tgt_members.get(fixed + optionals).and_then(|m| match view.get(*m) {
                Type::RestElem(array) => match view.get(*array) {
                    Type::Array(elem) => Some(*elem),
                    _ => None,
                },
                _ => None,
            });
            // Anything after a rest element is unmodeled.
            if tgt_members.len() > fixed + optionals + usize::from(rest.is_some()) {
                return Relation::Unknown;
            }
            if src_members.len() < fixed {
                return Relation::False;
            }
            if rest.is_none() && src_members.len() > fixed + optionals {
                return Relation::False;
            }
            let mut result = Relation::True;
            for (i, src_member) in src_members.iter().enumerate() {
                let target_elem = if i < fixed {
                    tgt_members[i]
                } else if i < fixed + optionals {
                    match view.get(tgt_members[i]) {
                        Type::OptionalElem(inner) => *inner,
                        _ => tgt_members[i],
                    }
                } else {
                    match rest {
                        Some(elem) => elem,
                        None => return Relation::False,
                    }
                };
                match relate_inner(view, *src_member, target_elem, depth + 1) {
                    Relation::False => return Relation::False,
                    relation => result = result.and(relation),
                }
            }
            result
        }
        (Type::Array(_), Type::Tuple(_)) => Relation::False,

        // Structural object comparison.
        (Type::Object(src_shape), Type::Object(tgt_shape)) => {
            relate_objects(view, src_shape, tgt_shape, depth)
        }

        // Arrays/functions/primitives against object shapes have apparent
        // members (`length`, `toString`, call signatures) v0 cannot see.
        (_, Type::Object(_)) => Relation::Unknown,

        // Function relations: parameters check bivariantly (tsc methods are
        // bivariant; only report False when BOTH directions definitely fail,
        // which is safe for strict function properties too). Returns are
        // covariant.
        (Type::Function(s_shape), Type::Function(t_shape)) => {
            let mut result = Relation::Unknown; // arity/this/overloads unmodeled
            let pairs = s_shape.params.len().min(t_shape.params.len());
            for i in 0..pairs {
                let (sp, tp) = (s_shape.params[i].ty, t_shape.params[i].ty);
                let contra = relate_inner(view, tp, sp, depth + 1);
                let co = relate_inner(view, sp, tp, depth + 1);
                if contra == Relation::False && co == Relation::False {
                    return Relation::False;
                }
            }
            let ret = relate_inner(view, s_shape.ret, t_shape.ret, depth + 1);
            if ret == Relation::False
                && !matches!(view.get(t_shape.ret), Type::Void | Type::Any | Type::Unknown)
            {
                return Relation::False;
            }
            result = result.and(Relation::Unknown);
            result
        }

        // A non-primitive source never fits a primitive/literal target.
        (Type::Object(_) | Type::Array(_) | Type::Tuple(_) | Type::Function(_), _) => {
            Relation::False
        }

        // Remaining combinations are leaf/leaf mismatches (string vs number,
        // null vs string under strict, void vs number, ...).
        _ => Relation::False,
    }
}

fn relate_objects(
    view: &TypeView<'_>,
    src: &ObjectShape,
    tgt: &ObjectShape,
    depth: u32,
) -> Relation {
    let mut result = Relation::True;
    for tgt_member in &tgt.members {
        if let Some(src_member) = src.members.iter().find(|m| m.name == tgt_member.name) {
            if src_member.optional && !tgt_member.optional {
                // Source member may be absent — needs `| undefined` handling
                // v0 doesn't model precisely.
                result = result.and(Relation::Unknown);
                continue;
            }
            match relate_inner(view, src_member.ty, tgt_member.ty, depth + 1) {
                Relation::False => return Relation::False,
                relation => result = result.and(relation),
            }
        } else {
            if tgt_member.optional {
                continue;
            }
            if src.inexact {
                // The source may have inherited/index members we can't see.
                result = result.and(Relation::Unknown);
            } else {
                // A definitely-missing required member is a definite error,
                // even when the target has members we can't see.
                return Relation::False;
            }
        }
    }
    if tgt.inexact {
        // All declared members fit, but inherited target members are unknown.
        return result.and(Relation::Unknown);
    }
    result
}

fn enum_member_is_string(view: &TypeView<'_>, symbol: SymbolId, index: u32) -> bool {
    match &view.env.symbol(symbol).kind {
        SymbolKind::Enum { members } => members.get(index as usize).is_some_and(|m| m.is_string),
        _ => false,
    }
}

fn enum_is_all_string(view: &TypeView<'_>, symbol: SymbolId) -> Option<bool> {
    match &view.env.symbol(symbol).kind {
        SymbolKind::Enum { members } => {
            if members.iter().all(|m| m.is_string) {
                Some(true)
            } else if members.iter().all(|m| !m.is_string) {
                Some(false)
            } else {
                None // mixed enum
            }
        }
        _ => None,
    }
}

/// An enum-typed source against a non-enum target.
#[expect(clippy::match_same_arms)] // rule-by-rule layout mirrors tsc
fn relate_enum_to_concrete(
    view: &TypeView<'_>,
    symbol: SymbolId,
    src: TypeId,
    tgt: TypeId,
    depth: u32,
) -> Relation {
    match view.get(tgt) {
        Type::Any | Type::Unknown => Relation::True,
        Type::Never => Relation::False,
        Type::Number => match enum_is_all_string(view, symbol) {
            Some(false) => Relation::True,
            Some(true) => Relation::False,
            None => Relation::Unknown,
        },
        Type::String => match enum_is_all_string(view, symbol) {
            Some(true) => Relation::True,
            Some(false) => Relation::False,
            None => Relation::Unknown,
        },
        Type::Union(members) => {
            let members = members.clone();
            let mut all_false = true;
            for member in &members {
                match relate_inner(view, src, *member, depth + 1) {
                    Relation::True => return Relation::True,
                    Relation::Unknown => all_false = false,
                    Relation::False => {}
                }
            }
            if all_false { Relation::False } else { Relation::Unknown }
        }
        Type::Boolean
        | Type::BigInt
        | Type::Null
        | Type::Undefined
        | Type::Void
        | Type::StringLiteral(_)
        | Type::NumberLiteral(_)
        | Type::BooleanLiteral(_)
        | Type::BigIntLiteral(_) => Relation::False,
        _ => Relation::Unknown,
    }
}

/// A non-enum source against an enum-typed target (nominal: only the enum's
/// own members are assignable).
fn relate_concrete_to_enum(
    view: &TypeView<'_>,
    src: TypeId,
    symbol: SymbolId,
    tgt: TypeId,
    depth: u32,
) -> Relation {
    match view.get(src) {
        Type::Any | Type::Never => Relation::True,
        Type::EnumMember { symbol: s, .. } => bool_relation(*s == symbol),
        Type::Union(members) => {
            let members = members.clone();
            let mut result = Relation::True;
            for member in &members {
                match relate_inner(view, *member, tgt, depth + 1) {
                    Relation::False => return Relation::False,
                    relation => result = result.and(relation),
                }
            }
            result
        }
        // Literals and other concrete types never satisfy a nominal enum.
        Type::StringLiteral(_)
        | Type::NumberLiteral(_)
        | Type::BooleanLiteral(_)
        | Type::BigIntLiteral(_)
        | Type::String
        | Type::Number
        | Type::Boolean
        | Type::BigInt
        | Type::Null
        | Type::Undefined
        | Type::Void
        | Type::Object(_)
        | Type::Array(_)
        | Type::Tuple(_)
        | Type::Function(_) => Relation::False,
        _ => Relation::Unknown,
    }
}

fn bool_relation(value: bool) -> Relation {
    if value { Relation::True } else { Relation::False }
}
