// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

use crate::ast::StructDetails;

/// Details of how `#[ast]` macro should modify structs.
#[expect(clippy::unreadable_literal)]
pub static STRUCTS: phf::Map<&'static str, StructDetails> = ::phf::Map {
    key: 16287231350648472473,
    disps: &[
        (0, 1),
        (0, 5),
        (0, 29),
        (0, 13),
        (0, 3),
        (0, 37),
        (0, 1),
        (0, 0),
        (0, 1),
        (0, 0),
        (0, 110),
        (0, 24),
        (0, 55),
        (0, 14),
        (0, 35),
        (0, 21),
        (0, 1),
        (0, 1),
        (0, 22),
        (0, 44),
        (0, 19),
        (0, 12),
        (0, 55),
        (0, 0),
        (0, 34),
        (0, 74),
        (0, 3),
        (0, 6),
        (0, 1),
        (0, 8),
        (0, 3),
        (0, 23),
        (0, 0),
        (0, 9),
        (0, 70),
        (0, 8),
        (0, 12),
        (0, 96),
        (0, 37),
        (0, 170),
        (0, 1),
        (0, 25),
        (0, 68),
        (0, 203),
        (0, 6),
        (0, 17),
        (0, 0),
        (0, 16),
        (0, 3),
        (0, 0),
        (0, 193),
        (0, 0),
        (0, 133),
        (0, 6),
        (0, 0),
        (0, 62),
        (0, 12),
        (0, 102),
        (0, 5),
        (0, 66),
        (1, 59),
        (0, 172),
        (0, 104),
        (0, 5),
        (0, 13),
        (0, 16),
        (0, 77),
        (0, 27),
        (0, 131),
        (0, 16),
        (0, 12),
        (0, 40),
        (0, 67),
        (2, 234),
        (0, 5),
        (0, 0),
        (0, 221),
        (0, 6),
        (0, 6),
    ],
    entries: &[
        (
            "TSBigIntKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "VariableDeclarator",
            StructDetails {
                field_order: Some(&[1, 0, 2, 4, 5, 6, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("Modifier", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "YieldExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSThisParameter",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "CatchParameter",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ImportAttribute",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "CharacterClass",
            StructDetails {
                field_order: Some(&[0, 2, 3, 4, 1]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "EcmaScriptModule",
            StructDetails {
                field_order: Some(&[4, 0, 1, 2, 3]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "UpdateExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "CatchClause",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "NewTarget",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "JSXExpressionContainer",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSEnumMember",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "IndexedReference",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "Directive",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSXClosingFragment",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "ComputedMemberExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "StaticBlock",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "AssignmentExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Error",
            StructDetails {
                field_order: Some(&[4, 0, 1, 2, 3]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "MethodDefinition",
            StructDetails {
                field_order: Some(&[1, 0, 2, 6, 7, 8, 3, 4, 5, 9, 10, 11]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSNamedTupleMember",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTypeQuery",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "RawTransferMetadata",
            StructDetails {
                field_order: Some(&[0, 3, 4, 5, 1, 2]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "ClassBody",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "PropertyDefinition",
            StructDetails {
                field_order: Some(&[1, 0, 2, 6, 7, 8, 9, 3, 4, 5, 10, 11, 12, 13, 14]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTypeAssertion",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "WithStatement",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ClassString",
            StructDetails { field_order: Some(&[0, 2, 1]), is_node: false, is_transparent: false },
        ),
        (
            "TSTypeOperator",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSConstructSignatureDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "FixedSizeAllocatorMetadata",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "V8IntrinsicExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSIndexSignature",
            StructDetails {
                field_order: Some(&[1, 0, 4, 5, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "SwitchCase",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSInterfaceDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 7, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSXEmptyExpression",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "ClassStringDisjunction",
            StructDetails { field_order: Some(&[0, 2, 1]), is_node: false, is_transparent: false },
        ),
        (
            "ArrayExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "FormalParameterRest",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Character",
            StructDetails { field_order: Some(&[0, 2, 1]), is_node: false, is_transparent: false },
        ),
        (
            "TSExportAssignment",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSTypePredicate",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "IdentifierName",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        ("ImportEntry", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "TSIntrinsicKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "ErrorLabel",
            StructDetails { field_order: Some(&[1, 0]), is_node: false, is_transparent: false },
        ),
        (
            "BreakStatement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "NamedReference",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "AccessorProperty",
            StructDetails {
                field_order: Some(&[1, 0, 2, 6, 7, 8, 9, 3, 4, 5, 10, 11]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Hashbang",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        ("NodeId", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        ("ReferenceId", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "ArrowFunctionExpression",
            StructDetails {
                field_order: Some(&[1, 0, 7, 8, 3, 4, 5, 6, 2, 9, 10]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSXSpreadChild",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        ("NonMaxU32", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "CallExpression",
            StructDetails {
                field_order: Some(&[1, 0, 4, 5, 6, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSStringKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "JSXOpeningFragment",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TSGlobalDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ForStatement",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Program",
            StructDetails {
                field_order: Some(&[1, 0, 8, 3, 4, 5, 6, 7, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "FormalParameter",
            StructDetails {
                field_order: Some(&[1, 0, 6, 7, 8, 9, 2, 3, 4, 5]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "NameSpan",
            StructDetails { field_order: Some(&[1, 0]), is_node: false, is_transparent: false },
        ),
        (
            "TSImportTypeQualifiedName",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "BigIntLiteral",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ExportNamedDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 2, 6]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("Disjunction", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "TSInterfaceHeritage",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "RawTransferData",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "TSCallSignatureDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "AssignmentPattern",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSExternalModuleReference",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "NewExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSXIdentifier",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSInterfaceBody",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "Span",
            StructDetails { field_order: Some(&[1, 2, 0]), is_node: false, is_transparent: false },
        ),
        (
            "TSClassImplements",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSNullKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TSTypeReference",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSXSpreadAttribute",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSNonNullExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSVoidKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "CapturingGroup",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "Class",
            StructDetails {
                field_order: Some(&[1, 0, 10, 3, 4, 5, 6, 7, 8, 9, 11, 12, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSPropertySignature",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4, 5, 6]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("Pattern", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "ExportDefaultDeclaration",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSNumberKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "UnaryExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "SequenceExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "BindingRestElement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSNeverKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "BooleanLiteral",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ThisExpression",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "LabeledStatement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ExportSpecifier",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "DoWhileStatement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "SwitchStatement",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ForOfStatement",
            StructDetails {
                field_order: Some(&[1, 0, 6, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("RegExp", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "TSTypeParameter",
            StructDetails {
                field_order: Some(&[1, 0, 5, 6, 7, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("RegExpFlags", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "RegExpLiteral",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "PrivateInExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSMappedType",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 7, 8, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTypeAliasDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSModuleBlock",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSFunctionType",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("Modifiers", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "LookAroundAssertion",
            StructDetails { field_order: Some(&[0, 2, 1]), is_node: false, is_transparent: false },
        ),
        (
            "DebuggerStatement",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TSLiteralType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "JSXAttribute",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSThisType",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TSUnionType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSModuleDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSSatisfiesExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "FunctionBody",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "AssignmentTargetRest",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "BindingIdentifier",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSUndefinedKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "Decorator",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ImportNamespaceSpecifier",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "JSXElement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTupleType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "JSXOpeningElement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "NumericLiteral",
            StructDetails {
                field_order: Some(&[1, 0, 4, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ImportDefaultSpecifier",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSTypeLiteral",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "JSXClosingElement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "IdentifierReference",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSDocNonNullableType",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSAsExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ContinueStatement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "SpreadElement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSArrayType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSEnumBody",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("I32Dummy", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "WhileStatement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ForInStatement",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSUnknownKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "PrivateIdentifier",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ImportDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 4, 5, 2, 6, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "EmptyStatement",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TemplateElementValue",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "TSImportType",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4, 5]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ObjectPattern",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSNamespaceExportDeclaration",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "StaticImport",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "BoundaryAssertion",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "TSConditionalType",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 6, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "LabelIdentifier",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSQualifiedName",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ImportExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("SourceType", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "ObjectAssignmentTarget",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "AwaitExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "JSXFragment",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTemplateLiteralType",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ArrayPattern",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "WithClause",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSObjectKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        ("Dot", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "Super",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TSInstantiationExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSDocNullableType",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TemplateLiteral",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ChainExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "PrivateFieldExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TemplateElement",
            StructDetails {
                field_order: Some(&[1, 0, 4, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSEnumDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 4, 5, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Quantifier",
            StructDetails {
                field_order: Some(&[0, 1, 2, 4, 3]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "TSMethodSignature",
            StructDetails {
                field_order: Some(&[1, 0, 3, 8, 9, 10, 4, 5, 6, 7, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "JSDocUnknownType",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "JSXText",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSIntersectionType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TaggedTemplateExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ImportSpecifier",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTypeAnnotation",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ObjectExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ExportAllDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSIndexedAccessType",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Elision",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "ObjectProperty",
            StructDetails {
                field_order: Some(&[1, 0, 2, 6, 7, 3, 4, 5]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "BlockStatement",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ThrowStatement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "CommentNewlines",
            StructDetails { field_order: None, is_node: false, is_transparent: true },
        ),
        ("IgnoreGroup", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "UnicodePropertyEscape",
            StructDetails {
                field_order: Some(&[0, 3, 4, 1, 2]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "ReturnStatement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "NullLiteral",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        ("SymbolId", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "TSAnyKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "CharacterClassRange",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "JSXMemberExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ImportMeta",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "StaticExport",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "TSOptionalType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "RegExpPattern",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "VariableDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 2, 4, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "StaticMemberExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "ParenthesizedExpression",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "BinaryExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSImportEqualsDeclaration",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSIndexSignatureName",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTypeParameterDeclaration",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TSInferType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ArrayAssignmentTarget",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "Function",
            StructDetails {
                field_order: Some(&[1, 0, 9, 3, 10, 11, 12, 4, 5, 6, 7, 8, 2, 13, 14]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSBooleanKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "TSSymbolKeyword",
            StructDetails { field_order: Some(&[1, 0]), is_node: true, is_transparent: false },
        ),
        (
            "IfStatement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "FormalParameters",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "AssignmentTargetPropertyProperty",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSTypeParameterInstantiation",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "AssignmentTargetPropertyIdentifier",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("ScopeId", StructDetails { field_order: None, is_node: false, is_transparent: true }),
        (
            "TSRestType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "ExportEntry",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4, 5, 6]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "AssignmentTargetWithDefault",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "DynamicImport",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "TSParenthesizedType",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        (
            "TryStatement",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "LogicalExpression",
            StructDetails {
                field_order: Some(&[1, 0, 3, 2, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "StringLiteral",
            StructDetails {
                field_order: Some(&[1, 0, 3, 4, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        ("Comment", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "ExpressionStatement",
            StructDetails { field_order: Some(&[1, 0, 2]), is_node: true, is_transparent: false },
        ),
        ("Alternative", StructDetails { field_order: None, is_node: false, is_transparent: false }),
        (
            "BindingProperty",
            StructDetails {
                field_order: Some(&[1, 0, 4, 5, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "CharacterClassEscape",
            StructDetails { field_order: None, is_node: false, is_transparent: false },
        ),
        (
            "ConditionalExpression",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3, 4]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "TSConstructorType",
            StructDetails {
                field_order: Some(&[1, 0, 6, 3, 4, 5, 2]),
                is_node: true,
                is_transparent: false,
            },
        ),
        (
            "RawTransferMetadata2",
            StructDetails {
                field_order: Some(&[0, 3, 4, 5, 1, 2]),
                is_node: false,
                is_transparent: false,
            },
        ),
        (
            "JSXNamespacedName",
            StructDetails {
                field_order: Some(&[1, 0, 2, 3]),
                is_node: true,
                is_transparent: false,
            },
        ),
    ],
};
