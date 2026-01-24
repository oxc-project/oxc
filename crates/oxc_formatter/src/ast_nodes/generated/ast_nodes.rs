// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/ast_nodes.rs`.

use std::mem::transmute;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Ident};

use crate::ast_nodes::AstNode;
use crate::formatter::{
    Format, Formatter,
    trivia::{format_leading_comments, format_trailing_comments},
};

#[inline]
pub(super) fn transmute_self<'a, T>(s: &AstNode<'a, T>) -> &'a AstNode<'a, T> {
    #[expect(clippy::undocumented_unsafe_blocks)]
    unsafe {
        transmute(s)
    }
}

#[derive(Clone, Copy)]
pub enum AstNodes<'a> {
    Dummy(),
    Program(&'a AstNode<'a, Program<'a>>),
    IdentifierName(&'a AstNode<'a, IdentifierName<'a>>),
    IdentifierReference(&'a AstNode<'a, IdentifierReference<'a>>),
    BindingIdentifier(&'a AstNode<'a, BindingIdentifier<'a>>),
    LabelIdentifier(&'a AstNode<'a, LabelIdentifier<'a>>),
    ThisExpression(&'a AstNode<'a, ThisExpression>),
    ArrayExpression(&'a AstNode<'a, ArrayExpression<'a>>),
    Elision(&'a AstNode<'a, Elision>),
    ObjectExpression(&'a AstNode<'a, ObjectExpression<'a>>),
    ObjectProperty(&'a AstNode<'a, ObjectProperty<'a>>),
    TemplateLiteral(&'a AstNode<'a, TemplateLiteral<'a>>),
    TaggedTemplateExpression(&'a AstNode<'a, TaggedTemplateExpression<'a>>),
    TemplateElement(&'a AstNode<'a, TemplateElement<'a>>),
    ComputedMemberExpression(&'a AstNode<'a, ComputedMemberExpression<'a>>),
    StaticMemberExpression(&'a AstNode<'a, StaticMemberExpression<'a>>),
    PrivateFieldExpression(&'a AstNode<'a, PrivateFieldExpression<'a>>),
    CallExpression(&'a AstNode<'a, CallExpression<'a>>),
    NewExpression(&'a AstNode<'a, NewExpression<'a>>),
    MetaProperty(&'a AstNode<'a, MetaProperty<'a>>),
    SpreadElement(&'a AstNode<'a, SpreadElement<'a>>),
    UpdateExpression(&'a AstNode<'a, UpdateExpression<'a>>),
    UnaryExpression(&'a AstNode<'a, UnaryExpression<'a>>),
    BinaryExpression(&'a AstNode<'a, BinaryExpression<'a>>),
    PrivateInExpression(&'a AstNode<'a, PrivateInExpression<'a>>),
    LogicalExpression(&'a AstNode<'a, LogicalExpression<'a>>),
    ConditionalExpression(&'a AstNode<'a, ConditionalExpression<'a>>),
    AssignmentExpression(&'a AstNode<'a, AssignmentExpression<'a>>),
    ArrayAssignmentTarget(&'a AstNode<'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(&'a AstNode<'a, ObjectAssignmentTarget<'a>>),
    AssignmentTargetRest(&'a AstNode<'a, AssignmentTargetRest<'a>>),
    AssignmentTargetWithDefault(&'a AstNode<'a, AssignmentTargetWithDefault<'a>>),
    AssignmentTargetPropertyIdentifier(&'a AstNode<'a, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(&'a AstNode<'a, AssignmentTargetPropertyProperty<'a>>),
    SequenceExpression(&'a AstNode<'a, SequenceExpression<'a>>),
    Super(&'a AstNode<'a, Super>),
    AwaitExpression(&'a AstNode<'a, AwaitExpression<'a>>),
    ChainExpression(&'a AstNode<'a, ChainExpression<'a>>),
    ParenthesizedExpression(&'a AstNode<'a, ParenthesizedExpression<'a>>),
    Directive(&'a AstNode<'a, Directive<'a>>),
    Hashbang(&'a AstNode<'a, Hashbang<'a>>),
    BlockStatement(&'a AstNode<'a, BlockStatement<'a>>),
    VariableDeclaration(&'a AstNode<'a, VariableDeclaration<'a>>),
    VariableDeclarator(&'a AstNode<'a, VariableDeclarator<'a>>),
    EmptyStatement(&'a AstNode<'a, EmptyStatement>),
    ExpressionStatement(&'a AstNode<'a, ExpressionStatement<'a>>),
    IfStatement(&'a AstNode<'a, IfStatement<'a>>),
    DoWhileStatement(&'a AstNode<'a, DoWhileStatement<'a>>),
    WhileStatement(&'a AstNode<'a, WhileStatement<'a>>),
    ForStatement(&'a AstNode<'a, ForStatement<'a>>),
    ForInStatement(&'a AstNode<'a, ForInStatement<'a>>),
    ForOfStatement(&'a AstNode<'a, ForOfStatement<'a>>),
    ContinueStatement(&'a AstNode<'a, ContinueStatement<'a>>),
    BreakStatement(&'a AstNode<'a, BreakStatement<'a>>),
    ReturnStatement(&'a AstNode<'a, ReturnStatement<'a>>),
    WithStatement(&'a AstNode<'a, WithStatement<'a>>),
    SwitchStatement(&'a AstNode<'a, SwitchStatement<'a>>),
    SwitchCase(&'a AstNode<'a, SwitchCase<'a>>),
    LabeledStatement(&'a AstNode<'a, LabeledStatement<'a>>),
    ThrowStatement(&'a AstNode<'a, ThrowStatement<'a>>),
    TryStatement(&'a AstNode<'a, TryStatement<'a>>),
    CatchClause(&'a AstNode<'a, CatchClause<'a>>),
    CatchParameter(&'a AstNode<'a, CatchParameter<'a>>),
    DebuggerStatement(&'a AstNode<'a, DebuggerStatement>),
    AssignmentPattern(&'a AstNode<'a, AssignmentPattern<'a>>),
    ObjectPattern(&'a AstNode<'a, ObjectPattern<'a>>),
    BindingProperty(&'a AstNode<'a, BindingProperty<'a>>),
    ArrayPattern(&'a AstNode<'a, ArrayPattern<'a>>),
    BindingRestElement(&'a AstNode<'a, BindingRestElement<'a>>),
    Function(&'a AstNode<'a, Function<'a>>),
    FormalParameters(&'a AstNode<'a, FormalParameters<'a>>),
    FormalParameter(&'a AstNode<'a, FormalParameter<'a>>),
    FormalParameterRest(&'a AstNode<'a, FormalParameterRest<'a>>),
    FunctionBody(&'a AstNode<'a, FunctionBody<'a>>),
    ArrowFunctionExpression(&'a AstNode<'a, ArrowFunctionExpression<'a>>),
    YieldExpression(&'a AstNode<'a, YieldExpression<'a>>),
    Class(&'a AstNode<'a, Class<'a>>),
    ClassBody(&'a AstNode<'a, ClassBody<'a>>),
    MethodDefinition(&'a AstNode<'a, MethodDefinition<'a>>),
    PropertyDefinition(&'a AstNode<'a, PropertyDefinition<'a>>),
    PrivateIdentifier(&'a AstNode<'a, PrivateIdentifier<'a>>),
    StaticBlock(&'a AstNode<'a, StaticBlock<'a>>),
    AccessorProperty(&'a AstNode<'a, AccessorProperty<'a>>),
    ImportExpression(&'a AstNode<'a, ImportExpression<'a>>),
    ImportDeclaration(&'a AstNode<'a, ImportDeclaration<'a>>),
    ImportSpecifier(&'a AstNode<'a, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(&'a AstNode<'a, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(&'a AstNode<'a, ImportNamespaceSpecifier<'a>>),
    WithClause(&'a AstNode<'a, WithClause<'a>>),
    ImportAttribute(&'a AstNode<'a, ImportAttribute<'a>>),
    ExportNamedDeclaration(&'a AstNode<'a, ExportNamedDeclaration<'a>>),
    ExportDefaultDeclaration(&'a AstNode<'a, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(&'a AstNode<'a, ExportAllDeclaration<'a>>),
    ExportSpecifier(&'a AstNode<'a, ExportSpecifier<'a>>),
    V8IntrinsicExpression(&'a AstNode<'a, V8IntrinsicExpression<'a>>),
    BooleanLiteral(&'a AstNode<'a, BooleanLiteral>),
    NullLiteral(&'a AstNode<'a, NullLiteral>),
    NumericLiteral(&'a AstNode<'a, NumericLiteral<'a>>),
    StringLiteral(&'a AstNode<'a, StringLiteral<'a>>),
    BigIntLiteral(&'a AstNode<'a, BigIntLiteral<'a>>),
    RegExpLiteral(&'a AstNode<'a, RegExpLiteral<'a>>),
    JSXElement(&'a AstNode<'a, JSXElement<'a>>),
    JSXOpeningElement(&'a AstNode<'a, JSXOpeningElement<'a>>),
    JSXClosingElement(&'a AstNode<'a, JSXClosingElement<'a>>),
    JSXFragment(&'a AstNode<'a, JSXFragment<'a>>),
    JSXOpeningFragment(&'a AstNode<'a, JSXOpeningFragment>),
    JSXClosingFragment(&'a AstNode<'a, JSXClosingFragment>),
    JSXNamespacedName(&'a AstNode<'a, JSXNamespacedName<'a>>),
    JSXMemberExpression(&'a AstNode<'a, JSXMemberExpression<'a>>),
    JSXExpressionContainer(&'a AstNode<'a, JSXExpressionContainer<'a>>),
    JSXEmptyExpression(&'a AstNode<'a, JSXEmptyExpression>),
    JSXAttribute(&'a AstNode<'a, JSXAttribute<'a>>),
    JSXSpreadAttribute(&'a AstNode<'a, JSXSpreadAttribute<'a>>),
    JSXIdentifier(&'a AstNode<'a, JSXIdentifier<'a>>),
    JSXSpreadChild(&'a AstNode<'a, JSXSpreadChild<'a>>),
    JSXText(&'a AstNode<'a, JSXText<'a>>),
    TSThisParameter(&'a AstNode<'a, TSThisParameter<'a>>),
    TSEnumDeclaration(&'a AstNode<'a, TSEnumDeclaration<'a>>),
    TSEnumBody(&'a AstNode<'a, TSEnumBody<'a>>),
    TSEnumMember(&'a AstNode<'a, TSEnumMember<'a>>),
    TSTypeAnnotation(&'a AstNode<'a, TSTypeAnnotation<'a>>),
    TSLiteralType(&'a AstNode<'a, TSLiteralType<'a>>),
    TSConditionalType(&'a AstNode<'a, TSConditionalType<'a>>),
    TSUnionType(&'a AstNode<'a, TSUnionType<'a>>),
    TSIntersectionType(&'a AstNode<'a, TSIntersectionType<'a>>),
    TSParenthesizedType(&'a AstNode<'a, TSParenthesizedType<'a>>),
    TSTypeOperator(&'a AstNode<'a, TSTypeOperator<'a>>),
    TSArrayType(&'a AstNode<'a, TSArrayType<'a>>),
    TSIndexedAccessType(&'a AstNode<'a, TSIndexedAccessType<'a>>),
    TSTupleType(&'a AstNode<'a, TSTupleType<'a>>),
    TSNamedTupleMember(&'a AstNode<'a, TSNamedTupleMember<'a>>),
    TSOptionalType(&'a AstNode<'a, TSOptionalType<'a>>),
    TSRestType(&'a AstNode<'a, TSRestType<'a>>),
    TSAnyKeyword(&'a AstNode<'a, TSAnyKeyword>),
    TSStringKeyword(&'a AstNode<'a, TSStringKeyword>),
    TSBooleanKeyword(&'a AstNode<'a, TSBooleanKeyword>),
    TSNumberKeyword(&'a AstNode<'a, TSNumberKeyword>),
    TSNeverKeyword(&'a AstNode<'a, TSNeverKeyword>),
    TSIntrinsicKeyword(&'a AstNode<'a, TSIntrinsicKeyword>),
    TSUnknownKeyword(&'a AstNode<'a, TSUnknownKeyword>),
    TSNullKeyword(&'a AstNode<'a, TSNullKeyword>),
    TSUndefinedKeyword(&'a AstNode<'a, TSUndefinedKeyword>),
    TSVoidKeyword(&'a AstNode<'a, TSVoidKeyword>),
    TSSymbolKeyword(&'a AstNode<'a, TSSymbolKeyword>),
    TSThisType(&'a AstNode<'a, TSThisType>),
    TSObjectKeyword(&'a AstNode<'a, TSObjectKeyword>),
    TSBigIntKeyword(&'a AstNode<'a, TSBigIntKeyword>),
    TSTypeReference(&'a AstNode<'a, TSTypeReference<'a>>),
    TSQualifiedName(&'a AstNode<'a, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(&'a AstNode<'a, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(&'a AstNode<'a, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(&'a AstNode<'a, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(&'a AstNode<'a, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(&'a AstNode<'a, TSClassImplements<'a>>),
    TSInterfaceDeclaration(&'a AstNode<'a, TSInterfaceDeclaration<'a>>),
    TSInterfaceBody(&'a AstNode<'a, TSInterfaceBody<'a>>),
    TSPropertySignature(&'a AstNode<'a, TSPropertySignature<'a>>),
    TSIndexSignature(&'a AstNode<'a, TSIndexSignature<'a>>),
    TSCallSignatureDeclaration(&'a AstNode<'a, TSCallSignatureDeclaration<'a>>),
    TSMethodSignature(&'a AstNode<'a, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(&'a AstNode<'a, TSConstructSignatureDeclaration<'a>>),
    TSIndexSignatureName(&'a AstNode<'a, TSIndexSignatureName<'a>>),
    TSInterfaceHeritage(&'a AstNode<'a, TSInterfaceHeritage<'a>>),
    TSTypePredicate(&'a AstNode<'a, TSTypePredicate<'a>>),
    TSModuleDeclaration(&'a AstNode<'a, TSModuleDeclaration<'a>>),
    TSGlobalDeclaration(&'a AstNode<'a, TSGlobalDeclaration<'a>>),
    TSModuleBlock(&'a AstNode<'a, TSModuleBlock<'a>>),
    TSTypeLiteral(&'a AstNode<'a, TSTypeLiteral<'a>>),
    TSInferType(&'a AstNode<'a, TSInferType<'a>>),
    TSTypeQuery(&'a AstNode<'a, TSTypeQuery<'a>>),
    TSImportType(&'a AstNode<'a, TSImportType<'a>>),
    TSImportTypeQualifiedName(&'a AstNode<'a, TSImportTypeQualifiedName<'a>>),
    TSFunctionType(&'a AstNode<'a, TSFunctionType<'a>>),
    TSConstructorType(&'a AstNode<'a, TSConstructorType<'a>>),
    TSMappedType(&'a AstNode<'a, TSMappedType<'a>>),
    TSTemplateLiteralType(&'a AstNode<'a, TSTemplateLiteralType<'a>>),
    TSAsExpression(&'a AstNode<'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(&'a AstNode<'a, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(&'a AstNode<'a, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(&'a AstNode<'a, TSImportEqualsDeclaration<'a>>),
    TSExternalModuleReference(&'a AstNode<'a, TSExternalModuleReference<'a>>),
    TSNonNullExpression(&'a AstNode<'a, TSNonNullExpression<'a>>),
    Decorator(&'a AstNode<'a, Decorator<'a>>),
    TSExportAssignment(&'a AstNode<'a, TSExportAssignment<'a>>),
    TSNamespaceExportDeclaration(&'a AstNode<'a, TSNamespaceExportDeclaration<'a>>),
    TSInstantiationExpression(&'a AstNode<'a, TSInstantiationExpression<'a>>),
    JSDocNullableType(&'a AstNode<'a, JSDocNullableType<'a>>),
    JSDocNonNullableType(&'a AstNode<'a, JSDocNonNullableType<'a>>),
    JSDocUnknownType(&'a AstNode<'a, JSDocUnknownType>),
}
impl AstNodes<'_> {
    #[inline]
    pub fn span(&self) -> Span {
        match self {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
            Self::Program(n) => n.span(),
            Self::IdentifierName(n) => n.span(),
            Self::IdentifierReference(n) => n.span(),
            Self::BindingIdentifier(n) => n.span(),
            Self::LabelIdentifier(n) => n.span(),
            Self::ThisExpression(n) => n.span(),
            Self::ArrayExpression(n) => n.span(),
            Self::Elision(n) => n.span(),
            Self::ObjectExpression(n) => n.span(),
            Self::ObjectProperty(n) => n.span(),
            Self::TemplateLiteral(n) => n.span(),
            Self::TaggedTemplateExpression(n) => n.span(),
            Self::TemplateElement(n) => n.span(),
            Self::ComputedMemberExpression(n) => n.span(),
            Self::StaticMemberExpression(n) => n.span(),
            Self::PrivateFieldExpression(n) => n.span(),
            Self::CallExpression(n) => n.span(),
            Self::NewExpression(n) => n.span(),
            Self::MetaProperty(n) => n.span(),
            Self::SpreadElement(n) => n.span(),
            Self::UpdateExpression(n) => n.span(),
            Self::UnaryExpression(n) => n.span(),
            Self::BinaryExpression(n) => n.span(),
            Self::PrivateInExpression(n) => n.span(),
            Self::LogicalExpression(n) => n.span(),
            Self::ConditionalExpression(n) => n.span(),
            Self::AssignmentExpression(n) => n.span(),
            Self::ArrayAssignmentTarget(n) => n.span(),
            Self::ObjectAssignmentTarget(n) => n.span(),
            Self::AssignmentTargetRest(n) => n.span(),
            Self::AssignmentTargetWithDefault(n) => n.span(),
            Self::AssignmentTargetPropertyIdentifier(n) => n.span(),
            Self::AssignmentTargetPropertyProperty(n) => n.span(),
            Self::SequenceExpression(n) => n.span(),
            Self::Super(n) => n.span(),
            Self::AwaitExpression(n) => n.span(),
            Self::ChainExpression(n) => n.span(),
            Self::ParenthesizedExpression(n) => n.span(),
            Self::Directive(n) => n.span(),
            Self::Hashbang(n) => n.span(),
            Self::BlockStatement(n) => n.span(),
            Self::VariableDeclaration(n) => n.span(),
            Self::VariableDeclarator(n) => n.span(),
            Self::EmptyStatement(n) => n.span(),
            Self::ExpressionStatement(n) => n.span(),
            Self::IfStatement(n) => n.span(),
            Self::DoWhileStatement(n) => n.span(),
            Self::WhileStatement(n) => n.span(),
            Self::ForStatement(n) => n.span(),
            Self::ForInStatement(n) => n.span(),
            Self::ForOfStatement(n) => n.span(),
            Self::ContinueStatement(n) => n.span(),
            Self::BreakStatement(n) => n.span(),
            Self::ReturnStatement(n) => n.span(),
            Self::WithStatement(n) => n.span(),
            Self::SwitchStatement(n) => n.span(),
            Self::SwitchCase(n) => n.span(),
            Self::LabeledStatement(n) => n.span(),
            Self::ThrowStatement(n) => n.span(),
            Self::TryStatement(n) => n.span(),
            Self::CatchClause(n) => n.span(),
            Self::CatchParameter(n) => n.span(),
            Self::DebuggerStatement(n) => n.span(),
            Self::AssignmentPattern(n) => n.span(),
            Self::ObjectPattern(n) => n.span(),
            Self::BindingProperty(n) => n.span(),
            Self::ArrayPattern(n) => n.span(),
            Self::BindingRestElement(n) => n.span(),
            Self::Function(n) => n.span(),
            Self::FormalParameters(n) => n.span(),
            Self::FormalParameter(n) => n.span(),
            Self::FormalParameterRest(n) => n.span(),
            Self::FunctionBody(n) => n.span(),
            Self::ArrowFunctionExpression(n) => n.span(),
            Self::YieldExpression(n) => n.span(),
            Self::Class(n) => n.span(),
            Self::ClassBody(n) => n.span(),
            Self::MethodDefinition(n) => n.span(),
            Self::PropertyDefinition(n) => n.span(),
            Self::PrivateIdentifier(n) => n.span(),
            Self::StaticBlock(n) => n.span(),
            Self::AccessorProperty(n) => n.span(),
            Self::ImportExpression(n) => n.span(),
            Self::ImportDeclaration(n) => n.span(),
            Self::ImportSpecifier(n) => n.span(),
            Self::ImportDefaultSpecifier(n) => n.span(),
            Self::ImportNamespaceSpecifier(n) => n.span(),
            Self::WithClause(n) => n.span(),
            Self::ImportAttribute(n) => n.span(),
            Self::ExportNamedDeclaration(n) => n.span(),
            Self::ExportDefaultDeclaration(n) => n.span(),
            Self::ExportAllDeclaration(n) => n.span(),
            Self::ExportSpecifier(n) => n.span(),
            Self::V8IntrinsicExpression(n) => n.span(),
            Self::BooleanLiteral(n) => n.span(),
            Self::NullLiteral(n) => n.span(),
            Self::NumericLiteral(n) => n.span(),
            Self::StringLiteral(n) => n.span(),
            Self::BigIntLiteral(n) => n.span(),
            Self::RegExpLiteral(n) => n.span(),
            Self::JSXElement(n) => n.span(),
            Self::JSXOpeningElement(n) => n.span(),
            Self::JSXClosingElement(n) => n.span(),
            Self::JSXFragment(n) => n.span(),
            Self::JSXOpeningFragment(n) => n.span(),
            Self::JSXClosingFragment(n) => n.span(),
            Self::JSXNamespacedName(n) => n.span(),
            Self::JSXMemberExpression(n) => n.span(),
            Self::JSXExpressionContainer(n) => n.span(),
            Self::JSXEmptyExpression(n) => n.span(),
            Self::JSXAttribute(n) => n.span(),
            Self::JSXSpreadAttribute(n) => n.span(),
            Self::JSXIdentifier(n) => n.span(),
            Self::JSXSpreadChild(n) => n.span(),
            Self::JSXText(n) => n.span(),
            Self::TSThisParameter(n) => n.span(),
            Self::TSEnumDeclaration(n) => n.span(),
            Self::TSEnumBody(n) => n.span(),
            Self::TSEnumMember(n) => n.span(),
            Self::TSTypeAnnotation(n) => n.span(),
            Self::TSLiteralType(n) => n.span(),
            Self::TSConditionalType(n) => n.span(),
            Self::TSUnionType(n) => n.span(),
            Self::TSIntersectionType(n) => n.span(),
            Self::TSParenthesizedType(n) => n.span(),
            Self::TSTypeOperator(n) => n.span(),
            Self::TSArrayType(n) => n.span(),
            Self::TSIndexedAccessType(n) => n.span(),
            Self::TSTupleType(n) => n.span(),
            Self::TSNamedTupleMember(n) => n.span(),
            Self::TSOptionalType(n) => n.span(),
            Self::TSRestType(n) => n.span(),
            Self::TSAnyKeyword(n) => n.span(),
            Self::TSStringKeyword(n) => n.span(),
            Self::TSBooleanKeyword(n) => n.span(),
            Self::TSNumberKeyword(n) => n.span(),
            Self::TSNeverKeyword(n) => n.span(),
            Self::TSIntrinsicKeyword(n) => n.span(),
            Self::TSUnknownKeyword(n) => n.span(),
            Self::TSNullKeyword(n) => n.span(),
            Self::TSUndefinedKeyword(n) => n.span(),
            Self::TSVoidKeyword(n) => n.span(),
            Self::TSSymbolKeyword(n) => n.span(),
            Self::TSThisType(n) => n.span(),
            Self::TSObjectKeyword(n) => n.span(),
            Self::TSBigIntKeyword(n) => n.span(),
            Self::TSTypeReference(n) => n.span(),
            Self::TSQualifiedName(n) => n.span(),
            Self::TSTypeParameterInstantiation(n) => n.span(),
            Self::TSTypeParameter(n) => n.span(),
            Self::TSTypeParameterDeclaration(n) => n.span(),
            Self::TSTypeAliasDeclaration(n) => n.span(),
            Self::TSClassImplements(n) => n.span(),
            Self::TSInterfaceDeclaration(n) => n.span(),
            Self::TSInterfaceBody(n) => n.span(),
            Self::TSPropertySignature(n) => n.span(),
            Self::TSIndexSignature(n) => n.span(),
            Self::TSCallSignatureDeclaration(n) => n.span(),
            Self::TSMethodSignature(n) => n.span(),
            Self::TSConstructSignatureDeclaration(n) => n.span(),
            Self::TSIndexSignatureName(n) => n.span(),
            Self::TSInterfaceHeritage(n) => n.span(),
            Self::TSTypePredicate(n) => n.span(),
            Self::TSModuleDeclaration(n) => n.span(),
            Self::TSGlobalDeclaration(n) => n.span(),
            Self::TSModuleBlock(n) => n.span(),
            Self::TSTypeLiteral(n) => n.span(),
            Self::TSInferType(n) => n.span(),
            Self::TSTypeQuery(n) => n.span(),
            Self::TSImportType(n) => n.span(),
            Self::TSImportTypeQualifiedName(n) => n.span(),
            Self::TSFunctionType(n) => n.span(),
            Self::TSConstructorType(n) => n.span(),
            Self::TSMappedType(n) => n.span(),
            Self::TSTemplateLiteralType(n) => n.span(),
            Self::TSAsExpression(n) => n.span(),
            Self::TSSatisfiesExpression(n) => n.span(),
            Self::TSTypeAssertion(n) => n.span(),
            Self::TSImportEqualsDeclaration(n) => n.span(),
            Self::TSExternalModuleReference(n) => n.span(),
            Self::TSNonNullExpression(n) => n.span(),
            Self::Decorator(n) => n.span(),
            Self::TSExportAssignment(n) => n.span(),
            Self::TSNamespaceExportDeclaration(n) => n.span(),
            Self::TSInstantiationExpression(n) => n.span(),
            Self::JSDocNullableType(n) => n.span(),
            Self::JSDocNonNullableType(n) => n.span(),
            Self::JSDocUnknownType(n) => n.span(),
        }
    }
    #[inline]
    pub fn parent(&self) -> &Self {
        match self {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
            Self::Program(n) => n.parent(),
            Self::IdentifierName(n) => n.parent(),
            Self::IdentifierReference(n) => n.parent(),
            Self::BindingIdentifier(n) => n.parent(),
            Self::LabelIdentifier(n) => n.parent(),
            Self::ThisExpression(n) => n.parent(),
            Self::ArrayExpression(n) => n.parent(),
            Self::Elision(n) => n.parent(),
            Self::ObjectExpression(n) => n.parent(),
            Self::ObjectProperty(n) => n.parent(),
            Self::TemplateLiteral(n) => n.parent(),
            Self::TaggedTemplateExpression(n) => n.parent(),
            Self::TemplateElement(n) => n.parent(),
            Self::ComputedMemberExpression(n) => n.parent(),
            Self::StaticMemberExpression(n) => n.parent(),
            Self::PrivateFieldExpression(n) => n.parent(),
            Self::CallExpression(n) => n.parent(),
            Self::NewExpression(n) => n.parent(),
            Self::MetaProperty(n) => n.parent(),
            Self::SpreadElement(n) => n.parent(),
            Self::UpdateExpression(n) => n.parent(),
            Self::UnaryExpression(n) => n.parent(),
            Self::BinaryExpression(n) => n.parent(),
            Self::PrivateInExpression(n) => n.parent(),
            Self::LogicalExpression(n) => n.parent(),
            Self::ConditionalExpression(n) => n.parent(),
            Self::AssignmentExpression(n) => n.parent(),
            Self::ArrayAssignmentTarget(n) => n.parent(),
            Self::ObjectAssignmentTarget(n) => n.parent(),
            Self::AssignmentTargetRest(n) => n.parent(),
            Self::AssignmentTargetWithDefault(n) => n.parent(),
            Self::AssignmentTargetPropertyIdentifier(n) => n.parent(),
            Self::AssignmentTargetPropertyProperty(n) => n.parent(),
            Self::SequenceExpression(n) => n.parent(),
            Self::Super(n) => n.parent(),
            Self::AwaitExpression(n) => n.parent(),
            Self::ChainExpression(n) => n.parent(),
            Self::ParenthesizedExpression(n) => n.parent(),
            Self::Directive(n) => n.parent(),
            Self::Hashbang(n) => n.parent(),
            Self::BlockStatement(n) => n.parent(),
            Self::VariableDeclaration(n) => n.parent(),
            Self::VariableDeclarator(n) => n.parent(),
            Self::EmptyStatement(n) => n.parent(),
            Self::ExpressionStatement(n) => n.parent(),
            Self::IfStatement(n) => n.parent(),
            Self::DoWhileStatement(n) => n.parent(),
            Self::WhileStatement(n) => n.parent(),
            Self::ForStatement(n) => n.parent(),
            Self::ForInStatement(n) => n.parent(),
            Self::ForOfStatement(n) => n.parent(),
            Self::ContinueStatement(n) => n.parent(),
            Self::BreakStatement(n) => n.parent(),
            Self::ReturnStatement(n) => n.parent(),
            Self::WithStatement(n) => n.parent(),
            Self::SwitchStatement(n) => n.parent(),
            Self::SwitchCase(n) => n.parent(),
            Self::LabeledStatement(n) => n.parent(),
            Self::ThrowStatement(n) => n.parent(),
            Self::TryStatement(n) => n.parent(),
            Self::CatchClause(n) => n.parent(),
            Self::CatchParameter(n) => n.parent(),
            Self::DebuggerStatement(n) => n.parent(),
            Self::AssignmentPattern(n) => n.parent(),
            Self::ObjectPattern(n) => n.parent(),
            Self::BindingProperty(n) => n.parent(),
            Self::ArrayPattern(n) => n.parent(),
            Self::BindingRestElement(n) => n.parent(),
            Self::Function(n) => n.parent(),
            Self::FormalParameters(n) => n.parent(),
            Self::FormalParameter(n) => n.parent(),
            Self::FormalParameterRest(n) => n.parent(),
            Self::FunctionBody(n) => n.parent(),
            Self::ArrowFunctionExpression(n) => n.parent(),
            Self::YieldExpression(n) => n.parent(),
            Self::Class(n) => n.parent(),
            Self::ClassBody(n) => n.parent(),
            Self::MethodDefinition(n) => n.parent(),
            Self::PropertyDefinition(n) => n.parent(),
            Self::PrivateIdentifier(n) => n.parent(),
            Self::StaticBlock(n) => n.parent(),
            Self::AccessorProperty(n) => n.parent(),
            Self::ImportExpression(n) => n.parent(),
            Self::ImportDeclaration(n) => n.parent(),
            Self::ImportSpecifier(n) => n.parent(),
            Self::ImportDefaultSpecifier(n) => n.parent(),
            Self::ImportNamespaceSpecifier(n) => n.parent(),
            Self::WithClause(n) => n.parent(),
            Self::ImportAttribute(n) => n.parent(),
            Self::ExportNamedDeclaration(n) => n.parent(),
            Self::ExportDefaultDeclaration(n) => n.parent(),
            Self::ExportAllDeclaration(n) => n.parent(),
            Self::ExportSpecifier(n) => n.parent(),
            Self::V8IntrinsicExpression(n) => n.parent(),
            Self::BooleanLiteral(n) => n.parent(),
            Self::NullLiteral(n) => n.parent(),
            Self::NumericLiteral(n) => n.parent(),
            Self::StringLiteral(n) => n.parent(),
            Self::BigIntLiteral(n) => n.parent(),
            Self::RegExpLiteral(n) => n.parent(),
            Self::JSXElement(n) => n.parent(),
            Self::JSXOpeningElement(n) => n.parent(),
            Self::JSXClosingElement(n) => n.parent(),
            Self::JSXFragment(n) => n.parent(),
            Self::JSXOpeningFragment(n) => n.parent(),
            Self::JSXClosingFragment(n) => n.parent(),
            Self::JSXNamespacedName(n) => n.parent(),
            Self::JSXMemberExpression(n) => n.parent(),
            Self::JSXExpressionContainer(n) => n.parent(),
            Self::JSXEmptyExpression(n) => n.parent(),
            Self::JSXAttribute(n) => n.parent(),
            Self::JSXSpreadAttribute(n) => n.parent(),
            Self::JSXIdentifier(n) => n.parent(),
            Self::JSXSpreadChild(n) => n.parent(),
            Self::JSXText(n) => n.parent(),
            Self::TSThisParameter(n) => n.parent(),
            Self::TSEnumDeclaration(n) => n.parent(),
            Self::TSEnumBody(n) => n.parent(),
            Self::TSEnumMember(n) => n.parent(),
            Self::TSTypeAnnotation(n) => n.parent(),
            Self::TSLiteralType(n) => n.parent(),
            Self::TSConditionalType(n) => n.parent(),
            Self::TSUnionType(n) => n.parent(),
            Self::TSIntersectionType(n) => n.parent(),
            Self::TSParenthesizedType(n) => n.parent(),
            Self::TSTypeOperator(n) => n.parent(),
            Self::TSArrayType(n) => n.parent(),
            Self::TSIndexedAccessType(n) => n.parent(),
            Self::TSTupleType(n) => n.parent(),
            Self::TSNamedTupleMember(n) => n.parent(),
            Self::TSOptionalType(n) => n.parent(),
            Self::TSRestType(n) => n.parent(),
            Self::TSAnyKeyword(n) => n.parent(),
            Self::TSStringKeyword(n) => n.parent(),
            Self::TSBooleanKeyword(n) => n.parent(),
            Self::TSNumberKeyword(n) => n.parent(),
            Self::TSNeverKeyword(n) => n.parent(),
            Self::TSIntrinsicKeyword(n) => n.parent(),
            Self::TSUnknownKeyword(n) => n.parent(),
            Self::TSNullKeyword(n) => n.parent(),
            Self::TSUndefinedKeyword(n) => n.parent(),
            Self::TSVoidKeyword(n) => n.parent(),
            Self::TSSymbolKeyword(n) => n.parent(),
            Self::TSThisType(n) => n.parent(),
            Self::TSObjectKeyword(n) => n.parent(),
            Self::TSBigIntKeyword(n) => n.parent(),
            Self::TSTypeReference(n) => n.parent(),
            Self::TSQualifiedName(n) => n.parent(),
            Self::TSTypeParameterInstantiation(n) => n.parent(),
            Self::TSTypeParameter(n) => n.parent(),
            Self::TSTypeParameterDeclaration(n) => n.parent(),
            Self::TSTypeAliasDeclaration(n) => n.parent(),
            Self::TSClassImplements(n) => n.parent(),
            Self::TSInterfaceDeclaration(n) => n.parent(),
            Self::TSInterfaceBody(n) => n.parent(),
            Self::TSPropertySignature(n) => n.parent(),
            Self::TSIndexSignature(n) => n.parent(),
            Self::TSCallSignatureDeclaration(n) => n.parent(),
            Self::TSMethodSignature(n) => n.parent(),
            Self::TSConstructSignatureDeclaration(n) => n.parent(),
            Self::TSIndexSignatureName(n) => n.parent(),
            Self::TSInterfaceHeritage(n) => n.parent(),
            Self::TSTypePredicate(n) => n.parent(),
            Self::TSModuleDeclaration(n) => n.parent(),
            Self::TSGlobalDeclaration(n) => n.parent(),
            Self::TSModuleBlock(n) => n.parent(),
            Self::TSTypeLiteral(n) => n.parent(),
            Self::TSInferType(n) => n.parent(),
            Self::TSTypeQuery(n) => n.parent(),
            Self::TSImportType(n) => n.parent(),
            Self::TSImportTypeQualifiedName(n) => n.parent(),
            Self::TSFunctionType(n) => n.parent(),
            Self::TSConstructorType(n) => n.parent(),
            Self::TSMappedType(n) => n.parent(),
            Self::TSTemplateLiteralType(n) => n.parent(),
            Self::TSAsExpression(n) => n.parent(),
            Self::TSSatisfiesExpression(n) => n.parent(),
            Self::TSTypeAssertion(n) => n.parent(),
            Self::TSImportEqualsDeclaration(n) => n.parent(),
            Self::TSExternalModuleReference(n) => n.parent(),
            Self::TSNonNullExpression(n) => n.parent(),
            Self::Decorator(n) => n.parent(),
            Self::TSExportAssignment(n) => n.parent(),
            Self::TSNamespaceExportDeclaration(n) => n.parent(),
            Self::TSInstantiationExpression(n) => n.parent(),
            Self::JSDocNullableType(n) => n.parent(),
            Self::JSDocNonNullableType(n) => n.parent(),
            Self::JSDocUnknownType(n) => n.parent(),
        }
    }
    #[inline]
    pub fn debug_name(&self) -> &'static str {
        match self {
            Self::Dummy() => "Dummy",
            Self::Program(_) => "Program",
            Self::IdentifierName(_) => "IdentifierName",
            Self::IdentifierReference(_) => "IdentifierReference",
            Self::BindingIdentifier(_) => "BindingIdentifier",
            Self::LabelIdentifier(_) => "LabelIdentifier",
            Self::ThisExpression(_) => "ThisExpression",
            Self::ArrayExpression(_) => "ArrayExpression",
            Self::Elision(_) => "Elision",
            Self::ObjectExpression(_) => "ObjectExpression",
            Self::ObjectProperty(_) => "ObjectProperty",
            Self::TemplateLiteral(_) => "TemplateLiteral",
            Self::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
            Self::TemplateElement(_) => "TemplateElement",
            Self::ComputedMemberExpression(_) => "ComputedMemberExpression",
            Self::StaticMemberExpression(_) => "StaticMemberExpression",
            Self::PrivateFieldExpression(_) => "PrivateFieldExpression",
            Self::CallExpression(_) => "CallExpression",
            Self::NewExpression(_) => "NewExpression",
            Self::MetaProperty(_) => "MetaProperty",
            Self::SpreadElement(_) => "SpreadElement",
            Self::UpdateExpression(_) => "UpdateExpression",
            Self::UnaryExpression(_) => "UnaryExpression",
            Self::BinaryExpression(_) => "BinaryExpression",
            Self::PrivateInExpression(_) => "PrivateInExpression",
            Self::LogicalExpression(_) => "LogicalExpression",
            Self::ConditionalExpression(_) => "ConditionalExpression",
            Self::AssignmentExpression(_) => "AssignmentExpression",
            Self::ArrayAssignmentTarget(_) => "ArrayAssignmentTarget",
            Self::ObjectAssignmentTarget(_) => "ObjectAssignmentTarget",
            Self::AssignmentTargetRest(_) => "AssignmentTargetRest",
            Self::AssignmentTargetWithDefault(_) => "AssignmentTargetWithDefault",
            Self::AssignmentTargetPropertyIdentifier(_) => "AssignmentTargetPropertyIdentifier",
            Self::AssignmentTargetPropertyProperty(_) => "AssignmentTargetPropertyProperty",
            Self::SequenceExpression(_) => "SequenceExpression",
            Self::Super(_) => "Super",
            Self::AwaitExpression(_) => "AwaitExpression",
            Self::ChainExpression(_) => "ChainExpression",
            Self::ParenthesizedExpression(_) => "ParenthesizedExpression",
            Self::Directive(_) => "Directive",
            Self::Hashbang(_) => "Hashbang",
            Self::BlockStatement(_) => "BlockStatement",
            Self::VariableDeclaration(_) => "VariableDeclaration",
            Self::VariableDeclarator(_) => "VariableDeclarator",
            Self::EmptyStatement(_) => "EmptyStatement",
            Self::ExpressionStatement(_) => "ExpressionStatement",
            Self::IfStatement(_) => "IfStatement",
            Self::DoWhileStatement(_) => "DoWhileStatement",
            Self::WhileStatement(_) => "WhileStatement",
            Self::ForStatement(_) => "ForStatement",
            Self::ForInStatement(_) => "ForInStatement",
            Self::ForOfStatement(_) => "ForOfStatement",
            Self::ContinueStatement(_) => "ContinueStatement",
            Self::BreakStatement(_) => "BreakStatement",
            Self::ReturnStatement(_) => "ReturnStatement",
            Self::WithStatement(_) => "WithStatement",
            Self::SwitchStatement(_) => "SwitchStatement",
            Self::SwitchCase(_) => "SwitchCase",
            Self::LabeledStatement(_) => "LabeledStatement",
            Self::ThrowStatement(_) => "ThrowStatement",
            Self::TryStatement(_) => "TryStatement",
            Self::CatchClause(_) => "CatchClause",
            Self::CatchParameter(_) => "CatchParameter",
            Self::DebuggerStatement(_) => "DebuggerStatement",
            Self::AssignmentPattern(_) => "AssignmentPattern",
            Self::ObjectPattern(_) => "ObjectPattern",
            Self::BindingProperty(_) => "BindingProperty",
            Self::ArrayPattern(_) => "ArrayPattern",
            Self::BindingRestElement(_) => "BindingRestElement",
            Self::Function(_) => "Function",
            Self::FormalParameters(_) => "FormalParameters",
            Self::FormalParameter(_) => "FormalParameter",
            Self::FormalParameterRest(_) => "FormalParameterRest",
            Self::FunctionBody(_) => "FunctionBody",
            Self::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
            Self::YieldExpression(_) => "YieldExpression",
            Self::Class(_) => "Class",
            Self::ClassBody(_) => "ClassBody",
            Self::MethodDefinition(_) => "MethodDefinition",
            Self::PropertyDefinition(_) => "PropertyDefinition",
            Self::PrivateIdentifier(_) => "PrivateIdentifier",
            Self::StaticBlock(_) => "StaticBlock",
            Self::AccessorProperty(_) => "AccessorProperty",
            Self::ImportExpression(_) => "ImportExpression",
            Self::ImportDeclaration(_) => "ImportDeclaration",
            Self::ImportSpecifier(_) => "ImportSpecifier",
            Self::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier",
            Self::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier",
            Self::WithClause(_) => "WithClause",
            Self::ImportAttribute(_) => "ImportAttribute",
            Self::ExportNamedDeclaration(_) => "ExportNamedDeclaration",
            Self::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
            Self::ExportAllDeclaration(_) => "ExportAllDeclaration",
            Self::ExportSpecifier(_) => "ExportSpecifier",
            Self::V8IntrinsicExpression(_) => "V8IntrinsicExpression",
            Self::BooleanLiteral(_) => "BooleanLiteral",
            Self::NullLiteral(_) => "NullLiteral",
            Self::NumericLiteral(_) => "NumericLiteral",
            Self::StringLiteral(_) => "StringLiteral",
            Self::BigIntLiteral(_) => "BigIntLiteral",
            Self::RegExpLiteral(_) => "RegExpLiteral",
            Self::JSXElement(_) => "JSXElement",
            Self::JSXOpeningElement(_) => "JSXOpeningElement",
            Self::JSXClosingElement(_) => "JSXClosingElement",
            Self::JSXFragment(_) => "JSXFragment",
            Self::JSXOpeningFragment(_) => "JSXOpeningFragment",
            Self::JSXClosingFragment(_) => "JSXClosingFragment",
            Self::JSXNamespacedName(_) => "JSXNamespacedName",
            Self::JSXMemberExpression(_) => "JSXMemberExpression",
            Self::JSXExpressionContainer(_) => "JSXExpressionContainer",
            Self::JSXEmptyExpression(_) => "JSXEmptyExpression",
            Self::JSXAttribute(_) => "JSXAttribute",
            Self::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
            Self::JSXIdentifier(_) => "JSXIdentifier",
            Self::JSXSpreadChild(_) => "JSXSpreadChild",
            Self::JSXText(_) => "JSXText",
            Self::TSThisParameter(_) => "TSThisParameter",
            Self::TSEnumDeclaration(_) => "TSEnumDeclaration",
            Self::TSEnumBody(_) => "TSEnumBody",
            Self::TSEnumMember(_) => "TSEnumMember",
            Self::TSTypeAnnotation(_) => "TSTypeAnnotation",
            Self::TSLiteralType(_) => "TSLiteralType",
            Self::TSConditionalType(_) => "TSConditionalType",
            Self::TSUnionType(_) => "TSUnionType",
            Self::TSIntersectionType(_) => "TSIntersectionType",
            Self::TSParenthesizedType(_) => "TSParenthesizedType",
            Self::TSTypeOperator(_) => "TSTypeOperator",
            Self::TSArrayType(_) => "TSArrayType",
            Self::TSIndexedAccessType(_) => "TSIndexedAccessType",
            Self::TSTupleType(_) => "TSTupleType",
            Self::TSNamedTupleMember(_) => "TSNamedTupleMember",
            Self::TSOptionalType(_) => "TSOptionalType",
            Self::TSRestType(_) => "TSRestType",
            Self::TSAnyKeyword(_) => "TSAnyKeyword",
            Self::TSStringKeyword(_) => "TSStringKeyword",
            Self::TSBooleanKeyword(_) => "TSBooleanKeyword",
            Self::TSNumberKeyword(_) => "TSNumberKeyword",
            Self::TSNeverKeyword(_) => "TSNeverKeyword",
            Self::TSIntrinsicKeyword(_) => "TSIntrinsicKeyword",
            Self::TSUnknownKeyword(_) => "TSUnknownKeyword",
            Self::TSNullKeyword(_) => "TSNullKeyword",
            Self::TSUndefinedKeyword(_) => "TSUndefinedKeyword",
            Self::TSVoidKeyword(_) => "TSVoidKeyword",
            Self::TSSymbolKeyword(_) => "TSSymbolKeyword",
            Self::TSThisType(_) => "TSThisType",
            Self::TSObjectKeyword(_) => "TSObjectKeyword",
            Self::TSBigIntKeyword(_) => "TSBigIntKeyword",
            Self::TSTypeReference(_) => "TSTypeReference",
            Self::TSQualifiedName(_) => "TSQualifiedName",
            Self::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation",
            Self::TSTypeParameter(_) => "TSTypeParameter",
            Self::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration",
            Self::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
            Self::TSClassImplements(_) => "TSClassImplements",
            Self::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
            Self::TSInterfaceBody(_) => "TSInterfaceBody",
            Self::TSPropertySignature(_) => "TSPropertySignature",
            Self::TSIndexSignature(_) => "TSIndexSignature",
            Self::TSCallSignatureDeclaration(_) => "TSCallSignatureDeclaration",
            Self::TSMethodSignature(_) => "TSMethodSignature",
            Self::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration",
            Self::TSIndexSignatureName(_) => "TSIndexSignatureName",
            Self::TSInterfaceHeritage(_) => "TSInterfaceHeritage",
            Self::TSTypePredicate(_) => "TSTypePredicate",
            Self::TSModuleDeclaration(_) => "TSModuleDeclaration",
            Self::TSGlobalDeclaration(_) => "TSGlobalDeclaration",
            Self::TSModuleBlock(_) => "TSModuleBlock",
            Self::TSTypeLiteral(_) => "TSTypeLiteral",
            Self::TSInferType(_) => "TSInferType",
            Self::TSTypeQuery(_) => "TSTypeQuery",
            Self::TSImportType(_) => "TSImportType",
            Self::TSImportTypeQualifiedName(_) => "TSImportTypeQualifiedName",
            Self::TSFunctionType(_) => "TSFunctionType",
            Self::TSConstructorType(_) => "TSConstructorType",
            Self::TSMappedType(_) => "TSMappedType",
            Self::TSTemplateLiteralType(_) => "TSTemplateLiteralType",
            Self::TSAsExpression(_) => "TSAsExpression",
            Self::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
            Self::TSTypeAssertion(_) => "TSTypeAssertion",
            Self::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
            Self::TSExternalModuleReference(_) => "TSExternalModuleReference",
            Self::TSNonNullExpression(_) => "TSNonNullExpression",
            Self::Decorator(_) => "Decorator",
            Self::TSExportAssignment(_) => "TSExportAssignment",
            Self::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration",
            Self::TSInstantiationExpression(_) => "TSInstantiationExpression",
            Self::JSDocNullableType(_) => "JSDocNullableType",
            Self::JSDocNonNullableType(_) => "JSDocNonNullableType",
            Self::JSDocUnknownType(_) => "JSDocUnknownType",
        }
    }
}

impl<'a> AstNode<'a, Program<'a>> {
    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.inner.source_type
    }

    #[inline]
    pub fn source_text(&self) -> &'a str {
        self.inner.source_text
    }

    #[inline]
    pub fn comments(&self) -> &Vec<'a, Comment> {
        &self.inner.comments
    }

    #[inline]
    pub fn hashbang(&self) -> Option<&AstNode<'a, Hashbang<'a>>> {
        let following_span_start = self
            .inner
            .directives
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.body.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.hashbang.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Program(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .body
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: AstNodes::Program(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::Program(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Expression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Expression::BooleanLiteral(s) => {
                AstNodes::BooleanLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::NullLiteral(s) => AstNodes::NullLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Expression::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::BigIntLiteral(s) => {
                AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::RegExpLiteral(s) => {
                AstNodes::RegExpLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::Identifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::MetaProperty(s) => AstNodes::MetaProperty(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Expression::Super(s) => AstNodes::Super(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Expression::ArrayExpression(s) => {
                AstNodes::ArrayExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ArrowFunctionExpression(s) => {
                AstNodes::ArrowFunctionExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::AssignmentExpression(s) => {
                AstNodes::AssignmentExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::AwaitExpression(s) => {
                AstNodes::AwaitExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::BinaryExpression(s) => {
                AstNodes::BinaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ChainExpression(s) => {
                AstNodes::ChainExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ClassExpression(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Expression::ConditionalExpression(s) => {
                AstNodes::ConditionalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::FunctionExpression(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ImportExpression(s) => {
                AstNodes::ImportExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::LogicalExpression(s) => {
                AstNodes::LogicalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::NewExpression(s) => {
                AstNodes::NewExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ObjectExpression(s) => {
                AstNodes::ObjectExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ParenthesizedExpression(s) => {
                AstNodes::ParenthesizedExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::SequenceExpression(s) => {
                AstNodes::SequenceExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TaggedTemplateExpression(s) => {
                AstNodes::TaggedTemplateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::UpdateExpression(s) => {
                AstNodes::UpdateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::YieldExpression(s) => {
                AstNodes::YieldExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::PrivateInExpression(s) => {
                AstNodes::PrivateInExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::JSXElement(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Expression::JSXFragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Expression::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSInstantiationExpression(s) => {
                AstNodes::TSInstantiationExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::V8IntrinsicExpression(s) => {
                AstNodes::V8IntrinsicExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(Expression) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, IdentifierName<'a>> {
    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, IdentifierReference<'a>> {
    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BindingIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, LabelIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ThisExpression> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: AstNodes::ArrayExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ArrayExpressionElement::Elision(s) => {
                AstNodes::Elision(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ArrayExpressionElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, Elision> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ObjectExpression<'a>> {
    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: AstNodes::ObjectExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ObjectPropertyKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNodes::ObjectProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ObjectProperty<'a>> {
    #[inline]
    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::ObjectProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: AstNodes::ObjectProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn method(&self) -> bool {
        self.inner.method
    }

    #[inline]
    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, PropertyKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNodes::PrivateIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(PropertyKey) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
        let following_span_start = self
            .inner
            .expressions
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: AstNodes::TemplateLiteral(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn expressions(&self) -> &AstNode<'a, Vec<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: AstNodes::TemplateLiteral(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    pub fn tag(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.quasi.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.tag,
            allocator: self.allocator,
            parent: AstNodes::TaggedTemplateExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.inner.quasi.span().start;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TaggedTemplateExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn quasi(&self) -> &AstNode<'a, TemplateLiteral<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasi,
            allocator: self.allocator,
            parent: AstNodes::TaggedTemplateExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TemplateElement<'a>> {
    #[inline]
    pub fn value(&self) -> &TemplateElementValue<'a> {
        &self.inner.value
    }

    #[inline]
    pub fn tail(&self) -> bool {
        self.inner.tail
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, MemberExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                AstNodes::ComputedMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            MemberExpression::StaticMemberExpression(s) => {
                AstNodes::StaticMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            MemberExpression::PrivateFieldExpression(s) => {
                AstNodes::PrivateFieldExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ComputedMemberExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.expression.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::ComputedMemberExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ComputedMemberExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, StaticMemberExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.property.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::StaticMemberExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: AstNodes::StaticMemberExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.field.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::PrivateFieldExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn field(&self) -> &AstNode<'a, PrivateIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.field,
            allocator: self.allocator,
            parent: AstNodes::PrivateFieldExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, CallExpression<'a>> {
    #[inline]
    pub fn callee(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: AstNodes::CallExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::CallExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: AstNodes::CallExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, NewExpression<'a>> {
    #[inline]
    pub fn callee(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: AstNodes::NewExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::NewExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: AstNodes::NewExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, MetaProperty<'a>> {
    #[inline]
    pub fn meta(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self.inner.property.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.meta,
            allocator: self.allocator,
            parent: AstNodes::MetaProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: AstNodes::MetaProperty(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, SpreadElement<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::SpreadElement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Argument<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Argument::SpreadElement(s) => AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            it @ match_expression!(Argument) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, UpdateExpression<'a>> {
    #[inline]
    pub fn operator(&self) -> UpdateOperator {
        self.inner.operator
    }

    #[inline]
    pub fn prefix(&self) -> bool {
        self.inner.prefix
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, SimpleAssignmentTarget<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::UpdateExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, UnaryExpression<'a>> {
    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::UnaryExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BinaryExpression<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::BinaryExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn operator(&self) -> BinaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::BinaryExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, PrivateInExpression<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, PrivateIdentifier<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::PrivateInExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::PrivateInExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, LogicalExpression<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::LogicalExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn operator(&self) -> LogicalOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::LogicalExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ConditionalExpression<'a>> {
    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.consequent.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::ConditionalExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.alternate.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: AstNodes::ConditionalExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn alternate(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.alternate,
            allocator: self.allocator,
            parent: AstNodes::ConditionalExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AssignmentExpression<'a>> {
    #[inline]
    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::AssignmentExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::AssignmentExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        #[expect(clippy::needless_return)]
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_simple_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target_pattern(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        }
    }
}

impl<'a> AstNode<'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, AssignmentTargetPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNodes::ArrayAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNodes::ObjectAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        let following_span_start = self
            .inner
            .rest
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: AstNodes::ArrayAssignmentTarget(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, AssignmentTargetRest<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrayAssignmentTarget(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>> {
        let following_span_start = self
            .inner
            .rest
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: AstNodes::ObjectAssignmentTarget(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, AssignmentTargetRest<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ObjectAssignmentTarget(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AssignmentTargetRest<'a>> {
    #[inline]
    pub fn target(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.target,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetRest(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNodes::AssignmentTargetWithDefault(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    pub fn binding(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_span_start = self.inner.init.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetWithDefault(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn init(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.init,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetWithDefault(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                AstNodes::AssignmentTargetPropertyIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                AstNodes::AssignmentTargetPropertyProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    pub fn binding(&self) -> &AstNode<'a, IdentifierReference<'a>> {
        let following_span_start = self
            .inner
            .init
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetPropertyIdentifier(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::AssignmentTargetPropertyIdentifier(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self.inner.binding.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetPropertyProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn binding(&self) -> &AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetPropertyProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, SequenceExpression<'a>> {
    #[inline]
    pub fn expressions(&self) -> &AstNode<'a, Vec<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: AstNodes::SequenceExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Super> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, AwaitExpression<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::AwaitExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ChainExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, ChainElement<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ChainExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ChainElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ChainElement::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ChainElement::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(ChainElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ParenthesizedExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ParenthesizedExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Statement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Statement::BlockStatement(s) => {
                AstNodes::BlockStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::BreakStatement(s) => {
                AstNodes::BreakStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ContinueStatement(s) => {
                AstNodes::ContinueStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::DebuggerStatement(s) => {
                AstNodes::DebuggerStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::DoWhileStatement(s) => {
                AstNodes::DoWhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::EmptyStatement(s) => {
                AstNodes::EmptyStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ExpressionStatement(s) => {
                AstNodes::ExpressionStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForInStatement(s) => {
                AstNodes::ForInStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForOfStatement(s) => {
                AstNodes::ForOfStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForStatement(s) => AstNodes::ForStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Statement::IfStatement(s) => AstNodes::IfStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Statement::LabeledStatement(s) => {
                AstNodes::LabeledStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ReturnStatement(s) => {
                AstNodes::ReturnStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::SwitchStatement(s) => {
                AstNodes::SwitchStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ThrowStatement(s) => {
                AstNodes::ThrowStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::TryStatement(s) => AstNodes::TryStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Statement::WhileStatement(s) => {
                AstNodes::WhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::WithStatement(s) => AstNodes::WithStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            it @ match_declaration!(Statement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
            it @ match_module_declaration!(Statement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_module_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, Directive<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::Directive(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn directive(&self) -> Atom<'a> {
        self.inner.directive
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Hashbang<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BlockStatement<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::BlockStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Declaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Declaration::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::ClassDeclaration(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            Declaration::TSTypeAliasDeclaration(s) => {
                AstNodes::TSTypeAliasDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSEnumDeclaration(s) => {
                AstNodes::TSEnumDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSGlobalDeclaration(s) => {
                AstNodes::TSGlobalDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNodes::TSImportEqualsDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, VariableDeclaration<'a>> {
    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declarations(&self) -> &AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.declarations,
            allocator: self.allocator,
            parent: AstNodes::VariableDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, VariableDeclarator<'a>> {
    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.init.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::VariableDeclarator(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .init
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::VariableDeclarator(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::VariableDeclarator(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, EmptyStatement> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ExpressionStatement<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ExpressionStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, IfStatement<'a>> {
    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.consequent.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::IfStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = self.inner.alternate.as_ref().map_or(0, |n| n.span().start);
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: AstNodes::IfStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn alternate(&self) -> Option<&AstNode<'a, Statement<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.alternate.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::IfStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, DoWhileStatement<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = self.inner.test.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::DoWhileStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::DoWhileStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, WhileStatement<'a>> {
    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::WhileStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::WhileStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ForStatement<'a>> {
    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, ForStatementInit<'a>>> {
        let following_span_start = self
            .inner
            .test
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.update.as_ref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ForStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn test(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self
            .inner
            .update
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ForStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn update(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator
            .alloc(self.inner.update.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ForStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ForStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ForStatementInit) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ForInStatement<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, ForStatementLeft<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::ForInStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::ForInStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ForInStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ForStatementLeft<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementLeft::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ForOfStatement<'a>> {
    #[inline]
    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, ForStatementLeft<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::ForOfStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::ForOfStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ForOfStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ContinueStatement<'a>> {
    #[inline]
    pub fn label(&self) -> Option<&AstNode<'a, LabelIdentifier<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ContinueStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BreakStatement<'a>> {
    #[inline]
    pub fn label(&self) -> Option<&AstNode<'a, LabelIdentifier<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::BreakStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ReturnStatement<'a>> {
    #[inline]
    pub fn argument(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ReturnStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, WithStatement<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::WithStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::WithStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, SwitchStatement<'a>> {
    #[inline]
    pub fn discriminant(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.cases.first().map_or(0, |n| n.span().start);
        self.allocator.alloc(AstNode {
            inner: &self.inner.discriminant,
            allocator: self.allocator,
            parent: AstNodes::SwitchStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn cases(&self) -> &AstNode<'a, Vec<'a, SwitchCase<'a>>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.cases,
            allocator: self.allocator,
            parent: AstNodes::SwitchStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, SwitchCase<'a>> {
    #[inline]
    pub fn test(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self
            .inner
            .consequent
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::SwitchCase(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: AstNodes::SwitchCase(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, LabeledStatement<'a>> {
    #[inline]
    pub fn label(&self) -> &AstNode<'a, LabelIdentifier<'a>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: AstNodes::LabeledStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::LabeledStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ThrowStatement<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::ThrowStatement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TryStatement<'a>> {
    #[inline]
    pub fn block(&self) -> &AstNode<'a, BlockStatement<'a>> {
        let following_span_start = self
            .inner
            .handler
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.finalizer.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.block.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TryStatement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn handler(&self) -> Option<&AstNode<'a, CatchClause<'a>>> {
        let following_span_start = self.inner.finalizer.as_deref().map_or(0, |n| n.span().start);
        self.allocator
            .alloc(self.inner.handler.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TryStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn finalizer(&self) -> Option<&AstNode<'a, BlockStatement<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.finalizer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TryStatement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, CatchClause<'a>> {
    #[inline]
    pub fn param(&self) -> Option<&AstNode<'a, CatchParameter<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator
            .alloc(self.inner.param.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::CatchClause(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, BlockStatement<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::CatchClause(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, CatchParameter<'a>> {
    #[inline]
    pub fn pattern(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: AstNodes::CatchParameter(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::CatchParameter(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, DebuggerStatement> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BindingPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            BindingPattern::BindingIdentifier(s) => {
                AstNodes::BindingIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::ObjectPattern(s) => {
                AstNodes::ObjectPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::ArrayPattern(s) => {
                AstNodes::ArrayPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::AssignmentPattern(s) => {
                AstNodes::AssignmentPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, AssignmentPattern<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::AssignmentPattern(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::AssignmentPattern(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ObjectPattern<'a>> {
    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, BindingProperty<'a>>> {
        let following_span_start = self
            .inner
            .rest
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: AstNodes::ObjectPattern(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ObjectPattern(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BindingProperty<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::BindingProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: AstNodes::BindingProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ArrayPattern<'a>> {
    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, Option<BindingPattern<'a>>>> {
        let following_span_start = self
            .inner
            .rest
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: AstNodes::ArrayPattern(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrayPattern(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BindingRestElement<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::BindingRestElement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Function<'a>> {
    #[inline]
    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    #[inline]
    pub fn id(&self) -> Option<&AstNode<'a, BindingIdentifier<'a>>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Function(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn generator(&self) -> bool {
        self.inner.generator
    }

    #[inline]
    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.body.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::Function(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .body
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> Option<&AstNode<'a, FunctionBody<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    #[inline]
    pub fn pife(&self) -> bool {
        self.inner.pife
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, FormalParameters<'a>> {
    #[inline]
    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    #[inline]
    pub fn items(&self) -> &AstNode<'a, Vec<'a, FormalParameter<'a>>> {
        let following_span_start = self
            .inner
            .rest
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.items,
            allocator: self.allocator,
            parent: AstNodes::FormalParameters(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, FormalParameterRest<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameters(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, FormalParameter<'a>> {
    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.pattern.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::FormalParameter(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn pattern(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.initializer.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: AstNodes::FormalParameter(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .initializer
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameter(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn initializer(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameter(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, FormalParameterRest<'a>> {
    #[inline]
    pub fn rest(&self) -> &AstNode<'a, BindingRestElement<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.rest,
            allocator: self.allocator,
            parent: AstNodes::FormalParameterRest(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameterRest(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, FunctionBody<'a>> {
    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .statements
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: AstNodes::FunctionBody(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn statements(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.statements,
            allocator: self.allocator,
            parent: AstNodes::FunctionBody(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ArrowFunctionExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> bool {
        self.inner.expression
    }

    #[inline]
    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrowFunctionExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::ArrowFunctionExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrowFunctionExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, FunctionBody<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::ArrowFunctionExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    #[inline]
    pub fn pife(&self) -> bool {
        self.inner.pife
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, YieldExpression<'a>> {
    #[inline]
    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    #[inline]
    pub fn argument(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::YieldExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Class<'a>> {
    #[inline]
    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self
            .inner
            .id
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_parameters.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.super_class.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::Class(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn id(&self) -> Option<&AstNode<'a, BindingIdentifier<'a>>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.super_class.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Class(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .super_class
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Class(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn super_class(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self
            .inner
            .super_type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.super_class.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Class(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn super_type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .implements
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.super_type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Class(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn implements(&self) -> &AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.implements,
            allocator: self.allocator,
            parent: AstNodes::Class(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, ClassBody<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::Class(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ClassBody<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, ClassElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ClassBody(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ClassElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ClassElement::StaticBlock(s) => AstNodes::StaticBlock(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            ClassElement::MethodDefinition(s) => {
                AstNodes::MethodDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::PropertyDefinition(s) => {
                AstNodes::PropertyDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::AccessorProperty(s) => {
                AstNodes::AccessorProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, MethodDefinition<'a>> {
    #[inline]
    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::MethodDefinition(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::MethodDefinition(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, Function<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.value.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::MethodDefinition(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn kind(&self) -> MethodDefinitionKind {
        self.inner.kind
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, PropertyDefinition<'a>> {
    #[inline]
    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::PropertyDefinition(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::PropertyDefinition(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::PropertyDefinition(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::PropertyDefinition(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, PrivateIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, StaticBlock<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::StaticBlock(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNodes::ImportDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                AstNodes::ExportAllDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNodes::ExportDefaultDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNodes::ExportNamedDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                AstNodes::TSExportAssignment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                AstNodes::TSNamespaceExportDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, AccessorProperty<'a>> {
    #[inline]
    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::AccessorProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::AccessorProperty(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::AccessorProperty(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::AccessorProperty(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportExpression<'a>> {
    #[inline]
    pub fn source(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .options
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::ImportExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn options(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ImportExpression(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportDeclaration<'a>> {
    #[inline]
    pub fn specifiers(&self) -> Option<&AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        let following_span_start = self.inner.source.span().start;
        self.allocator
            .alloc(self.inner.specifiers.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ImportDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn source(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::ImportDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ImportDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNodes::ImportSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNodes::ImportDefaultSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNodes::ImportNamespaceSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ImportSpecifier<'a>> {
    #[inline]
    pub fn imported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_span_start = self.inner.local.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.imported,
            allocator: self.allocator,
            parent: AstNodes::ImportSpecifier(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ImportSpecifier(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ImportDefaultSpecifier(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ImportNamespaceSpecifier(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, WithClause<'a>> {
    #[inline]
    pub fn keyword(&self) -> WithClauseKeyword {
        self.inner.keyword
    }

    #[inline]
    pub fn with_entries(&self) -> &AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.with_entries,
            allocator: self.allocator,
            parent: AstNodes::WithClause(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportAttribute<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, ImportAttributeKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::ImportAttribute(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: AstNodes::ImportAttribute(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ImportAttributeKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ImportAttributeKey::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ExportNamedDeclaration<'a>> {
    #[inline]
    pub fn declaration(&self) -> Option<&AstNode<'a, Declaration<'a>>> {
        let following_span_start = self
            .inner
            .specifiers
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.source.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.with_clause.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.declaration.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ExportNamedDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn specifiers(&self) -> &AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
        let following_span_start = self
            .inner
            .source
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.with_clause.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.specifiers,
            allocator: self.allocator,
            parent: AstNodes::ExportNamedDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn source(&self) -> Option<&AstNode<'a, StringLiteral<'a>>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        self.allocator
            .alloc(self.inner.source.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ExportNamedDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ExportNamedDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    pub fn declaration(&self) -> &AstNode<'a, ExportDefaultDeclarationKind<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.declaration,
            allocator: self.allocator,
            parent: AstNodes::ExportDefaultDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ExportAllDeclaration<'a>> {
    #[inline]
    pub fn exported(&self) -> Option<&AstNode<'a, ModuleExportName<'a>>> {
        let following_span_start = self.inner.source.span().start;
        self.allocator
            .alloc(self.inner.exported.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ExportAllDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn source(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::ExportAllDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ExportAllDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ExportSpecifier<'a>> {
    #[inline]
    pub fn local(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_span_start = self.inner.exported.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ExportSpecifier(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn exported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: AstNodes::ExportSpecifier(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNodes::Class(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, ModuleExportName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleExportName::IdentifierName(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleExportName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleExportName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, V8IntrinsicExpression<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::V8IntrinsicExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: AstNodes::V8IntrinsicExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BooleanLiteral> {
    #[inline]
    pub fn value(&self) -> bool {
        self.inner.value
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, NullLiteral> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, NumericLiteral<'a>> {
    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(&self) -> NumberBase {
        self.inner.base
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, StringLiteral<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(&self) -> BigintBase {
        self.inner.base
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    pub fn regex(&self) -> &RegExp<'a> {
        &self.inner.regex
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXElement<'a>> {
    #[inline]
    pub fn opening_element(&self) -> &AstNode<'a, JSXOpeningElement<'a>> {
        let following_span_start = self
            .inner
            .children
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.closing_element.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.opening_element.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::JSXElement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn children(&self) -> &AstNode<'a, Vec<'a, JSXChild<'a>>> {
        let following_span_start = self
            .inner
            .closing_element
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: AstNodes::JSXElement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn closing_element(&self) -> Option<&AstNode<'a, JSXClosingElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.closing_element.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::JSXElement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXOpeningElement<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXElementName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.attributes.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXOpeningElement(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .attributes
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::JSXOpeningElement(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn attributes(&self) -> &AstNode<'a, Vec<'a, JSXAttributeItem<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes,
            allocator: self.allocator,
            parent: AstNodes::JSXOpeningElement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXClosingElement<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXElementName<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXClosingElement(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXFragment<'a>> {
    #[inline]
    pub fn opening_fragment(&self) -> &AstNode<'a, JSXOpeningFragment> {
        let following_span_start = self
            .inner
            .children
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.closing_fragment.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.opening_fragment,
            allocator: self.allocator,
            parent: AstNodes::JSXFragment(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn children(&self) -> &AstNode<'a, Vec<'a, JSXChild<'a>>> {
        let following_span_start = self.inner.closing_fragment.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: AstNodes::JSXFragment(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn closing_fragment(&self) -> &AstNode<'a, JSXClosingFragment> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.closing_fragment,
            allocator: self.allocator,
            parent: AstNodes::JSXFragment(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXOpeningFragment> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXClosingFragment> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXElementName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXElementName::Identifier(s) => {
                AstNodes::JSXIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXNamespacedName<'a>> {
    #[inline]
    pub fn namespace(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_span_start = self.inner.name.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.namespace,
            allocator: self.allocator,
            parent: AstNodes::JSXNamespacedName(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXNamespacedName(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, JSXMemberExpressionObject<'a>> {
        let following_span_start = self.inner.property.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::JSXMemberExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: AstNodes::JSXMemberExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXExpressionContainer<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, JSXExpression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::JSXExpressionContainer(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXExpression::EmptyExpression(s) => {
                AstNodes::JSXEmptyExpression(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(JSXExpression) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXEmptyExpression> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXAttributeItem<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeItem::Attribute(s) => {
                AstNodes::JSXAttribute(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeItem::SpreadAttribute(s) => {
                AstNodes::JSXSpreadAttribute(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXAttribute<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXAttributeName<'a>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXAttribute(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, JSXAttributeValue<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::JSXAttribute(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXSpreadAttribute<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::JSXSpreadAttribute(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXAttributeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeName::Identifier(s) => {
                AstNodes::JSXIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXAttributeValue<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            JSXAttributeValue::Fragment(s) => {
                AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXChild<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXChild::Text(s) => AstNodes::JSXText(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            JSXChild::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            JSXChild::Fragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            JSXChild::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXChild::Spread(s) => AstNodes::JSXSpreadChild(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, JSXSpreadChild<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::JSXSpreadChild(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSXText<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSThisParameter<'a>> {
    #[inline]
    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSThisParameter(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSEnumDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSEnumDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSEnumBody<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSEnumDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSEnumBody<'a>> {
    #[inline]
    pub fn members(&self) -> &AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: AstNodes::TSEnumBody(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSEnumMember<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, TSEnumMemberName<'a>> {
        let following_span_start = self
            .inner
            .initializer
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSEnumMember(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn initializer(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSEnumMember(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSEnumMemberName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSEnumMemberName::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSEnumMemberName::String(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSEnumMemberName::ComputedString(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSTypeAnnotation<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAnnotation(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSLiteralType<'a>> {
    #[inline]
    pub fn literal(&self) -> &AstNode<'a, TSLiteral<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.literal,
            allocator: self.allocator,
            parent: AstNodes::TSLiteralType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSLiteral<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSLiteral::BooleanLiteral(s) => {
                AstNodes::BooleanLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::BigIntLiteral(s) => AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::StringLiteral(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSType<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSType::TSAnyKeyword(s) => AstNodes::TSAnyKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSBigIntKeyword(s) => {
                AstNodes::TSBigIntKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSBooleanKeyword(s) => {
                AstNodes::TSBooleanKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSIntrinsicKeyword(s) => {
                AstNodes::TSIntrinsicKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSNeverKeyword(s) => AstNodes::TSNeverKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSNullKeyword(s) => AstNodes::TSNullKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSNumberKeyword(s) => {
                AstNodes::TSNumberKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSObjectKeyword(s) => {
                AstNodes::TSObjectKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSStringKeyword(s) => {
                AstNodes::TSStringKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSSymbolKeyword(s) => {
                AstNodes::TSSymbolKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUndefinedKeyword(s) => {
                AstNodes::TSUndefinedKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUnknownKeyword(s) => {
                AstNodes::TSUnknownKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSVoidKeyword(s) => AstNodes::TSVoidKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSArrayType(s) => AstNodes::TSArrayType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSConditionalType(s) => {
                AstNodes::TSConditionalType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSConstructorType(s) => {
                AstNodes::TSConstructorType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSFunctionType(s) => AstNodes::TSFunctionType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSImportType(s) => AstNodes::TSImportType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSIndexedAccessType(s) => {
                AstNodes::TSIndexedAccessType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSInferType(s) => AstNodes::TSInferType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSIntersectionType(s) => {
                AstNodes::TSIntersectionType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSLiteralType(s) => AstNodes::TSLiteralType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSMappedType(s) => AstNodes::TSMappedType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSNamedTupleMember(s) => {
                AstNodes::TSNamedTupleMember(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTemplateLiteralType(s) => {
                AstNodes::TSTemplateLiteralType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSThisType(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTupleType(s) => AstNodes::TSTupleType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeLiteral(s) => AstNodes::TSTypeLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeOperatorType(s) => {
                AstNodes::TSTypeOperator(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypePredicate(s) => {
                AstNodes::TSTypePredicate(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypeQuery(s) => AstNodes::TSTypeQuery(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeReference(s) => {
                AstNodes::TSTypeReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUnionType(s) => AstNodes::TSUnionType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            TSType::TSParenthesizedType(s) => {
                AstNodes::TSParenthesizedType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocNullableType(s) => {
                AstNodes::JSDocNullableType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocNonNullableType(s) => {
                AstNodes::JSDocNonNullableType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocUnknownType(s) => {
                AstNodes::JSDocUnknownType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSConditionalType<'a>> {
    #[inline]
    pub fn check_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.inner.extends_type.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.check_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn extends_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.inner.true_type.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn true_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.inner.false_type.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.true_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn false_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.false_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSUnionType<'a>> {
    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: AstNodes::TSUnionType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSIntersectionType<'a>> {
    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: AstNodes::TSIntersectionType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSParenthesizedType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSParenthesizedType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeOperator<'a>> {
    #[inline]
    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeOperator(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSArrayType<'a>> {
    #[inline]
    pub fn element_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: AstNodes::TSArrayType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSIndexedAccessType<'a>> {
    #[inline]
    pub fn object_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.inner.index_type.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.object_type,
            allocator: self.allocator,
            parent: AstNodes::TSIndexedAccessType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn index_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.index_type,
            allocator: self.allocator,
            parent: AstNodes::TSIndexedAccessType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTupleType<'a>> {
    #[inline]
    pub fn element_types(&self) -> &AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_types,
            allocator: self.allocator,
            parent: AstNodes::TSTupleType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSNamedTupleMember<'a>> {
    #[inline]
    pub fn label(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self.inner.element_type.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: AstNodes::TSNamedTupleMember(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn element_type(&self) -> &AstNode<'a, TSTupleElement<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: AstNodes::TSNamedTupleMember(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSOptionalType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSOptionalType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSRestType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSRestType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                AstNodes::TSOptionalType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTupleElement::TSRestType(s) => AstNodes::TSRestType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
            it @ match_ts_type!(TSTupleElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSAnyKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSStringKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSBooleanKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSNumberKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSNeverKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSIntrinsicKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSUnknownKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSNullKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSUndefinedKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSVoidKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSSymbolKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSThisType> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSObjectKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSBigIntKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeReference<'a>> {
    #[inline]
    pub fn type_name(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_name,
            allocator: self.allocator,
            parent: AstNodes::TSTypeReference(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypeReference(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypeName::QualifiedName(s) => {
                AstNodes::TSQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypeName::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSQualifiedName<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::TSQualifiedName(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::TSQualifiedName(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    pub fn params(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: AstNodes::TSTypeParameterInstantiation(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeParameter<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .constraint
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.default.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::TSTypeParameter(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn constraint(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_span_start = self
            .inner
            .default
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.constraint.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSTypeParameter(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn default(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.default.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSTypeParameter(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn r#in(&self) -> bool {
        self.inner.r#in
    }

    #[inline]
    pub fn out(&self) -> bool {
        self.inner.out
    }

    #[inline]
    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    pub fn params(&self) -> &AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: AstNodes::TSTypeParameterDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.type_annotation.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAliasDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.type_annotation.span().start;
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypeAliasDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAliasDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSClassImplements<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSClassImplements(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSClassImplements(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.extends.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .extends
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSInterfaceDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn extends(&self) -> &AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSInterfaceBody<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSInterfaceBody<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, TSSignature<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceBody(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSPropertySignature<'a>> {
    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::TSPropertySignature(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSPropertySignature(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSSignature<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSSignature::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSPropertySignature(s) => {
                AstNodes::TSPropertySignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                AstNodes::TSCallSignatureDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNodes::TSConstructSignatureDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSMethodSignature(s) => {
                AstNodes::TSMethodSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSIndexSignature<'a>> {
    #[inline]
    pub fn parameters(&self) -> &AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
        let following_span_start = self.inner.type_annotation.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameters,
            allocator: self.allocator,
            parent: AstNodes::TSIndexSignature(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSIndexSignature(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSCallSignatureDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSCallSignatureDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSCallSignatureDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSCallSignatureDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSMethodSignature<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::TSMethodSignature(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn kind(&self) -> TSMethodSignatureKind {
        self.inner.kind
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSMethodSignature(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSMethodSignature(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSMethodSignature(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSMethodSignature(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSConstructSignatureDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSConstructSignatureDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSConstructSignatureDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSIndexSignatureName<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSIndexSignatureName(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSInterfaceHeritage<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceHeritage(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSInterfaceHeritage(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypePredicate<'a>> {
    #[inline]
    pub fn parameter_name(&self) -> &AstNode<'a, TSTypePredicateName<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameter_name,
            allocator: self.allocator,
            parent: AstNodes::TSTypePredicate(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypePredicate(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypePredicateName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypePredicateName::This(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s,
                parent,
                allocator: self.allocator,
                following_span_start: self.following_span_start,
            })),
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSModuleDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, TSModuleDeclarationName<'a>> {
        let following_span_start = self.inner.body.as_ref().map_or(0, |n| n.span().start);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSModuleDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> Option<&AstNode<'a, TSModuleDeclarationBody<'a>>> {
        let following_span_start = 0;
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSModuleDeclaration(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn kind(&self) -> TSModuleDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSModuleDeclarationName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                AstNodes::BindingIdentifier(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSModuleDeclarationBody<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNodes::TSModuleBlock(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSGlobalDeclaration<'a>> {
    #[inline]
    pub fn global_span(&self) -> Span {
        self.inner.global_span
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSModuleBlock<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSGlobalDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSModuleBlock<'a>> {
    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .body
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: AstNodes::TSModuleBlock(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSModuleBlock(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeLiteral<'a>> {
    #[inline]
    pub fn members(&self) -> &AstNode<'a, Vec<'a, TSSignature<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: AstNodes::TSTypeLiteral(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSInferType<'a>> {
    #[inline]
    pub fn type_parameter(&self) -> &AstNode<'a, TSTypeParameter<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSInferType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeQuery<'a>> {
    #[inline]
    pub fn expr_name(&self) -> &AstNode<'a, TSTypeQueryExprName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.expr_name,
            allocator: self.allocator,
            parent: AstNodes::TSTypeQuery(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypeQuery(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeQueryExprName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNodes::TSImportType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type_name(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSImportType<'a>> {
    #[inline]
    pub fn source(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_span_start = self
            .inner
            .options
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.qualifier.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::TSImportType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn options(&self) -> Option<&AstNode<'a, ObjectExpression<'a>>> {
        let following_span_start = self
            .inner
            .qualifier
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSImportType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn qualifier(&self) -> Option<&AstNode<'a, TSImportTypeQualifier<'a>>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.qualifier.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSImportType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSImportType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSImportTypeQualifier<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSImportTypeQualifier::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSImportTypeQualifier::QualifiedName(s) => {
                AstNodes::TSImportTypeQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSImportTypeQualifiedName<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, TSImportTypeQualifier<'a>> {
        let following_span_start = self.inner.right.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::TSImportTypeQualifiedName(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::TSImportTypeQualifiedName(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSFunctionType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSFunctionType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self.inner.return_type.span().start;
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSFunctionType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSFunctionType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSConstructorType<'a>> {
    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSConstructorType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_span_start = self.inner.return_type.span().start;
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSConstructorType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSConstructorType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSMappedType<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self.inner.constraint.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::TSMappedType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn constraint(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self
            .inner
            .name_type
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_annotation.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.constraint,
            allocator: self.allocator,
            parent: AstNodes::TSMappedType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn name_type(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator
            .alloc(self.inner.name_type.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSMappedType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSMappedType(transmute_self(self)),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn optional(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.optional
    }

    #[inline]
    pub fn readonly(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.readonly
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTemplateLiteralType<'a>> {
    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
        let following_span_start = self
            .inner
            .types
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: AstNodes::TSTemplateLiteralType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: AstNodes::TSTemplateLiteralType(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSAsExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.type_annotation.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSAsExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSAsExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSSatisfiesExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.type_annotation.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSSatisfiesExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSSatisfiesExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSTypeAssertion<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.inner.expression.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAssertion(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAssertion(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_span_start = self.inner.module_reference.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSImportEqualsDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn module_reference(&self) -> &AstNode<'a, TSModuleReference<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.module_reference,
            allocator: self.allocator,
            parent: AstNodes::TSImportEqualsDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNodes::TSExternalModuleReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleReference::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleReference::QualifiedName(s) => {
                AstNodes::TSQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, TSExternalModuleReference<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSExternalModuleReference(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSNonNullExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSNonNullExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, Decorator<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::Decorator(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSExportAssignment<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSExportAssignment(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_span_start = 0;
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSNamespaceExportDeclaration(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, TSInstantiationExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_span_start = self.inner.type_arguments.span().start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSInstantiationExpression(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> &AstNode<'a, TSTypeParameterInstantiation<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_arguments.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSInstantiationExpression(transmute_self(self)),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSDocNullableType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::JSDocNullableType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSDocNonNullableType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::JSDocNonNullableType(transmute_self(self)),
            following_span_start,
        })
    }

    #[inline]
    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, JSDocUnknownType> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}
