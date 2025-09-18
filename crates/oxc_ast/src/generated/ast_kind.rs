// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_kind.rs`.

#![expect(missing_docs)]
// FIXME (in ast_tools/src/generators/ast_kind.rs)

use std::ptr;

use oxc_allocator::{Address, GetAddress};
use oxc_span::{GetSpan, Span};

use crate::ast::*;

/// The largest integer value that can be mapped to an `AstType`/`AstKind` enum variant.
pub const AST_TYPE_MAX: u8 = 185;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AstType {
    Program = 0,
    IdentifierName = 1,
    IdentifierReference = 2,
    BindingIdentifier = 3,
    LabelIdentifier = 4,
    ThisExpression = 5,
    ArrayExpression = 6,
    Elision = 7,
    ObjectExpression = 8,
    ObjectProperty = 9,
    TemplateLiteral = 10,
    TaggedTemplateExpression = 11,
    TemplateElement = 12,
    ComputedMemberExpression = 13,
    StaticMemberExpression = 14,
    PrivateFieldExpression = 15,
    CallExpression = 16,
    NewExpression = 17,
    MetaProperty = 18,
    SpreadElement = 19,
    UpdateExpression = 20,
    UnaryExpression = 21,
    BinaryExpression = 22,
    PrivateInExpression = 23,
    LogicalExpression = 24,
    ConditionalExpression = 25,
    AssignmentExpression = 26,
    ArrayAssignmentTarget = 27,
    ObjectAssignmentTarget = 28,
    AssignmentTargetRest = 29,
    AssignmentTargetWithDefault = 30,
    AssignmentTargetPropertyIdentifier = 31,
    AssignmentTargetPropertyProperty = 32,
    SequenceExpression = 33,
    Super = 34,
    AwaitExpression = 35,
    ChainExpression = 36,
    ParenthesizedExpression = 37,
    Directive = 38,
    Hashbang = 39,
    BlockStatement = 40,
    VariableDeclaration = 41,
    VariableDeclarator = 42,
    EmptyStatement = 43,
    ExpressionStatement = 44,
    IfStatement = 45,
    DoWhileStatement = 46,
    WhileStatement = 47,
    ForStatement = 48,
    ForInStatement = 49,
    ForOfStatement = 50,
    ContinueStatement = 51,
    BreakStatement = 52,
    ReturnStatement = 53,
    WithStatement = 54,
    SwitchStatement = 55,
    SwitchCase = 56,
    LabeledStatement = 57,
    ThrowStatement = 58,
    TryStatement = 59,
    CatchClause = 60,
    CatchParameter = 61,
    DebuggerStatement = 62,
    AssignmentPattern = 63,
    ObjectPattern = 64,
    BindingProperty = 65,
    ArrayPattern = 66,
    BindingRestElement = 67,
    Function = 68,
    FormalParameters = 69,
    FormalParameter = 70,
    FunctionBody = 71,
    ArrowFunctionExpression = 72,
    YieldExpression = 73,
    Class = 74,
    ClassBody = 75,
    MethodDefinition = 76,
    PropertyDefinition = 77,
    PrivateIdentifier = 78,
    StaticBlock = 79,
    AccessorProperty = 80,
    ImportExpression = 81,
    ImportDeclaration = 82,
    ImportSpecifier = 83,
    ImportDefaultSpecifier = 84,
    ImportNamespaceSpecifier = 85,
    WithClause = 86,
    ImportAttribute = 87,
    ExportNamedDeclaration = 88,
    ExportDefaultDeclaration = 89,
    ExportAllDeclaration = 90,
    ExportSpecifier = 91,
    V8IntrinsicExpression = 92,
    BooleanLiteral = 93,
    NullLiteral = 94,
    NumericLiteral = 95,
    StringLiteral = 96,
    BigIntLiteral = 97,
    RegExpLiteral = 98,
    JSXElement = 99,
    JSXOpeningElement = 100,
    JSXClosingElement = 101,
    JSXFragment = 102,
    JSXOpeningFragment = 103,
    JSXClosingFragment = 104,
    JSXNamespacedName = 105,
    JSXMemberExpression = 106,
    JSXExpressionContainer = 107,
    JSXEmptyExpression = 108,
    JSXAttribute = 109,
    JSXSpreadAttribute = 110,
    JSXIdentifier = 111,
    JSXSpreadChild = 112,
    JSXText = 113,
    TSThisParameter = 114,
    TSEnumDeclaration = 115,
    TSEnumBody = 116,
    TSEnumMember = 117,
    TSTypeAnnotation = 118,
    TSLiteralType = 119,
    TSConditionalType = 120,
    TSUnionType = 121,
    TSIntersectionType = 122,
    TSParenthesizedType = 123,
    TSTypeOperator = 124,
    TSArrayType = 125,
    TSIndexedAccessType = 126,
    TSTupleType = 127,
    TSNamedTupleMember = 128,
    TSOptionalType = 129,
    TSRestType = 130,
    TSAnyKeyword = 131,
    TSStringKeyword = 132,
    TSBooleanKeyword = 133,
    TSNumberKeyword = 134,
    TSNeverKeyword = 135,
    TSIntrinsicKeyword = 136,
    TSUnknownKeyword = 137,
    TSNullKeyword = 138,
    TSUndefinedKeyword = 139,
    TSVoidKeyword = 140,
    TSSymbolKeyword = 141,
    TSThisType = 142,
    TSObjectKeyword = 143,
    TSBigIntKeyword = 144,
    TSTypeReference = 145,
    TSQualifiedName = 146,
    TSTypeParameterInstantiation = 147,
    TSTypeParameter = 148,
    TSTypeParameterDeclaration = 149,
    TSTypeAliasDeclaration = 150,
    TSClassImplements = 151,
    TSInterfaceDeclaration = 152,
    TSInterfaceBody = 153,
    TSPropertySignature = 154,
    TSIndexSignature = 155,
    TSCallSignatureDeclaration = 156,
    TSMethodSignature = 157,
    TSConstructSignatureDeclaration = 158,
    TSIndexSignatureName = 159,
    TSInterfaceHeritage = 160,
    TSTypePredicate = 161,
    TSModuleDeclaration = 162,
    TSModuleBlock = 163,
    TSTypeLiteral = 164,
    TSInferType = 165,
    TSTypeQuery = 166,
    TSImportType = 167,
    TSImportTypeQualifiedName = 168,
    TSFunctionType = 169,
    TSConstructorType = 170,
    TSMappedType = 171,
    TSTemplateLiteralType = 172,
    TSAsExpression = 173,
    TSSatisfiesExpression = 174,
    TSTypeAssertion = 175,
    TSImportEqualsDeclaration = 176,
    TSExternalModuleReference = 177,
    TSNonNullExpression = 178,
    Decorator = 179,
    TSExportAssignment = 180,
    TSNamespaceExportDeclaration = 181,
    TSInstantiationExpression = 182,
    JSDocNullableType = 183,
    JSDocNonNullableType = 184,
    JSDocUnknownType = 185,
}

