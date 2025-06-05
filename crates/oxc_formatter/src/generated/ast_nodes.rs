// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter2.rs`.

#![allow(clippy::undocumented_unsafe_blocks)]
use oxc_allocator::{Allocator, Vec};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    parentheses::NeedsParentheses,
    write::FormatWrite,
};

pub enum AstNodes<'a, 'b> {
    DUMMY(),
    Program(AstNode<'a, 'b, Program<'a>>),
    IdentifierName(AstNode<'a, 'b, IdentifierName<'a>>),
    IdentifierReference(AstNode<'a, 'b, IdentifierReference<'a>>),
    BindingIdentifier(AstNode<'a, 'b, BindingIdentifier<'a>>),
    LabelIdentifier(AstNode<'a, 'b, LabelIdentifier<'a>>),
    ThisExpression(AstNode<'a, 'b, ThisExpression>),
    ArrayExpression(AstNode<'a, 'b, ArrayExpression<'a>>),
    ArrayExpressionElement(AstNode<'a, 'b, ArrayExpressionElement<'a>>),
    Elision(AstNode<'a, 'b, Elision>),
    ObjectExpression(AstNode<'a, 'b, ObjectExpression<'a>>),
    ObjectProperty(AstNode<'a, 'b, ObjectProperty<'a>>),
    PropertyKey(AstNode<'a, 'b, PropertyKey<'a>>),
    TemplateLiteral(AstNode<'a, 'b, TemplateLiteral<'a>>),
    TaggedTemplateExpression(AstNode<'a, 'b, TaggedTemplateExpression<'a>>),
    TemplateElement(AstNode<'a, 'b, TemplateElement<'a>>),
    MemberExpression(AstNode<'a, 'b, MemberExpression<'a>>),
    ComputedMemberExpression(AstNode<'a, 'b, ComputedMemberExpression<'a>>),
    StaticMemberExpression(AstNode<'a, 'b, StaticMemberExpression<'a>>),
    PrivateFieldExpression(AstNode<'a, 'b, PrivateFieldExpression<'a>>),
    CallExpression(AstNode<'a, 'b, CallExpression<'a>>),
    NewExpression(AstNode<'a, 'b, NewExpression<'a>>),
    MetaProperty(AstNode<'a, 'b, MetaProperty<'a>>),
    SpreadElement(AstNode<'a, 'b, SpreadElement<'a>>),
    Argument(AstNode<'a, 'b, Argument<'a>>),
    UpdateExpression(AstNode<'a, 'b, UpdateExpression<'a>>),
    UnaryExpression(AstNode<'a, 'b, UnaryExpression<'a>>),
    BinaryExpression(AstNode<'a, 'b, BinaryExpression<'a>>),
    PrivateInExpression(AstNode<'a, 'b, PrivateInExpression<'a>>),
    LogicalExpression(AstNode<'a, 'b, LogicalExpression<'a>>),
    ConditionalExpression(AstNode<'a, 'b, ConditionalExpression<'a>>),
    AssignmentExpression(AstNode<'a, 'b, AssignmentExpression<'a>>),
    AssignmentTarget(AstNode<'a, 'b, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(AstNode<'a, 'b, SimpleAssignmentTarget<'a>>),
    AssignmentTargetPattern(AstNode<'a, 'b, AssignmentTargetPattern<'a>>),
    ArrayAssignmentTarget(AstNode<'a, 'b, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(AstNode<'a, 'b, ObjectAssignmentTarget<'a>>),
    AssignmentTargetRest(AstNode<'a, 'b, AssignmentTargetRest<'a>>),
    AssignmentTargetWithDefault(AstNode<'a, 'b, AssignmentTargetWithDefault<'a>>),
    AssignmentTargetPropertyIdentifier(AstNode<'a, 'b, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(AstNode<'a, 'b, AssignmentTargetPropertyProperty<'a>>),
    SequenceExpression(AstNode<'a, 'b, SequenceExpression<'a>>),
    Super(AstNode<'a, 'b, Super>),
    AwaitExpression(AstNode<'a, 'b, AwaitExpression<'a>>),
    ChainExpression(AstNode<'a, 'b, ChainExpression<'a>>),
    ParenthesizedExpression(AstNode<'a, 'b, ParenthesizedExpression<'a>>),
    Directive(AstNode<'a, 'b, Directive<'a>>),
    Hashbang(AstNode<'a, 'b, Hashbang<'a>>),
    BlockStatement(AstNode<'a, 'b, BlockStatement<'a>>),
    VariableDeclaration(AstNode<'a, 'b, VariableDeclaration<'a>>),
    VariableDeclarator(AstNode<'a, 'b, VariableDeclarator<'a>>),
    EmptyStatement(AstNode<'a, 'b, EmptyStatement>),
    ExpressionStatement(AstNode<'a, 'b, ExpressionStatement<'a>>),
    IfStatement(AstNode<'a, 'b, IfStatement<'a>>),
    DoWhileStatement(AstNode<'a, 'b, DoWhileStatement<'a>>),
    WhileStatement(AstNode<'a, 'b, WhileStatement<'a>>),
    ForStatement(AstNode<'a, 'b, ForStatement<'a>>),
    ForStatementInit(AstNode<'a, 'b, ForStatementInit<'a>>),
    ForInStatement(AstNode<'a, 'b, ForInStatement<'a>>),
    ForOfStatement(AstNode<'a, 'b, ForOfStatement<'a>>),
    ContinueStatement(AstNode<'a, 'b, ContinueStatement<'a>>),
    BreakStatement(AstNode<'a, 'b, BreakStatement<'a>>),
    ReturnStatement(AstNode<'a, 'b, ReturnStatement<'a>>),
    WithStatement(AstNode<'a, 'b, WithStatement<'a>>),
    SwitchStatement(AstNode<'a, 'b, SwitchStatement<'a>>),
    SwitchCase(AstNode<'a, 'b, SwitchCase<'a>>),
    LabeledStatement(AstNode<'a, 'b, LabeledStatement<'a>>),
    ThrowStatement(AstNode<'a, 'b, ThrowStatement<'a>>),
    TryStatement(AstNode<'a, 'b, TryStatement<'a>>),
    CatchClause(AstNode<'a, 'b, CatchClause<'a>>),
    CatchParameter(AstNode<'a, 'b, CatchParameter<'a>>),
    DebuggerStatement(AstNode<'a, 'b, DebuggerStatement>),
    BindingPattern(AstNode<'a, 'b, BindingPattern<'a>>),
    AssignmentPattern(AstNode<'a, 'b, AssignmentPattern<'a>>),
    ObjectPattern(AstNode<'a, 'b, ObjectPattern<'a>>),
    BindingProperty(AstNode<'a, 'b, BindingProperty<'a>>),
    ArrayPattern(AstNode<'a, 'b, ArrayPattern<'a>>),
    BindingRestElement(AstNode<'a, 'b, BindingRestElement<'a>>),
    Function(AstNode<'a, 'b, Function<'a>>),
    FormalParameters(AstNode<'a, 'b, FormalParameters<'a>>),
    FormalParameter(AstNode<'a, 'b, FormalParameter<'a>>),
    FunctionBody(AstNode<'a, 'b, FunctionBody<'a>>),
    ArrowFunctionExpression(AstNode<'a, 'b, ArrowFunctionExpression<'a>>),
    YieldExpression(AstNode<'a, 'b, YieldExpression<'a>>),
    Class(AstNode<'a, 'b, Class<'a>>),
    ClassBody(AstNode<'a, 'b, ClassBody<'a>>),
    MethodDefinition(AstNode<'a, 'b, MethodDefinition<'a>>),
    PropertyDefinition(AstNode<'a, 'b, PropertyDefinition<'a>>),
    PrivateIdentifier(AstNode<'a, 'b, PrivateIdentifier<'a>>),
    StaticBlock(AstNode<'a, 'b, StaticBlock<'a>>),
    ModuleDeclaration(AstNode<'a, 'b, ModuleDeclaration<'a>>),
    AccessorProperty(AstNode<'a, 'b, AccessorProperty<'a>>),
    ImportExpression(AstNode<'a, 'b, ImportExpression<'a>>),
    ImportDeclaration(AstNode<'a, 'b, ImportDeclaration<'a>>),
    ImportSpecifier(AstNode<'a, 'b, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(AstNode<'a, 'b, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(AstNode<'a, 'b, ImportNamespaceSpecifier<'a>>),
    WithClause(AstNode<'a, 'b, WithClause<'a>>),
    ImportAttribute(AstNode<'a, 'b, ImportAttribute<'a>>),
    ExportNamedDeclaration(AstNode<'a, 'b, ExportNamedDeclaration<'a>>),
    ExportDefaultDeclaration(AstNode<'a, 'b, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(AstNode<'a, 'b, ExportAllDeclaration<'a>>),
    ExportSpecifier(AstNode<'a, 'b, ExportSpecifier<'a>>),
    V8IntrinsicExpression(AstNode<'a, 'b, V8IntrinsicExpression<'a>>),
    BooleanLiteral(AstNode<'a, 'b, BooleanLiteral>),
    NullLiteral(AstNode<'a, 'b, NullLiteral>),
    NumericLiteral(AstNode<'a, 'b, NumericLiteral<'a>>),
    StringLiteral(AstNode<'a, 'b, StringLiteral<'a>>),
    BigIntLiteral(AstNode<'a, 'b, BigIntLiteral<'a>>),
    RegExpLiteral(AstNode<'a, 'b, RegExpLiteral<'a>>),
    JSXElement(AstNode<'a, 'b, JSXElement<'a>>),
    JSXOpeningElement(AstNode<'a, 'b, JSXOpeningElement<'a>>),
    JSXClosingElement(AstNode<'a, 'b, JSXClosingElement<'a>>),
    JSXFragment(AstNode<'a, 'b, JSXFragment<'a>>),
    JSXOpeningFragment(AstNode<'a, 'b, JSXOpeningFragment>),
    JSXClosingFragment(AstNode<'a, 'b, JSXClosingFragment>),
    JSXElementName(AstNode<'a, 'b, JSXElementName<'a>>),
    JSXNamespacedName(AstNode<'a, 'b, JSXNamespacedName<'a>>),
    JSXMemberExpression(AstNode<'a, 'b, JSXMemberExpression<'a>>),
    JSXMemberExpressionObject(AstNode<'a, 'b, JSXMemberExpressionObject<'a>>),
    JSXExpressionContainer(AstNode<'a, 'b, JSXExpressionContainer<'a>>),
    JSXEmptyExpression(AstNode<'a, 'b, JSXEmptyExpression>),
    JSXAttributeItem(AstNode<'a, 'b, JSXAttributeItem<'a>>),
    JSXAttribute(AstNode<'a, 'b, JSXAttribute<'a>>),
    JSXSpreadAttribute(AstNode<'a, 'b, JSXSpreadAttribute<'a>>),
    JSXIdentifier(AstNode<'a, 'b, JSXIdentifier<'a>>),
    JSXSpreadChild(AstNode<'a, 'b, JSXSpreadChild<'a>>),
    JSXText(AstNode<'a, 'b, JSXText<'a>>),
    TSThisParameter(AstNode<'a, 'b, TSThisParameter<'a>>),
    TSEnumDeclaration(AstNode<'a, 'b, TSEnumDeclaration<'a>>),
    TSEnumBody(AstNode<'a, 'b, TSEnumBody<'a>>),
    TSEnumMember(AstNode<'a, 'b, TSEnumMember<'a>>),
    TSTypeAnnotation(AstNode<'a, 'b, TSTypeAnnotation<'a>>),
    TSLiteralType(AstNode<'a, 'b, TSLiteralType<'a>>),
    TSConditionalType(AstNode<'a, 'b, TSConditionalType<'a>>),
    TSUnionType(AstNode<'a, 'b, TSUnionType<'a>>),
    TSIntersectionType(AstNode<'a, 'b, TSIntersectionType<'a>>),
    TSParenthesizedType(AstNode<'a, 'b, TSParenthesizedType<'a>>),
    TSTypeOperator(AstNode<'a, 'b, TSTypeOperator<'a>>),
    TSArrayType(AstNode<'a, 'b, TSArrayType<'a>>),
    TSIndexedAccessType(AstNode<'a, 'b, TSIndexedAccessType<'a>>),
    TSTupleType(AstNode<'a, 'b, TSTupleType<'a>>),
    TSNamedTupleMember(AstNode<'a, 'b, TSNamedTupleMember<'a>>),
    TSOptionalType(AstNode<'a, 'b, TSOptionalType<'a>>),
    TSRestType(AstNode<'a, 'b, TSRestType<'a>>),
    TSAnyKeyword(AstNode<'a, 'b, TSAnyKeyword>),
    TSStringKeyword(AstNode<'a, 'b, TSStringKeyword>),
    TSBooleanKeyword(AstNode<'a, 'b, TSBooleanKeyword>),
    TSNumberKeyword(AstNode<'a, 'b, TSNumberKeyword>),
    TSNeverKeyword(AstNode<'a, 'b, TSNeverKeyword>),
    TSIntrinsicKeyword(AstNode<'a, 'b, TSIntrinsicKeyword>),
    TSUnknownKeyword(AstNode<'a, 'b, TSUnknownKeyword>),
    TSNullKeyword(AstNode<'a, 'b, TSNullKeyword>),
    TSUndefinedKeyword(AstNode<'a, 'b, TSUndefinedKeyword>),
    TSVoidKeyword(AstNode<'a, 'b, TSVoidKeyword>),
    TSSymbolKeyword(AstNode<'a, 'b, TSSymbolKeyword>),
    TSThisType(AstNode<'a, 'b, TSThisType>),
    TSObjectKeyword(AstNode<'a, 'b, TSObjectKeyword>),
    TSBigIntKeyword(AstNode<'a, 'b, TSBigIntKeyword>),
    TSTypeReference(AstNode<'a, 'b, TSTypeReference<'a>>),
    TSTypeName(AstNode<'a, 'b, TSTypeName<'a>>),
    TSQualifiedName(AstNode<'a, 'b, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(AstNode<'a, 'b, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(AstNode<'a, 'b, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(AstNode<'a, 'b, TSClassImplements<'a>>),
    TSInterfaceDeclaration(AstNode<'a, 'b, TSInterfaceDeclaration<'a>>),
    TSInterfaceBody(AstNode<'a, 'b, TSInterfaceBody<'a>>),
    TSPropertySignature(AstNode<'a, 'b, TSPropertySignature<'a>>),
    TSIndexSignature(AstNode<'a, 'b, TSIndexSignature<'a>>),
    TSCallSignatureDeclaration(AstNode<'a, 'b, TSCallSignatureDeclaration<'a>>),
    TSMethodSignature(AstNode<'a, 'b, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(AstNode<'a, 'b, TSConstructSignatureDeclaration<'a>>),
    TSIndexSignatureName(AstNode<'a, 'b, TSIndexSignatureName<'a>>),
    TSInterfaceHeritage(AstNode<'a, 'b, TSInterfaceHeritage<'a>>),
    TSTypePredicate(AstNode<'a, 'b, TSTypePredicate<'a>>),
    TSModuleDeclaration(AstNode<'a, 'b, TSModuleDeclaration<'a>>),
    TSModuleBlock(AstNode<'a, 'b, TSModuleBlock<'a>>),
    TSTypeLiteral(AstNode<'a, 'b, TSTypeLiteral<'a>>),
    TSInferType(AstNode<'a, 'b, TSInferType<'a>>),
    TSTypeQuery(AstNode<'a, 'b, TSTypeQuery<'a>>),
    TSImportType(AstNode<'a, 'b, TSImportType<'a>>),
    TSFunctionType(AstNode<'a, 'b, TSFunctionType<'a>>),
    TSConstructorType(AstNode<'a, 'b, TSConstructorType<'a>>),
    TSMappedType(AstNode<'a, 'b, TSMappedType<'a>>),
    TSTemplateLiteralType(AstNode<'a, 'b, TSTemplateLiteralType<'a>>),
    TSAsExpression(AstNode<'a, 'b, TSAsExpression<'a>>),
    TSSatisfiesExpression(AstNode<'a, 'b, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(AstNode<'a, 'b, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(AstNode<'a, 'b, TSImportEqualsDeclaration<'a>>),
    TSModuleReference(AstNode<'a, 'b, TSModuleReference<'a>>),
    TSExternalModuleReference(AstNode<'a, 'b, TSExternalModuleReference<'a>>),
    TSNonNullExpression(AstNode<'a, 'b, TSNonNullExpression<'a>>),
    Decorator(AstNode<'a, 'b, Decorator<'a>>),
    TSExportAssignment(AstNode<'a, 'b, TSExportAssignment<'a>>),
    TSNamespaceExportDeclaration(AstNode<'a, 'b, TSNamespaceExportDeclaration<'a>>),
    TSInstantiationExpression(AstNode<'a, 'b, TSInstantiationExpression<'a>>),
    JSDocNullableType(AstNode<'a, 'b, JSDocNullableType<'a>>),
    JSDocNonNullableType(AstNode<'a, 'b, JSDocNonNullableType<'a>>),
    JSDocUnknownType(AstNode<'a, 'b, JSDocUnknownType>),
}

pub struct AstNode<'a, 'b, T> {
    inner: &'b T,
    parent: &'a AstNodes<'a, 'b>,
    allocator: &'a Allocator,
}

impl<'a, 'b, T> AstNode<'a, 'b, T> {
    pub fn new(inner: &'b T, parent: &'a AstNodes<'a, 'b>, allocator: &'a Allocator) -> Self {
        AstNode { inner, parent, allocator }
    }
    pub fn inner(&self) -> &'b T {
        self.inner
    }
    pub fn parent(&self) -> &'a AstNodes<'a, 'b> {
        self.parent
    }
}

impl<'a, 'b, T> AstNode<'a, 'b, Vec<'a, T>> {
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn to_vec(&self) -> Vec<'a, AstNode<'a, 'b, T>> {
        let iter = self.inner.iter().map(|inner| AstNode {
            inner,
            parent: self.parent,
            allocator: self.allocator,
        });
        Vec::from_iter_in(iter, self.allocator)
    }
}

pub struct AstNodeIterator<'a, 'b, T> {
    inner: std::slice::Iter<'b, T>,
    parent: &'b AstNodes<'a, 'b>,
    allocator: &'a Allocator,
}
/// @@line_break
impl<'a, 'b, T> Iterator for AstNodeIterator<'a, 'b, T> {
    type Item = &'a AstNode<'a, 'b, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        let parent = self.parent;
        self.allocator
            .alloc(self.inner.next().map(|inner| AstNode { parent, inner, allocator }))
            .as_ref()
    }
}

impl<'a, 'b, T> IntoIterator for &AstNode<'a, 'b, Vec<'a, T>> {
    type Item = &'a AstNode<'a, 'b, T>;
    type IntoIter = AstNodeIterator<'a, 'b, T>;
    fn into_iter(self) -> Self::IntoIter {
        let parent = self.parent;
        AstNodeIterator::<T> { inner: self.inner.iter(), parent, allocator: self.allocator }
    }
}

impl<'a, 'b> AstNode<'a, 'b, Program<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn source_type(&self) -> SourceType {
        self.inner.source_type
    }

    pub fn source_text(&self) -> &'a str {
        self.inner.source_text
    }

    pub fn comments(&self) -> &AstNode<'a, 'b, Vec<'a, Comment>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.comments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn hashbang(&self) -> Option<&AstNode<'a, 'b, Hashbang>> {
        self.allocator
            .alloc(self.inner.hashbang.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Program(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn directives(&self) -> &AstNode<'a, 'b, Vec<'a, Directive<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Expression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            Expression::BooleanLiteral(s) => {
                AstNode::<'a, 'b, BooleanLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::NullLiteral(s) => {
                AstNode::<'a, 'b, NullLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::NumericLiteral(s) => {
                AstNode::<'a, 'b, NumericLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::BigIntLiteral(s) => {
                AstNode::<'a, 'b, BigIntLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::RegExpLiteral(s) => {
                AstNode::<'a, 'b, RegExpLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TemplateLiteral(s) => {
                AstNode::<'a, 'b, TemplateLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::Identifier(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::MetaProperty(s) => {
                AstNode::<'a, 'b, MetaProperty> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::Super(s) => {
                AstNode::<'a, 'b, Super> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::ArrayExpression(s) => {
                AstNode::<'a, 'b, ArrayExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ArrowFunctionExpression(s) => AstNode::<'a, 'b, ArrowFunctionExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::AssignmentExpression(s) => AstNode::<'a, 'b, AssignmentExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::AwaitExpression(s) => {
                AstNode::<'a, 'b, AwaitExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::BinaryExpression(s) => {
                AstNode::<'a, 'b, BinaryExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::CallExpression(s) => {
                AstNode::<'a, 'b, CallExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ChainExpression(s) => {
                AstNode::<'a, 'b, ChainExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ClassExpression(s) => {
                AstNode::<'a, 'b, Class> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::ConditionalExpression(s) => AstNode::<'a, 'b, ConditionalExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::FunctionExpression(s) => {
                AstNode::<'a, 'b, Function> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::ImportExpression(s) => {
                AstNode::<'a, 'b, ImportExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::LogicalExpression(s) => {
                AstNode::<'a, 'b, LogicalExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::NewExpression(s) => {
                AstNode::<'a, 'b, NewExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ObjectExpression(s) => {
                AstNode::<'a, 'b, ObjectExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ParenthesizedExpression(s) => AstNode::<'a, 'b, ParenthesizedExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::SequenceExpression(s) => AstNode::<'a, 'b, SequenceExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::TaggedTemplateExpression(s) => {
                AstNode::<'a, 'b, TaggedTemplateExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            Expression::ThisExpression(s) => {
                AstNode::<'a, 'b, ThisExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::UnaryExpression(s) => {
                AstNode::<'a, 'b, UnaryExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::UpdateExpression(s) => {
                AstNode::<'a, 'b, UpdateExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::YieldExpression(s) => {
                AstNode::<'a, 'b, YieldExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::PrivateInExpression(s) => AstNode::<'a, 'b, PrivateInExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::JSXElement(s) => {
                AstNode::<'a, 'b, JSXElement> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::JSXFragment(s) => {
                AstNode::<'a, 'b, JSXFragment> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TSAsExpression(s) => {
                AstNode::<'a, 'b, TSAsExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TSSatisfiesExpression(s) => AstNode::<'a, 'b, TSSatisfiesExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::TSTypeAssertion(s) => {
                AstNode::<'a, 'b, TSTypeAssertion> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TSNonNullExpression(s) => AstNode::<'a, 'b, TSNonNullExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::TSInstantiationExpression(s) => {
                AstNode::<'a, 'b, TSInstantiationExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            Expression::V8IntrinsicExpression(s) => AstNode::<'a, 'b, V8IntrinsicExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_member_expression!(Expression) => AstNode::<'a, 'b, MemberExpression> {
                inner: it.to_member_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Expression<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, IdentifierName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, IdentifierReference<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, LabelIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, ThisExpression> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrayExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn elements(&self) -> &AstNode<'a, 'b, Vec<'a, ArrayExpressionElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ArrayExpressionElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::ArrayExpressionElement(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNode::<'a, 'b, SpreadElement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ArrayExpressionElement::Elision(s) => {
                AstNode::<'a, 'b, Elision> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            it @ match_expression!(ArrayExpressionElement) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ArrayExpressionElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, Elision> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn properties(&self) -> &AstNode<'a, 'b, Vec<'a, ObjectPropertyKind<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ObjectPropertyKind<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNode::<'a, 'b, ObjectProperty> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNode::<'a, 'b, SpreadElement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ObjectPropertyKind<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectProperty(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn method(&self) -> bool {
        self.inner.method
    }

    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, PropertyKey<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::PropertyKey(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNode::<'a, 'b, PrivateIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_expression!(PropertyKey) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, PropertyKey<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TemplateLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn quasis(&self) -> &AstNode<'a, 'b, Vec<'a, TemplateElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn expressions(&self) -> &AstNode<'a, 'b, Vec<'a, Expression<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TaggedTemplateExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn tag(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.tag,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn quasi(&self) -> &AstNode<'a, 'b, TemplateLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasi,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TemplateElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> &TemplateElementValue {
        &self.inner.value
    }

    pub fn tail(&self) -> bool {
        self.inner.tail
    }

    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, MemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::MemberExpression(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                AstNode::<'a, 'b, ComputedMemberExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            MemberExpression::StaticMemberExpression(s) => {
                AstNode::<'a, 'b, StaticMemberExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            MemberExpression::PrivateFieldExpression(s) => {
                AstNode::<'a, 'b, PrivateFieldExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, MemberExpression<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ComputedMemberExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ComputedMemberExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ComputedMemberExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, StaticMemberExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticMemberExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn property(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticMemberExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, PrivateFieldExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateFieldExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn field(&self) -> &AstNode<'a, 'b, PrivateIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.field,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateFieldExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, CallExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn callee(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::CallExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn arguments(&self) -> &AstNode<'a, 'b, Vec<'a, Argument<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, NewExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn callee(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::NewExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn arguments(&self) -> &AstNode<'a, 'b, Vec<'a, Argument<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, MetaProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn meta(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.meta,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn property(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, SpreadElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SpreadElement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Argument<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::Argument(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            Argument::SpreadElement(s) => {
                AstNode::<'a, 'b, SpreadElement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_expression!(Argument) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Argument<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, UpdateExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> UpdateOperator {
        self.inner.operator
    }

    pub fn prefix(&self) -> bool {
        self.inner.prefix
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, SimpleAssignmentTarget> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, UnaryExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UnaryExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, BinaryExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn operator(&self) -> BinaryOperator {
        self.inner.operator
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, PrivateInExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, PrivateIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, LogicalExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn operator(&self) -> LogicalOperator {
        self.inner.operator
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ConditionalExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn consequent(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn alternate(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.alternate,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    pub fn left(&self) -> &AstNode<'a, 'b, AssignmentTarget> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTarget(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                AstNode::<'a, 'b, SimpleAssignmentTarget> {
                    inner: it.to_simple_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                AstNode::<'a, 'b, AssignmentTargetPattern> {
                    inner: it.to_assignment_target_pattern(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTarget<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::SimpleAssignmentTarget(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNode::<'a, 'b, IdentifierReference> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNode::<'a, 'b, TSAsExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNode::<'a, 'b, TSSatisfiesExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNode::<'a, 'b, TSNonNullExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNode::<'a, 'b, TSTypeAssertion> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                AstNode::<'a, 'b, MemberExpression> {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTargetPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTargetPattern(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNode::<'a, 'b, ArrayAssignmentTarget> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNode::<'a, 'b, ObjectAssignmentTarget> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTargetPattern<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrayAssignmentTarget<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn elements(&self) -> &AstNode<'a, 'b, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn rest(&self) -> Option<&AssignmentTargetRest> {
        self.inner.rest.as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectAssignmentTarget<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn properties(&self) -> &AstNode<'a, 'b, Vec<'a, AssignmentTargetProperty<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectAssignmentTarget(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn rest(&self) -> Option<&AssignmentTargetRest> {
        self.inner.rest.as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetRest<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn target(&self) -> &AstNode<'a, 'b, AssignmentTarget> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.target,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNode::<'a, 'b, AssignmentTargetWithDefault> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                AstNode::<'a, 'b, AssignmentTarget> {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetWithDefault<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn binding(&self) -> &AstNode<'a, 'b, AssignmentTarget> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn init(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.init,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentTargetWithDefault(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTargetProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                AstNode::<'a, 'b, AssignmentTargetPropertyIdentifier> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                AstNode::<'a, 'b, AssignmentTargetPropertyProperty> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTargetProperty<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetPropertyIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn binding(&self) -> &AstNode<'a, 'b, IdentifierReference> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentTargetPropertyIdentifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn init(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::AssignmentTargetPropertyIdentifier(
                    AstNode { inner: self.inner, parent: self.parent, allocator: self.allocator },
                )),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetPropertyProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn binding(&self) -> &AstNode<'a, 'b, AssignmentTargetMaybeDefault> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentTargetPropertyProperty(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}
impl<'a, 'b> AstNode<'a, 'b, SequenceExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expressions(&self) -> &AstNode<'a, 'b, Vec<'a, Expression<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SequenceExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, Super> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, AwaitExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AwaitExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ChainExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, ChainElement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ChainExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ChainElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ChainElement::CallExpression(s) => {
                AstNode::<'a, 'b, CallExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ChainElement::TSNonNullExpression(s) => AstNode::<'a, 'b, TSNonNullExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_member_expression!(ChainElement) => AstNode::<'a, 'b, MemberExpression> {
                inner: it.to_member_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ChainElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ParenthesizedExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ParenthesizedExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Statement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            Statement::BlockStatement(s) => {
                AstNode::<'a, 'b, BlockStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::BreakStatement(s) => {
                AstNode::<'a, 'b, BreakStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ContinueStatement(s) => {
                AstNode::<'a, 'b, ContinueStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::DebuggerStatement(s) => {
                AstNode::<'a, 'b, DebuggerStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::DoWhileStatement(s) => {
                AstNode::<'a, 'b, DoWhileStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::EmptyStatement(s) => {
                AstNode::<'a, 'b, EmptyStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ExpressionStatement(s) => AstNode::<'a, 'b, ExpressionStatement> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Statement::ForInStatement(s) => {
                AstNode::<'a, 'b, ForInStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ForOfStatement(s) => {
                AstNode::<'a, 'b, ForOfStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ForStatement(s) => {
                AstNode::<'a, 'b, ForStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::IfStatement(s) => {
                AstNode::<'a, 'b, IfStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::LabeledStatement(s) => {
                AstNode::<'a, 'b, LabeledStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ReturnStatement(s) => {
                AstNode::<'a, 'b, ReturnStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::SwitchStatement(s) => {
                AstNode::<'a, 'b, SwitchStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ThrowStatement(s) => {
                AstNode::<'a, 'b, ThrowStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::TryStatement(s) => {
                AstNode::<'a, 'b, TryStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::WhileStatement(s) => {
                AstNode::<'a, 'b, WhileStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::WithStatement(s) => {
                AstNode::<'a, 'b, WithStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_declaration!(Statement) => AstNode::<'a, 'b, Declaration> {
                inner: it.to_declaration(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_module_declaration!(Statement) => AstNode::<'a, 'b, ModuleDeclaration> {
                inner: it.to_module_declaration(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Statement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, Directive<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, StringLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Directive(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn directive(&self) -> Atom<'a> {
        self.inner.directive
    }
}
impl<'a, 'b> AstNode<'a, 'b, Hashbang<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }
}
impl<'a, 'b> AstNode<'a, 'b, BlockStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BlockStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Declaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            Declaration::VariableDeclaration(s) => AstNode::<'a, 'b, VariableDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::FunctionDeclaration(s) => {
                AstNode::<'a, 'b, Function> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Declaration::ClassDeclaration(s) => {
                AstNode::<'a, 'b, Class> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Declaration::TSTypeAliasDeclaration(s) => AstNode::<'a, 'b, TSTypeAliasDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::TSInterfaceDeclaration(s) => AstNode::<'a, 'b, TSInterfaceDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::TSEnumDeclaration(s) => {
                AstNode::<'a, 'b, TSEnumDeclaration> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Declaration::TSModuleDeclaration(s) => AstNode::<'a, 'b, TSModuleDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNode::<'a, 'b, TSImportEqualsDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Declaration<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, VariableDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    pub fn declarations(&self) -> &AstNode<'a, 'b, Vec<'a, VariableDeclarator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.declarations,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::VariableDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, VariableDeclarator<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    pub fn id(&self) -> &BindingPattern {
        &self.inner.id
    }

    pub fn init(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::VariableDeclarator(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn definite(&self) -> bool {
        self.inner.definite
    }
}
impl<'a, 'b> AstNode<'a, 'b, EmptyStatement> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExpressionStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExpressionStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, IfStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn consequent(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn alternate(&self) -> Option<&AstNode<'a, 'b, Statement>> {
        self.allocator
            .alloc(self.inner.alternate.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::IfStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, DoWhileStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, WhileStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ForStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn init(&self) -> Option<&AstNode<'a, 'b, ForStatementInit>> {
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn test(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn update(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.update.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ForStatementInit<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::ForStatementInit(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            ForStatementInit::VariableDeclaration(s) => AstNode::<'a, 'b, VariableDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_expression!(ForStatementInit) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ForStatementInit<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ForInStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, ForStatementLeft> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ForStatementLeft<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ForStatementLeft::VariableDeclaration(s) => AstNode::<'a, 'b, VariableDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_assignment_target!(ForStatementLeft) => {
                AstNode::<'a, 'b, AssignmentTarget> {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ForStatementLeft<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ForOfStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    pub fn left(&self) -> &AstNode<'a, 'b, ForStatementLeft> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ContinueStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> Option<&AstNode<'a, 'b, LabelIdentifier>> {
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ContinueStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, BreakStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> Option<&AstNode<'a, 'b, LabelIdentifier>> {
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::BreakStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ReturnStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ReturnStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, WithStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, SwitchStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn discriminant(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.discriminant,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn cases(&self) -> &AstNode<'a, 'b, Vec<'a, SwitchCase<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.cases,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, SwitchCase<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::SwitchCase(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn consequent(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchCase(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, LabeledStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> &AstNode<'a, 'b, LabelIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ThrowStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ThrowStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TryStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn block(&self) -> &AstNode<'a, 'b, BlockStatement> {
        self.allocator.alloc(AstNode {
            inner: self.inner.block.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TryStatement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn handler(&self) -> Option<&AstNode<'a, 'b, CatchClause>> {
        self.allocator
            .alloc(self.inner.handler.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn finalizer(&self) -> Option<&AstNode<'a, 'b, BlockStatement>> {
        self.allocator
            .alloc(self.inner.finalizer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, CatchClause<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn param(&self) -> Option<&AstNode<'a, 'b, CatchParameter>> {
        self.allocator
            .alloc(self.inner.param.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::CatchClause(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn body(&self) -> &AstNode<'a, 'b, BlockStatement> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchClause(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, CatchParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn pattern(&self) -> &BindingPattern {
        &self.inner.pattern
    }
}
impl<'a, 'b> AstNode<'a, 'b, DebuggerStatement> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingPattern<'a>> {
    pub fn kind(&self) -> &AstNode<'a, 'b, BindingPatternKind> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.kind,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BindingPattern(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::BindingPattern(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, BindingPatternKind<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            BindingPatternKind::BindingIdentifier(s) => {
                AstNode::<'a, 'b, BindingIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            BindingPatternKind::ObjectPattern(s) => {
                AstNode::<'a, 'b, ObjectPattern> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            BindingPatternKind::ArrayPattern(s) => {
                AstNode::<'a, 'b, ArrayPattern> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            BindingPatternKind::AssignmentPattern(s) => {
                AstNode::<'a, 'b, AssignmentPattern> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, BindingPatternKind<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentPattern<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &BindingPattern {
        &self.inner.left
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectPattern<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn properties(&self) -> &AstNode<'a, 'b, Vec<'a, BindingProperty<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectPattern(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, BindingRestElement>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ObjectPattern(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn value(&self) -> &BindingPattern {
        &self.inner.value
    }

    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrayPattern<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn elements(&self) -> &AstNode<'a, 'b, Vec<'a, Option<BindingPattern<'a>>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayPattern(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, BindingRestElement>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayPattern(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingRestElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &BindingPattern {
        &self.inner.argument
    }
}
impl<'a, 'b> AstNode<'a, 'b, Function<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    pub fn id(&self) -> Option<&AstNode<'a, 'b, BindingIdentifier>> {
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn generator(&self) -> bool {
        self.inner.generator
    }

    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Function(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn body(&self) -> Option<&AstNode<'a, 'b, FunctionBody>> {
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, FormalParameters<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    pub fn items(&self) -> &AstNode<'a, 'b, Vec<'a, FormalParameter<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.items,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameters(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, BindingRestElement>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::FormalParameters(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, FormalParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameter(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn pattern(&self) -> &BindingPattern {
        &self.inner.pattern
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }
}
impl<'a, 'b> AstNode<'a, 'b, FunctionBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn directives(&self) -> &AstNode<'a, 'b, Vec<'a, Directive<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn statements(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.statements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrowFunctionExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> bool {
        self.inner.expression
    }

    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn body(&self) -> &AstNode<'a, 'b, FunctionBody> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, YieldExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    pub fn argument(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::YieldExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, Class<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn id(&self) -> Option<&AstNode<'a, 'b, BindingIdentifier>> {
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn super_class(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.super_class.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn super_type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.super_type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn implements(&self) -> &AstNode<'a, 'b, Vec<'a, TSClassImplements<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.implements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, ClassBody> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, ClassBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, ClassElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ClassBody(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ClassElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ClassElement::StaticBlock(s) => {
                AstNode::<'a, 'b, StaticBlock> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ClassElement::MethodDefinition(s) => {
                AstNode::<'a, 'b, MethodDefinition> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ClassElement::PropertyDefinition(s) => AstNode::<'a, 'b, PropertyDefinition> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ClassElement::AccessorProperty(s) => {
                AstNode::<'a, 'b, AccessorProperty> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ClassElement::TSIndexSignature(s) => {
                AstNode::<'a, 'b, TSIndexSignature> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ClassElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, MethodDefinition<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, Function> {
        self.allocator.alloc(AstNode {
            inner: self.inner.value.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn kind(&self) -> MethodDefinitionKind {
        self.inner.kind
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }
}
impl<'a, 'b> AstNode<'a, 'b, PropertyDefinition<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::PropertyDefinition(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn value(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::PropertyDefinition(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }
}
impl<'a, 'b> AstNode<'a, 'b, PrivateIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, StaticBlock<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticBlock(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ModuleDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::ModuleDeclaration(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNode::<'a, 'b, ImportDeclaration> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ModuleDeclaration::ExportAllDeclaration(s) => AstNode::<'a, 'b, ExportAllDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNode::<'a, 'b, ExportDefaultDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNode::<'a, 'b, ExportNamedDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            ModuleDeclaration::TSExportAssignment(s) => AstNode::<'a, 'b, TSExportAssignment> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                AstNode::<'a, 'b, TSNamespaceExportDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ModuleDeclaration<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AccessorProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AccessorProperty(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::AccessorProperty(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn value(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::AccessorProperty(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn source(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn options(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportExpression(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn specifiers(&self) -> Option<&AstNode<'a, 'b, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        self.allocator
            .alloc(self.inner.specifiers.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn source(&self) -> &AstNode<'a, 'b, StringLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    pub fn with_clause(&self) -> Option<&AstNode<'a, 'b, WithClause>> {
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ImportDeclarationSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNode::<'a, 'b, ImportSpecifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNode::<'a, 'b, ImportDefaultSpecifier> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNode::<'a, 'b, ImportNamespaceSpecifier> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ImportDeclarationSpecifier<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn imported(&self) -> &AstNode<'a, 'b, ModuleExportName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.imported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn local(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportDefaultSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn local(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDefaultSpecifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportNamespaceSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn local(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportNamespaceSpecifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, WithClause<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn attributes_keyword(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes_keyword,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithClause(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn with_entries(&self) -> &AstNode<'a, 'b, Vec<'a, ImportAttribute<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.with_entries,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithClause(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportAttribute<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn key(&self) -> &AstNode<'a, 'b, ImportAttributeKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportAttribute(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, StringLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportAttribute(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ImportAttributeKey<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ImportAttributeKey::Identifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ImportAttributeKey<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportNamedDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn declaration(&self) -> Option<&AstNode<'a, 'b, Declaration>> {
        self.allocator
            .alloc(self.inner.declaration.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn specifiers(&self) -> &AstNode<'a, 'b, Vec<'a, ExportSpecifier<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.specifiers,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn source(&self) -> Option<&AstNode<'a, 'b, StringLiteral>> {
        self.allocator
            .alloc(self.inner.source.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn with_clause(&self) -> Option<&AstNode<'a, 'b, WithClause>> {
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportDefaultDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn exported(&self) -> &AstNode<'a, 'b, ModuleExportName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn declaration(&self) -> &AstNode<'a, 'b, ExportDefaultDeclarationKind> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.declaration,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportAllDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn exported(&self) -> Option<&AstNode<'a, 'b, ModuleExportName>> {
        self.allocator
            .alloc(self.inner.exported.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn source(&self) -> &AstNode<'a, 'b, StringLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn with_clause(&self) -> Option<&AstNode<'a, 'b, WithClause>> {
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn local(&self) -> &AstNode<'a, 'b, ModuleExportName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn exported(&self) -> &AstNode<'a, 'b, ModuleExportName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNode::<'a, 'b, Function> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNode::<'a, 'b, Class> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNode::<'a, 'b, TSInterfaceDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ModuleExportName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ModuleExportName::IdentifierName(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ModuleExportName::IdentifierReference(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ModuleExportName::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ModuleExportName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, V8IntrinsicExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn arguments(&self) -> &AstNode<'a, 'b, Vec<'a, Argument<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, BooleanLiteral> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> bool {
        self.inner.value
    }
}
impl<'a, 'b> AstNode<'a, 'b, NullLiteral> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, NumericLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> f64 {
        self.inner.value
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn base(&self) -> NumberBase {
        self.inner.base
    }
}
impl<'a, 'b> AstNode<'a, 'b, StringLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }
}
impl<'a, 'b> AstNode<'a, 'b, BigIntLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn raw(&self) -> Atom<'a> {
        self.inner.raw
    }

    pub fn base(&self) -> BigintBase {
        self.inner.base
    }
}
impl<'a, 'b> AstNode<'a, 'b, RegExpLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn regex(&self) -> &RegExp {
        &self.inner.regex
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn opening_element(&self) -> &AstNode<'a, 'b, JSXOpeningElement> {
        self.allocator.alloc(AstNode {
            inner: self.inner.opening_element.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn children(&self) -> &AstNode<'a, 'b, Vec<'a, JSXChild<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn closing_element(&self) -> Option<&AstNode<'a, 'b, JSXClosingElement>> {
        self.allocator
            .alloc(self.inner.closing_element.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXElement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXOpeningElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXElementName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXOpeningElement(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn attributes(&self) -> &AstNode<'a, 'b, Vec<'a, JSXAttributeItem<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXClosingElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXElementName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXFragment<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn opening_fragment(&self) -> &JSXOpeningFragment {
        &self.inner.opening_fragment
    }

    pub fn children(&self) -> &AstNode<'a, 'b, Vec<'a, JSXChild<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn closing_fragment(&self) -> &JSXClosingFragment {
        &self.inner.closing_fragment
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXOpeningFragment> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXClosingFragment> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXElementName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::JSXElementName(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            JSXElementName::Identifier(s) => {
                AstNode::<'a, 'b, JSXIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXElementName::IdentifierReference(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            JSXElementName::NamespacedName(s) => {
                AstNode::<'a, 'b, JSXNamespacedName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXElementName::MemberExpression(s) => AstNode::<'a, 'b, JSXMemberExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            JSXElementName::ThisExpression(s) => {
                AstNode::<'a, 'b, ThisExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXElementName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXNamespacedName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn namespace(&self) -> &AstNode<'a, 'b, JSXIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.namespace,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXMemberExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, JSXMemberExpressionObject> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn property(&self) -> &AstNode<'a, 'b, JSXIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::JSXMemberExpressionObject(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNode::<'a, 'b, IdentifierReference> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNode::<'a, 'b, JSXMemberExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNode::<'a, 'b, ThisExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXExpressionContainer<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, JSXExpression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXExpressionContainer(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXExpression::EmptyExpression(s) => AstNode::<'a, 'b, JSXEmptyExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_expression!(JSXExpression) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXExpression<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXEmptyExpression> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXAttributeItem<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::JSXAttributeItem(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            JSXAttributeItem::Attribute(s) => {
                AstNode::<'a, 'b, JSXAttribute> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXAttributeItem::SpreadAttribute(s) => AstNode::<'a, 'b, JSXSpreadAttribute> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXAttributeItem<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXAttribute<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXAttributeName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXAttribute(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn value(&self) -> Option<&AstNode<'a, 'b, JSXAttributeValue>> {
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXAttribute(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXSpreadAttribute<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadAttribute(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXAttributeName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXAttributeName::Identifier(s) => {
                AstNode::<'a, 'b, JSXIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXAttributeName::NamespacedName(s) => {
                AstNode::<'a, 'b, JSXNamespacedName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXAttributeName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXAttributeValue<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNode::<'a, 'b, JSXExpressionContainer> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            JSXAttributeValue::Element(s) => {
                AstNode::<'a, 'b, JSXElement> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            JSXAttributeValue::Fragment(s) => {
                AstNode::<'a, 'b, JSXFragment> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXAttributeValue<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXChild<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXChild::Text(s) => {
                AstNode::<'a, 'b, JSXText> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            JSXChild::Element(s) => {
                AstNode::<'a, 'b, JSXElement> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            JSXChild::Fragment(s) => {
                AstNode::<'a, 'b, JSXFragment> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXChild::ExpressionContainer(s) => AstNode::<'a, 'b, JSXExpressionContainer> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            JSXChild::Spread(s) => {
                AstNode::<'a, 'b, JSXSpreadChild> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXChild<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXSpreadChild<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadChild(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXText<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSThisParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSThisParameter(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSEnumDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, TSEnumBody> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSEnumBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn members(&self) -> &AstNode<'a, 'b, Vec<'a, TSEnumMember<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumBody(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSEnumMember<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, TSEnumMemberName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumMember(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn initializer(&self) -> Option<&AstNode<'a, 'b, Expression>> {
        self.allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSEnumMember(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSEnumMemberName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSEnumMemberName::Identifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSEnumMemberName::String(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSEnumMemberName::ComputedString(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNode::<'a, 'b, TemplateLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSEnumMemberName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeAnnotation<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAnnotation(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSLiteralType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn literal(&self) -> &AstNode<'a, 'b, TSLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.literal,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSLiteralType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSLiteral::BooleanLiteral(s) => {
                AstNode::<'a, 'b, BooleanLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::NumericLiteral(s) => {
                AstNode::<'a, 'b, NumericLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::BigIntLiteral(s) => {
                AstNode::<'a, 'b, BigIntLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::TemplateLiteral(s) => {
                AstNode::<'a, 'b, TemplateLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::UnaryExpression(s) => {
                AstNode::<'a, 'b, UnaryExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSLiteral<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSType::TSAnyKeyword(s) => {
                AstNode::<'a, 'b, TSAnyKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSBigIntKeyword(s) => {
                AstNode::<'a, 'b, TSBigIntKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSBooleanKeyword(s) => {
                AstNode::<'a, 'b, TSBooleanKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSIntrinsicKeyword(s) => AstNode::<'a, 'b, TSIntrinsicKeyword> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSNeverKeyword(s) => {
                AstNode::<'a, 'b, TSNeverKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSNullKeyword(s) => {
                AstNode::<'a, 'b, TSNullKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSNumberKeyword(s) => {
                AstNode::<'a, 'b, TSNumberKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSObjectKeyword(s) => {
                AstNode::<'a, 'b, TSObjectKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSStringKeyword(s) => {
                AstNode::<'a, 'b, TSStringKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSSymbolKeyword(s) => {
                AstNode::<'a, 'b, TSSymbolKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSUndefinedKeyword(s) => AstNode::<'a, 'b, TSUndefinedKeyword> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSUnknownKeyword(s) => {
                AstNode::<'a, 'b, TSUnknownKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSVoidKeyword(s) => {
                AstNode::<'a, 'b, TSVoidKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSArrayType(s) => {
                AstNode::<'a, 'b, TSArrayType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSConditionalType(s) => {
                AstNode::<'a, 'b, TSConditionalType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSConstructorType(s) => {
                AstNode::<'a, 'b, TSConstructorType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSFunctionType(s) => {
                AstNode::<'a, 'b, TSFunctionType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSImportType(s) => {
                AstNode::<'a, 'b, TSImportType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSIndexedAccessType(s) => AstNode::<'a, 'b, TSIndexedAccessType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSInferType(s) => {
                AstNode::<'a, 'b, TSInferType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSIntersectionType(s) => AstNode::<'a, 'b, TSIntersectionType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSLiteralType(s) => {
                AstNode::<'a, 'b, TSLiteralType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSMappedType(s) => {
                AstNode::<'a, 'b, TSMappedType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSNamedTupleMember(s) => AstNode::<'a, 'b, TSNamedTupleMember> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSTemplateLiteralType(s) => AstNode::<'a, 'b, TSTemplateLiteralType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSThisType(s) => {
                AstNode::<'a, 'b, TSThisType> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            TSType::TSTupleType(s) => {
                AstNode::<'a, 'b, TSTupleType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeLiteral(s) => {
                AstNode::<'a, 'b, TSTypeLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeOperatorType(s) => {
                AstNode::<'a, 'b, TSTypeOperator> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypePredicate(s) => {
                AstNode::<'a, 'b, TSTypePredicate> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeQuery(s) => {
                AstNode::<'a, 'b, TSTypeQuery> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeReference(s) => {
                AstNode::<'a, 'b, TSTypeReference> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSUnionType(s) => {
                AstNode::<'a, 'b, TSUnionType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSParenthesizedType(s) => AstNode::<'a, 'b, TSParenthesizedType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::JSDocNullableType(s) => {
                AstNode::<'a, 'b, JSDocNullableType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::JSDocNonNullableType(s) => AstNode::<'a, 'b, JSDocNonNullableType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::JSDocUnknownType(s) => {
                AstNode::<'a, 'b, JSDocUnknownType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSType<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSConditionalType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn check_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.check_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn extends_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn true_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.true_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn false_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.false_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSUnionType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn types(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSUnionType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIntersectionType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn types(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIntersectionType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSParenthesizedType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSParenthesizedType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeOperator<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeOperator(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSArrayType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn element_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSArrayType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIndexedAccessType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn index_type(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.index_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTupleType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn element_types(&self) -> &AstNode<'a, 'b, Vec<'a, TSTupleElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTupleType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNamedTupleMember<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn element_type(&self) -> &AstNode<'a, 'b, TSTupleElement> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSOptionalType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSOptionalType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSRestType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSRestType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTupleElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                AstNode::<'a, 'b, TSOptionalType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSTupleElement::TSRestType(s) => {
                AstNode::<'a, 'b, TSRestType> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            it @ match_ts_type!(TSTupleElement) => AstNode::<'a, 'b, TSType> {
                inner: it.to_ts_type(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTupleElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSAnyKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSStringKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSBooleanKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNumberKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNeverKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIntrinsicKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSUnknownKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNullKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSUndefinedKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSVoidKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSSymbolKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSThisType> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSObjectKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSBigIntKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeReference<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_name(&self) -> &AstNode<'a, 'b, TSTypeName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeReference(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTypeName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::TSTypeName(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            TSTypeName::IdentifierReference(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSTypeName::QualifiedName(s) => {
                AstNode::<'a, 'b, TSQualifiedName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTypeName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSQualifiedName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, TSTypeName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeParameterInstantiation<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn params(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeParameterInstantiation(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeParameter(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn constraint(&self) -> Option<&AstNode<'a, 'b, TSType>> {
        self.allocator
            .alloc(self.inner.constraint.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeParameter(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn default(&self) -> Option<&AstNode<'a, 'b, TSType>> {
        self.allocator
            .alloc(self.inner.default.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeParameter(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn r#in(&self) -> bool {
        self.inner.r#in
    }

    pub fn out(&self) -> bool {
        self.inner.out
    }

    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeParameterDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn params(&self) -> &AstNode<'a, 'b, Vec<'a, TSTypeParameter<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeParameterDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeAliasDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSClassImplements<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, TSTypeName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSClassImplements(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInterfaceDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn extends(&self) -> &AstNode<'a, 'b, Vec<'a, TSInterfaceHeritage<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, TSInterfaceBody> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInterfaceBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, TSSignature<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceBody(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSPropertySignature<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSPropertySignature(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSSignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSSignature::TSIndexSignature(s) => {
                AstNode::<'a, 'b, TSIndexSignature> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSSignature::TSPropertySignature(s) => AstNode::<'a, 'b, TSPropertySignature> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSSignature::TSCallSignatureDeclaration(s) => {
                AstNode::<'a, 'b, TSCallSignatureDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNode::<'a, 'b, TSConstructSignatureDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            TSSignature::TSMethodSignature(s) => {
                AstNode::<'a, 'b, TSMethodSignature> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSSignature<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIndexSignature<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn parameters(&self) -> &AstNode<'a, 'b, Vec<'a, TSIndexSignatureName<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameters,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexSignature(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSTypeAnnotation> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexSignature(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSCallSignatureDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSCallSignatureDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSCallSignatureDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSCallSignatureDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSCallSignatureDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSMethodSignature<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn kind(&self) -> TSMethodSignatureKind {
        self.inner.kind
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMethodSignature(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSConstructSignatureDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSConstructSignatureDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConstructSignatureDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSConstructSignatureDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIndexSignatureName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSTypeAnnotation> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexSignatureName(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInterfaceHeritage<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypePredicate<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn parameter_name(&self) -> &AstNode<'a, 'b, TSTypePredicateName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameter_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypePredicate(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypePredicate(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTypePredicateName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSTypePredicateName::This(s) => {
                AstNode::<'a, 'b, TSThisType> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTypePredicateName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSModuleDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, TSModuleDeclarationName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> Option<&AstNode<'a, 'b, TSModuleDeclarationBody>> {
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn kind(&self) -> TSModuleDeclarationKind {
        self.inner.kind
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                AstNode::<'a, 'b, BindingIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSModuleDeclarationBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                AstNode::<'a, 'b, TSModuleDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNode::<'a, 'b, TSModuleBlock> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSModuleDeclarationBody<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSModuleBlock<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn directives(&self) -> &AstNode<'a, 'b, Vec<'a, Directive<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn members(&self) -> &AstNode<'a, 'b, Vec<'a, TSSignature<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeLiteral(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInferType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameter(&self) -> &AstNode<'a, 'b, TSTypeParameter> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInferType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeQuery<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expr_name(&self) -> &AstNode<'a, 'b, TSTypeQueryExprName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expr_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeQuery(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeQuery(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNode::<'a, 'b, TSImportType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => AstNode::<'a, 'b, TSTypeName> {
                inner: it.to_ts_type_name(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSImportType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn options(&self) -> Option<&AstNode<'a, 'b, ObjectExpression>> {
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn qualifier(&self) -> Option<&AstNode<'a, 'b, TSTypeName>> {
        self.allocator
            .alloc(self.inner.qualifier.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSFunctionType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSFunctionType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSFunctionType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSFunctionType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> &AstNode<'a, 'b, TSTypeAnnotation> {
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSFunctionType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSConstructorType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSConstructorType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConstructorType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn return_type(&self) -> &AstNode<'a, 'b, TSTypeAnnotation> {
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConstructorType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSMappedType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameter(&self) -> &AstNode<'a, 'b, TSTypeParameter> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMappedType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn name_type(&self) -> Option<&AstNode<'a, 'b, TSType>> {
        self.allocator
            .alloc(self.inner.name_type.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMappedType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSType>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMappedType(AstNode {
                    inner: self.inner,
                    parent: self.parent,
                    allocator: self.allocator,
                })),
            }))
            .as_ref()
    }

    pub fn optional(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.optional
    }

    pub fn readonly(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.readonly
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTemplateLiteralType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn quasis(&self) -> &AstNode<'a, 'b, Vec<'a, TemplateElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn types(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSAsExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSSatisfiesExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeAssertion<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSImportEqualsDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportEqualsDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn module_reference(&self) -> &AstNode<'a, 'b, TSModuleReference> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.module_reference,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSModuleReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::TSModuleReference(AstNode {
            inner: self.inner,
            parent: self.parent,
            allocator: self.allocator,
        }));
        match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNode::<'a, 'b, TSExternalModuleReference> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_ts_type_name!(TSModuleReference) => AstNode::<'a, 'b, TSTypeName> {
                inner: it.to_ts_type_name(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSModuleReference<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSExternalModuleReference<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, StringLiteral> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExternalModuleReference(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNonNullExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNonNullExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, Decorator<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Decorator(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSExportAssignment<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExportAssignment(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNamespaceExportDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, IdentifierName> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamespaceExportDeclaration(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInstantiationExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn type_arguments(&self) -> &AstNode<'a, 'b, TSTypeParameterInstantiation> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_arguments.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSDocNullableType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSDocNullableType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSDocNonNullableType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSDocNonNullableType(AstNode {
                inner: self.inner,
                parent: self.parent,
                allocator: self.allocator,
            })),
        })
    }

    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSDocUnknownType> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
