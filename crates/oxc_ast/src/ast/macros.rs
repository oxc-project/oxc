/// Macro to inherit enum variants from another enum.
///
/// (for further details see <https://github.com/oxc-project/oxc/pull/3115>)
///
/// # Types which can be inherited
///
/// The following types' variants can be inherited:
///
/// * `Expression`
/// * `MemberExpression`
/// * `AssignmentTarget`
/// * `SimpleAssignmentTarget`
/// * `AssignmentTargetPattern`
/// * `Declaration`
/// * `ModuleDeclaration`
/// * `TSType`
/// * `TSTypeName`
///
/// # Expansion
///
/// ```
/// inherit_variants! {
///     #[ast]
///     enum Statement<'a> {
///         pub enum Statement<'a> {
///             BlockStatement(Box<'a, BlockStatement<'a>>) = 0,
///             BreakStatement(Box<'a, BreakStatement<'a>>) = 1,
///             @inherit Declaration
///             @inherit ModuleDeclaration
///         }
///     }
/// }
/// ```
///
/// expands to:
///
/// ```
/// #[ast]
/// enum Statement<'a> {
///     pub enum Statement<'a> {
///         BlockStatement(Box<'a, BlockStatement<'a>>) = 0,
///         BreakStatement(Box<'a, BreakStatement<'a>>) = 1,
///
///         // Inherited from `Declaration`
///         VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 32,
///         FunctionDeclaration(Box<'a, Function<'a>>) = 33,
///         // ...and many more
///
///         // Inherited from `ModuleDeclaration`
///         ImportDeclaration(Box<'a, ImportDeclaration<'a>>) = 64,
///         ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>) = 65,
///         // ...and many more
///     }
/// }
///
/// shared_enum_variants!(
///     Statement, Declaration,
///     is_declaration,
///     into_declaration,
///     as_declaration, as_declaration_mut,
///     to_declaration, to_declaration_mut,
///     [VariableDeclaration, FunctionDeclaration, ...more]
/// )
///
/// shared_enum_variants!(
///     Statement, ModuleDeclaration,
///     is_module_declaration,
///     into_module_declaration,
///     as_module_declaration, as_module_declaration_mut,
///     to_module_declaration, to_module_declaration_mut,
///     [ImportDeclaration, ExportAllDeclaration, ...more]
/// )
/// ```
macro_rules! inherit_variants {
    // Inherit `Expression`'s variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit Expression
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                // `Expression`'s own variants

                /// Inherited from [`Expression`]
                BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
                /// Inherited from [`Expression`]
                NullLiteral(Box<'a, NullLiteral>) = 1,
                /// Inherited from [`Expression`]
                NumericLiteral(Box<'a, NumericLiteral<'a>>) = 2,
                /// Inherited from [`Expression`]
                BigIntLiteral(Box<'a, BigIntLiteral<'a>>) = 3,
                /// Inherited from [`Expression`]
                RegExpLiteral(Box<'a, RegExpLiteral<'a>>) = 4,
                /// Inherited from [`Expression`]
                StringLiteral(Box<'a, StringLiteral<'a>>) = 5,
                /// Inherited from [`Expression`]
                TemplateLiteral(Box<'a, TemplateLiteral<'a>>) = 6,

                /// Inherited from [`Expression`]
                Identifier(Box<'a, IdentifierReference<'a>>) = 7,

                /// Inherited from [`Expression`]
                MetaProperty(Box<'a, MetaProperty<'a>>) = 8,
                /// Inherited from [`Expression`]
                Super(Box<'a, Super>) = 9,

                /// Inherited from [`Expression`]
                ArrayExpression(Box<'a, ArrayExpression<'a>>) = 10,
                /// Inherited from [`Expression`]
                ArrowFunctionExpression(Box<'a, ArrowFunctionExpression<'a>>) = 11,
                /// Inherited from [`Expression`]
                AssignmentExpression(Box<'a, AssignmentExpression<'a>>) = 12,
                /// Inherited from [`Expression`]
                AwaitExpression(Box<'a, AwaitExpression<'a>>) = 13,
                /// Inherited from [`Expression`]
                BinaryExpression(Box<'a, BinaryExpression<'a>>) = 14,
                /// Inherited from [`Expression`]
                CallExpression(Box<'a, CallExpression<'a>>) = 15,
                /// Inherited from [`Expression`]
                ChainExpression(Box<'a, ChainExpression<'a>>) = 16,
                /// Inherited from [`Expression`]
                ClassExpression(Box<'a, Class<'a>>) = 17,
                /// Inherited from [`Expression`]
                ConditionalExpression(Box<'a, ConditionalExpression<'a>>) = 18,
                /// Inherited from [`Expression`]
                FunctionExpression(Box<'a, Function<'a>>) = 19,
                /// Inherited from [`Expression`]
                ImportExpression(Box<'a, ImportExpression<'a>>) = 20,
                /// Inherited from [`Expression`]
                LogicalExpression(Box<'a, LogicalExpression<'a>>) = 21,
                /// Inherited from [`Expression`]
                NewExpression(Box<'a, NewExpression<'a>>) = 22,
                /// Inherited from [`Expression`]
                ObjectExpression(Box<'a, ObjectExpression<'a>>) = 23,
                /// Inherited from [`Expression`]
                ParenthesizedExpression(Box<'a, ParenthesizedExpression<'a>>) = 24,
                /// Inherited from [`Expression`]
                SequenceExpression(Box<'a, SequenceExpression<'a>>) = 25,
                /// Inherited from [`Expression`]
                TaggedTemplateExpression(Box<'a, TaggedTemplateExpression<'a>>) = 26,
                /// Inherited from [`Expression`]
                ThisExpression(Box<'a, ThisExpression>) = 27,
                /// Inherited from [`Expression`]
                UnaryExpression(Box<'a, UnaryExpression<'a>>) = 28,
                /// Inherited from [`Expression`]
                UpdateExpression(Box<'a, UpdateExpression<'a>>) = 29,
                /// Inherited from [`Expression`]
                YieldExpression(Box<'a, YieldExpression<'a>>) = 30,
                /// Inherited from [`Expression`]
                PrivateInExpression(Box<'a, PrivateInExpression<'a>>) = 31,

                /// Inherited from [`Expression`]
                JSXElement(Box<'a, JSXElement<'a>>) = 32,
                /// Inherited from [`Expression`]
                JSXFragment(Box<'a, JSXFragment<'a>>) = 33,

                /// Inherited from [`Expression`]
                TSAsExpression(Box<'a, TSAsExpression<'a>>) = 34,
                /// Inherited from [`Expression`]
                TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>) = 35,
                /// Inherited from [`Expression`]
                TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>) = 36,
                /// Inherited from [`Expression`]
                TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>) = 37,
                /// Inherited from [`Expression`]
                TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>) = 38,

                // Inherited from `MemberExpression`
                @inherit MemberExpression

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            Expression,
            is_expression,
            into_expression,
            as_expression,
            as_expression_mut,
            to_expression,
            to_expression_mut,
            [
                BooleanLiteral,
                NullLiteral,
                NumericLiteral,
                BigIntLiteral,
                RegExpLiteral,
                StringLiteral,
                TemplateLiteral,
                Identifier,
                MetaProperty,
                Super,
                ArrayExpression,
                ArrowFunctionExpression,
                AssignmentExpression,
                AwaitExpression,
                BinaryExpression,
                CallExpression,
                ChainExpression,
                ClassExpression,
                ConditionalExpression,
                FunctionExpression,
                ImportExpression,
                LogicalExpression,
                NewExpression,
                ObjectExpression,
                ParenthesizedExpression,
                SequenceExpression,
                TaggedTemplateExpression,
                ThisExpression,
                UnaryExpression,
                UpdateExpression,
                YieldExpression,
                PrivateInExpression,
                JSXElement,
                JSXFragment,
                TSAsExpression,
                TSSatisfiesExpression,
                TSTypeAssertion,
                TSNonNullExpression,
                TSInstantiationExpression,
                ComputedMemberExpression,
                StaticMemberExpression,
                PrivateFieldExpression,
            ]
        );
    };

    // Inherit `MemberExpression`'s variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit MemberExpression
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                /// Inherited from [`MemberExpression`].
                ///
                /// `MemberExpression[?Yield, ?Await] [ Expression[+In, ?Yield, ?Await] ]`
                ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>) = 48,
                /// Inherited from [`MemberExpression`].
                ///
                /// `MemberExpression[?Yield, ?Await] . IdentifierName`
                StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>) = 49,
                /// Inherited from [`MemberExpression`].
                ///
                /// `MemberExpression[?Yield, ?Await] . PrivateIdentifier`
                PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>) = 50,

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            MemberExpression,
            is_member_expression,
            into_member_expression,
            as_member_expression,
            as_member_expression_mut,
            to_member_expression,
            to_member_expression_mut,
            [ComputedMemberExpression, StaticMemberExpression, PrivateFieldExpression]
        );
    };

    // Inherit `AssignmentTarget` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit AssignmentTarget
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                @inherit SimpleAssignmentTarget
                @inherit AssignmentTargetPattern

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            AssignmentTarget,
            is_assignment_target,
            into_assignment_target,
            as_assignment_target,
            as_assignment_target_mut,
            to_assignment_target,
            to_assignment_target_mut,
            [
                AssignmentTargetIdentifier,
                ComputedMemberExpression,
                StaticMemberExpression,
                PrivateFieldExpression,
                TSAsExpression,
                TSSatisfiesExpression,
                TSNonNullExpression,
                TSTypeAssertion,
                TSInstantiationExpression,
                ArrayAssignmentTarget,
                ObjectAssignmentTarget,
            ]
        );
    };

    // Inherit `SimpleAssignmentTarget` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit SimpleAssignmentTarget
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                /// Inherited from [`SimpleAssignmentTarget`]
                AssignmentTargetIdentifier(Box<'a, IdentifierReference<'a>>) = 0,

                /// Inherited from [`SimpleAssignmentTarget`]
                TSAsExpression(Box<'a, TSAsExpression<'a>>) = 1,
                /// Inherited from [`SimpleAssignmentTarget`]
                TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>) = 2,
                /// Inherited from [`SimpleAssignmentTarget`]
                TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>) = 3,
                /// Inherited from [`SimpleAssignmentTarget`]
                TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>) = 4,
                /// Inherited from [`SimpleAssignmentTarget`]
                TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>) = 5,

                // Inherited from `MemberExpression`
                @inherit MemberExpression

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            SimpleAssignmentTarget,
            is_simple_assignment_target,
            into_simple_assignment_target,
            as_simple_assignment_target,
            as_simple_assignment_target_mut,
            to_simple_assignment_target,
            to_simple_assignment_target_mut,
            [
                AssignmentTargetIdentifier,
                ComputedMemberExpression,
                StaticMemberExpression,
                PrivateFieldExpression,
                TSAsExpression,
                TSSatisfiesExpression,
                TSNonNullExpression,
                TSTypeAssertion,
                TSInstantiationExpression
            ]
        );
    };

    // Inherit `AssignmentTargetPattern` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit AssignmentTargetPattern
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                /// Inherited from [`AssignmentTargetPattern`]
                ArrayAssignmentTarget(Box<'a, ArrayAssignmentTarget<'a>>) = 8,
                /// Inherited from [`AssignmentTargetPattern`]
                ObjectAssignmentTarget(Box<'a, ObjectAssignmentTarget<'a>>) = 9,

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            AssignmentTargetPattern,
            is_assignment_target_pattern,
            into_assignment_target_pattern,
            as_assignment_target_pattern,
            as_assignment_target_pattern_mut,
            to_assignment_target_pattern,
            to_assignment_target_pattern_mut,
            [ArrayAssignmentTarget, ObjectAssignmentTarget]
        );
    };

    // Inherit `Declaration` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit Declaration
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                /// Inherited from [`Declaration`]
                VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 32,
                /// Inherited from [`Declaration`]
                FunctionDeclaration(Box<'a, Function<'a>>) = 33,
                /// Inherited from [`Declaration`]
                ClassDeclaration(Box<'a, Class<'a>>) = 34,

                /// Inherited from [`Declaration`]
                TSTypeAliasDeclaration(Box<'a, TSTypeAliasDeclaration<'a>>) = 35,
                /// Inherited from [`Declaration`]
                TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>) = 36,
                /// Inherited from [`Declaration`]
                TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>) = 37,
                /// Inherited from [`Declaration`]
                TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>) = 38,
                /// Inherited from [`Declaration`]
                TSImportEqualsDeclaration(Box<'a, TSImportEqualsDeclaration<'a>>) = 39,

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            Declaration,
            is_declaration,
            into_declaration,
            as_declaration,
            as_declaration_mut,
            to_declaration,
            to_declaration_mut,
            [
                VariableDeclaration,
                FunctionDeclaration,
                ClassDeclaration,
                TSTypeAliasDeclaration,
                TSInterfaceDeclaration,
                TSEnumDeclaration,
                TSModuleDeclaration,
                TSImportEqualsDeclaration,
            ]
        );
    };

    // Inherit `ModuleDeclaration` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit ModuleDeclaration
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                /// Inherited from [`ModuleDeclaration`].
                /// `import hello from './world.js';`
                /// `import * as t from './world.js';`
                ImportDeclaration(Box<'a, ImportDeclaration<'a>>) = 64,
                /// Inherited from [`ModuleDeclaration`].
                /// `export * as numbers from '../numbers.js'`
                ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>) = 65,
                /// Inherited from [`ModuleDeclaration`].
                /// `export default 5;`
                ExportDefaultDeclaration(Box<'a, ExportDefaultDeclaration<'a>>) = 66,
                /// Inherited from [`ModuleDeclaration`].
                /// `export {five} from './numbers.js';`
                /// `export {six, seven};`
                ExportNamedDeclaration(Box<'a, ExportNamedDeclaration<'a>>) = 67,

                /// Inherited from [`ModuleDeclaration`].
                /// `export = 5;`
                TSExportAssignment(Box<'a, TSExportAssignment<'a>>) = 68,
                /// Inherited from [`ModuleDeclaration`].
                /// `export as namespace React;`
                TSNamespaceExportDeclaration(Box<'a, TSNamespaceExportDeclaration<'a>>) = 69,

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            ModuleDeclaration,
            is_module_declaration,
            into_module_declaration,
            as_module_declaration,
            as_module_declaration_mut,
            to_module_declaration,
            to_module_declaration_mut,
            [
                ImportDeclaration,
                ExportAllDeclaration,
                ExportDefaultDeclaration,
                ExportNamedDeclaration,
                TSExportAssignment,
                TSNamespaceExportDeclaration,
            ]
        );
    };

    // Inherit `TSType` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit TSType
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                // Keyword
                /// Inherited from [`TSType`]
                TSAnyKeyword(Box<'a, TSAnyKeyword>) = 0,
                /// Inherited from [`TSType`]
                TSBigIntKeyword(Box<'a, TSBigIntKeyword>) = 1,
                /// Inherited from [`TSType`]
                TSBooleanKeyword(Box<'a, TSBooleanKeyword>) = 2,
                /// Inherited from [`TSType`]
                TSIntrinsicKeyword(Box<'a, TSIntrinsicKeyword>) = 3,
                /// Inherited from [`TSType`]
                TSNeverKeyword(Box<'a, TSNeverKeyword>) = 4,
                /// Inherited from [`TSType`]
                TSNullKeyword(Box<'a, TSNullKeyword>) = 5,
                /// Inherited from [`TSType`]
                TSNumberKeyword(Box<'a, TSNumberKeyword>) = 6,
                /// Inherited from [`TSType`]
                TSObjectKeyword(Box<'a, TSObjectKeyword>) = 7,
                /// Inherited from [`TSType`]
                TSStringKeyword(Box<'a, TSStringKeyword>) = 8,
                /// Inherited from [`TSType`]
                TSSymbolKeyword(Box<'a, TSSymbolKeyword>) = 9,
                /// Inherited from [`TSType`]
                TSThisType(Box<'a, TSThisType>) = 10,
                /// Inherited from [`TSType`]
                TSUndefinedKeyword(Box<'a, TSUndefinedKeyword>) = 11,
                /// Inherited from [`TSType`]
                TSUnknownKeyword(Box<'a, TSUnknownKeyword>) = 12,
                /// Inherited from [`TSType`]
                TSVoidKeyword(Box<'a, TSVoidKeyword>) = 13,

                // Compound
                /// Inherited from [`TSType`]
                TSArrayType(Box<'a, TSArrayType<'a>>) = 14,
                /// Inherited from [`TSType`]
                TSConditionalType(Box<'a, TSConditionalType<'a>>) = 15,
                /// Inherited from [`TSType`]
                TSConstructorType(Box<'a, TSConstructorType<'a>>) = 16,
                /// Inherited from [`TSType`]
                TSFunctionType(Box<'a, TSFunctionType<'a>>) = 17,
                /// Inherited from [`TSType`]
                TSImportType(Box<'a, TSImportType<'a>>) = 18,
                /// Inherited from [`TSType`]
                TSIndexedAccessType(Box<'a, TSIndexedAccessType<'a>>) = 19,
                /// Inherited from [`TSType`]
                TSInferType(Box<'a, TSInferType<'a>>) = 20,
                /// Inherited from [`TSType`]
                TSIntersectionType(Box<'a, TSIntersectionType<'a>>) = 21,
                /// Inherited from [`TSType`]
                TSLiteralType(Box<'a, TSLiteralType<'a>>) = 22,
                /// Inherited from [`TSType`]
                TSMappedType(Box<'a, TSMappedType<'a>>) = 23,
                /// Inherited from [`TSType`]
                TSNamedTupleMember(Box<'a, TSNamedTupleMember<'a>>) = 24,
                /// Inherited from [`TSType`]
                TSQualifiedName(Box<'a, TSQualifiedName<'a>>) = 25,
                /// Inherited from [`TSType`]
                TSTemplateLiteralType(Box<'a, TSTemplateLiteralType<'a>>) = 26,
                /// Inherited from [`TSType`]
                TSTupleType(Box<'a, TSTupleType<'a>>) = 27,
                /// Inherited from [`TSType`]
                TSTypeLiteral(Box<'a, TSTypeLiteral<'a>>) = 28,
                /// Inherited from [`TSType`]
                TSTypeOperatorType(Box<'a, TSTypeOperator<'a>>) = 29,
                /// Inherited from [`TSType`]
                TSTypePredicate(Box<'a, TSTypePredicate<'a>>) = 30,
                /// Inherited from [`TSType`]
                TSTypeQuery(Box<'a, TSTypeQuery<'a>>) = 31,
                /// Inherited from [`TSType`]
                TSTypeReference(Box<'a, TSTypeReference<'a>>) = 32,
                /// Inherited from [`TSType`]
                TSUnionType(Box<'a, TSUnionType<'a>>) = 33,
                /// Inherited from [`TSType`]
                TSParenthesizedType(Box<'a, TSParenthesizedType<'a>>) = 34,

                // JSDoc
                /// Inherited from [`TSType`]
                JSDocNullableType(Box<'a, JSDocNullableType<'a>>) = 35,
                /// Inherited from [`TSType`]
                JSDocNonNullableType(Box<'a, JSDocNonNullableType<'a>>) = 36,
                /// Inherited from [`TSType`]
                JSDocUnknownType(Box<'a, JSDocUnknownType>) = 37,

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            TSType,
            is_ts_type,
            into_ts_type,
            as_ts_type,
            as_ts_type_mut,
            to_ts_type,
            to_ts_type_mut,
            [
                TSAnyKeyword,
                TSBigIntKeyword,
                TSBooleanKeyword,
                TSIntrinsicKeyword,
                TSNeverKeyword,
                TSNullKeyword,
                TSNumberKeyword,
                TSObjectKeyword,
                TSStringKeyword,
                TSSymbolKeyword,
                TSThisType,
                TSUndefinedKeyword,
                TSUnknownKeyword,
                TSVoidKeyword,
                TSArrayType,
                TSConditionalType,
                TSConstructorType,
                TSFunctionType,
                TSImportType,
                TSIndexedAccessType,
                TSInferType,
                TSIntersectionType,
                TSLiteralType,
                TSMappedType,
                TSNamedTupleMember,
                TSQualifiedName,
                TSTemplateLiteralType,
                TSTupleType,
                TSTypeLiteral,
                TSTypeOperatorType,
                TSTypePredicate,
                TSTypeQuery,
                TSTypeReference,
                TSUnionType,
                TSParenthesizedType,
                JSDocNullableType,
                JSDocNonNullableType,
                JSDocUnknownType,
            ]
        );
    };

    // Inherit `TSTypeName` variants
    (
        $(#[$attr:meta])*
        pub enum $ty:ident<'a> {
            $($(#[$variant_attr:meta])* $variant_name:ident($variant_type:ty) = $variant_discrim:literal,)*
            @inherit TSTypeName
            $($rest:tt)*
        }
    ) => {
        $crate::ast::macros::inherit_variants! {
            $(#[$attr])*
            pub enum $ty<'a> {
                $($(#[$variant_attr])* $variant_name($variant_type) = $variant_discrim,)*

                /// Inherited from [`TSTypeName`]
                IdentifierReference(Box<'a, IdentifierReference<'a>>) = 0,
                /// Inherited from [`TSTypeName`]
                QualifiedName(Box<'a, TSQualifiedName<'a>>) = 1,

                $($rest)*
            }
        }

        $crate::ast::macros::shared_enum_variants!(
            $ty,
            TSTypeName,
            is_ts_type_name,
            into_ts_type_name,
            as_ts_type_name,
            as_ts_type_name_mut,
            to_ts_type_name,
            to_ts_type_name_mut,
            [IdentifierReference, QualifiedName]
        );
    };

    // Passthrough - no further inheritance to handle
    ($($rest:tt)*) => {$($rest)*};
}
pub(crate) use inherit_variants;

/// Macro to allow conversion between 2 enum types where they share some of the same variants.
/// "Parent" enum contains all the "child"'s variants, plus parent contains further other variants.
/// e.g. `Statement` and `Declaration`.
///
/// The discriminants and types of the shared variants must be identical between the 2 enums.
/// All variants must have a `Box<_>` payload.
/// Equality of types is guaranteed by `From` and `TryFrom` impls this macro creates.
/// These will fail to compile if the types differ for any variant.
/// Equality of discriminants is checked with a compile-time assertion.
///
/// # SAFETY
/// Both enums must be `#[repr(C, u8)]` or using this macro is unsound.
///
/// # Expansion
///
/// NB: For illustration only - `Statement` and `Declaration` in reality share 9 variants, not 2.
///
/// ```
/// shared_enum_variants!(
///     Statement, Declaration,
///     is_declaration,
///     into_declaration,
///     as_declaration, as_declaration_mut,
///     to_declaration, to_declaration_mut,
///     [VariableDeclaration, FunctionDeclaration]
/// )
/// ```
///
/// expands to:
///
/// ```
/// const _: () = {
///     assert!(discriminant!(Statement::VariableDeclaration) == discriminant!(Declaration::VariableDeclaration));
///     assert!(discriminant!(Statement::FunctionDeclaration) == discriminant!(Declaration::FunctionDeclaration));
/// };
///
/// impl<'a> Statement<'a> {
///     /// Return if a `Statement` is a `Declaration`.
///     #[inline]
///     pub fn is_declaration(&self) -> bool {
///         match self {
///             Self::VariableDeclaration(_) | Self::FunctionDeclaration(_) => true,
///             _ => false,
///         }
///     }
///
///     /// Convert `Statement` to `Declaration`.
///     /// # Panic
///     /// Panics if not convertible.
///     #[inline]
///     pub fn into_declaration(self) -> Declaration<'a> {
///         Declaration::try_from(self).unwrap()
///     }
///
///     /// Convert `&Statement` to `&Declaration`.
///     #[inline]
///     pub fn as_declaration(&self) -> Option<&Declaration<'a>> {
///         if self.is_declaration() {
///             Some(unsafe { &*(self as *const _ as *const Declaration) })
///         } else {
///             None
///         }
///     }
///
///     /// Convert `&mut Statement` to `&mut Declaration`.
///     #[inline]
///     pub fn as_declaration_mut(&mut self) -> Option<&mut Declaration<'a>> {
///         if self.is_declaration() {
///             Some(unsafe { &mut *(self as *mut _ as *mut Declaration) })
///         } else {
///             None
///         }
///     }
///
///     /// Convert `&Statement` to `&Declaration`.
///     /// # Panic
///     /// Panics if not convertible.
///     #[inline]
///     pub fn to_declaration(&self) -> &Declaration<'a> {
///         self.as_declaration().unwrap()
///     }
///
///     /// Convert `&mut Statement` to `&mut Declaration`.
///     /// # Panic
///     /// Panics if not convertible.
///     #[inline]
///     pub fn to_declaration_mut(&mut self) -> &mut Declaration<'a> {
///         self.as_declaration_mut().unwrap()
///     }
/// }
///
/// impl<'a> TryFrom<Statement<'a>> for Declaration<'a> {
///     type Error = ();
///
///     /// "Convert `Statement` to `Declaration`.
///     #[inline]
///     fn try_from(value: Statement<'a>) -> Result<Self, Self::Error> {
///         match value {
///             Statement::VariableDeclaration(o) => Ok(Declaration::VariableDeclaration(o)),
///             Statement::FunctionDeclaration(o) => Ok(Declaration::FunctionDeclaration(o)),
///             _ => Err(()),
///         }
///     }
/// }
///
/// impl<'a> From<Declaration<'a>> for Statement<'a> {
///     /// Convert `Declaration` to `Statement`.
///     #[inline]
///     fn from(value: Declaration<'a>) -> Self {
///         match value {
///             Declaration::VariableDeclaration(o) => Statement::VariableDeclaration(o),
///             Declaration::FunctionDeclaration(o) => Statement::FunctionDeclaration(o),
///         }
///     }
/// }
/// ```
macro_rules! shared_enum_variants {
    (
        $parent:ident, $child:ident,
        $is_child:ident,
        $into_child:ident,
        $as_child:ident, $as_child_mut:ident,
        $to_child:ident, $to_child_mut:ident,
        [$($variant:ident),+ $(,)?]
    ) => {
        // Ensure discriminants match for all variants between parent and child types
        const _: () = {
            $(
                assert!(
                    $crate::ast::macros::discriminant!($parent::$variant)
                    == $crate::ast::macros::discriminant!($child::$variant),
                    concat!(
                        "Non-matching discriminants for `", stringify!($variant),
                        "` between `", stringify!($parent), "` and `", stringify!($child), "`"
                    )
                );
            )+
        };

        impl<'a> $parent<'a> {
            #[doc = concat!("Return if a `", stringify!($parent), "` is a `", stringify!($child), "`.")]
            #[inline]
            pub fn $is_child(&self) -> bool {
                matches!(
                    self,
                    $(Self::$variant(_))|+
                )
            }

            #[doc = concat!("Convert `", stringify!($parent), "` to `", stringify!($child), "`.")]
            #[doc = "# Panic"]
            #[doc = "Panics if not convertible."]
            #[inline]
            pub fn $into_child(self) -> $child<'a> {
                $child::try_from(self).unwrap()
            }

            #[doc = concat!("Convert `&", stringify!($parent), "` to `&", stringify!($child), "`.")]
            #[inline]
            pub fn $as_child(&self) -> Option<&$child<'a>> {
                if self.$is_child() {
                    #[allow(unsafe_code)]
                    // SAFETY: Transmute is safe because discriminants + types are identical between
                    // `$parent` and `$child` for $child variants
                    Some(unsafe { &*std::ptr::from_ref(self).cast::<$child>() })
                } else {
                    None
                }
            }

            #[doc = concat!("Convert `&mut ", stringify!($parent), "` to `&mut ", stringify!($child), "`.")]
            #[inline]
            pub fn $as_child_mut(&mut self) -> Option<&mut $child<'a>> {
                if self.$is_child() {
                    #[allow(unsafe_code)]
                    // SAFETY: Transmute is safe because discriminants + types are identical between
                    // `$parent` and `$child` for $child variants
                    Some(unsafe { &mut *std::ptr::from_mut(self).cast::<$child>() })
                } else {
                    None
                }
            }

            #[doc = concat!("Convert `&", stringify!($parent), "` to `&", stringify!($child), "`.")]
            #[doc = "# Panic"]
            #[doc = "Panics if not convertible."]
            #[inline]
            pub fn $to_child(&self) -> &$child<'a> {
                self.$as_child().unwrap()
            }

            #[doc = concat!("Convert `&mut ", stringify!($parent), "` to `&mut ", stringify!($child), "`.")]
            #[doc = "# Panic"]
            #[doc = "Panics if not convertible."]
            #[inline]
            pub fn $to_child_mut(&mut self) -> &mut $child<'a> {
                self.$as_child_mut().unwrap()
            }
        }

        impl<'a> TryFrom<$parent<'a>> for $child<'a> {
            type Error = ();

            #[doc = concat!("Convert `", stringify!($parent), "` to `", stringify!($child), "`.")]
            #[inline]
            fn try_from(value: $parent<'a>) -> Result<Self, Self::Error> {
                // Compiler should implement this as a check of discriminant and then zero-cost transmute,
                // as discriminants for `$parent` and `$child` are aligned
                match value {
                    $($parent::$variant(o) => Ok($child::$variant(o)),)+
                    _ => Err(())
                }
            }
        }

        impl<'a> From<$child<'a>> for $parent<'a> {
            #[doc = concat!("Convert `", stringify!($child), "` to `", stringify!($parent), "`.")]
            #[inline]
            fn from(value: $child<'a>) -> Self {
                // Compiler should implement this as zero-cost transmute as discriminants
                // for `$child` and `$parent` are aligned
                match value {
                    $($child::$variant(o) => $parent::$variant(o),)+
                }
            }
        }
    }
}
pub(crate) use shared_enum_variants;

/// Macro to get discriminant of an enum.
/// # SAFETY
/// Enum must be `#[repr(C, u8)]` or using this macro is unsound.
/// <https://doc.rust-lang.org/std/mem/fn.discriminant.html>
macro_rules! discriminant {
    ($ty:ident :: $variant:ident) => {{
        #[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
        unsafe {
            let t = std::mem::ManuallyDrop::new($ty::$variant(oxc_allocator::Box::dangling()));
            *(std::ptr::addr_of!(t).cast::<u8>())
        }
    }};
}
pub(crate) use discriminant;
