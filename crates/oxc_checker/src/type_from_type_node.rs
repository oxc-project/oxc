use oxc_ast::ast::TSType;
use oxc_span::CompactStr;
use oxc_types::{LiteralType, ObjectFlags, TypeData, TypeFlags, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Resolve a `TSType` AST node to a `TypeId`.
    ///
    /// This is the main entry point for converting type syntax into the
    /// checker's internal type representation.
    pub fn get_type_from_type_node(&mut self, ts_type: &TSType<'_>) -> TypeId {
        match ts_type {
            // Keyword types -> pre-allocated intrinsic types
            TSType::TSAnyKeyword(_) => self.any_type,
            TSType::TSUnknownKeyword(_) => self.unknown_type,
            TSType::TSStringKeyword(_) => self.string_type,
            TSType::TSNumberKeyword(_) => self.number_type,
            TSType::TSBigIntKeyword(_) => self.bigint_type,
            TSType::TSBooleanKeyword(_) => self.boolean_type,
            TSType::TSSymbolKeyword(_) => self.es_symbol_type,
            TSType::TSVoidKeyword(_) => self.void_type,
            TSType::TSUndefinedKeyword(_) => self.undefined_type,
            TSType::TSNullKeyword(_) => self.null_type,
            TSType::TSNeverKeyword(_) => self.never_type,
            TSType::TSObjectKeyword(_) => self.non_primitive_type,
            TSType::TSIntrinsicKeyword(_) => self.unknown_type, // intrinsic keyword is contextual

            // Parenthesized types unwrap to the inner type
            TSType::TSParenthesizedType(t) => self.get_type_from_type_node(&t.type_annotation),

            // Union types
            TSType::TSUnionType(union) => self.get_type_from_union_type_node(union),

            // Literal types in annotations: `let x: 42`, `let x: "hello"`, etc.
            TSType::TSLiteralType(lit) => self.get_type_from_literal_type_node(lit),

            // Intersection types: `A & B`
            TSType::TSIntersectionType(intersection) => {
                self.get_type_from_intersection_type_node(intersection)
            }

            // Type reference: `Foo`, `Array<T>`, `MyInterface`, etc.
            TSType::TSTypeReference(ref_type) => {
                self.get_type_from_type_reference(ref_type)
            }

            // Not yet implemented — return `any` as a placeholder
            TSType::TSArrayType(_)
            | TSType::TSConditionalType(_)
            | TSType::TSConstructorType(_)
            | TSType::TSFunctionType(_)
            | TSType::TSImportType(_)
            | TSType::TSIndexedAccessType(_)
            | TSType::TSInferType(_)
            | TSType::TSMappedType(_)
            | TSType::TSNamedTupleMember(_)
            | TSType::TSTemplateLiteralType(_)
            | TSType::TSThisType(_)
            | TSType::TSTupleType(_)
            | TSType::TSTypeLiteral(_)
            | TSType::TSTypeOperatorType(_)
            | TSType::TSTypePredicate(_)
            | TSType::TSTypeQuery(_)
            | TSType::JSDocNullableType(_)
            | TSType::JSDocNonNullableType(_)
            | TSType::JSDocUnknownType(_) => {
                // TODO: implement these
                self.any_type
            }
        }
    }

    fn get_type_from_union_type_node(&mut self, union: &oxc_ast::ast::TSUnionType<'_>) -> TypeId {
        let types: Vec<TypeId> = union
            .types
            .iter()
            .map(|t| self.get_type_from_type_node(t))
            .collect();
        self.get_or_create_union_type(types)
    }

    fn get_type_from_intersection_type_node(
        &mut self,
        intersection: &oxc_ast::ast::TSIntersectionType<'_>,
    ) -> TypeId {
        let types: Vec<TypeId> = intersection
            .types
            .iter()
            .map(|t| self.get_type_from_type_node(t))
            .collect();
        self.get_or_create_intersection_type(types)
    }

    fn get_type_from_literal_type_node(
        &mut self,
        lit: &oxc_ast::ast::TSLiteralType<'_>,
    ) -> TypeId {
        use oxc_ast::ast::TSLiteral;
        match &lit.literal {
            TSLiteral::BooleanLiteral(b) => {
                if b.value {
                    self.true_type
                } else {
                    self.false_type
                }
            }
            TSLiteral::NumericLiteral(n) => self.type_arena.new_type(
                TypeFlags::NumberLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::Number(n.value)),
                None,
            ),
            TSLiteral::BigIntLiteral(n) => self.type_arena.new_type(
                TypeFlags::BigIntLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::BigInt(CompactStr::new(n.value.as_str()))),
                None,
            ),
            TSLiteral::StringLiteral(s) => self.type_arena.new_type(
                TypeFlags::StringLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::String(CompactStr::new(&s.value))),
                None,
            ),
            // Template literals and unary expressions in type position — fallback
            TSLiteral::TemplateLiteral(_) | TSLiteral::UnaryExpression(_) => self.any_type,
        }
    }

    /// Resolve a type reference (`Foo`, `Array<T>`, etc.) to a TypeId.
    ///
    /// Looks up the type name in the symbol table, then delegates to
    /// `get_declared_type_of_symbol` for type-namespace resolution.
    fn get_type_from_type_reference(
        &mut self,
        ref_type: &oxc_ast::ast::TSTypeReference<'_>,
    ) -> TypeId {
        use oxc_ast::ast::TSTypeName;

        // Only handle simple identifier references for now (not `A.B.C`)
        let TSTypeName::IdentifierReference(ident) = &ref_type.type_name else {
            return self.any_type;
        };

        let Some(reference_id) = ident.reference_id.get() else {
            return self.any_type;
        };

        let reference = self.semantic().scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            // Unresolved reference — could be a global type (Array, Promise, etc.)
            // Check the global type environment
            return self.get_global_type(&ident.name);
        };

        // TODO: handle type_arguments for generic instantiation
        self.get_declared_type_of_symbol(symbol_id)
    }
}
