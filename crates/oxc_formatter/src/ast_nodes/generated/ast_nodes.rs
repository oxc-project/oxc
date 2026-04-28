// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/ast_nodes.rs`.

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_str::Ident;
use oxc_syntax::node::NodeId;

use crate::ast_nodes::AstNode;
use crate::formatter::{
    Format, Formatter,
    trivia::{format_leading_comments, format_trailing_comments},
};

/// Reference-holding parent enum used in `AstNode<...>::parent` fields.
///
/// Each variant holds a `&'me AstNode<...>` borrowing the parent's stack frame.
/// Cheap to copy (single discriminant + reference).
///
/// Also returned (as a reference) by `AstNode<EnumType>::as_ast_nodes()`, where the
/// referenced wrapper is currently allocated in the arena.
#[derive(Clone, Copy)]
pub enum AstNodes<'me, 'a> {
    Dummy(),
    Program(&'me AstNode<'me, 'a, Program<'a>>),
    IdentifierName(&'me AstNode<'me, 'a, IdentifierName<'a>>),
    IdentifierReference(&'me AstNode<'me, 'a, IdentifierReference<'a>>),
    BindingIdentifier(&'me AstNode<'me, 'a, BindingIdentifier<'a>>),
    LabelIdentifier(&'me AstNode<'me, 'a, LabelIdentifier<'a>>),
    ThisExpression(&'me AstNode<'me, 'a, ThisExpression>),
    ArrayExpression(&'me AstNode<'me, 'a, ArrayExpression<'a>>),
    Elision(&'me AstNode<'me, 'a, Elision>),
    ObjectExpression(&'me AstNode<'me, 'a, ObjectExpression<'a>>),
    ObjectProperty(&'me AstNode<'me, 'a, ObjectProperty<'a>>),
    TemplateLiteral(&'me AstNode<'me, 'a, TemplateLiteral<'a>>),
    TaggedTemplateExpression(&'me AstNode<'me, 'a, TaggedTemplateExpression<'a>>),
    TemplateElement(&'me AstNode<'me, 'a, TemplateElement<'a>>),
    ComputedMemberExpression(&'me AstNode<'me, 'a, ComputedMemberExpression<'a>>),
    StaticMemberExpression(&'me AstNode<'me, 'a, StaticMemberExpression<'a>>),
    PrivateFieldExpression(&'me AstNode<'me, 'a, PrivateFieldExpression<'a>>),
    CallExpression(&'me AstNode<'me, 'a, CallExpression<'a>>),
    NewExpression(&'me AstNode<'me, 'a, NewExpression<'a>>),
    MetaProperty(&'me AstNode<'me, 'a, MetaProperty<'a>>),
    SpreadElement(&'me AstNode<'me, 'a, SpreadElement<'a>>),
    UpdateExpression(&'me AstNode<'me, 'a, UpdateExpression<'a>>),
    UnaryExpression(&'me AstNode<'me, 'a, UnaryExpression<'a>>),
    BinaryExpression(&'me AstNode<'me, 'a, BinaryExpression<'a>>),
    PrivateInExpression(&'me AstNode<'me, 'a, PrivateInExpression<'a>>),
    LogicalExpression(&'me AstNode<'me, 'a, LogicalExpression<'a>>),
    ConditionalExpression(&'me AstNode<'me, 'a, ConditionalExpression<'a>>),
    AssignmentExpression(&'me AstNode<'me, 'a, AssignmentExpression<'a>>),
    ArrayAssignmentTarget(&'me AstNode<'me, 'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(&'me AstNode<'me, 'a, ObjectAssignmentTarget<'a>>),
    AssignmentTargetRest(&'me AstNode<'me, 'a, AssignmentTargetRest<'a>>),
    AssignmentTargetWithDefault(&'me AstNode<'me, 'a, AssignmentTargetWithDefault<'a>>),
    AssignmentTargetPropertyIdentifier(
        &'me AstNode<'me, 'a, AssignmentTargetPropertyIdentifier<'a>>,
    ),
    AssignmentTargetPropertyProperty(&'me AstNode<'me, 'a, AssignmentTargetPropertyProperty<'a>>),
    SequenceExpression(&'me AstNode<'me, 'a, SequenceExpression<'a>>),
    Super(&'me AstNode<'me, 'a, Super>),
    AwaitExpression(&'me AstNode<'me, 'a, AwaitExpression<'a>>),
    ChainExpression(&'me AstNode<'me, 'a, ChainExpression<'a>>),
    ParenthesizedExpression(&'me AstNode<'me, 'a, ParenthesizedExpression<'a>>),
    Directive(&'me AstNode<'me, 'a, Directive<'a>>),
    Hashbang(&'me AstNode<'me, 'a, Hashbang<'a>>),
    BlockStatement(&'me AstNode<'me, 'a, BlockStatement<'a>>),
    VariableDeclaration(&'me AstNode<'me, 'a, VariableDeclaration<'a>>),
    VariableDeclarator(&'me AstNode<'me, 'a, VariableDeclarator<'a>>),
    EmptyStatement(&'me AstNode<'me, 'a, EmptyStatement>),
    ExpressionStatement(&'me AstNode<'me, 'a, ExpressionStatement<'a>>),
    IfStatement(&'me AstNode<'me, 'a, IfStatement<'a>>),
    DoWhileStatement(&'me AstNode<'me, 'a, DoWhileStatement<'a>>),
    WhileStatement(&'me AstNode<'me, 'a, WhileStatement<'a>>),
    ForStatement(&'me AstNode<'me, 'a, ForStatement<'a>>),
    ForInStatement(&'me AstNode<'me, 'a, ForInStatement<'a>>),
    ForOfStatement(&'me AstNode<'me, 'a, ForOfStatement<'a>>),
    ContinueStatement(&'me AstNode<'me, 'a, ContinueStatement<'a>>),
    BreakStatement(&'me AstNode<'me, 'a, BreakStatement<'a>>),
    ReturnStatement(&'me AstNode<'me, 'a, ReturnStatement<'a>>),
    WithStatement(&'me AstNode<'me, 'a, WithStatement<'a>>),
    SwitchStatement(&'me AstNode<'me, 'a, SwitchStatement<'a>>),
    SwitchCase(&'me AstNode<'me, 'a, SwitchCase<'a>>),
    LabeledStatement(&'me AstNode<'me, 'a, LabeledStatement<'a>>),
    ThrowStatement(&'me AstNode<'me, 'a, ThrowStatement<'a>>),
    TryStatement(&'me AstNode<'me, 'a, TryStatement<'a>>),
    CatchClause(&'me AstNode<'me, 'a, CatchClause<'a>>),
    CatchParameter(&'me AstNode<'me, 'a, CatchParameter<'a>>),
    DebuggerStatement(&'me AstNode<'me, 'a, DebuggerStatement>),
    AssignmentPattern(&'me AstNode<'me, 'a, AssignmentPattern<'a>>),
    ObjectPattern(&'me AstNode<'me, 'a, ObjectPattern<'a>>),
    BindingProperty(&'me AstNode<'me, 'a, BindingProperty<'a>>),
    ArrayPattern(&'me AstNode<'me, 'a, ArrayPattern<'a>>),
    BindingRestElement(&'me AstNode<'me, 'a, BindingRestElement<'a>>),
    Function(&'me AstNode<'me, 'a, Function<'a>>),
    FormalParameters(&'me AstNode<'me, 'a, FormalParameters<'a>>),
    FormalParameter(&'me AstNode<'me, 'a, FormalParameter<'a>>),
    FormalParameterRest(&'me AstNode<'me, 'a, FormalParameterRest<'a>>),
    FunctionBody(&'me AstNode<'me, 'a, FunctionBody<'a>>),
    ArrowFunctionExpression(&'me AstNode<'me, 'a, ArrowFunctionExpression<'a>>),
    YieldExpression(&'me AstNode<'me, 'a, YieldExpression<'a>>),
    Class(&'me AstNode<'me, 'a, Class<'a>>),
    ClassBody(&'me AstNode<'me, 'a, ClassBody<'a>>),
    MethodDefinition(&'me AstNode<'me, 'a, MethodDefinition<'a>>),
    PropertyDefinition(&'me AstNode<'me, 'a, PropertyDefinition<'a>>),
    PrivateIdentifier(&'me AstNode<'me, 'a, PrivateIdentifier<'a>>),
    StaticBlock(&'me AstNode<'me, 'a, StaticBlock<'a>>),
    AccessorProperty(&'me AstNode<'me, 'a, AccessorProperty<'a>>),
    ImportExpression(&'me AstNode<'me, 'a, ImportExpression<'a>>),
    ImportDeclaration(&'me AstNode<'me, 'a, ImportDeclaration<'a>>),
    ImportSpecifier(&'me AstNode<'me, 'a, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(&'me AstNode<'me, 'a, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(&'me AstNode<'me, 'a, ImportNamespaceSpecifier<'a>>),
    WithClause(&'me AstNode<'me, 'a, WithClause<'a>>),
    ImportAttribute(&'me AstNode<'me, 'a, ImportAttribute<'a>>),
    ExportNamedDeclaration(&'me AstNode<'me, 'a, ExportNamedDeclaration<'a>>),
    ExportDefaultDeclaration(&'me AstNode<'me, 'a, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(&'me AstNode<'me, 'a, ExportAllDeclaration<'a>>),
    ExportSpecifier(&'me AstNode<'me, 'a, ExportSpecifier<'a>>),
    V8IntrinsicExpression(&'me AstNode<'me, 'a, V8IntrinsicExpression<'a>>),
    BooleanLiteral(&'me AstNode<'me, 'a, BooleanLiteral>),
    NullLiteral(&'me AstNode<'me, 'a, NullLiteral>),
    NumericLiteral(&'me AstNode<'me, 'a, NumericLiteral<'a>>),
    StringLiteral(&'me AstNode<'me, 'a, StringLiteral<'a>>),
    BigIntLiteral(&'me AstNode<'me, 'a, BigIntLiteral<'a>>),
    RegExpLiteral(&'me AstNode<'me, 'a, RegExpLiteral<'a>>),
    JSXElement(&'me AstNode<'me, 'a, JSXElement<'a>>),
    JSXOpeningElement(&'me AstNode<'me, 'a, JSXOpeningElement<'a>>),
    JSXClosingElement(&'me AstNode<'me, 'a, JSXClosingElement<'a>>),
    JSXFragment(&'me AstNode<'me, 'a, JSXFragment<'a>>),
    JSXOpeningFragment(&'me AstNode<'me, 'a, JSXOpeningFragment>),
    JSXClosingFragment(&'me AstNode<'me, 'a, JSXClosingFragment>),
    JSXNamespacedName(&'me AstNode<'me, 'a, JSXNamespacedName<'a>>),
    JSXMemberExpression(&'me AstNode<'me, 'a, JSXMemberExpression<'a>>),
    JSXExpressionContainer(&'me AstNode<'me, 'a, JSXExpressionContainer<'a>>),
    JSXEmptyExpression(&'me AstNode<'me, 'a, JSXEmptyExpression>),
    JSXAttribute(&'me AstNode<'me, 'a, JSXAttribute<'a>>),
    JSXSpreadAttribute(&'me AstNode<'me, 'a, JSXSpreadAttribute<'a>>),
    JSXIdentifier(&'me AstNode<'me, 'a, JSXIdentifier<'a>>),
    JSXSpreadChild(&'me AstNode<'me, 'a, JSXSpreadChild<'a>>),
    JSXText(&'me AstNode<'me, 'a, JSXText<'a>>),
    TSThisParameter(&'me AstNode<'me, 'a, TSThisParameter<'a>>),
    TSEnumDeclaration(&'me AstNode<'me, 'a, TSEnumDeclaration<'a>>),
    TSEnumBody(&'me AstNode<'me, 'a, TSEnumBody<'a>>),
    TSEnumMember(&'me AstNode<'me, 'a, TSEnumMember<'a>>),
    TSTypeAnnotation(&'me AstNode<'me, 'a, TSTypeAnnotation<'a>>),
    TSLiteralType(&'me AstNode<'me, 'a, TSLiteralType<'a>>),
    TSConditionalType(&'me AstNode<'me, 'a, TSConditionalType<'a>>),
    TSUnionType(&'me AstNode<'me, 'a, TSUnionType<'a>>),
    TSIntersectionType(&'me AstNode<'me, 'a, TSIntersectionType<'a>>),
    TSParenthesizedType(&'me AstNode<'me, 'a, TSParenthesizedType<'a>>),
    TSTypeOperator(&'me AstNode<'me, 'a, TSTypeOperator<'a>>),
    TSArrayType(&'me AstNode<'me, 'a, TSArrayType<'a>>),
    TSIndexedAccessType(&'me AstNode<'me, 'a, TSIndexedAccessType<'a>>),
    TSTupleType(&'me AstNode<'me, 'a, TSTupleType<'a>>),
    TSNamedTupleMember(&'me AstNode<'me, 'a, TSNamedTupleMember<'a>>),
    TSOptionalType(&'me AstNode<'me, 'a, TSOptionalType<'a>>),
    TSRestType(&'me AstNode<'me, 'a, TSRestType<'a>>),
    TSAnyKeyword(&'me AstNode<'me, 'a, TSAnyKeyword>),
    TSStringKeyword(&'me AstNode<'me, 'a, TSStringKeyword>),
    TSBooleanKeyword(&'me AstNode<'me, 'a, TSBooleanKeyword>),
    TSNumberKeyword(&'me AstNode<'me, 'a, TSNumberKeyword>),
    TSNeverKeyword(&'me AstNode<'me, 'a, TSNeverKeyword>),
    TSIntrinsicKeyword(&'me AstNode<'me, 'a, TSIntrinsicKeyword>),
    TSUnknownKeyword(&'me AstNode<'me, 'a, TSUnknownKeyword>),
    TSNullKeyword(&'me AstNode<'me, 'a, TSNullKeyword>),
    TSUndefinedKeyword(&'me AstNode<'me, 'a, TSUndefinedKeyword>),
    TSVoidKeyword(&'me AstNode<'me, 'a, TSVoidKeyword>),
    TSSymbolKeyword(&'me AstNode<'me, 'a, TSSymbolKeyword>),
    TSThisType(&'me AstNode<'me, 'a, TSThisType>),
    TSObjectKeyword(&'me AstNode<'me, 'a, TSObjectKeyword>),
    TSBigIntKeyword(&'me AstNode<'me, 'a, TSBigIntKeyword>),
    TSTypeReference(&'me AstNode<'me, 'a, TSTypeReference<'a>>),
    TSQualifiedName(&'me AstNode<'me, 'a, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(&'me AstNode<'me, 'a, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(&'me AstNode<'me, 'a, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(&'me AstNode<'me, 'a, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(&'me AstNode<'me, 'a, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(&'me AstNode<'me, 'a, TSClassImplements<'a>>),
    TSInterfaceDeclaration(&'me AstNode<'me, 'a, TSInterfaceDeclaration<'a>>),
    TSInterfaceBody(&'me AstNode<'me, 'a, TSInterfaceBody<'a>>),
    TSPropertySignature(&'me AstNode<'me, 'a, TSPropertySignature<'a>>),
    TSIndexSignature(&'me AstNode<'me, 'a, TSIndexSignature<'a>>),
    TSCallSignatureDeclaration(&'me AstNode<'me, 'a, TSCallSignatureDeclaration<'a>>),
    TSMethodSignature(&'me AstNode<'me, 'a, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(&'me AstNode<'me, 'a, TSConstructSignatureDeclaration<'a>>),
    TSIndexSignatureName(&'me AstNode<'me, 'a, TSIndexSignatureName<'a>>),
    TSInterfaceHeritage(&'me AstNode<'me, 'a, TSInterfaceHeritage<'a>>),
    TSTypePredicate(&'me AstNode<'me, 'a, TSTypePredicate<'a>>),
    TSModuleDeclaration(&'me AstNode<'me, 'a, TSModuleDeclaration<'a>>),
    TSGlobalDeclaration(&'me AstNode<'me, 'a, TSGlobalDeclaration<'a>>),
    TSModuleBlock(&'me AstNode<'me, 'a, TSModuleBlock<'a>>),
    TSTypeLiteral(&'me AstNode<'me, 'a, TSTypeLiteral<'a>>),
    TSInferType(&'me AstNode<'me, 'a, TSInferType<'a>>),
    TSTypeQuery(&'me AstNode<'me, 'a, TSTypeQuery<'a>>),
    TSImportType(&'me AstNode<'me, 'a, TSImportType<'a>>),
    TSImportTypeQualifiedName(&'me AstNode<'me, 'a, TSImportTypeQualifiedName<'a>>),
    TSFunctionType(&'me AstNode<'me, 'a, TSFunctionType<'a>>),
    TSConstructorType(&'me AstNode<'me, 'a, TSConstructorType<'a>>),
    TSMappedType(&'me AstNode<'me, 'a, TSMappedType<'a>>),
    TSTemplateLiteralType(&'me AstNode<'me, 'a, TSTemplateLiteralType<'a>>),
    TSAsExpression(&'me AstNode<'me, 'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(&'me AstNode<'me, 'a, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(&'me AstNode<'me, 'a, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(&'me AstNode<'me, 'a, TSImportEqualsDeclaration<'a>>),
    TSExternalModuleReference(&'me AstNode<'me, 'a, TSExternalModuleReference<'a>>),
    TSNonNullExpression(&'me AstNode<'me, 'a, TSNonNullExpression<'a>>),
    Decorator(&'me AstNode<'me, 'a, Decorator<'a>>),
    TSExportAssignment(&'me AstNode<'me, 'a, TSExportAssignment<'a>>),
    TSNamespaceExportDeclaration(&'me AstNode<'me, 'a, TSNamespaceExportDeclaration<'a>>),
    TSInstantiationExpression(&'me AstNode<'me, 'a, TSInstantiationExpression<'a>>),
    JSDocNullableType(&'me AstNode<'me, 'a, JSDocNullableType<'a>>),
    JSDocNonNullableType(&'me AstNode<'me, 'a, JSDocNonNullableType<'a>>),
    JSDocUnknownType(&'me AstNode<'me, 'a, JSDocUnknownType>),
}

impl<'me, 'a> AstNodes<'me, 'a> {
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

impl<'me, 'a> AstNode<'me, 'a, Program<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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
    pub fn hashbang<'this>(&'this self) -> Option<AstNode<'this, 'a, Hashbang<'a>>> {
        let following_span_start = self
            .inner
            .directives
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.body.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.hashbang.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::Program(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn directives<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .body
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.directives,
            parent: AstNodes::Program(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode { inner: &self.inner.body, parent: AstNodes::Program(self), following_span_start }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Expression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            Expression::BooleanLiteral(s) => AstNodes::BooleanLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::NullLiteral(s) => AstNodes::NullLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::NumericLiteral(s) => AstNodes::NumericLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::BigIntLiteral(s) => AstNodes::BigIntLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::RegExpLiteral(s) => AstNodes::RegExpLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::StringLiteral(s) => AstNodes::StringLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::TemplateLiteral(s) => AstNodes::TemplateLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::Identifier(s) => AstNodes::IdentifierReference(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::MetaProperty(s) => AstNodes::MetaProperty(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::Super(s) => AstNodes::Super(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ArrayExpression(s) => AstNodes::ArrayExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ArrowFunctionExpression(s) => {
                AstNodes::ArrowFunctionExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::AssignmentExpression(s) => {
                AstNodes::AssignmentExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::AwaitExpression(s) => AstNodes::AwaitExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::BinaryExpression(s) => {
                AstNodes::BinaryExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::CallExpression(s) => AstNodes::CallExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ChainExpression(s) => AstNodes::ChainExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ClassExpression(s) => AstNodes::Class(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ConditionalExpression(s) => {
                AstNodes::ConditionalExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::FunctionExpression(s) => AstNodes::Function(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ImportExpression(s) => {
                AstNodes::ImportExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::LogicalExpression(s) => {
                AstNodes::LogicalExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::NewExpression(s) => AstNodes::NewExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::ObjectExpression(s) => {
                AstNodes::ObjectExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ParenthesizedExpression(s) => {
                AstNodes::ParenthesizedExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::SequenceExpression(s) => {
                AstNodes::SequenceExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TaggedTemplateExpression(s) => {
                AstNodes::TaggedTemplateExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ThisExpression(s) => AstNodes::ThisExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::UnaryExpression(s) => AstNodes::UnaryExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::UpdateExpression(s) => {
                AstNodes::UpdateExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::YieldExpression(s) => AstNodes::YieldExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::PrivateInExpression(s) => {
                AstNodes::PrivateInExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::JSXElement(s) => AstNodes::JSXElement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::JSXFragment(s) => AstNodes::JSXFragment(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::TSAsExpression(s) => AstNodes::TSAsExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSTypeAssertion(s) => AstNodes::TSTypeAssertion(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Expression::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSInstantiationExpression(s) => {
                AstNodes::TSInstantiationExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::V8IntrinsicExpression(s) => {
                AstNodes::V8IntrinsicExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(Expression) => {
                return AstNode {
                    inner: it.to_member_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, IdentifierName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, IdentifierReference<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, BindingIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, LabelIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, ThisExpression> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ArrayExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, ArrayExpressionElement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.elements,
            parent: AstNodes::ArrayExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ArrayExpressionElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNodes::SpreadElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ArrayExpressionElement::Elision(s) => AstNodes::Elision(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            it @ match_expression!(ArrayExpressionElement) => {
                return AstNode {
                    inner: it.to_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, Elision> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ObjectExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, ObjectPropertyKind<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.properties,
            parent: AstNodes::ObjectExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ObjectPropertyKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNodes::ObjectProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNodes::SpreadElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ObjectProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::ObjectProperty(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn value<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.value,
            parent: AstNodes::ObjectProperty(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, PropertyKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNodes::PrivateIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(PropertyKey) => {
                return AstNode {
                    inner: it.to_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TemplateLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn quasis<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TemplateElement<'a>>> {
        let following_span_start = self
            .inner
            .expressions
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.quasis,
            parent: AstNodes::TemplateLiteral(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn expressions<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expressions,
            parent: AstNodes::TemplateLiteral(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TaggedTemplateExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn tag<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.quasi.span().start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.tag,
            parent: AstNodes::TaggedTemplateExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.inner.quasi.span().start;
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TaggedTemplateExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn quasi<'this>(&'this self) -> AstNode<'this, 'a, TemplateLiteral<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.quasi,
            parent: AstNodes::TaggedTemplateExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TemplateElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, MemberExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                AstNodes::ComputedMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            MemberExpression::StaticMemberExpression(s) => {
                AstNodes::StaticMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            MemberExpression::PrivateFieldExpression(s) => {
                AstNodes::PrivateFieldExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ComputedMemberExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.expression.span().start;
        AstNode {
            inner: &self.inner.object,
            parent: AstNodes::ComputedMemberExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::ComputedMemberExpression(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, StaticMemberExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.property.span().start;
        AstNode {
            inner: &self.inner.object,
            parent: AstNodes::StaticMemberExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn property<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.property,
            parent: AstNodes::StaticMemberExpression(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, PrivateFieldExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.field.span().start;
        AstNode {
            inner: &self.inner.object,
            parent: AstNodes::PrivateFieldExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn field<'this>(&'this self) -> AstNode<'this, 'a, PrivateIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.field,
            parent: AstNodes::PrivateFieldExpression(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, CallExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn callee<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.callee,
            parent: AstNodes::CallExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::CallExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn arguments<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.arguments,
            parent: AstNodes::CallExpression(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, NewExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn callee<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.callee,
            parent: AstNodes::NewExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::NewExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn arguments<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.arguments,
            parent: AstNodes::NewExpression(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, MetaProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn meta<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self.inner.property.span().start;
        AstNode {
            inner: &self.inner.meta,
            parent: AstNodes::MetaProperty(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn property<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.property,
            parent: AstNodes::MetaProperty(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, SpreadElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::SpreadElement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Argument<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            Argument::SpreadElement(s) => AstNodes::SpreadElement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            it @ match_expression!(Argument) => {
                return AstNode {
                    inner: it.to_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, UpdateExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> UpdateOperator {
        self.inner.operator
    }

    #[inline]
    pub fn prefix(&self) -> bool {
        self.inner.prefix
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, SimpleAssignmentTarget<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::UpdateExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, UnaryExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::UnaryExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, BinaryExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::BinaryExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn operator(&self) -> BinaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::BinaryExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, PrivateInExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, PrivateIdentifier<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::PrivateInExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::PrivateInExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, LogicalExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::LogicalExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn operator(&self) -> LogicalOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::LogicalExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ConditionalExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.consequent.span().start;
        AstNode {
            inner: &self.inner.test,
            parent: AstNodes::ConditionalExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn consequent<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.alternate.span().start;
        AstNode {
            inner: &self.inner.consequent,
            parent: AstNodes::ConditionalExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn alternate<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.alternate,
            parent: AstNodes::ConditionalExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, AssignmentTarget<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::AssignmentExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::AssignmentExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        #[expect(clippy::needless_return)]
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                return AstNode {
                    inner: it.to_simple_assignment_target(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                return AstNode {
                    inner: it.to_assignment_target_pattern(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNodes::TSAsExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                return AstNode {
                    inner: it.to_member_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNodes::ArrayAssignmentTarget(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNodes::ObjectAssignmentTarget(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements<'this>(
        &'this self,
    ) -> AstNode<'this, 'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.elements,
            parent: AstNodes::ArrayAssignmentTarget(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn rest<'this>(&'this self) -> Option<AstNode<'this, 'a, AssignmentTargetRest<'a>>> {
        let following_span_start = 0;
        self.inner.rest.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ArrayAssignmentTarget(self),
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

impl<'me, 'a> AstNode<'me, 'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties<'this>(
        &'this self,
    ) -> AstNode<'this, 'a, Vec<'a, AssignmentTargetProperty<'a>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.properties,
            parent: AstNodes::ObjectAssignmentTarget(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn rest<'this>(&'this self) -> Option<AstNode<'this, 'a, AssignmentTargetRest<'a>>> {
        let following_span_start = 0;
        self.inner.rest.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ObjectAssignmentTarget(self),
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

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetRest<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn target<'this>(&'this self) -> AstNode<'this, 'a, AssignmentTarget<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.target,
            parent: AstNodes::AssignmentTargetRest(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNodes::AssignmentTargetWithDefault(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                return AstNode {
                    inner: it.to_assignment_target(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn binding<'this>(&'this self) -> AstNode<'this, 'a, AssignmentTarget<'a>> {
        let following_span_start = self.inner.init.span().start;
        AstNode {
            inner: &self.inner.binding,
            parent: AstNodes::AssignmentTargetWithDefault(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn init<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.init,
            parent: AstNodes::AssignmentTargetWithDefault(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetProperty<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                AstNodes::AssignmentTargetPropertyIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                AstNodes::AssignmentTargetPropertyProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn binding<'this>(&'this self) -> AstNode<'this, 'a, IdentifierReference<'a>> {
        let following_span_start = self
            .inner
            .init
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.binding,
            parent: AstNodes::AssignmentTargetPropertyIdentifier(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn init<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.init.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::AssignmentTargetPropertyIdentifier(self),
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

impl<'me, 'a> AstNode<'me, 'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self.inner.binding.span().start;
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::AssignmentTargetPropertyProperty(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn binding<'this>(&'this self) -> AstNode<'this, 'a, AssignmentTargetMaybeDefault<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.binding,
            parent: AstNodes::AssignmentTargetPropertyProperty(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, SequenceExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expressions<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expressions,
            parent: AstNodes::SequenceExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Super> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, AwaitExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::AwaitExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ChainExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, ChainElement<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::ChainExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ChainElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ChainElement::CallExpression(s) => AstNodes::CallExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            ChainElement::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(ChainElement) => {
                return AstNode {
                    inner: it.to_member_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ParenthesizedExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::ParenthesizedExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Statement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            Statement::BlockStatement(s) => AstNodes::BlockStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::BreakStatement(s) => AstNodes::BreakStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::ContinueStatement(s) => {
                AstNodes::ContinueStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::DebuggerStatement(s) => {
                AstNodes::DebuggerStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::DoWhileStatement(s) => {
                AstNodes::DoWhileStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::EmptyStatement(s) => AstNodes::EmptyStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::ExpressionStatement(s) => {
                AstNodes::ExpressionStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForInStatement(s) => AstNodes::ForInStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::ForOfStatement(s) => AstNodes::ForOfStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::ForStatement(s) => AstNodes::ForStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::IfStatement(s) => AstNodes::IfStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::LabeledStatement(s) => {
                AstNodes::LabeledStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ReturnStatement(s) => AstNodes::ReturnStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::SwitchStatement(s) => AstNodes::SwitchStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::ThrowStatement(s) => AstNodes::ThrowStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::TryStatement(s) => AstNodes::TryStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::WhileStatement(s) => AstNodes::WhileStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Statement::WithStatement(s) => AstNodes::WithStatement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            it @ match_declaration!(Statement) => {
                return AstNode {
                    inner: it.to_declaration(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
            it @ match_module_declaration!(Statement) => {
                return AstNode {
                    inner: it.to_module_declaration(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, Directive<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::Directive(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn directive(&self) -> Str<'a> {
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

impl<'me, 'a> AstNode<'me, 'a, Hashbang<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
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

impl<'me, 'a> AstNode<'me, 'a, BlockStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::BlockStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Declaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            Declaration::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::FunctionDeclaration(s) => AstNodes::Function(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Declaration::ClassDeclaration(s) => AstNodes::Class(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            Declaration::TSTypeAliasDeclaration(s) => {
                AstNodes::TSTypeAliasDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSEnumDeclaration(s) => {
                AstNodes::TSEnumDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSGlobalDeclaration(s) => {
                AstNodes::TSGlobalDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNodes::TSImportEqualsDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, VariableDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declarations<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, VariableDeclarator<'a>>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.declarations,
            parent: AstNodes::VariableDeclaration(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, VariableDeclarator<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.init.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::VariableDeclarator(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .init
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::VariableDeclarator(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn init<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.init.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::VariableDeclarator(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, EmptyStatement> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ExpressionStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::ExpressionStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, IfStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.consequent.span().start;
        AstNode {
            inner: &self.inner.test,
            parent: AstNodes::IfStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn consequent<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = self.inner.alternate.as_ref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.consequent,
            parent: AstNodes::IfStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn alternate<'this>(&'this self) -> Option<AstNode<'this, 'a, Statement<'a>>> {
        let following_span_start = 0;
        self.inner.alternate.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::IfStatement(self),
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

impl<'me, 'a> AstNode<'me, 'a, DoWhileStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = self.inner.test.span().start;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::DoWhileStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn test<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.test,
            parent: AstNodes::DoWhileStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, WhileStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.test,
            parent: AstNodes::WhileStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::WhileStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ForStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn init<'this>(&'this self) -> Option<AstNode<'this, 'a, ForStatementInit<'a>>> {
        let following_span_start = self
            .inner
            .test
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.update.as_ref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.inner.init.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ForStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn test<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self
            .inner
            .update
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.inner.test.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ForStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn update<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.inner.update.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ForStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::ForStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ForStatementInit<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ForStatementInit) => {
                return AstNode {
                    inner: it.to_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ForInStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, ForStatementLeft<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::ForInStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::ForInStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::ForInStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ForStatementLeft<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ForStatementLeft::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                return AstNode {
                    inner: it.to_assignment_target(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ForOfStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, ForStatementLeft<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::ForOfStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::ForOfStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::ForOfStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ContinueStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'this>(&'this self) -> Option<AstNode<'this, 'a, LabelIdentifier<'a>>> {
        let following_span_start = 0;
        self.inner.label.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ContinueStatement(self),
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

impl<'me, 'a> AstNode<'me, 'a, BreakStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'this>(&'this self) -> Option<AstNode<'this, 'a, LabelIdentifier<'a>>> {
        let following_span_start = 0;
        self.inner.label.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::BreakStatement(self),
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

impl<'me, 'a> AstNode<'me, 'a, ReturnStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = 0;
        self.inner.argument.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ReturnStatement(self),
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

impl<'me, 'a> AstNode<'me, 'a, WithStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.object,
            parent: AstNodes::WithStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::WithStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, SwitchStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn discriminant<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.cases.first().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.discriminant,
            parent: AstNodes::SwitchStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn cases<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, SwitchCase<'a>>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.cases,
            parent: AstNodes::SwitchStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, SwitchCase<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self
            .inner
            .consequent
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.test.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::SwitchCase(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn consequent<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.consequent,
            parent: AstNodes::SwitchCase(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, LabeledStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'this>(&'this self) -> AstNode<'this, 'a, LabelIdentifier<'a>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.label,
            parent: AstNodes::LabeledStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Statement<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::LabeledStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ThrowStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::ThrowStatement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TryStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn block<'this>(&'this self) -> AstNode<'this, 'a, BlockStatement<'a>> {
        let following_span_start = self
            .inner
            .handler
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.finalizer.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.block.as_ref(),
            parent: AstNodes::TryStatement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn handler<'this>(&'this self) -> Option<AstNode<'this, 'a, CatchClause<'a>>> {
        let following_span_start = self.inner.finalizer.as_deref().map_or(0, |n| n.span().start);
        self.inner.handler.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TryStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn finalizer<'this>(&'this self) -> Option<AstNode<'this, 'a, BlockStatement<'a>>> {
        let following_span_start = 0;
        self.inner.finalizer.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TryStatement(self),
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

impl<'me, 'a> AstNode<'me, 'a, CatchClause<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn param<'this>(&'this self) -> Option<AstNode<'this, 'a, CatchParameter<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.inner.param.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::CatchClause(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, BlockStatement<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.body.as_ref(),
            parent: AstNodes::CatchClause(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, CatchParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn pattern<'this>(&'this self) -> AstNode<'this, 'a, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.pattern,
            parent: AstNodes::CatchParameter(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::CatchParameter(self),
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

impl<'me, 'a> AstNode<'me, 'a, DebuggerStatement> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, BindingPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            BindingPattern::BindingIdentifier(s) => {
                AstNodes::BindingIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::ObjectPattern(s) => AstNodes::ObjectPattern(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            BindingPattern::ArrayPattern(s) => AstNodes::ArrayPattern(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            BindingPattern::AssignmentPattern(s) => {
                AstNodes::AssignmentPattern(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, AssignmentPattern<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, BindingPattern<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::AssignmentPattern(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::AssignmentPattern(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ObjectPattern<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, BindingProperty<'a>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.properties,
            parent: AstNodes::ObjectPattern(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn rest<'this>(&'this self) -> Option<AstNode<'this, 'a, BindingRestElement<'a>>> {
        let following_span_start = 0;
        self.inner.rest.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ObjectPattern(self),
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

impl<'me, 'a> AstNode<'me, 'a, BindingProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::BindingProperty(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn value<'this>(&'this self) -> AstNode<'this, 'a, BindingPattern<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.value,
            parent: AstNodes::BindingProperty(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, ArrayPattern<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Option<BindingPattern<'a>>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.elements,
            parent: AstNodes::ArrayPattern(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn rest<'this>(&'this self) -> Option<AstNode<'this, 'a, BindingRestElement<'a>>> {
        let following_span_start = 0;
        self.inner.rest.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ArrayPattern(self),
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

impl<'me, 'a> AstNode<'me, 'a, BindingRestElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, BindingPattern<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::BindingRestElement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Function<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    #[inline]
    pub fn id<'this>(&'this self) -> Option<AstNode<'this, 'a, BindingIdentifier<'a>>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.inner.id.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::Function(self),
            following_span_start,
        })
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
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::Function(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn this_param<'this>(&'this self) -> Option<AstNode<'this, 'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.this_param.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::Function(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.body.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::Function(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .body
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.return_type.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::Function(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'this>(&'this self) -> Option<AstNode<'this, 'a, FunctionBody<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.body.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::Function(self),
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

impl<'me, 'a> AstNode<'me, 'a, FormalParameters<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    #[inline]
    pub fn items<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, FormalParameter<'a>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.items,
            parent: AstNodes::FormalParameters(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn rest<'this>(&'this self) -> Option<AstNode<'this, 'a, FormalParameterRest<'a>>> {
        let following_span_start = 0;
        self.inner.rest.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::FormalParameters(self),
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

impl<'me, 'a> AstNode<'me, 'a, FormalParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn decorators<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.pattern.span().start;
        AstNode {
            inner: &self.inner.decorators,
            parent: AstNodes::FormalParameter(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn pattern<'this>(&'this self) -> AstNode<'this, 'a, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.initializer.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.pattern,
            parent: AstNodes::FormalParameter(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .initializer
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::FormalParameter(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn initializer<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.initializer.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::FormalParameter(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, FormalParameterRest<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn decorators<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.rest.span().start;
        AstNode {
            inner: &self.inner.decorators,
            parent: AstNodes::FormalParameterRest(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn rest<'this>(&'this self) -> AstNode<'this, 'a, BindingRestElement<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.rest,
            parent: AstNodes::FormalParameterRest(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::FormalParameterRest(self),
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

impl<'me, 'a> AstNode<'me, 'a, FunctionBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn directives<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .statements
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.directives,
            parent: AstNodes::FunctionBody(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn statements<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.statements,
            parent: AstNodes::FunctionBody(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ArrowFunctionExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(&self) -> bool {
        self.inner.expression
    }

    #[inline]
    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ArrowFunctionExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::ArrowFunctionExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.inner.body.span().start;
        self.inner.return_type.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ArrowFunctionExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, FunctionBody<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.body.as_ref(),
            parent: AstNodes::ArrowFunctionExpression(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, YieldExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.argument.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::YieldExpression(self),
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

impl<'me, 'a> AstNode<'me, 'a, Class<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Decorator<'a>>> {
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
        AstNode {
            inner: &self.inner.decorators,
            parent: AstNodes::Class(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn id<'this>(&'this self) -> Option<AstNode<'this, 'a, BindingIdentifier<'a>>> {
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
        self.inner.id.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .super_class
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn super_class<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self
            .inner
            .super_type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.inner.super_class.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn super_type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .implements
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.inner.super_type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn implements<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSClassImplements<'a>>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.implements,
            parent: AstNodes::Class(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, ClassBody<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.body.as_ref(),
            parent: AstNodes::Class(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, ClassBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, ClassElement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode { inner: &self.inner.body, parent: AstNodes::ClassBody(self), following_span_start }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ClassElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ClassElement::StaticBlock(s) => AstNodes::StaticBlock(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            ClassElement::MethodDefinition(s) => {
                AstNodes::MethodDefinition(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::PropertyDefinition(s) => {
                AstNodes::PropertyDefinition(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::AccessorProperty(s) => {
                AstNodes::AccessorProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, MethodDefinition<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        AstNode {
            inner: &self.inner.decorators,
            parent: AstNodes::MethodDefinition(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::MethodDefinition(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn value<'this>(&'this self) -> AstNode<'this, 'a, Function<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.value.as_ref(),
            parent: AstNodes::MethodDefinition(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, PropertyDefinition<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        AstNode {
            inner: &self.inner.decorators,
            parent: AstNodes::PropertyDefinition(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::PropertyDefinition(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::PropertyDefinition(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.value.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::PropertyDefinition(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, PrivateIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, StaticBlock<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::StaticBlock(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ModuleDeclaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNodes::ImportDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                AstNodes::ExportAllDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNodes::ExportDefaultDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNodes::ExportNamedDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                AstNodes::TSExportAssignment(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                AstNodes::TSNamespaceExportDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, AccessorProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        AstNode {
            inner: &self.inner.decorators,
            parent: AstNodes::AccessorProperty(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::AccessorProperty(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::AccessorProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.value.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::AccessorProperty(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, ImportExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .options
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.source,
            parent: AstNodes::ImportExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn options<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.options.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ImportExpression(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, ImportDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn specifiers<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        let following_span_start = self.inner.source.span().start;
        self.inner.specifiers.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ImportDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn source<'this>(&'this self) -> AstNode<'this, 'a, StringLiteral<'a>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.source,
            parent: AstNodes::ImportDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    #[inline]
    pub fn with_clause<'this>(&'this self) -> Option<AstNode<'this, 'a, WithClause<'a>>> {
        let following_span_start = 0;
        self.inner.with_clause.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ImportDeclaration(self),
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

impl<'me, 'a> AstNode<'me, 'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNodes::ImportSpecifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNodes::ImportDefaultSpecifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNodes::ImportNamespaceSpecifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ImportSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn imported<'this>(&'this self) -> AstNode<'this, 'a, ModuleExportName<'a>> {
        let following_span_start = self.inner.local.span().start;
        AstNode {
            inner: &self.inner.imported,
            parent: AstNodes::ImportSpecifier(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn local<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.local,
            parent: AstNodes::ImportSpecifier(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.local,
            parent: AstNodes::ImportDefaultSpecifier(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.local,
            parent: AstNodes::ImportNamespaceSpecifier(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, WithClause<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn keyword(&self) -> WithClauseKeyword {
        self.inner.keyword
    }

    #[inline]
    pub fn with_entries<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, ImportAttribute<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.with_entries,
            parent: AstNodes::WithClause(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ImportAttribute<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, ImportAttributeKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::ImportAttribute(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn value<'this>(&'this self) -> AstNode<'this, 'a, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.value,
            parent: AstNodes::ImportAttribute(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ImportAttributeKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ImportAttributeKey::Identifier(s) => {
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ExportNamedDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn declaration<'this>(&'this self) -> Option<AstNode<'this, 'a, Declaration<'a>>> {
        let following_span_start = self
            .inner
            .specifiers
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.source.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.with_clause.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        self.inner.declaration.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ExportNamedDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn specifiers<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, ExportSpecifier<'a>>> {
        let following_span_start = self
            .inner
            .source
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.with_clause.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.specifiers,
            parent: AstNodes::ExportNamedDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn source<'this>(&'this self) -> Option<AstNode<'this, 'a, StringLiteral<'a>>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        self.inner.source.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ExportNamedDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    #[inline]
    pub fn with_clause<'this>(&'this self) -> Option<AstNode<'this, 'a, WithClause<'a>>> {
        let following_span_start = 0;
        self.inner.with_clause.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ExportNamedDeclaration(self),
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

impl<'me, 'a> AstNode<'me, 'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn declaration<'this>(&'this self) -> AstNode<'this, 'a, ExportDefaultDeclarationKind<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.declaration,
            parent: AstNodes::ExportDefaultDeclaration(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, ExportAllDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn exported<'this>(&'this self) -> Option<AstNode<'this, 'a, ModuleExportName<'a>>> {
        let following_span_start = self.inner.source.span().start;
        self.inner.exported.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::ExportAllDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn source<'this>(&'this self) -> AstNode<'this, 'a, StringLiteral<'a>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.source,
            parent: AstNodes::ExportAllDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn with_clause<'this>(&'this self) -> Option<AstNode<'this, 'a, WithClause<'a>>> {
        let following_span_start = 0;
        self.inner.with_clause.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::ExportAllDeclaration(self),
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

impl<'me, 'a> AstNode<'me, 'a, ExportSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local<'this>(&'this self) -> AstNode<'this, 'a, ModuleExportName<'a>> {
        let following_span_start = self.inner.exported.span().start;
        AstNode {
            inner: &self.inner.local,
            parent: AstNodes::ExportSpecifier(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn exported<'this>(&'this self) -> AstNode<'this, 'a, ModuleExportName<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.exported,
            parent: AstNodes::ExportSpecifier(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNodes::Function(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNodes::Class(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                return AstNode {
                    inner: it.to_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, ModuleExportName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            ModuleExportName::IdentifierName(s) => {
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleExportName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleExportName::StringLiteral(s) => {
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, V8IntrinsicExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::V8IntrinsicExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn arguments<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.arguments,
            parent: AstNodes::V8IntrinsicExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, BooleanLiteral> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

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

impl<'me, 'a> AstNode<'me, 'a, NullLiteral> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, NumericLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
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

impl<'me, 'a> AstNode<'me, 'a, StringLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
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

impl<'me, 'a> AstNode<'me, 'a, BigIntLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
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

impl<'me, 'a> AstNode<'me, 'a, RegExpLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn regex(&self) -> &RegExp<'a> {
        &self.inner.regex
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
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

impl<'me, 'a> AstNode<'me, 'a, JSXElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn opening_element<'this>(&'this self) -> AstNode<'this, 'a, JSXOpeningElement<'a>> {
        let following_span_start = self
            .inner
            .children
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.closing_element.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.opening_element.as_ref(),
            parent: AstNodes::JSXElement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn children<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, JSXChild<'a>>> {
        let following_span_start = self
            .inner
            .closing_element
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.children,
            parent: AstNodes::JSXElement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn closing_element<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, JSXClosingElement<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.closing_element.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::JSXElement(self),
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

impl<'me, 'a> AstNode<'me, 'a, JSXOpeningElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, JSXElementName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.attributes.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::JSXOpeningElement(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .attributes
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::JSXOpeningElement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn attributes<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, JSXAttributeItem<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.attributes,
            parent: AstNodes::JSXOpeningElement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXClosingElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, JSXElementName<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::JSXClosingElement(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXFragment<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn opening_fragment<'this>(&'this self) -> AstNode<'this, 'a, JSXOpeningFragment> {
        let following_span_start = self
            .inner
            .children
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.closing_fragment.span().start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.opening_fragment,
            parent: AstNodes::JSXFragment(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn children<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, JSXChild<'a>>> {
        let following_span_start = self.inner.closing_fragment.span().start;
        AstNode {
            inner: &self.inner.children,
            parent: AstNodes::JSXFragment(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn closing_fragment<'this>(&'this self) -> AstNode<'this, 'a, JSXClosingFragment> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.closing_fragment,
            parent: AstNodes::JSXFragment(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXOpeningFragment> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXClosingFragment> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXElementName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXElementName::Identifier(s) => AstNodes::JSXIdentifier(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXElementName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::ThisExpression(s) => {
                AstNodes::ThisExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXNamespacedName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn namespace<'this>(&'this self) -> AstNode<'this, 'a, JSXIdentifier<'a>> {
        let following_span_start = self.inner.name.span().start;
        AstNode {
            inner: &self.inner.namespace,
            parent: AstNodes::JSXNamespacedName(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, JSXIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::JSXNamespacedName(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXMemberExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'this>(&'this self) -> AstNode<'this, 'a, JSXMemberExpressionObject<'a>> {
        let following_span_start = self.inner.property.span().start;
        AstNode {
            inner: &self.inner.object,
            parent: AstNodes::JSXMemberExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn property<'this>(&'this self) -> AstNode<'this, 'a, JSXIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.property,
            parent: AstNodes::JSXMemberExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNodes::ThisExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXExpressionContainer<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, JSXExpression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::JSXExpressionContainer(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXExpression::EmptyExpression(s) => {
                AstNodes::JSXEmptyExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(JSXExpression) => {
                return AstNode {
                    inner: it.to_expression(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXEmptyExpression> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXAttributeItem<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXAttributeItem::Attribute(s) => AstNodes::JSXAttribute(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXAttributeItem::SpreadAttribute(s) => {
                AstNodes::JSXSpreadAttribute(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXAttribute<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, JSXAttributeName<'a>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::JSXAttribute(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn value<'this>(&'this self) -> Option<AstNode<'this, 'a, JSXAttributeValue<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.value.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::JSXAttribute(self),
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

impl<'me, 'a> AstNode<'me, 'a, JSXSpreadAttribute<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.argument,
            parent: AstNodes::JSXSpreadAttribute(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXAttributeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXAttributeName::Identifier(s) => AstNodes::JSXIdentifier(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXAttributeName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXAttributeValue<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::Element(s) => AstNodes::JSXElement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXAttributeValue::Fragment(s) => AstNodes::JSXFragment(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Str<'a> {
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

impl<'me, 'a> AstNode<'me, 'a, JSXChild<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            JSXChild::Text(s) => AstNodes::JSXText(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXChild::Element(s) => AstNodes::JSXElement(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXChild::Fragment(s) => AstNodes::JSXFragment(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            JSXChild::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXChild::Spread(s) => AstNodes::JSXSpreadChild(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXSpreadChild<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::JSXSpreadChild(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSXText<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
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

impl<'me, 'a> AstNode<'me, 'a, TSThisParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSThisParameter(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSEnumDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSEnumDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, TSEnumBody<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::TSEnumDeclaration(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSEnumBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn members<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSEnumMember<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.members,
            parent: AstNodes::TSEnumBody(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSEnumMember<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, TSEnumMemberName<'a>> {
        let following_span_start = self
            .inner
            .initializer
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSEnumMember(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn initializer<'this>(&'this self) -> Option<AstNode<'this, 'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.initializer.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSEnumMember(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSEnumMemberName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSEnumMemberName::Identifier(s) => AstNodes::IdentifierName(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSEnumMemberName::String(s) => AstNodes::StringLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSEnumMemberName::ComputedString(s) => {
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNodes::TemplateLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeAnnotation<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSTypeAnnotation(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSLiteralType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn literal<'this>(&'this self) -> AstNode<'this, 'a, TSLiteral<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.literal,
            parent: AstNodes::TSLiteralType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSLiteral<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSLiteral::BooleanLiteral(s) => AstNodes::BooleanLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::NumericLiteral(s) => AstNodes::NumericLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::BigIntLiteral(s) => AstNodes::BigIntLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::StringLiteral(s) => AstNodes::StringLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::TemplateLiteral(s) => AstNodes::TemplateLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSLiteral::UnaryExpression(s) => AstNodes::UnaryExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSType<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSType::TSAnyKeyword(s) => AstNodes::TSAnyKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSBigIntKeyword(s) => AstNodes::TSBigIntKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSBooleanKeyword(s) => AstNodes::TSBooleanKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSIntrinsicKeyword(s) => {
                AstNodes::TSIntrinsicKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSNeverKeyword(s) => AstNodes::TSNeverKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSNullKeyword(s) => AstNodes::TSNullKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSNumberKeyword(s) => AstNodes::TSNumberKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSObjectKeyword(s) => AstNodes::TSObjectKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSStringKeyword(s) => AstNodes::TSStringKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSSymbolKeyword(s) => AstNodes::TSSymbolKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSUndefinedKeyword(s) => {
                AstNodes::TSUndefinedKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUnknownKeyword(s) => AstNodes::TSUnknownKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSVoidKeyword(s) => AstNodes::TSVoidKeyword(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSArrayType(s) => AstNodes::TSArrayType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSConditionalType(s) => AstNodes::TSConditionalType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSConstructorType(s) => AstNodes::TSConstructorType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSFunctionType(s) => AstNodes::TSFunctionType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSImportType(s) => AstNodes::TSImportType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSIndexedAccessType(s) => {
                AstNodes::TSIndexedAccessType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSInferType(s) => AstNodes::TSInferType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSIntersectionType(s) => {
                AstNodes::TSIntersectionType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSLiteralType(s) => AstNodes::TSLiteralType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSMappedType(s) => AstNodes::TSMappedType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSNamedTupleMember(s) => {
                AstNodes::TSNamedTupleMember(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTemplateLiteralType(s) => {
                AstNodes::TSTemplateLiteralType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSThisType(s) => AstNodes::TSThisType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTupleType(s) => AstNodes::TSTupleType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeLiteral(s) => AstNodes::TSTypeLiteral(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeOperatorType(s) => AstNodes::TSTypeOperator(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypePredicate(s) => AstNodes::TSTypePredicate(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeQuery(s) => AstNodes::TSTypeQuery(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSTypeReference(s) => AstNodes::TSTypeReference(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSUnionType(s) => AstNodes::TSUnionType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::TSParenthesizedType(s) => {
                AstNodes::TSParenthesizedType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocNullableType(s) => AstNodes::JSDocNullableType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSType::JSDocNonNullableType(s) => {
                AstNodes::JSDocNonNullableType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocUnknownType(s) => AstNodes::JSDocUnknownType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSConditionalType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn check_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.inner.extends_type.span().start;
        AstNode {
            inner: &self.inner.check_type,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn extends_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.inner.true_type.span().start;
        AstNode {
            inner: &self.inner.extends_type,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn true_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.inner.false_type.span().start;
        AstNode {
            inner: &self.inner.true_type,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn false_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.false_type,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSUnionType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn types<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.types,
            parent: AstNodes::TSUnionType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSIntersectionType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn types<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.types,
            parent: AstNodes::TSIntersectionType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSParenthesizedType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSParenthesizedType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeOperator<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSTypeOperator(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSArrayType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn element_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.element_type,
            parent: AstNodes::TSArrayType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSIndexedAccessType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.inner.index_type.span().start;
        AstNode {
            inner: &self.inner.object_type,
            parent: AstNodes::TSIndexedAccessType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn index_type<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.index_type,
            parent: AstNodes::TSIndexedAccessType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTupleType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn element_types<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSTupleElement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.element_types,
            parent: AstNodes::TSTupleType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSNamedTupleMember<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self.inner.element_type.span().start;
        AstNode {
            inner: &self.inner.label,
            parent: AstNodes::TSNamedTupleMember(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn element_type<'this>(&'this self) -> AstNode<'this, 'a, TSTupleElement<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.element_type,
            parent: AstNodes::TSNamedTupleMember(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSOptionalType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSOptionalType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSRestType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSRestType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTupleElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                AstNodes::TSOptionalType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTupleElement::TSRestType(s) => AstNodes::TSRestType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            it @ match_ts_type!(TSTupleElement) => {
                return AstNode {
                    inner: it.to_ts_type(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSAnyKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSStringKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSBooleanKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSNumberKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSNeverKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSIntrinsicKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSUnknownKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSNullKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSUndefinedKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSVoidKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSSymbolKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSThisType> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSObjectKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSBigIntKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeReference<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_name<'this>(&'this self) -> AstNode<'this, 'a, TSTypeName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.type_name,
            parent: AstNodes::TSTypeReference(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSTypeReference(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSTypeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSTypeName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypeName::QualifiedName(s) => AstNodes::TSQualifiedName(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
            TSTypeName::ThisExpression(s) => AstNodes::ThisExpression(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSQualifiedName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, TSTypeName<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::TSQualifiedName(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::TSQualifiedName(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.params,
            parent: AstNodes::TSTypeParameterInstantiation(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .constraint
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.default.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.name,
            parent: AstNodes::TSTypeParameter(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn constraint<'this>(&'this self) -> Option<AstNode<'this, 'a, TSType<'a>>> {
        let following_span_start = self
            .inner
            .default
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.constraint.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSTypeParameter(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn default<'this>(&'this self) -> Option<AstNode<'this, 'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.default.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSTypeParameter(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSTypeParameter<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.params,
            parent: AstNodes::TSTypeParameterDeclaration(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.type_annotation.span().start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSTypeAliasDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.type_annotation.span().start;
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSTypeAliasDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSTypeAliasDeclaration(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSClassImplements<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, TSTypeName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSClassImplements(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSClassImplements(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.extends.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .extends
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn extends<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSInterfaceHeritage<'a>>> {
        let following_span_start = self.inner.body.span().start;
        AstNode {
            inner: &self.inner.extends,
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, TSInterfaceBody<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: self.inner.body.as_ref(),
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSInterfaceBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSSignature<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::TSInterfaceBody(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSPropertySignature<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
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
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::TSPropertySignature(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSPropertySignature(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSSignature<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSSignature::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSPropertySignature(s) => {
                AstNodes::TSPropertySignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                AstNodes::TSCallSignatureDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNodes::TSConstructSignatureDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSMethodSignature(s) => {
                AstNodes::TSMethodSignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSIndexSignature<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn parameters<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSIndexSignatureName<'a>>> {
        let following_span_start = self.inner.type_annotation.span().start;
        AstNode {
            inner: &self.inner.parameters,
            parent: AstNodes::TSIndexSignature(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.type_annotation.as_ref(),
            parent: AstNodes::TSIndexSignature(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSCallSignatureDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn this_param<'this>(&'this self) -> Option<AstNode<'this, 'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.this_param.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSCallSignatureDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::TSCallSignatureDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.return_type.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSCallSignatureDeclaration(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSMethodSignature<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::TSMethodSignature(self),
            following_span_start,
        }
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
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSMethodSignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn this_param<'this>(&'this self) -> Option<AstNode<'this, 'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.this_param.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSMethodSignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::TSMethodSignature(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.return_type.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSMethodSignature(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSConstructSignatureDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::TSConstructSignatureDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.return_type.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSConstructSignatureDeclaration(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSIndexSignatureName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Str<'a> {
        self.inner.name
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.type_annotation.as_ref(),
            parent: AstNodes::TSIndexSignatureName(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSInterfaceHeritage<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSInterfaceHeritage(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSInterfaceHeritage(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSTypePredicate<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn parameter_name<'this>(&'this self) -> AstNode<'this, 'a, TSTypePredicateName<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.parameter_name,
            parent: AstNodes::TSTypePredicate(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSTypePredicate(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSTypePredicateName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypePredicateName::This(s) => AstNodes::TSThisType(allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            })),
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSModuleDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, TSModuleDeclarationName<'a>> {
        let following_span_start = self.inner.body.as_ref().map_or(0, |n| n.span().start);
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSModuleDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> Option<AstNode<'this, 'a, TSModuleDeclarationBody<'a>>> {
        let following_span_start = 0;
        self.inner.body.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSModuleDeclaration(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, TSModuleDeclarationName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                AstNodes::BindingIdentifier(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s,
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSModuleDeclarationBody<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNodes::TSModuleBlock(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSGlobalDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn global_span(&self) -> Span {
        self.inner.global_span
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, TSModuleBlock<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::TSGlobalDeclaration(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSModuleBlock<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn directives<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .body
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.directives,
            parent: AstNodes::TSModuleBlock(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.body,
            parent: AstNodes::TSModuleBlock(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn members<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSSignature<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.members,
            parent: AstNodes::TSTypeLiteral(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSInferType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameter<'this>(&'this self) -> AstNode<'this, 'a, TSTypeParameter<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.type_parameter.as_ref(),
            parent: AstNodes::TSInferType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeQuery<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expr_name<'this>(&'this self) -> AstNode<'this, 'a, TSTypeQueryExprName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.expr_name,
            parent: AstNodes::TSTypeQuery(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSTypeQuery(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSTypeQueryExprName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNodes::TSImportType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                return AstNode {
                    inner: it.to_ts_type_name(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }
                .as_ast_nodes(allocator);
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSImportType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source<'this>(&'this self) -> AstNode<'this, 'a, StringLiteral<'a>> {
        let following_span_start = self
            .inner
            .options
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.qualifier.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.source,
            parent: AstNodes::TSImportType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn options<'this>(&'this self) -> Option<AstNode<'this, 'a, ObjectExpression<'a>>> {
        let following_span_start = self
            .inner
            .qualifier
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.options.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSImportType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn qualifier<'this>(&'this self) -> Option<AstNode<'this, 'a, TSImportTypeQualifier<'a>>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.qualifier.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSImportType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_arguments.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSImportType(self),
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

impl<'me, 'a> AstNode<'me, 'a, TSImportTypeQualifier<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSImportTypeQualifier::Identifier(s) => {
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSImportTypeQualifier::QualifiedName(s) => {
                AstNodes::TSImportTypeQualifiedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSImportTypeQualifiedName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'this>(&'this self) -> AstNode<'this, 'a, TSImportTypeQualifier<'a>> {
        let following_span_start = self.inner.right.span().start;
        AstNode {
            inner: &self.inner.left,
            parent: AstNodes::TSImportTypeQualifiedName(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn right<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.right,
            parent: AstNodes::TSImportTypeQualifiedName(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSFunctionType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSFunctionType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn this_param<'this>(&'this self) -> Option<AstNode<'this, 'a, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.this_param.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSFunctionType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self.inner.return_type.span().start;
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::TSFunctionType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> AstNode<'this, 'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.return_type.as_ref(),
            parent: AstNodes::TSFunctionType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSConstructorType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn type_parameters<'this>(
        &'this self,
    ) -> Option<AstNode<'this, 'a, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        self.inner.type_parameters.as_ref().map(|inner| AstNode {
            inner: inner.as_ref(),
            parent: AstNodes::TSConstructorType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn params<'this>(&'this self) -> AstNode<'this, 'a, FormalParameters<'a>> {
        let following_span_start = self.inner.return_type.span().start;
        AstNode {
            inner: self.inner.params.as_ref(),
            parent: AstNodes::TSConstructorType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn return_type<'this>(&'this self) -> AstNode<'this, 'a, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.return_type.as_ref(),
            parent: AstNodes::TSConstructorType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSMappedType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self.inner.constraint.span().start;
        AstNode {
            inner: &self.inner.key,
            parent: AstNodes::TSMappedType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn constraint<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self
            .inner
            .name_type
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_annotation.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.constraint,
            parent: AstNodes::TSMappedType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn name_type<'this>(&'this self) -> Option<AstNode<'this, 'a, TSType<'a>>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        self.inner.name_type.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSMappedType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> Option<AstNode<'this, 'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        self.inner.type_annotation.as_ref().map(|inner| AstNode {
            inner,
            parent: AstNodes::TSMappedType(self),
            following_span_start,
        })
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

impl<'me, 'a> AstNode<'me, 'a, TSTemplateLiteralType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn quasis<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TemplateElement<'a>>> {
        let following_span_start = self
            .inner
            .types
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        AstNode {
            inner: &self.inner.quasis,
            parent: AstNodes::TSTemplateLiteralType(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn types<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.types,
            parent: AstNodes::TSTemplateLiteralType(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSAsExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.type_annotation.span().start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSAsExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSAsExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSSatisfiesExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.type_annotation.span().start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSSatisfiesExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSSatisfiesExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSTypeAssertion<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.inner.expression.span().start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::TSTypeAssertion(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSTypeAssertion(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, BindingIdentifier<'a>> {
        let following_span_start = self.inner.module_reference.span().start;
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSImportEqualsDeclaration(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn module_reference<'this>(&'this self) -> AstNode<'this, 'a, TSModuleReference<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.module_reference,
            parent: AstNodes::TSImportEqualsDeclaration(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, TSModuleReference<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self, allocator: &'a Allocator) -> AstNodes<'me, 'a> {
        match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNodes::TSExternalModuleReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleReference::IdentifierReference(s) => {
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleReference::QualifiedName(s) => {
                AstNodes::TSQualifiedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent: self.parent,
                    following_span_start: self.following_span_start,
                }))
            }
        }
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSExternalModuleReference<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSExternalModuleReference(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSNonNullExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSNonNullExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, Decorator<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::Decorator(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSExportAssignment<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSExportAssignment(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'this>(&'this self) -> AstNode<'this, 'a, IdentifierName<'a>> {
        let following_span_start = 0;
        AstNode {
            inner: &self.inner.id,
            parent: AstNodes::TSNamespaceExportDeclaration(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, TSInstantiationExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
        let following_span_start = self.inner.type_arguments.span().start;
        AstNode {
            inner: &self.inner.expression,
            parent: AstNodes::TSInstantiationExpression(self),
            following_span_start,
        }
    }

    #[inline]
    pub fn type_arguments<'this>(
        &'this self,
    ) -> AstNode<'this, 'a, TSTypeParameterInstantiation<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: self.inner.type_arguments.as_ref(),
            parent: AstNodes::TSInstantiationExpression(self),
            following_span_start,
        }
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'me, 'a> AstNode<'me, 'a, JSDocNullableType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::JSDocNullableType(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, JSDocNonNullableType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'this>(&'this self) -> AstNode<'this, 'a, TSType<'a>> {
        let following_span_start = self.following_span_start;
        AstNode {
            inner: &self.inner.type_annotation,
            parent: AstNodes::JSDocNonNullableType(self),
            following_span_start,
        }
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

impl<'me, 'a> AstNode<'me, 'a, JSDocUnknownType> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}
