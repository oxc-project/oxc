use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Semantic;
use oxc_types::{IntrinsicType, ObjectFlags, TypeArena, TypeData, TypeFlags, TypeId};

/// TypeScript type checker.
///
/// The checker runs after semantic analysis and resolves types for all
/// expressions and declarations, emitting diagnostics for type errors.
///
/// Types are stored in a `TypeArena` and referenced by `TypeId`.
/// Well-known types (primitives) are pre-allocated during construction.
pub struct Checker<'a> {
    /// The semantic analysis result for the program being checked.
    pub(crate) semantic: Semantic<'a>,

    /// Arena storing all types created during checking.
    pub(crate) type_arena: TypeArena,

    /// Diagnostics collected during type checking.
    pub(crate) diagnostics: Vec<OxcDiagnostic>,

    /// Recursion depth counter to prevent stack overflow.
    pub(crate) recursion_depth: u32,

    // Well-known types, pre-allocated during construction.
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    pub number_type: TypeId,
    pub bigint_type: TypeId,
    pub boolean_type: TypeId,
    pub es_symbol_type: TypeId,
    pub void_type: TypeId,
    pub undefined_type: TypeId,
    pub null_type: TypeId,
    pub never_type: TypeId,
    /// The `object` non-primitive type (not `Object` interface).
    pub non_primitive_type: TypeId,
    /// The `true` literal type.
    pub true_type: TypeId,
    /// The `false` literal type.
    pub false_type: TypeId,
}

impl<'a> Checker<'a> {
    /// Create a new type checker from a completed semantic analysis.
    pub fn new(semantic: Semantic<'a>) -> Self {
        let mut type_arena = TypeArena::with_capacity(64);

        let any_type = new_intrinsic(&mut type_arena, TypeFlags::Any, "any");
        let unknown_type = new_intrinsic(&mut type_arena, TypeFlags::Unknown, "unknown");
        let string_type = new_intrinsic(&mut type_arena, TypeFlags::String, "string");
        let number_type = new_intrinsic(&mut type_arena, TypeFlags::Number, "number");
        let bigint_type = new_intrinsic(&mut type_arena, TypeFlags::BigInt, "bigint");
        let boolean_type = new_intrinsic(&mut type_arena, TypeFlags::Boolean, "boolean");
        let es_symbol_type = new_intrinsic(&mut type_arena, TypeFlags::ESSymbol, "symbol");
        let void_type = new_intrinsic(&mut type_arena, TypeFlags::Void, "void");
        let undefined_type = new_intrinsic(&mut type_arena, TypeFlags::Undefined, "undefined");
        let null_type = new_intrinsic(&mut type_arena, TypeFlags::Null, "null");
        let never_type = new_intrinsic(&mut type_arena, TypeFlags::Never, "never");
        let non_primitive_type =
            new_intrinsic(&mut type_arena, TypeFlags::NonPrimitive, "object");

        let true_type = type_arena.new_type(
            TypeFlags::BooleanLiteral,
            ObjectFlags::None,
            TypeData::Literal(oxc_types::LiteralType::Boolean(true)),
            None,
        );
        let false_type = type_arena.new_type(
            TypeFlags::BooleanLiteral,
            ObjectFlags::None,
            TypeData::Literal(oxc_types::LiteralType::Boolean(false)),
            None,
        );

        Self {
            semantic,
            type_arena,
            diagnostics: Vec::new(),
            recursion_depth: 0,
            any_type,
            unknown_type,
            string_type,
            number_type,
            bigint_type,
            boolean_type,
            es_symbol_type,
            void_type,
            undefined_type,
            null_type,
            never_type,
            non_primitive_type,
            true_type,
            false_type,
        }
    }

    /// Run the type checker and return collected diagnostics.
    pub fn check(mut self) -> Vec<OxcDiagnostic> {
        // TODO: Walk AST and check all declarations/statements/expressions.
        let _ = &self.semantic;
        std::mem::take(&mut self.diagnostics)
    }

    /// Get the type arena (for testing/inspection).
    pub fn type_arena(&self) -> &TypeArena {
        &self.type_arena
    }

    /// Get the semantic analysis result.
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }
}

fn new_intrinsic(arena: &mut TypeArena, flags: TypeFlags, name: &'static str) -> TypeId {
    arena.new_type(
        flags,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: name }),
        None,
    )
}
