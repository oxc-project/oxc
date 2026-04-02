use std::sync::Arc;

use smallvec::smallvec;

use crate::{IntrinsicType, ObjectFlags, TypeArena, TypeData, TypeFlags, UnionType};

#[test]
fn create_intrinsic_types() {
    let arena = TypeArena::new();

    let string_type = arena.new_type(
        TypeFlags::String,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "string" }),
        None,
    );

    let number_type = arena.new_type(
        TypeFlags::Number,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "number" }),
        None,
    );

    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get_flags(string_type), TypeFlags::String);
    assert_eq!(arena.get_flags(number_type), TypeFlags::Number);

    assert!(arena.get_flags(string_type).intersects(TypeFlags::StringLike));
    assert!(!arena.get_flags(string_type).intersects(TypeFlags::NumberLike));
    assert!(arena.get_flags(number_type).intersects(TypeFlags::NumberLike));

    match arena.get_data(string_type) {
        TypeData::Intrinsic(t) => assert_eq!(t.intrinsic_name, "string"),
        _ => panic!("expected IntrinsicType"),
    }
}

#[test]
fn create_union_type() {
    let arena = TypeArena::new();

    let string_type = arena.new_type(
        TypeFlags::String,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "string" }),
        None,
    );

    let number_type = arena.new_type(
        TypeFlags::Number,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "number" }),
        None,
    );

    let union_type = arena.new_type(
        TypeFlags::Union,
        ObjectFlags::None,
        TypeData::Union(UnionType { types: Arc::new(smallvec![string_type, number_type]) }),
        None,
    );

    assert_eq!(arena.get_flags(union_type), TypeFlags::Union);
    assert!(arena.get_flags(union_type).intersects(TypeFlags::UnionOrIntersection));

    match arena.get_data(union_type) {
        TypeData::Union(u) => {
            assert_eq!(u.types.len(), 2);
            assert_eq!(u.types[0], string_type);
            assert_eq!(u.types[1], number_type);
        }
        _ => panic!("expected UnionType"),
    }
}

#[test]
fn type_flags_composite_checks() {
    // Verify composite flag relationships match TypeScript's
    assert!(TypeFlags::String.intersects(TypeFlags::StringLike));
    assert!(TypeFlags::StringLiteral.intersects(TypeFlags::StringLike));
    assert!(TypeFlags::TemplateLiteral.intersects(TypeFlags::StringLike));
    assert!(!TypeFlags::Number.intersects(TypeFlags::StringLike));

    assert!(TypeFlags::Undefined.intersects(TypeFlags::Nullable));
    assert!(TypeFlags::Null.intersects(TypeFlags::Nullable));
    assert!(!TypeFlags::String.intersects(TypeFlags::Nullable));

    assert!(TypeFlags::Any.intersects(TypeFlags::Intrinsic));
    assert!(TypeFlags::Never.intersects(TypeFlags::Intrinsic));
    assert!(TypeFlags::Void.intersects(TypeFlags::Intrinsic));
    assert!(!TypeFlags::Object.intersects(TypeFlags::Intrinsic));
    assert!(!TypeFlags::Union.intersects(TypeFlags::Intrinsic));

    assert!(TypeFlags::Object.intersects(TypeFlags::StructuredType));
    assert!(TypeFlags::Union.intersects(TypeFlags::StructuredType));
    assert!(TypeFlags::Intersection.intersects(TypeFlags::StructuredType));
    assert!(!TypeFlags::String.intersects(TypeFlags::StructuredType));

    assert!(TypeFlags::TypeParameter.intersects(TypeFlags::Instantiable));
    assert!(TypeFlags::Conditional.intersects(TypeFlags::Instantiable));
    assert!(TypeFlags::Index.intersects(TypeFlags::Instantiable));
    assert!(!TypeFlags::String.intersects(TypeFlags::Instantiable));
}

#[test]
fn arena_symbol_association() {
    use oxc_syntax::symbol::SymbolId;

    let arena = TypeArena::new();

    let type_without_symbol = arena.new_type(
        TypeFlags::String,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "string" }),
        None,
    );

    let symbol_id = SymbolId::from_usize(42);
    let type_with_symbol = arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Class,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "MyClass" }),
        Some((0, symbol_id)),
    );

    assert_eq!(arena.get_symbol(type_without_symbol), None);
    assert_eq!(arena.get_symbol(type_with_symbol), Some((0, symbol_id)));
    assert_eq!(arena.get_object_flags(type_with_symbol), ObjectFlags::Class);
}

#[test]
fn arena_alias_symbol() {
    use oxc_syntax::symbol::SymbolId;

    let arena = TypeArena::new();

    // Create an anonymous body type (like a type literal { x: number })
    let body_type = arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Anonymous,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: "anonymous" }),
        None,
    );

    // Body type has no symbol or alias
    assert_eq!(arena.get_symbol(body_type), None);
    assert_eq!(arena.get_alias_symbol(body_type), None);

    // Clone with alias (like attaching `type A = { x: number }`)
    let alias_symbol = SymbolId::from_usize(7);
    let aliased_type = arena.clone_type_with_alias(body_type, (0, alias_symbol));

    // Aliased type has the alias but no intrinsic symbol
    assert_eq!(arena.get_symbol(aliased_type), None);
    assert_eq!(arena.get_alias_symbol(aliased_type), Some((0, alias_symbol)));
    // Flags are preserved
    assert_eq!(arena.get_flags(aliased_type), TypeFlags::Object);
    assert_eq!(arena.get_object_flags(aliased_type), ObjectFlags::Anonymous);

    // clone_type_with_symbol does NOT set alias
    let intrinsic_symbol = SymbolId::from_usize(8);
    let symbol_type = arena.clone_type_with_symbol(body_type, Some((0, intrinsic_symbol)));
    assert_eq!(arena.get_symbol(symbol_type), Some((0, intrinsic_symbol)));
    assert_eq!(arena.get_alias_symbol(symbol_type), None);
}

#[test]
fn arena_with_capacity() {
    let arena = TypeArena::with_capacity(100);
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn type_data_size() {
    use crate::TypeData;
    let size = std::mem::size_of::<TypeData>();
    println!("TypeData size: {size} bytes");
    assert!(
        size <= 40,
        "TypeData enum grew beyond 40 bytes: {size} bytes. Consider boxing large variants.",
    );
}
