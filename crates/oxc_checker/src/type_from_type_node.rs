use oxc_ast::ast::TSType;

use crate::Checker;
use oxc_types::TypeId;

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

            // Not yet implemented — return `any` as a placeholder
            TSType::TSArrayType(_)
            | TSType::TSConditionalType(_)
            | TSType::TSConstructorType(_)
            | TSType::TSFunctionType(_)
            | TSType::TSImportType(_)
            | TSType::TSIndexedAccessType(_)
            | TSType::TSInferType(_)
            | TSType::TSIntersectionType(_)
            | TSType::TSLiteralType(_)
            | TSType::TSMappedType(_)
            | TSType::TSNamedTupleMember(_)
            | TSType::TSTemplateLiteralType(_)
            | TSType::TSThisType(_)
            | TSType::TSTupleType(_)
            | TSType::TSTypeLiteral(_)
            | TSType::TSTypeOperatorType(_)
            | TSType::TSTypePredicate(_)
            | TSType::TSTypeQuery(_)
            | TSType::TSTypeReference(_)
            | TSType::TSUnionType(_)
            | TSType::JSDocNullableType(_)
            | TSType::JSDocNonNullableType(_)
            | TSType::JSDocUnknownType(_) => {
                // TODO: implement these
                self.any_type
            }
        }
    }
}