/// Untyped AST Node Kind
#[derive(Debug, Clone, Copy)]
#[repr(C, u8)]
pub enum AstKind<'a> {
    Program(&'a Program<'a>) = AstType::Program as u8,
    IdentifierName(&'a IdentifierName<'a>) = AstType::IdentifierName as u8,
    IdentifierReference(&'a IdentifierReference<'a>) = AstType::IdentifierReference as u8,
    BindingIdentifier(&'a BindingIdentifier<'a>) = AstType::BindingIdentifier as u8,
    LabelIdentifier(&'a LabelIdentifier<'a>) = AstType::LabelIdentifier as u8,
    ThisExpression(&'a ThisExpression) = AstType::ThisExpression as u8,
    ArrayExpression(&'a ArrayExpression<'a>) = AstType::ArrayExpression as u8,
    Elision(&'a Elision) = AstType::Elision as u8,
    ObjectExpression(&'a ObjectExpression<'a>) = AstType::ObjectExpression as u8,
    ObjectProperty(&'a ObjectProperty<'a>) = AstType::ObjectProperty as u8,
    TemplateLiteral(&'a TemplateLiteral<'a>) = AstType::TemplateLiteral as u8,
    TaggedTemplateExpression(&'a TaggedTemplateExpression<'a>) =
        AstType::TaggedTemplateExpression as u8,
    TemplateElement(&'a TemplateElement<'a>) = AstType::TemplateElement as u8,
    ComputedMemberExpression(&'a ComputedMemberExpression<'a>) =
        AstType::ComputedMemberExpression as u8,
    StaticMemberExpression(&'a StaticMemberExpression<'a>) = AstType::StaticMemberExpression as u8,
    PrivateFieldExpression(&'a PrivateFieldExpression<'a>) = AstType::PrivateFieldExpression as u8,
    CallExpression(&'a CallExpression<'a>) = AstType::CallExpression as u8,
    NewExpression(&'a NewExpression<'a>) = AstType::NewExpression as u8,
    MetaProperty(&'a MetaProperty<'a>) = AstType::MetaProperty as u8,
    SpreadElement(&'a SpreadElement<'a>) = AstType::SpreadElement as u8,
    UpdateExpression(&'a UpdateExpression<'a>) = AstType::UpdateExpression as u8,
    UnaryExpression(&'a UnaryExpression<'a>) = AstType::UnaryExpression as u8,
    BinaryExpression(&'a BinaryExpression<'a>) = AstType::BinaryExpression as u8,
    PrivateInExpression(&'a PrivateInExpression<'a>) = AstType::PrivateInExpression as u8,
    LogicalExpression(&'a LogicalExpression<'a>) = AstType::LogicalExpression as u8,
    ConditionalExpression(&'a ConditionalExpression<'a>) = AstType::ConditionalExpression as u8,
    AssignmentExpression(&'a AssignmentExpression<'a>) = AstType::AssignmentExpression as u8,
    ArrayAssignmentTarget(&'a ArrayAssignmentTarget<'a>) = AstType::ArrayAssignmentTarget as u8,
    ObjectAssignmentTarget(&'a ObjectAssignmentTarget<'a>) = AstType::ObjectAssignmentTarget as u8,
    AssignmentTargetRest(&'a AssignmentTargetRest<'a>) = AstType::AssignmentTargetRest as u8,
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>) =
        AstType::AssignmentTargetWithDefault as u8,
    AssignmentTargetPropertyIdentifier(&'a AssignmentTargetPropertyIdentifier<'a>) =
        AstType::AssignmentTargetPropertyIdentifier as u8,
    AssignmentTargetPropertyProperty(&'a AssignmentTargetPropertyProperty<'a>) =
        AstType::AssignmentTargetPropertyProperty as u8,
    SequenceExpression(&'a SequenceExpression<'a>) = AstType::SequenceExpression as u8,
    Super(&'a Super) = AstType::Super as u8,
    AwaitExpression(&'a AwaitExpression<'a>) = AstType::AwaitExpression as u8,
    ChainExpression(&'a ChainExpression<'a>) = AstType::ChainExpression as u8,
    ParenthesizedExpression(&'a ParenthesizedExpression<'a>) =
        AstType::ParenthesizedExpression as u8,
    Directive(&'a Directive<'a>) = AstType::Directive as u8,
    Hashbang(&'a Hashbang<'a>) = AstType::Hashbang as u8,
    BlockStatement(&'a BlockStatement<'a>) = AstType::BlockStatement as u8,
    VariableDeclaration(&'a VariableDeclaration<'a>) = AstType::VariableDeclaration as u8,
    VariableDeclarator(&'a VariableDeclarator<'a>) = AstType::VariableDeclarator as u8,
    EmptyStatement(&'a EmptyStatement) = AstType::EmptyStatement as u8,
    ExpressionStatement(&'a ExpressionStatement<'a>) = AstType::ExpressionStatement as u8,
    IfStatement(&'a IfStatement<'a>) = AstType::IfStatement as u8,
    DoWhileStatement(&'a DoWhileStatement<'a>) = AstType::DoWhileStatement as u8,
    WhileStatement(&'a WhileStatement<'a>) = AstType::WhileStatement as u8,
    ForStatement(&'a ForStatement<'a>) = AstType::ForStatement as u8,
    ForInStatement(&'a ForInStatement<'a>) = AstType::ForInStatement as u8,
    ForOfStatement(&'a ForOfStatement<'a>) = AstType::ForOfStatement as u8,
    ContinueStatement(&'a ContinueStatement<'a>) = AstType::ContinueStatement as u8,
    BreakStatement(&'a BreakStatement<'a>) = AstType::BreakStatement as u8,
    ReturnStatement(&'a ReturnStatement<'a>) = AstType::ReturnStatement as u8,
    WithStatement(&'a WithStatement<'a>) = AstType::WithStatement as u8,
    SwitchStatement(&'a SwitchStatement<'a>) = AstType::SwitchStatement as u8,
    SwitchCase(&'a SwitchCase<'a>) = AstType::SwitchCase as u8,
    LabeledStatement(&'a LabeledStatement<'a>) = AstType::LabeledStatement as u8,
    ThrowStatement(&'a ThrowStatement<'a>) = AstType::ThrowStatement as u8,
    TryStatement(&'a TryStatement<'a>) = AstType::TryStatement as u8,
    CatchClause(&'a CatchClause<'a>) = AstType::CatchClause as u8,
    CatchParameter(&'a CatchParameter<'a>) = AstType::CatchParameter as u8,
    DebuggerStatement(&'a DebuggerStatement) = AstType::DebuggerStatement as u8,
    AssignmentPattern(&'a AssignmentPattern<'a>) = AstType::AssignmentPattern as u8,
    ObjectPattern(&'a ObjectPattern<'a>) = AstType::ObjectPattern as u8,
    BindingProperty(&'a BindingProperty<'a>) = AstType::BindingProperty as u8,
    ArrayPattern(&'a ArrayPattern<'a>) = AstType::ArrayPattern as u8,
    BindingRestElement(&'a BindingRestElement<'a>) = AstType::BindingRestElement as u8,
    Function(&'a Function<'a>) = AstType::Function as u8,
    FormalParameters(&'a FormalParameters<'a>) = AstType::FormalParameters as u8,
    FormalParameter(&'a FormalParameter<'a>) = AstType::FormalParameter as u8,
    FunctionBody(&'a FunctionBody<'a>) = AstType::FunctionBody as u8,
    ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>) =
        AstType::ArrowFunctionExpression as u8,
    YieldExpression(&'a YieldExpression<'a>) = AstType::YieldExpression as u8,
    Class(&'a Class<'a>) = AstType::Class as u8,
    ClassBody(&'a ClassBody<'a>) = AstType::ClassBody as u8,
    MethodDefinition(&'a MethodDefinition<'a>) = AstType::MethodDefinition as u8,
    PropertyDefinition(&'a PropertyDefinition<'a>) = AstType::PropertyDefinition as u8,
    PrivateIdentifier(&'a PrivateIdentifier<'a>) = AstType::PrivateIdentifier as u8,
    StaticBlock(&'a StaticBlock<'a>) = AstType::StaticBlock as u8,
    AccessorProperty(&'a AccessorProperty<'a>) = AstType::AccessorProperty as u8,
    ImportExpression(&'a ImportExpression<'a>) = AstType::ImportExpression as u8,
    ImportDeclaration(&'a ImportDeclaration<'a>) = AstType::ImportDeclaration as u8,
    ImportSpecifier(&'a ImportSpecifier<'a>) = AstType::ImportSpecifier as u8,
    ImportDefaultSpecifier(&'a ImportDefaultSpecifier<'a>) = AstType::ImportDefaultSpecifier as u8,
    ImportNamespaceSpecifier(&'a ImportNamespaceSpecifier<'a>) =
        AstType::ImportNamespaceSpecifier as u8,
    WithClause(&'a WithClause<'a>) = AstType::WithClause as u8,
    ImportAttribute(&'a ImportAttribute<'a>) = AstType::ImportAttribute as u8,
    ExportNamedDeclaration(&'a ExportNamedDeclaration<'a>) = AstType::ExportNamedDeclaration as u8,
    ExportDefaultDeclaration(&'a ExportDefaultDeclaration<'a>) =
        AstType::ExportDefaultDeclaration as u8,
    ExportAllDeclaration(&'a ExportAllDeclaration<'a>) = AstType::ExportAllDeclaration as u8,
    ExportSpecifier(&'a ExportSpecifier<'a>) = AstType::ExportSpecifier as u8,
    V8IntrinsicExpression(&'a V8IntrinsicExpression<'a>) = AstType::V8IntrinsicExpression as u8,
    BooleanLiteral(&'a BooleanLiteral) = AstType::BooleanLiteral as u8,
    NullLiteral(&'a NullLiteral) = AstType::NullLiteral as u8,
    NumericLiteral(&'a NumericLiteral<'a>) = AstType::NumericLiteral as u8,
    StringLiteral(&'a StringLiteral<'a>) = AstType::StringLiteral as u8,
    BigIntLiteral(&'a BigIntLiteral<'a>) = AstType::BigIntLiteral as u8,
    RegExpLiteral(&'a RegExpLiteral<'a>) = AstType::RegExpLiteral as u8,
    JSXElement(&'a JSXElement<'a>) = AstType::JSXElement as u8,
    JSXOpeningElement(&'a JSXOpeningElement<'a>) = AstType::JSXOpeningElement as u8,
    JSXClosingElement(&'a JSXClosingElement<'a>) = AstType::JSXClosingElement as u8,
    JSXFragment(&'a JSXFragment<'a>) = AstType::JSXFragment as u8,
    JSXOpeningFragment(&'a JSXOpeningFragment) = AstType::JSXOpeningFragment as u8,
    JSXClosingFragment(&'a JSXClosingFragment) = AstType::JSXClosingFragment as u8,
    JSXNamespacedName(&'a JSXNamespacedName<'a>) = AstType::JSXNamespacedName as u8,
    JSXMemberExpression(&'a JSXMemberExpression<'a>) = AstType::JSXMemberExpression as u8,
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>) = AstType::JSXExpressionContainer as u8,
    JSXEmptyExpression(&'a JSXEmptyExpression) = AstType::JSXEmptyExpression as u8,
    JSXAttribute(&'a JSXAttribute<'a>) = AstType::JSXAttribute as u8,
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>) = AstType::JSXSpreadAttribute as u8,
    JSXIdentifier(&'a JSXIdentifier<'a>) = AstType::JSXIdentifier as u8,
    JSXSpreadChild(&'a JSXSpreadChild<'a>) = AstType::JSXSpreadChild as u8,
    JSXText(&'a JSXText<'a>) = AstType::JSXText as u8,
    TSThisParameter(&'a TSThisParameter<'a>) = AstType::TSThisParameter as u8,
    TSEnumDeclaration(&'a TSEnumDeclaration<'a>) = AstType::TSEnumDeclaration as u8,
    TSEnumBody(&'a TSEnumBody<'a>) = AstType::TSEnumBody as u8,
    TSEnumMember(&'a TSEnumMember<'a>) = AstType::TSEnumMember as u8,
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>) = AstType::TSTypeAnnotation as u8,
    TSLiteralType(&'a TSLiteralType<'a>) = AstType::TSLiteralType as u8,
    TSConditionalType(&'a TSConditionalType<'a>) = AstType::TSConditionalType as u8,
    TSUnionType(&'a TSUnionType<'a>) = AstType::TSUnionType as u8,
    TSIntersectionType(&'a TSIntersectionType<'a>) = AstType::TSIntersectionType as u8,
    TSParenthesizedType(&'a TSParenthesizedType<'a>) = AstType::TSParenthesizedType as u8,
    TSTypeOperator(&'a TSTypeOperator<'a>) = AstType::TSTypeOperator as u8,
    TSArrayType(&'a TSArrayType<'a>) = AstType::TSArrayType as u8,
    TSIndexedAccessType(&'a TSIndexedAccessType<'a>) = AstType::TSIndexedAccessType as u8,
    TSTupleType(&'a TSTupleType<'a>) = AstType::TSTupleType as u8,
    TSNamedTupleMember(&'a TSNamedTupleMember<'a>) = AstType::TSNamedTupleMember as u8,
    TSOptionalType(&'a TSOptionalType<'a>) = AstType::TSOptionalType as u8,
    TSRestType(&'a TSRestType<'a>) = AstType::TSRestType as u8,
    TSAnyKeyword(&'a TSAnyKeyword) = AstType::TSAnyKeyword as u8,
    TSStringKeyword(&'a TSStringKeyword) = AstType::TSStringKeyword as u8,
    TSBooleanKeyword(&'a TSBooleanKeyword) = AstType::TSBooleanKeyword as u8,
    TSNumberKeyword(&'a TSNumberKeyword) = AstType::TSNumberKeyword as u8,
    TSNeverKeyword(&'a TSNeverKeyword) = AstType::TSNeverKeyword as u8,
    TSIntrinsicKeyword(&'a TSIntrinsicKeyword) = AstType::TSIntrinsicKeyword as u8,
    TSUnknownKeyword(&'a TSUnknownKeyword) = AstType::TSUnknownKeyword as u8,
    TSNullKeyword(&'a TSNullKeyword) = AstType::TSNullKeyword as u8,
    TSUndefinedKeyword(&'a TSUndefinedKeyword) = AstType::TSUndefinedKeyword as u8,
    TSVoidKeyword(&'a TSVoidKeyword) = AstType::TSVoidKeyword as u8,
    TSSymbolKeyword(&'a TSSymbolKeyword) = AstType::TSSymbolKeyword as u8,
    TSThisType(&'a TSThisType) = AstType::TSThisType as u8,
    TSObjectKeyword(&'a TSObjectKeyword) = AstType::TSObjectKeyword as u8,
    TSBigIntKeyword(&'a TSBigIntKeyword) = AstType::TSBigIntKeyword as u8,
    TSTypeReference(&'a TSTypeReference<'a>) = AstType::TSTypeReference as u8,
    TSQualifiedName(&'a TSQualifiedName<'a>) = AstType::TSQualifiedName as u8,
    TSTypeParameterInstantiation(&'a TSTypeParameterInstantiation<'a>) =
        AstType::TSTypeParameterInstantiation as u8,
    TSTypeParameter(&'a TSTypeParameter<'a>) = AstType::TSTypeParameter as u8,
    TSTypeParameterDeclaration(&'a TSTypeParameterDeclaration<'a>) =
        AstType::TSTypeParameterDeclaration as u8,
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>) = AstType::TSTypeAliasDeclaration as u8,
    TSClassImplements(&'a TSClassImplements<'a>) = AstType::TSClassImplements as u8,
    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>) = AstType::TSInterfaceDeclaration as u8,
    TSInterfaceBody(&'a TSInterfaceBody<'a>) = AstType::TSInterfaceBody as u8,
    TSPropertySignature(&'a TSPropertySignature<'a>) = AstType::TSPropertySignature as u8,
    TSIndexSignature(&'a TSIndexSignature<'a>) = AstType::TSIndexSignature as u8,
    TSCallSignatureDeclaration(&'a TSCallSignatureDeclaration<'a>) =
        AstType::TSCallSignatureDeclaration as u8,
    TSMethodSignature(&'a TSMethodSignature<'a>) = AstType::TSMethodSignature as u8,
    TSConstructSignatureDeclaration(&'a TSConstructSignatureDeclaration<'a>) =
        AstType::TSConstructSignatureDeclaration as u8,
    TSIndexSignatureName(&'a TSIndexSignatureName<'a>) = AstType::TSIndexSignatureName as u8,
    TSInterfaceHeritage(&'a TSInterfaceHeritage<'a>) = AstType::TSInterfaceHeritage as u8,
    TSTypePredicate(&'a TSTypePredicate<'a>) = AstType::TSTypePredicate as u8,
    TSModuleDeclaration(&'a TSModuleDeclaration<'a>) = AstType::TSModuleDeclaration as u8,
    TSModuleBlock(&'a TSModuleBlock<'a>) = AstType::TSModuleBlock as u8,
    TSTypeLiteral(&'a TSTypeLiteral<'a>) = AstType::TSTypeLiteral as u8,
    TSInferType(&'a TSInferType<'a>) = AstType::TSInferType as u8,
    TSTypeQuery(&'a TSTypeQuery<'a>) = AstType::TSTypeQuery as u8,
    TSImportType(&'a TSImportType<'a>) = AstType::TSImportType as u8,
    TSImportTypeQualifiedName(&'a TSImportTypeQualifiedName<'a>) =
        AstType::TSImportTypeQualifiedName as u8,
    TSFunctionType(&'a TSFunctionType<'a>) = AstType::TSFunctionType as u8,
    TSConstructorType(&'a TSConstructorType<'a>) = AstType::TSConstructorType as u8,
    TSMappedType(&'a TSMappedType<'a>) = AstType::TSMappedType as u8,
    TSTemplateLiteralType(&'a TSTemplateLiteralType<'a>) = AstType::TSTemplateLiteralType as u8,
    TSAsExpression(&'a TSAsExpression<'a>) = AstType::TSAsExpression as u8,
    TSSatisfiesExpression(&'a TSSatisfiesExpression<'a>) = AstType::TSSatisfiesExpression as u8,
    TSTypeAssertion(&'a TSTypeAssertion<'a>) = AstType::TSTypeAssertion as u8,
    TSImportEqualsDeclaration(&'a TSImportEqualsDeclaration<'a>) =
        AstType::TSImportEqualsDeclaration as u8,
    TSExternalModuleReference(&'a TSExternalModuleReference<'a>) =
        AstType::TSExternalModuleReference as u8,
    TSNonNullExpression(&'a TSNonNullExpression<'a>) = AstType::TSNonNullExpression as u8,
    Decorator(&'a Decorator<'a>) = AstType::Decorator as u8,
    TSExportAssignment(&'a TSExportAssignment<'a>) = AstType::TSExportAssignment as u8,
    TSNamespaceExportDeclaration(&'a TSNamespaceExportDeclaration<'a>) =
        AstType::TSNamespaceExportDeclaration as u8,
    TSInstantiationExpression(&'a TSInstantiationExpression<'a>) =
        AstType::TSInstantiationExpression as u8,
    JSDocNullableType(&'a JSDocNullableType<'a>) = AstType::JSDocNullableType as u8,
    JSDocNonNullableType(&'a JSDocNonNullableType<'a>) = AstType::JSDocNonNullableType as u8,
    JSDocUnknownType(&'a JSDocUnknownType) = AstType::JSDocUnknownType as u8,
}

impl AstKind<'_> {
    /// Get the [`AstType`] of an [`AstKind`].
    #[inline]
    pub fn ty(&self) -> AstType {
        // SAFETY: `AstKind` is `#[repr(C, u8)]`, so discriminant is stored in first byte,
        // and it's valid to read it.
        // `AstType` is also `#[repr(u8)]` and `AstKind` and `AstType` both have the same
        // discriminants, so it's valid to read `AstKind`'s discriminant as `AstType`.
        unsafe { *ptr::from_ref(self).cast::<AstType>().as_ref().unwrap_unchecked() }
    }
}

impl GetSpan for AstKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Program(it) => it.span(),
            Self::IdentifierName(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::BindingIdentifier(it) => it.span(),
            Self::LabelIdentifier(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::Elision(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ObjectProperty(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::TemplateElement(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::SpreadElement(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
            Self::AssignmentTargetRest(it) => it.span(),
            Self::AssignmentTargetWithDefault(it) => it.span(),
            Self::AssignmentTargetPropertyIdentifier(it) => it.span(),
            Self::AssignmentTargetPropertyProperty(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::Directive(it) => it.span(),
            Self::Hashbang(it) => it.span(),
            Self::BlockStatement(it) => it.span(),
            Self::VariableDeclaration(it) => it.span(),
            Self::VariableDeclarator(it) => it.span(),
            Self::EmptyStatement(it) => it.span(),
            Self::ExpressionStatement(it) => it.span(),
            Self::IfStatement(it) => it.span(),
            Self::DoWhileStatement(it) => it.span(),
            Self::WhileStatement(it) => it.span(),
            Self::ForStatement(it) => it.span(),
            Self::ForInStatement(it) => it.span(),
            Self::ForOfStatement(it) => it.span(),
            Self::ContinueStatement(it) => it.span(),
            Self::BreakStatement(it) => it.span(),
            Self::ReturnStatement(it) => it.span(),
            Self::WithStatement(it) => it.span(),
            Self::SwitchStatement(it) => it.span(),
            Self::SwitchCase(it) => it.span(),
            Self::LabeledStatement(it) => it.span(),
            Self::ThrowStatement(it) => it.span(),
            Self::TryStatement(it) => it.span(),
            Self::CatchClause(it) => it.span(),
            Self::CatchParameter(it) => it.span(),
            Self::DebuggerStatement(it) => it.span(),
            Self::AssignmentPattern(it) => it.span(),
            Self::ObjectPattern(it) => it.span(),
            Self::BindingProperty(it) => it.span(),
            Self::ArrayPattern(it) => it.span(),
            Self::BindingRestElement(it) => it.span(),
            Self::Function(it) => it.span(),
            Self::FormalParameters(it) => it.span(),
            Self::FormalParameter(it) => it.span(),
            Self::FunctionBody(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::Class(it) => it.span(),
            Self::ClassBody(it) => it.span(),
            Self::MethodDefinition(it) => it.span(),
            Self::PropertyDefinition(it) => it.span(),
            Self::PrivateIdentifier(it) => it.span(),
            Self::StaticBlock(it) => it.span(),
            Self::AccessorProperty(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::ImportDeclaration(it) => it.span(),
            Self::ImportSpecifier(it) => it.span(),
            Self::ImportDefaultSpecifier(it) => it.span(),
            Self::ImportNamespaceSpecifier(it) => it.span(),
            Self::WithClause(it) => it.span(),
            Self::ImportAttribute(it) => it.span(),
            Self::ExportNamedDeclaration(it) => it.span(),
            Self::ExportDefaultDeclaration(it) => it.span(),
            Self::ExportAllDeclaration(it) => it.span(),
            Self::ExportSpecifier(it) => it.span(),
            Self::V8IntrinsicExpression(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXOpeningElement(it) => it.span(),
            Self::JSXClosingElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::JSXOpeningFragment(it) => it.span(),
            Self::JSXClosingFragment(it) => it.span(),
            Self::JSXNamespacedName(it) => it.span(),
            Self::JSXMemberExpression(it) => it.span(),
            Self::JSXExpressionContainer(it) => it.span(),
            Self::JSXEmptyExpression(it) => it.span(),
            Self::JSXAttribute(it) => it.span(),
            Self::JSXSpreadAttribute(it) => it.span(),
            Self::JSXIdentifier(it) => it.span(),
            Self::JSXSpreadChild(it) => it.span(),
            Self::JSXText(it) => it.span(),
            Self::TSThisParameter(it) => it.span(),
            Self::TSEnumDeclaration(it) => it.span(),
            Self::TSEnumBody(it) => it.span(),
            Self::TSEnumMember(it) => it.span(),
            Self::TSTypeAnnotation(it) => it.span(),
            Self::TSLiteralType(it) => it.span(),
            Self::TSConditionalType(it) => it.span(),
            Self::TSUnionType(it) => it.span(),
            Self::TSIntersectionType(it) => it.span(),
            Self::TSParenthesizedType(it) => it.span(),
            Self::TSTypeOperator(it) => it.span(),
            Self::TSArrayType(it) => it.span(),
            Self::TSIndexedAccessType(it) => it.span(),
            Self::TSTupleType(it) => it.span(),
            Self::TSNamedTupleMember(it) => it.span(),
            Self::TSOptionalType(it) => it.span(),
            Self::TSRestType(it) => it.span(),
            Self::TSAnyKeyword(it) => it.span(),
            Self::TSStringKeyword(it) => it.span(),
            Self::TSBooleanKeyword(it) => it.span(),
            Self::TSNumberKeyword(it) => it.span(),
            Self::TSNeverKeyword(it) => it.span(),
            Self::TSIntrinsicKeyword(it) => it.span(),
            Self::TSUnknownKeyword(it) => it.span(),
            Self::TSNullKeyword(it) => it.span(),
            Self::TSUndefinedKeyword(it) => it.span(),
            Self::TSVoidKeyword(it) => it.span(),
            Self::TSSymbolKeyword(it) => it.span(),
            Self::TSThisType(it) => it.span(),
            Self::TSObjectKeyword(it) => it.span(),
            Self::TSBigIntKeyword(it) => it.span(),
            Self::TSTypeReference(it) => it.span(),
            Self::TSQualifiedName(it) => it.span(),
            Self::TSTypeParameterInstantiation(it) => it.span(),
            Self::TSTypeParameter(it) => it.span(),
            Self::TSTypeParameterDeclaration(it) => it.span(),
            Self::TSTypeAliasDeclaration(it) => it.span(),
            Self::TSClassImplements(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::TSInterfaceBody(it) => it.span(),
            Self::TSPropertySignature(it) => it.span(),
            Self::TSIndexSignature(it) => it.span(),
            Self::TSCallSignatureDeclaration(it) => it.span(),
            Self::TSMethodSignature(it) => it.span(),
            Self::TSConstructSignatureDeclaration(it) => it.span(),
            Self::TSIndexSignatureName(it) => it.span(),
            Self::TSInterfaceHeritage(it) => it.span(),
            Self::TSTypePredicate(it) => it.span(),
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSModuleBlock(it) => it.span(),
            Self::TSTypeLiteral(it) => it.span(),
            Self::TSInferType(it) => it.span(),
            Self::TSTypeQuery(it) => it.span(),
            Self::TSImportType(it) => it.span(),
            Self::TSImportTypeQualifiedName(it) => it.span(),
            Self::TSFunctionType(it) => it.span(),
            Self::TSConstructorType(it) => it.span(),
            Self::TSMappedType(it) => it.span(),
            Self::TSTemplateLiteralType(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSImportEqualsDeclaration(it) => it.span(),
            Self::TSExternalModuleReference(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::Decorator(it) => it.span(),
            Self::TSExportAssignment(it) => it.span(),
            Self::TSNamespaceExportDeclaration(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::JSDocNullableType(it) => it.span(),
            Self::JSDocNonNullableType(it) => it.span(),
            Self::JSDocUnknownType(it) => it.span(),
        }
    }
}

impl GetAddress for AstKind<'_> {
    fn address(&self) -> Address {
        match *self {
            Self::Program(it) => Address::from_ptr(it),
            Self::IdentifierName(it) => Address::from_ptr(it),
            Self::IdentifierReference(it) => Address::from_ptr(it),
            Self::BindingIdentifier(it) => Address::from_ptr(it),
            Self::LabelIdentifier(it) => Address::from_ptr(it),
            Self::ThisExpression(it) => Address::from_ptr(it),
            Self::ArrayExpression(it) => Address::from_ptr(it),
            Self::Elision(it) => Address::from_ptr(it),
            Self::ObjectExpression(it) => Address::from_ptr(it),
            Self::ObjectProperty(it) => Address::from_ptr(it),
            Self::TemplateLiteral(it) => Address::from_ptr(it),
            Self::TaggedTemplateExpression(it) => Address::from_ptr(it),
            Self::TemplateElement(it) => Address::from_ptr(it),
            Self::ComputedMemberExpression(it) => Address::from_ptr(it),
            Self::StaticMemberExpression(it) => Address::from_ptr(it),
            Self::PrivateFieldExpression(it) => Address::from_ptr(it),
            Self::CallExpression(it) => Address::from_ptr(it),
            Self::NewExpression(it) => Address::from_ptr(it),
            Self::MetaProperty(it) => Address::from_ptr(it),
            Self::SpreadElement(it) => Address::from_ptr(it),
            Self::UpdateExpression(it) => Address::from_ptr(it),
            Self::UnaryExpression(it) => Address::from_ptr(it),
            Self::BinaryExpression(it) => Address::from_ptr(it),
            Self::PrivateInExpression(it) => Address::from_ptr(it),
            Self::LogicalExpression(it) => Address::from_ptr(it),
            Self::ConditionalExpression(it) => Address::from_ptr(it),
            Self::AssignmentExpression(it) => Address::from_ptr(it),
            Self::ArrayAssignmentTarget(it) => Address::from_ptr(it),
            Self::ObjectAssignmentTarget(it) => Address::from_ptr(it),
            Self::AssignmentTargetRest(it) => Address::from_ptr(it),
            Self::AssignmentTargetWithDefault(it) => Address::from_ptr(it),
            Self::AssignmentTargetPropertyIdentifier(it) => Address::from_ptr(it),
            Self::AssignmentTargetPropertyProperty(it) => Address::from_ptr(it),
            Self::SequenceExpression(it) => Address::from_ptr(it),
            Self::Super(it) => Address::from_ptr(it),
            Self::AwaitExpression(it) => Address::from_ptr(it),
            Self::ChainExpression(it) => Address::from_ptr(it),
            Self::ParenthesizedExpression(it) => Address::from_ptr(it),
            Self::Directive(it) => Address::from_ptr(it),
            Self::Hashbang(it) => Address::from_ptr(it),
            Self::BlockStatement(it) => Address::from_ptr(it),
            Self::VariableDeclaration(it) => Address::from_ptr(it),
            Self::VariableDeclarator(it) => Address::from_ptr(it),
            Self::EmptyStatement(it) => Address::from_ptr(it),
            Self::ExpressionStatement(it) => Address::from_ptr(it),
            Self::IfStatement(it) => Address::from_ptr(it),
            Self::DoWhileStatement(it) => Address::from_ptr(it),
            Self::WhileStatement(it) => Address::from_ptr(it),
            Self::ForStatement(it) => Address::from_ptr(it),
            Self::ForInStatement(it) => Address::from_ptr(it),
            Self::ForOfStatement(it) => Address::from_ptr(it),
            Self::ContinueStatement(it) => Address::from_ptr(it),
            Self::BreakStatement(it) => Address::from_ptr(it),
            Self::ReturnStatement(it) => Address::from_ptr(it),
            Self::WithStatement(it) => Address::from_ptr(it),
            Self::SwitchStatement(it) => Address::from_ptr(it),
            Self::SwitchCase(it) => Address::from_ptr(it),
            Self::LabeledStatement(it) => Address::from_ptr(it),
            Self::ThrowStatement(it) => Address::from_ptr(it),
            Self::TryStatement(it) => Address::from_ptr(it),
            Self::CatchClause(it) => Address::from_ptr(it),
            Self::CatchParameter(it) => Address::from_ptr(it),
            Self::DebuggerStatement(it) => Address::from_ptr(it),
            Self::AssignmentPattern(it) => Address::from_ptr(it),
            Self::ObjectPattern(it) => Address::from_ptr(it),
            Self::BindingProperty(it) => Address::from_ptr(it),
            Self::ArrayPattern(it) => Address::from_ptr(it),
            Self::BindingRestElement(it) => Address::from_ptr(it),
            Self::Function(it) => Address::from_ptr(it),
            Self::FormalParameters(it) => Address::from_ptr(it),
            Self::FormalParameter(it) => Address::from_ptr(it),
            Self::FunctionBody(it) => Address::from_ptr(it),
            Self::ArrowFunctionExpression(it) => Address::from_ptr(it),
            Self::YieldExpression(it) => Address::from_ptr(it),
            Self::Class(it) => Address::from_ptr(it),
            Self::ClassBody(it) => Address::from_ptr(it),
            Self::MethodDefinition(it) => Address::from_ptr(it),
            Self::PropertyDefinition(it) => Address::from_ptr(it),
            Self::PrivateIdentifier(it) => Address::from_ptr(it),
            Self::StaticBlock(it) => Address::from_ptr(it),
            Self::AccessorProperty(it) => Address::from_ptr(it),
            Self::ImportExpression(it) => Address::from_ptr(it),
            Self::ImportDeclaration(it) => Address::from_ptr(it),
            Self::ImportSpecifier(it) => Address::from_ptr(it),
            Self::ImportDefaultSpecifier(it) => Address::from_ptr(it),
            Self::ImportNamespaceSpecifier(it) => Address::from_ptr(it),
            Self::WithClause(it) => Address::from_ptr(it),
            Self::ImportAttribute(it) => Address::from_ptr(it),
            Self::ExportNamedDeclaration(it) => Address::from_ptr(it),
            Self::ExportDefaultDeclaration(it) => Address::from_ptr(it),
            Self::ExportAllDeclaration(it) => Address::from_ptr(it),
            Self::ExportSpecifier(it) => Address::from_ptr(it),
            Self::V8IntrinsicExpression(it) => Address::from_ptr(it),
            Self::BooleanLiteral(it) => Address::from_ptr(it),
            Self::NullLiteral(it) => Address::from_ptr(it),
            Self::NumericLiteral(it) => Address::from_ptr(it),
            Self::StringLiteral(it) => Address::from_ptr(it),
            Self::BigIntLiteral(it) => Address::from_ptr(it),
            Self::RegExpLiteral(it) => Address::from_ptr(it),
            Self::JSXElement(it) => Address::from_ptr(it),
            Self::JSXOpeningElement(it) => Address::from_ptr(it),
            Self::JSXClosingElement(it) => Address::from_ptr(it),
            Self::JSXFragment(it) => Address::from_ptr(it),
            Self::JSXOpeningFragment(it) => Address::from_ptr(it),
            Self::JSXClosingFragment(it) => Address::from_ptr(it),
            Self::JSXNamespacedName(it) => Address::from_ptr(it),
            Self::JSXMemberExpression(it) => Address::from_ptr(it),
            Self::JSXExpressionContainer(it) => Address::from_ptr(it),
            Self::JSXEmptyExpression(it) => Address::from_ptr(it),
            Self::JSXAttribute(it) => Address::from_ptr(it),
            Self::JSXSpreadAttribute(it) => Address::from_ptr(it),
            Self::JSXIdentifier(it) => Address::from_ptr(it),
            Self::JSXSpreadChild(it) => Address::from_ptr(it),
            Self::JSXText(it) => Address::from_ptr(it),
            Self::TSThisParameter(it) => Address::from_ptr(it),
            Self::TSEnumDeclaration(it) => Address::from_ptr(it),
            Self::TSEnumBody(it) => Address::from_ptr(it),
            Self::TSEnumMember(it) => Address::from_ptr(it),
            Self::TSTypeAnnotation(it) => Address::from_ptr(it),
            Self::TSLiteralType(it) => Address::from_ptr(it),
            Self::TSConditionalType(it) => Address::from_ptr(it),
            Self::TSUnionType(it) => Address::from_ptr(it),
            Self::TSIntersectionType(it) => Address::from_ptr(it),
            Self::TSParenthesizedType(it) => Address::from_ptr(it),
            Self::TSTypeOperator(it) => Address::from_ptr(it),
            Self::TSArrayType(it) => Address::from_ptr(it),
            Self::TSIndexedAccessType(it) => Address::from_ptr(it),
            Self::TSTupleType(it) => Address::from_ptr(it),
            Self::TSNamedTupleMember(it) => Address::from_ptr(it),
            Self::TSOptionalType(it) => Address::from_ptr(it),
            Self::TSRestType(it) => Address::from_ptr(it),
            Self::TSAnyKeyword(it) => Address::from_ptr(it),
            Self::TSStringKeyword(it) => Address::from_ptr(it),
            Self::TSBooleanKeyword(it) => Address::from_ptr(it),
            Self::TSNumberKeyword(it) => Address::from_ptr(it),
            Self::TSNeverKeyword(it) => Address::from_ptr(it),
            Self::TSIntrinsicKeyword(it) => Address::from_ptr(it),
            Self::TSUnknownKeyword(it) => Address::from_ptr(it),
            Self::TSNullKeyword(it) => Address::from_ptr(it),
            Self::TSUndefinedKeyword(it) => Address::from_ptr(it),
            Self::TSVoidKeyword(it) => Address::from_ptr(it),
            Self::TSSymbolKeyword(it) => Address::from_ptr(it),
            Self::TSThisType(it) => Address::from_ptr(it),
            Self::TSObjectKeyword(it) => Address::from_ptr(it),
            Self::TSBigIntKeyword(it) => Address::from_ptr(it),
            Self::TSTypeReference(it) => Address::from_ptr(it),
            Self::TSQualifiedName(it) => Address::from_ptr(it),
            Self::TSTypeParameterInstantiation(it) => Address::from_ptr(it),
            Self::TSTypeParameter(it) => Address::from_ptr(it),
            Self::TSTypeParameterDeclaration(it) => Address::from_ptr(it),
            Self::TSTypeAliasDeclaration(it) => Address::from_ptr(it),
            Self::TSClassImplements(it) => Address::from_ptr(it),
            Self::TSInterfaceDeclaration(it) => Address::from_ptr(it),
            Self::TSInterfaceBody(it) => Address::from_ptr(it),
            Self::TSPropertySignature(it) => Address::from_ptr(it),
            Self::TSIndexSignature(it) => Address::from_ptr(it),
            Self::TSCallSignatureDeclaration(it) => Address::from_ptr(it),
            Self::TSMethodSignature(it) => Address::from_ptr(it),
            Self::TSConstructSignatureDeclaration(it) => Address::from_ptr(it),
            Self::TSIndexSignatureName(it) => Address::from_ptr(it),
            Self::TSInterfaceHeritage(it) => Address::from_ptr(it),
            Self::TSTypePredicate(it) => Address::from_ptr(it),
            Self::TSModuleDeclaration(it) => Address::from_ptr(it),
            Self::TSModuleBlock(it) => Address::from_ptr(it),
            Self::TSTypeLiteral(it) => Address::from_ptr(it),
            Self::TSInferType(it) => Address::from_ptr(it),
            Self::TSTypeQuery(it) => Address::from_ptr(it),
            Self::TSImportType(it) => Address::from_ptr(it),
            Self::TSImportTypeQualifiedName(it) => Address::from_ptr(it),
            Self::TSFunctionType(it) => Address::from_ptr(it),
            Self::TSConstructorType(it) => Address::from_ptr(it),
            Self::TSMappedType(it) => Address::from_ptr(it),
            Self::TSTemplateLiteralType(it) => Address::from_ptr(it),
            Self::TSAsExpression(it) => Address::from_ptr(it),
            Self::TSSatisfiesExpression(it) => Address::from_ptr(it),
            Self::TSTypeAssertion(it) => Address::from_ptr(it),
            Self::TSImportEqualsDeclaration(it) => Address::from_ptr(it),
            Self::TSExternalModuleReference(it) => Address::from_ptr(it),
            Self::TSNonNullExpression(it) => Address::from_ptr(it),
            Self::Decorator(it) => Address::from_ptr(it),
            Self::TSExportAssignment(it) => Address::from_ptr(it),
            Self::TSNamespaceExportDeclaration(it) => Address::from_ptr(it),
            Self::TSInstantiationExpression(it) => Address::from_ptr(it),
            Self::JSDocNullableType(it) => Address::from_ptr(it),
            Self::JSDocNonNullableType(it) => Address::from_ptr(it),
            Self::JSDocUnknownType(it) => Address::from_ptr(it),
        }
    }
}

impl<'a> AstKind<'a> {
    #[inline]
    pub fn as_program(self) -> Option<&'a Program<'a>> {
        if let Self::Program(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_identifier_name(self) -> Option<&'a IdentifierName<'a>> {
        if let Self::IdentifierName(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_identifier_reference(self) -> Option<&'a IdentifierReference<'a>> {
        if let Self::IdentifierReference(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_binding_identifier(self) -> Option<&'a BindingIdentifier<'a>> {
        if let Self::BindingIdentifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_label_identifier(self) -> Option<&'a LabelIdentifier<'a>> {
        if let Self::LabelIdentifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_this_expression(self) -> Option<&'a ThisExpression> {
        if let Self::ThisExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_array_expression(self) -> Option<&'a ArrayExpression<'a>> {
        if let Self::ArrayExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_elision(self) -> Option<&'a Elision> {
        if let Self::Elision(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_object_expression(self) -> Option<&'a ObjectExpression<'a>> {
        if let Self::ObjectExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_object_property(self) -> Option<&'a ObjectProperty<'a>> {
        if let Self::ObjectProperty(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_template_literal(self) -> Option<&'a TemplateLiteral<'a>> {
        if let Self::TemplateLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_tagged_template_expression(self) -> Option<&'a TaggedTemplateExpression<'a>> {
        if let Self::TaggedTemplateExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_template_element(self) -> Option<&'a TemplateElement<'a>> {
        if let Self::TemplateElement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_computed_member_expression(self) -> Option<&'a ComputedMemberExpression<'a>> {
        if let Self::ComputedMemberExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_static_member_expression(self) -> Option<&'a StaticMemberExpression<'a>> {
        if let Self::StaticMemberExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_private_field_expression(self) -> Option<&'a PrivateFieldExpression<'a>> {
        if let Self::PrivateFieldExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_call_expression(self) -> Option<&'a CallExpression<'a>> {
        if let Self::CallExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_new_expression(self) -> Option<&'a NewExpression<'a>> {
        if let Self::NewExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_meta_property(self) -> Option<&'a MetaProperty<'a>> {
        if let Self::MetaProperty(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_spread_element(self) -> Option<&'a SpreadElement<'a>> {
        if let Self::SpreadElement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_update_expression(self) -> Option<&'a UpdateExpression<'a>> {
        if let Self::UpdateExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_unary_expression(self) -> Option<&'a UnaryExpression<'a>> {
        if let Self::UnaryExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_binary_expression(self) -> Option<&'a BinaryExpression<'a>> {
        if let Self::BinaryExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_private_in_expression(self) -> Option<&'a PrivateInExpression<'a>> {
        if let Self::PrivateInExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_logical_expression(self) -> Option<&'a LogicalExpression<'a>> {
        if let Self::LogicalExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_conditional_expression(self) -> Option<&'a ConditionalExpression<'a>> {
        if let Self::ConditionalExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_assignment_expression(self) -> Option<&'a AssignmentExpression<'a>> {
        if let Self::AssignmentExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_array_assignment_target(self) -> Option<&'a ArrayAssignmentTarget<'a>> {
        if let Self::ArrayAssignmentTarget(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_object_assignment_target(self) -> Option<&'a ObjectAssignmentTarget<'a>> {
        if let Self::ObjectAssignmentTarget(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_assignment_target_rest(self) -> Option<&'a AssignmentTargetRest<'a>> {
        if let Self::AssignmentTargetRest(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_assignment_target_with_default(self) -> Option<&'a AssignmentTargetWithDefault<'a>> {
        if let Self::AssignmentTargetWithDefault(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_assignment_target_property_identifier(
        self,
    ) -> Option<&'a AssignmentTargetPropertyIdentifier<'a>> {
        if let Self::AssignmentTargetPropertyIdentifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_assignment_target_property_property(
        self,
    ) -> Option<&'a AssignmentTargetPropertyProperty<'a>> {
        if let Self::AssignmentTargetPropertyProperty(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_sequence_expression(self) -> Option<&'a SequenceExpression<'a>> {
        if let Self::SequenceExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_super(self) -> Option<&'a Super> {
        if let Self::Super(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_await_expression(self) -> Option<&'a AwaitExpression<'a>> {
        if let Self::AwaitExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_chain_expression(self) -> Option<&'a ChainExpression<'a>> {
        if let Self::ChainExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_parenthesized_expression(self) -> Option<&'a ParenthesizedExpression<'a>> {
        if let Self::ParenthesizedExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_directive(self) -> Option<&'a Directive<'a>> {
        if let Self::Directive(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_hashbang(self) -> Option<&'a Hashbang<'a>> {
        if let Self::Hashbang(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_block_statement(self) -> Option<&'a BlockStatement<'a>> {
        if let Self::BlockStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_variable_declaration(self) -> Option<&'a VariableDeclaration<'a>> {
        if let Self::VariableDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_variable_declarator(self) -> Option<&'a VariableDeclarator<'a>> {
        if let Self::VariableDeclarator(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_empty_statement(self) -> Option<&'a EmptyStatement> {
        if let Self::EmptyStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_expression_statement(self) -> Option<&'a ExpressionStatement<'a>> {
        if let Self::ExpressionStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_if_statement(self) -> Option<&'a IfStatement<'a>> {
        if let Self::IfStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_do_while_statement(self) -> Option<&'a DoWhileStatement<'a>> {
        if let Self::DoWhileStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_while_statement(self) -> Option<&'a WhileStatement<'a>> {
        if let Self::WhileStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_for_statement(self) -> Option<&'a ForStatement<'a>> {
        if let Self::ForStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_for_in_statement(self) -> Option<&'a ForInStatement<'a>> {
        if let Self::ForInStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_for_of_statement(self) -> Option<&'a ForOfStatement<'a>> {
        if let Self::ForOfStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_continue_statement(self) -> Option<&'a ContinueStatement<'a>> {
        if let Self::ContinueStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_break_statement(self) -> Option<&'a BreakStatement<'a>> {
        if let Self::BreakStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_return_statement(self) -> Option<&'a ReturnStatement<'a>> {
        if let Self::ReturnStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_with_statement(self) -> Option<&'a WithStatement<'a>> {
        if let Self::WithStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_switch_statement(self) -> Option<&'a SwitchStatement<'a>> {
        if let Self::SwitchStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_switch_case(self) -> Option<&'a SwitchCase<'a>> {
        if let Self::SwitchCase(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_labeled_statement(self) -> Option<&'a LabeledStatement<'a>> {
        if let Self::LabeledStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_throw_statement(self) -> Option<&'a ThrowStatement<'a>> {
        if let Self::ThrowStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_try_statement(self) -> Option<&'a TryStatement<'a>> {
        if let Self::TryStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_catch_clause(self) -> Option<&'a CatchClause<'a>> {
        if let Self::CatchClause(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_catch_parameter(self) -> Option<&'a CatchParameter<'a>> {
        if let Self::CatchParameter(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_debugger_statement(self) -> Option<&'a DebuggerStatement> {
        if let Self::DebuggerStatement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_assignment_pattern(self) -> Option<&'a AssignmentPattern<'a>> {
        if let Self::AssignmentPattern(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_object_pattern(self) -> Option<&'a ObjectPattern<'a>> {
        if let Self::ObjectPattern(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_binding_property(self) -> Option<&'a BindingProperty<'a>> {
        if let Self::BindingProperty(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_array_pattern(self) -> Option<&'a ArrayPattern<'a>> {
        if let Self::ArrayPattern(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_binding_rest_element(self) -> Option<&'a BindingRestElement<'a>> {
        if let Self::BindingRestElement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_function(self) -> Option<&'a Function<'a>> {
        if let Self::Function(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_formal_parameters(self) -> Option<&'a FormalParameters<'a>> {
        if let Self::FormalParameters(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_formal_parameter(self) -> Option<&'a FormalParameter<'a>> {
        if let Self::FormalParameter(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_function_body(self) -> Option<&'a FunctionBody<'a>> {
        if let Self::FunctionBody(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_arrow_function_expression(self) -> Option<&'a ArrowFunctionExpression<'a>> {
        if let Self::ArrowFunctionExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_yield_expression(self) -> Option<&'a YieldExpression<'a>> {
        if let Self::YieldExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_class(self) -> Option<&'a Class<'a>> {
        if let Self::Class(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_class_body(self) -> Option<&'a ClassBody<'a>> {
        if let Self::ClassBody(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_method_definition(self) -> Option<&'a MethodDefinition<'a>> {
        if let Self::MethodDefinition(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_property_definition(self) -> Option<&'a PropertyDefinition<'a>> {
        if let Self::PropertyDefinition(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_private_identifier(self) -> Option<&'a PrivateIdentifier<'a>> {
        if let Self::PrivateIdentifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_static_block(self) -> Option<&'a StaticBlock<'a>> {
        if let Self::StaticBlock(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_accessor_property(self) -> Option<&'a AccessorProperty<'a>> {
        if let Self::AccessorProperty(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_import_expression(self) -> Option<&'a ImportExpression<'a>> {
        if let Self::ImportExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_import_declaration(self) -> Option<&'a ImportDeclaration<'a>> {
        if let Self::ImportDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_import_specifier(self) -> Option<&'a ImportSpecifier<'a>> {
        if let Self::ImportSpecifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_import_default_specifier(self) -> Option<&'a ImportDefaultSpecifier<'a>> {
        if let Self::ImportDefaultSpecifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_import_namespace_specifier(self) -> Option<&'a ImportNamespaceSpecifier<'a>> {
        if let Self::ImportNamespaceSpecifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_with_clause(self) -> Option<&'a WithClause<'a>> {
        if let Self::WithClause(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_import_attribute(self) -> Option<&'a ImportAttribute<'a>> {
        if let Self::ImportAttribute(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_export_named_declaration(self) -> Option<&'a ExportNamedDeclaration<'a>> {
        if let Self::ExportNamedDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_export_default_declaration(self) -> Option<&'a ExportDefaultDeclaration<'a>> {
        if let Self::ExportDefaultDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_export_all_declaration(self) -> Option<&'a ExportAllDeclaration<'a>> {
        if let Self::ExportAllDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_export_specifier(self) -> Option<&'a ExportSpecifier<'a>> {
        if let Self::ExportSpecifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_v_8_intrinsic_expression(self) -> Option<&'a V8IntrinsicExpression<'a>> {
        if let Self::V8IntrinsicExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_boolean_literal(self) -> Option<&'a BooleanLiteral> {
        if let Self::BooleanLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_null_literal(self) -> Option<&'a NullLiteral> {
        if let Self::NullLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_numeric_literal(self) -> Option<&'a NumericLiteral<'a>> {
        if let Self::NumericLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_string_literal(self) -> Option<&'a StringLiteral<'a>> {
        if let Self::StringLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_big_int_literal(self) -> Option<&'a BigIntLiteral<'a>> {
        if let Self::BigIntLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_reg_exp_literal(self) -> Option<&'a RegExpLiteral<'a>> {
        if let Self::RegExpLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_element(self) -> Option<&'a JSXElement<'a>> {
        if let Self::JSXElement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_opening_element(self) -> Option<&'a JSXOpeningElement<'a>> {
        if let Self::JSXOpeningElement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_closing_element(self) -> Option<&'a JSXClosingElement<'a>> {
        if let Self::JSXClosingElement(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_fragment(self) -> Option<&'a JSXFragment<'a>> {
        if let Self::JSXFragment(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_opening_fragment(self) -> Option<&'a JSXOpeningFragment> {
        if let Self::JSXOpeningFragment(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_closing_fragment(self) -> Option<&'a JSXClosingFragment> {
        if let Self::JSXClosingFragment(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_namespaced_name(self) -> Option<&'a JSXNamespacedName<'a>> {
        if let Self::JSXNamespacedName(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_member_expression(self) -> Option<&'a JSXMemberExpression<'a>> {
        if let Self::JSXMemberExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_expression_container(self) -> Option<&'a JSXExpressionContainer<'a>> {
        if let Self::JSXExpressionContainer(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_empty_expression(self) -> Option<&'a JSXEmptyExpression> {
        if let Self::JSXEmptyExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_attribute(self) -> Option<&'a JSXAttribute<'a>> {
        if let Self::JSXAttribute(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_spread_attribute(self) -> Option<&'a JSXSpreadAttribute<'a>> {
        if let Self::JSXSpreadAttribute(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_identifier(self) -> Option<&'a JSXIdentifier<'a>> {
        if let Self::JSXIdentifier(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_spread_child(self) -> Option<&'a JSXSpreadChild<'a>> {
        if let Self::JSXSpreadChild(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_jsx_text(self) -> Option<&'a JSXText<'a>> {
        if let Self::JSXText(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_this_parameter(self) -> Option<&'a TSThisParameter<'a>> {
        if let Self::TSThisParameter(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_enum_declaration(self) -> Option<&'a TSEnumDeclaration<'a>> {
        if let Self::TSEnumDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_enum_body(self) -> Option<&'a TSEnumBody<'a>> {
        if let Self::TSEnumBody(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_enum_member(self) -> Option<&'a TSEnumMember<'a>> {
        if let Self::TSEnumMember(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_annotation(self) -> Option<&'a TSTypeAnnotation<'a>> {
        if let Self::TSTypeAnnotation(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_literal_type(self) -> Option<&'a TSLiteralType<'a>> {
        if let Self::TSLiteralType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_conditional_type(self) -> Option<&'a TSConditionalType<'a>> {
        if let Self::TSConditionalType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_union_type(self) -> Option<&'a TSUnionType<'a>> {
        if let Self::TSUnionType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_intersection_type(self) -> Option<&'a TSIntersectionType<'a>> {
        if let Self::TSIntersectionType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_parenthesized_type(self) -> Option<&'a TSParenthesizedType<'a>> {
        if let Self::TSParenthesizedType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_operator(self) -> Option<&'a TSTypeOperator<'a>> {
        if let Self::TSTypeOperator(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_array_type(self) -> Option<&'a TSArrayType<'a>> {
        if let Self::TSArrayType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_indexed_access_type(self) -> Option<&'a TSIndexedAccessType<'a>> {
        if let Self::TSIndexedAccessType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_tuple_type(self) -> Option<&'a TSTupleType<'a>> {
        if let Self::TSTupleType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_named_tuple_member(self) -> Option<&'a TSNamedTupleMember<'a>> {
        if let Self::TSNamedTupleMember(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_optional_type(self) -> Option<&'a TSOptionalType<'a>> {
        if let Self::TSOptionalType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_rest_type(self) -> Option<&'a TSRestType<'a>> {
        if let Self::TSRestType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_any_keyword(self) -> Option<&'a TSAnyKeyword> {
        if let Self::TSAnyKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_string_keyword(self) -> Option<&'a TSStringKeyword> {
        if let Self::TSStringKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_boolean_keyword(self) -> Option<&'a TSBooleanKeyword> {
        if let Self::TSBooleanKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_number_keyword(self) -> Option<&'a TSNumberKeyword> {
        if let Self::TSNumberKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_never_keyword(self) -> Option<&'a TSNeverKeyword> {
        if let Self::TSNeverKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_intrinsic_keyword(self) -> Option<&'a TSIntrinsicKeyword> {
        if let Self::TSIntrinsicKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_unknown_keyword(self) -> Option<&'a TSUnknownKeyword> {
        if let Self::TSUnknownKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_null_keyword(self) -> Option<&'a TSNullKeyword> {
        if let Self::TSNullKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_undefined_keyword(self) -> Option<&'a TSUndefinedKeyword> {
        if let Self::TSUndefinedKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_void_keyword(self) -> Option<&'a TSVoidKeyword> {
        if let Self::TSVoidKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_symbol_keyword(self) -> Option<&'a TSSymbolKeyword> {
        if let Self::TSSymbolKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_this_type(self) -> Option<&'a TSThisType> {
        if let Self::TSThisType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_object_keyword(self) -> Option<&'a TSObjectKeyword> {
        if let Self::TSObjectKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_big_int_keyword(self) -> Option<&'a TSBigIntKeyword> {
        if let Self::TSBigIntKeyword(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_reference(self) -> Option<&'a TSTypeReference<'a>> {
        if let Self::TSTypeReference(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_qualified_name(self) -> Option<&'a TSQualifiedName<'a>> {
        if let Self::TSQualifiedName(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_parameter_instantiation(
        self,
    ) -> Option<&'a TSTypeParameterInstantiation<'a>> {
        if let Self::TSTypeParameterInstantiation(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_parameter(self) -> Option<&'a TSTypeParameter<'a>> {
        if let Self::TSTypeParameter(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_parameter_declaration(self) -> Option<&'a TSTypeParameterDeclaration<'a>> {
        if let Self::TSTypeParameterDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_alias_declaration(self) -> Option<&'a TSTypeAliasDeclaration<'a>> {
        if let Self::TSTypeAliasDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_class_implements(self) -> Option<&'a TSClassImplements<'a>> {
        if let Self::TSClassImplements(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_interface_declaration(self) -> Option<&'a TSInterfaceDeclaration<'a>> {
        if let Self::TSInterfaceDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_interface_body(self) -> Option<&'a TSInterfaceBody<'a>> {
        if let Self::TSInterfaceBody(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_property_signature(self) -> Option<&'a TSPropertySignature<'a>> {
        if let Self::TSPropertySignature(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_index_signature(self) -> Option<&'a TSIndexSignature<'a>> {
        if let Self::TSIndexSignature(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_call_signature_declaration(self) -> Option<&'a TSCallSignatureDeclaration<'a>> {
        if let Self::TSCallSignatureDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_method_signature(self) -> Option<&'a TSMethodSignature<'a>> {
        if let Self::TSMethodSignature(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_construct_signature_declaration(
        self,
    ) -> Option<&'a TSConstructSignatureDeclaration<'a>> {
        if let Self::TSConstructSignatureDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_index_signature_name(self) -> Option<&'a TSIndexSignatureName<'a>> {
        if let Self::TSIndexSignatureName(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_interface_heritage(self) -> Option<&'a TSInterfaceHeritage<'a>> {
        if let Self::TSInterfaceHeritage(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_predicate(self) -> Option<&'a TSTypePredicate<'a>> {
        if let Self::TSTypePredicate(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_module_declaration(self) -> Option<&'a TSModuleDeclaration<'a>> {
        if let Self::TSModuleDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_module_block(self) -> Option<&'a TSModuleBlock<'a>> {
        if let Self::TSModuleBlock(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_literal(self) -> Option<&'a TSTypeLiteral<'a>> {
        if let Self::TSTypeLiteral(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_infer_type(self) -> Option<&'a TSInferType<'a>> {
        if let Self::TSInferType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_query(self) -> Option<&'a TSTypeQuery<'a>> {
        if let Self::TSTypeQuery(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_import_type(self) -> Option<&'a TSImportType<'a>> {
        if let Self::TSImportType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_import_type_qualified_name(self) -> Option<&'a TSImportTypeQualifiedName<'a>> {
        if let Self::TSImportTypeQualifiedName(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_function_type(self) -> Option<&'a TSFunctionType<'a>> {
        if let Self::TSFunctionType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_constructor_type(self) -> Option<&'a TSConstructorType<'a>> {
        if let Self::TSConstructorType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_mapped_type(self) -> Option<&'a TSMappedType<'a>> {
        if let Self::TSMappedType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_template_literal_type(self) -> Option<&'a TSTemplateLiteralType<'a>> {
        if let Self::TSTemplateLiteralType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_as_expression(self) -> Option<&'a TSAsExpression<'a>> {
        if let Self::TSAsExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_satisfies_expression(self) -> Option<&'a TSSatisfiesExpression<'a>> {
        if let Self::TSSatisfiesExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_type_assertion(self) -> Option<&'a TSTypeAssertion<'a>> {
        if let Self::TSTypeAssertion(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_import_equals_declaration(self) -> Option<&'a TSImportEqualsDeclaration<'a>> {
        if let Self::TSImportEqualsDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_external_module_reference(self) -> Option<&'a TSExternalModuleReference<'a>> {
        if let Self::TSExternalModuleReference(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_non_null_expression(self) -> Option<&'a TSNonNullExpression<'a>> {
        if let Self::TSNonNullExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_decorator(self) -> Option<&'a Decorator<'a>> {
        if let Self::Decorator(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_export_assignment(self) -> Option<&'a TSExportAssignment<'a>> {
        if let Self::TSExportAssignment(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_namespace_export_declaration(
        self,
    ) -> Option<&'a TSNamespaceExportDeclaration<'a>> {
        if let Self::TSNamespaceExportDeclaration(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_ts_instantiation_expression(self) -> Option<&'a TSInstantiationExpression<'a>> {
        if let Self::TSInstantiationExpression(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_js_doc_nullable_type(self) -> Option<&'a JSDocNullableType<'a>> {
        if let Self::JSDocNullableType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_js_doc_non_nullable_type(self) -> Option<&'a JSDocNonNullableType<'a>> {
        if let Self::JSDocNonNullableType(v) = self { Some(v) } else { None }
    }

    #[inline]
    pub fn as_js_doc_unknown_type(self) -> Option<&'a JSDocUnknownType> {
        if let Self::JSDocUnknownType(v) = self { Some(v) } else { None }
    }
}
