// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_kind.rs`

use oxc_span::{GetSpan, Span};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

#[derive(Debug, Clone, Copy)]
pub enum AstType {
    BooleanLiteral,
    NullLiteral,
    NumericLiteral,
    BigIntLiteral,
    RegExpLiteral,
    StringLiteral,
    Program,
    IdentifierName,
    IdentifierReference,
    BindingIdentifier,
    LabelIdentifier,
    ThisExpression,
    ArrayExpression,
    ArrayExpressionElement,
    Elision,
    ObjectExpression,
    ObjectProperty,
    PropertyKey,
    TemplateLiteral,
    TaggedTemplateExpression,
    MemberExpression,
    CallExpression,
    NewExpression,
    MetaProperty,
    SpreadElement,
    Argument,
    UpdateExpression,
    UnaryExpression,
    BinaryExpression,
    PrivateInExpression,
    LogicalExpression,
    ConditionalExpression,
    AssignmentExpression,
    AssignmentTarget,
    SimpleAssignmentTarget,
    AssignmentTargetPattern,
    ArrayAssignmentTarget,
    ObjectAssignmentTarget,
    AssignmentTargetWithDefault,
    SequenceExpression,
    Super,
    AwaitExpression,
    ChainExpression,
    ParenthesizedExpression,
    Directive,
    Hashbang,
    BlockStatement,
    VariableDeclaration,
    VariableDeclarator,
    EmptyStatement,
    ExpressionStatement,
    IfStatement,
    DoWhileStatement,
    WhileStatement,
    ForStatement,
    ForStatementInit,
    ForInStatement,
    ForOfStatement,
    ContinueStatement,
    BreakStatement,
    ReturnStatement,
    WithStatement,
    SwitchStatement,
    SwitchCase,
    LabeledStatement,
    ThrowStatement,
    TryStatement,
    FinallyClause,
    CatchClause,
    CatchParameter,
    DebuggerStatement,
    AssignmentPattern,
    ObjectPattern,
    ArrayPattern,
    BindingRestElement,
    Function,
    FormalParameters,
    FormalParameter,
    FunctionBody,
    ArrowFunctionExpression,
    YieldExpression,
    Class,
    ClassHeritage,
    ClassBody,
    MethodDefinition,
    PropertyDefinition,
    PrivateIdentifier,
    StaticBlock,
    ModuleDeclaration,
    ImportExpression,
    ImportDeclaration,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    ExportNamedDeclaration,
    ExportDefaultDeclaration,
    ExportAllDeclaration,
    ExportSpecifier,
    TSThisParameter,
    TSEnumDeclaration,
    TSEnumMember,
    TSTypeAnnotation,
    TSLiteralType,
    TSConditionalType,
    TSUnionType,
    TSIntersectionType,
    TSParenthesizedType,
    TSIndexedAccessType,
    TSNamedTupleMember,
    TSAnyKeyword,
    TSStringKeyword,
    TSBooleanKeyword,
    TSNumberKeyword,
    TSNeverKeyword,
    TSIntrinsicKeyword,
    TSUnknownKeyword,
    TSNullKeyword,
    TSUndefinedKeyword,
    TSVoidKeyword,
    TSSymbolKeyword,
    TSThisType,
    TSObjectKeyword,
    TSBigIntKeyword,
    TSTypeReference,
    TSTypeName,
    TSQualifiedName,
    TSTypeParameterInstantiation,
    TSTypeParameter,
    TSTypeParameterDeclaration,
    TSTypeAliasDeclaration,
    TSClassImplements,
    TSInterfaceDeclaration,
    TSPropertySignature,
    TSMethodSignature,
    TSConstructSignatureDeclaration,
    TSInterfaceHeritage,
    TSModuleDeclaration,
    TSModuleBlock,
    TSTypeLiteral,
    TSInferType,
    TSTypeQuery,
    TSImportType,
    TSMappedType,
    TSTemplateLiteralType,
    TSAsExpression,
    TSSatisfiesExpression,
    TSTypeAssertion,
    TSImportEqualsDeclaration,
    TSModuleReference,
    TSExternalModuleReference,
    TSNonNullExpression,
    Decorator,
    TSExportAssignment,
    TSInstantiationExpression,
    JSXElement,
    JSXOpeningElement,
    JSXClosingElement,
    JSXFragment,
    JSXElementName,
    JSXNamespacedName,
    JSXMemberExpression,
    JSXMemberExpressionObject,
    JSXExpressionContainer,
    JSXAttributeItem,
    JSXSpreadAttribute,
    JSXIdentifier,
    JSXText,
    ExpressionArrayElement,
}

/// Untyped AST Node Kind
#[derive(Debug, Clone, Copy)]
pub enum AstKind<'a> {
    BooleanLiteral(&'a BooleanLiteral),
    NullLiteral(&'a NullLiteral),
    NumericLiteral(&'a NumericLiteral<'a>),
    BigIntLiteral(&'a BigIntLiteral<'a>),
    RegExpLiteral(&'a RegExpLiteral<'a>),
    StringLiteral(&'a StringLiteral<'a>),
    Program(&'a Program<'a>),
    IdentifierName(&'a IdentifierName<'a>),
    IdentifierReference(&'a IdentifierReference<'a>),
    BindingIdentifier(&'a BindingIdentifier<'a>),
    LabelIdentifier(&'a LabelIdentifier<'a>),
    ThisExpression(&'a ThisExpression),
    ArrayExpression(&'a ArrayExpression<'a>),
    ArrayExpressionElement(&'a ArrayExpressionElement<'a>),
    Elision(&'a Elision),
    ObjectExpression(&'a ObjectExpression<'a>),
    ObjectProperty(&'a ObjectProperty<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    TemplateLiteral(&'a TemplateLiteral<'a>),
    TaggedTemplateExpression(&'a TaggedTemplateExpression<'a>),
    MemberExpression(&'a MemberExpression<'a>),
    CallExpression(&'a CallExpression<'a>),
    NewExpression(&'a NewExpression<'a>),
    MetaProperty(&'a MetaProperty<'a>),
    SpreadElement(&'a SpreadElement<'a>),
    Argument(&'a Argument<'a>),
    UpdateExpression(&'a UpdateExpression<'a>),
    UnaryExpression(&'a UnaryExpression<'a>),
    BinaryExpression(&'a BinaryExpression<'a>),
    PrivateInExpression(&'a PrivateInExpression<'a>),
    LogicalExpression(&'a LogicalExpression<'a>),
    ConditionalExpression(&'a ConditionalExpression<'a>),
    AssignmentExpression(&'a AssignmentExpression<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
    AssignmentTargetPattern(&'a AssignmentTargetPattern<'a>),
    ArrayAssignmentTarget(&'a ArrayAssignmentTarget<'a>),
    ObjectAssignmentTarget(&'a ObjectAssignmentTarget<'a>),
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>),
    SequenceExpression(&'a SequenceExpression<'a>),
    Super(&'a Super),
    AwaitExpression(&'a AwaitExpression<'a>),
    ChainExpression(&'a ChainExpression<'a>),
    ParenthesizedExpression(&'a ParenthesizedExpression<'a>),
    Directive(&'a Directive<'a>),
    Hashbang(&'a Hashbang<'a>),
    BlockStatement(&'a BlockStatement<'a>),
    VariableDeclaration(&'a VariableDeclaration<'a>),
    VariableDeclarator(&'a VariableDeclarator<'a>),
    EmptyStatement(&'a EmptyStatement),
    ExpressionStatement(&'a ExpressionStatement<'a>),
    IfStatement(&'a IfStatement<'a>),
    DoWhileStatement(&'a DoWhileStatement<'a>),
    WhileStatement(&'a WhileStatement<'a>),
    ForStatement(&'a ForStatement<'a>),
    ForStatementInit(&'a ForStatementInit<'a>),
    ForInStatement(&'a ForInStatement<'a>),
    ForOfStatement(&'a ForOfStatement<'a>),
    ContinueStatement(&'a ContinueStatement<'a>),
    BreakStatement(&'a BreakStatement<'a>),
    ReturnStatement(&'a ReturnStatement<'a>),
    WithStatement(&'a WithStatement<'a>),
    SwitchStatement(&'a SwitchStatement<'a>),
    SwitchCase(&'a SwitchCase<'a>),
    LabeledStatement(&'a LabeledStatement<'a>),
    ThrowStatement(&'a ThrowStatement<'a>),
    TryStatement(&'a TryStatement<'a>),
    FinallyClause(&'a BlockStatement<'a>),
    CatchClause(&'a CatchClause<'a>),
    CatchParameter(&'a CatchParameter<'a>),
    DebuggerStatement(&'a DebuggerStatement),
    AssignmentPattern(&'a AssignmentPattern<'a>),
    ObjectPattern(&'a ObjectPattern<'a>),
    ArrayPattern(&'a ArrayPattern<'a>),
    BindingRestElement(&'a BindingRestElement<'a>),
    Function(&'a Function<'a>),
    FormalParameters(&'a FormalParameters<'a>),
    FormalParameter(&'a FormalParameter<'a>),
    FunctionBody(&'a FunctionBody<'a>),
    ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
    YieldExpression(&'a YieldExpression<'a>),
    Class(&'a Class<'a>),
    ClassHeritage(&'a Expression<'a>),
    ClassBody(&'a ClassBody<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),
    PropertyDefinition(&'a PropertyDefinition<'a>),
    PrivateIdentifier(&'a PrivateIdentifier<'a>),
    StaticBlock(&'a StaticBlock<'a>),
    ModuleDeclaration(&'a ModuleDeclaration<'a>),
    ImportExpression(&'a ImportExpression<'a>),
    ImportDeclaration(&'a ImportDeclaration<'a>),
    ImportSpecifier(&'a ImportSpecifier<'a>),
    ImportDefaultSpecifier(&'a ImportDefaultSpecifier<'a>),
    ImportNamespaceSpecifier(&'a ImportNamespaceSpecifier<'a>),
    ExportNamedDeclaration(&'a ExportNamedDeclaration<'a>),
    ExportDefaultDeclaration(&'a ExportDefaultDeclaration<'a>),
    ExportAllDeclaration(&'a ExportAllDeclaration<'a>),
    ExportSpecifier(&'a ExportSpecifier<'a>),
    TSThisParameter(&'a TSThisParameter<'a>),
    TSEnumDeclaration(&'a TSEnumDeclaration<'a>),
    TSEnumMember(&'a TSEnumMember<'a>),
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>),
    TSLiteralType(&'a TSLiteralType<'a>),
    TSConditionalType(&'a TSConditionalType<'a>),
    TSUnionType(&'a TSUnionType<'a>),
    TSIntersectionType(&'a TSIntersectionType<'a>),
    TSParenthesizedType(&'a TSParenthesizedType<'a>),
    TSIndexedAccessType(&'a TSIndexedAccessType<'a>),
    TSNamedTupleMember(&'a TSNamedTupleMember<'a>),
    TSAnyKeyword(&'a TSAnyKeyword),
    TSStringKeyword(&'a TSStringKeyword),
    TSBooleanKeyword(&'a TSBooleanKeyword),
    TSNumberKeyword(&'a TSNumberKeyword),
    TSNeverKeyword(&'a TSNeverKeyword),
    TSIntrinsicKeyword(&'a TSIntrinsicKeyword),
    TSUnknownKeyword(&'a TSUnknownKeyword),
    TSNullKeyword(&'a TSNullKeyword),
    TSUndefinedKeyword(&'a TSUndefinedKeyword),
    TSVoidKeyword(&'a TSVoidKeyword),
    TSSymbolKeyword(&'a TSSymbolKeyword),
    TSThisType(&'a TSThisType),
    TSObjectKeyword(&'a TSObjectKeyword),
    TSBigIntKeyword(&'a TSBigIntKeyword),
    TSTypeReference(&'a TSTypeReference<'a>),
    TSTypeName(&'a TSTypeName<'a>),
    TSQualifiedName(&'a TSQualifiedName<'a>),
    TSTypeParameterInstantiation(&'a TSTypeParameterInstantiation<'a>),
    TSTypeParameter(&'a TSTypeParameter<'a>),
    TSTypeParameterDeclaration(&'a TSTypeParameterDeclaration<'a>),
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>),
    TSClassImplements(&'a TSClassImplements<'a>),
    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>),
    TSPropertySignature(&'a TSPropertySignature<'a>),
    TSMethodSignature(&'a TSMethodSignature<'a>),
    TSConstructSignatureDeclaration(&'a TSConstructSignatureDeclaration<'a>),
    TSInterfaceHeritage(&'a TSInterfaceHeritage<'a>),
    TSModuleDeclaration(&'a TSModuleDeclaration<'a>),
    TSModuleBlock(&'a TSModuleBlock<'a>),
    TSTypeLiteral(&'a TSTypeLiteral<'a>),
    TSInferType(&'a TSInferType<'a>),
    TSTypeQuery(&'a TSTypeQuery<'a>),
    TSImportType(&'a TSImportType<'a>),
    TSMappedType(&'a TSMappedType<'a>),
    TSTemplateLiteralType(&'a TSTemplateLiteralType<'a>),
    TSAsExpression(&'a TSAsExpression<'a>),
    TSSatisfiesExpression(&'a TSSatisfiesExpression<'a>),
    TSTypeAssertion(&'a TSTypeAssertion<'a>),
    TSImportEqualsDeclaration(&'a TSImportEqualsDeclaration<'a>),
    TSModuleReference(&'a TSModuleReference<'a>),
    TSExternalModuleReference(&'a TSExternalModuleReference<'a>),
    TSNonNullExpression(&'a TSNonNullExpression<'a>),
    Decorator(&'a Decorator<'a>),
    TSExportAssignment(&'a TSExportAssignment<'a>),
    TSInstantiationExpression(&'a TSInstantiationExpression<'a>),
    JSXElement(&'a JSXElement<'a>),
    JSXOpeningElement(&'a JSXOpeningElement<'a>),
    JSXClosingElement(&'a JSXClosingElement<'a>),
    JSXFragment(&'a JSXFragment<'a>),
    JSXElementName(&'a JSXElementName<'a>),
    JSXNamespacedName(&'a JSXNamespacedName<'a>),
    JSXMemberExpression(&'a JSXMemberExpression<'a>),
    JSXMemberExpressionObject(&'a JSXMemberExpressionObject<'a>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXAttributeItem(&'a JSXAttributeItem<'a>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXIdentifier(&'a JSXIdentifier<'a>),
    JSXText(&'a JSXText<'a>),
    ExpressionArrayElement(&'a Expression<'a>),
}

impl<'a> GetSpan for AstKind<'a> {
    #[allow(clippy::match_same_arms)]
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::Program(it) => it.span(),
            Self::IdentifierName(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::BindingIdentifier(it) => it.span(),
            Self::LabelIdentifier(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrayExpressionElement(it) => it.span(),
            Self::Elision(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ObjectProperty(it) => it.span(),
            Self::PropertyKey(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::MemberExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::SpreadElement(it) => it.span(),
            Self::Argument(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AssignmentTarget(it) => it.span(),
            Self::SimpleAssignmentTarget(it) => it.span(),
            Self::AssignmentTargetPattern(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
            Self::AssignmentTargetWithDefault(it) => it.span(),
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
            Self::ForStatementInit(it) => it.span(),
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
            Self::FinallyClause(it) => it.span(),
            Self::CatchClause(it) => it.span(),
            Self::CatchParameter(it) => it.span(),
            Self::DebuggerStatement(it) => it.span(),
            Self::AssignmentPattern(it) => it.span(),
            Self::ObjectPattern(it) => it.span(),
            Self::ArrayPattern(it) => it.span(),
            Self::BindingRestElement(it) => it.span(),
            Self::Function(it) => it.span(),
            Self::FormalParameters(it) => it.span(),
            Self::FormalParameter(it) => it.span(),
            Self::FunctionBody(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::Class(it) => it.span(),
            Self::ClassHeritage(it) => it.span(),
            Self::ClassBody(it) => it.span(),
            Self::MethodDefinition(it) => it.span(),
            Self::PropertyDefinition(it) => it.span(),
            Self::PrivateIdentifier(it) => it.span(),
            Self::StaticBlock(it) => it.span(),
            Self::ModuleDeclaration(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::ImportDeclaration(it) => it.span(),
            Self::ImportSpecifier(it) => it.span(),
            Self::ImportDefaultSpecifier(it) => it.span(),
            Self::ImportNamespaceSpecifier(it) => it.span(),
            Self::ExportNamedDeclaration(it) => it.span(),
            Self::ExportDefaultDeclaration(it) => it.span(),
            Self::ExportAllDeclaration(it) => it.span(),
            Self::ExportSpecifier(it) => it.span(),
            Self::TSThisParameter(it) => it.span(),
            Self::TSEnumDeclaration(it) => it.span(),
            Self::TSEnumMember(it) => it.span(),
            Self::TSTypeAnnotation(it) => it.span(),
            Self::TSLiteralType(it) => it.span(),
            Self::TSConditionalType(it) => it.span(),
            Self::TSUnionType(it) => it.span(),
            Self::TSIntersectionType(it) => it.span(),
            Self::TSParenthesizedType(it) => it.span(),
            Self::TSIndexedAccessType(it) => it.span(),
            Self::TSNamedTupleMember(it) => it.span(),
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
            Self::TSTypeName(it) => it.span(),
            Self::TSQualifiedName(it) => it.span(),
            Self::TSTypeParameterInstantiation(it) => it.span(),
            Self::TSTypeParameter(it) => it.span(),
            Self::TSTypeParameterDeclaration(it) => it.span(),
            Self::TSTypeAliasDeclaration(it) => it.span(),
            Self::TSClassImplements(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::TSPropertySignature(it) => it.span(),
            Self::TSMethodSignature(it) => it.span(),
            Self::TSConstructSignatureDeclaration(it) => it.span(),
            Self::TSInterfaceHeritage(it) => it.span(),
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSModuleBlock(it) => it.span(),
            Self::TSTypeLiteral(it) => it.span(),
            Self::TSInferType(it) => it.span(),
            Self::TSTypeQuery(it) => it.span(),
            Self::TSImportType(it) => it.span(),
            Self::TSMappedType(it) => it.span(),
            Self::TSTemplateLiteralType(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSImportEqualsDeclaration(it) => it.span(),
            Self::TSModuleReference(it) => it.span(),
            Self::TSExternalModuleReference(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::Decorator(it) => it.span(),
            Self::TSExportAssignment(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXOpeningElement(it) => it.span(),
            Self::JSXClosingElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::JSXElementName(it) => it.span(),
            Self::JSXNamespacedName(it) => it.span(),
            Self::JSXMemberExpression(it) => it.span(),
            Self::JSXMemberExpressionObject(it) => it.span(),
            Self::JSXExpressionContainer(it) => it.span(),
            Self::JSXAttributeItem(it) => it.span(),
            Self::JSXSpreadAttribute(it) => it.span(),
            Self::JSXIdentifier(it) => it.span(),
            Self::JSXText(it) => it.span(),
            Self::ExpressionArrayElement(it) => it.span(),
        }
    }
}

impl<'a> AstKind<'a> {
    pub fn as_boolean_literal(&self) -> Option<&BooleanLiteral> {
        if let Self::BooleanLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_null_literal(&self) -> Option<&NullLiteral> {
        if let Self::NullLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_numeric_literal(&self) -> Option<&NumericLiteral<'a>> {
        if let Self::NumericLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_big_int_literal(&self) -> Option<&BigIntLiteral<'a>> {
        if let Self::BigIntLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_reg_exp_literal(&self) -> Option<&RegExpLiteral<'a>> {
        if let Self::RegExpLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_string_literal(&self) -> Option<&StringLiteral<'a>> {
        if let Self::StringLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_program(&self) -> Option<&Program<'a>> {
        if let Self::Program(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_identifier_name(&self) -> Option<&IdentifierName<'a>> {
        if let Self::IdentifierName(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_identifier_reference(&self) -> Option<&IdentifierReference<'a>> {
        if let Self::IdentifierReference(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_binding_identifier(&self) -> Option<&BindingIdentifier<'a>> {
        if let Self::BindingIdentifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_label_identifier(&self) -> Option<&LabelIdentifier<'a>> {
        if let Self::LabelIdentifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_this_expression(&self) -> Option<&ThisExpression> {
        if let Self::ThisExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_array_expression(&self) -> Option<&ArrayExpression<'a>> {
        if let Self::ArrayExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_array_expression_element(&self) -> Option<&ArrayExpressionElement<'a>> {
        if let Self::ArrayExpressionElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_elision(&self) -> Option<&Elision> {
        if let Self::Elision(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_object_expression(&self) -> Option<&ObjectExpression<'a>> {
        if let Self::ObjectExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_object_property(&self) -> Option<&ObjectProperty<'a>> {
        if let Self::ObjectProperty(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_property_key(&self) -> Option<&PropertyKey<'a>> {
        if let Self::PropertyKey(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_template_literal(&self) -> Option<&TemplateLiteral<'a>> {
        if let Self::TemplateLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_tagged_template_expression(&self) -> Option<&TaggedTemplateExpression<'a>> {
        if let Self::TaggedTemplateExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if let Self::MemberExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_call_expression(&self) -> Option<&CallExpression<'a>> {
        if let Self::CallExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_new_expression(&self) -> Option<&NewExpression<'a>> {
        if let Self::NewExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_meta_property(&self) -> Option<&MetaProperty<'a>> {
        if let Self::MetaProperty(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_spread_element(&self) -> Option<&SpreadElement<'a>> {
        if let Self::SpreadElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_argument(&self) -> Option<&Argument<'a>> {
        if let Self::Argument(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_update_expression(&self) -> Option<&UpdateExpression<'a>> {
        if let Self::UpdateExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_unary_expression(&self) -> Option<&UnaryExpression<'a>> {
        if let Self::UnaryExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_binary_expression(&self) -> Option<&BinaryExpression<'a>> {
        if let Self::BinaryExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_private_in_expression(&self) -> Option<&PrivateInExpression<'a>> {
        if let Self::PrivateInExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_logical_expression(&self) -> Option<&LogicalExpression<'a>> {
        if let Self::LogicalExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_conditional_expression(&self) -> Option<&ConditionalExpression<'a>> {
        if let Self::ConditionalExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_assignment_expression(&self) -> Option<&AssignmentExpression<'a>> {
        if let Self::AssignmentExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_assignment_target(&self) -> Option<&AssignmentTarget<'a>> {
        if let Self::AssignmentTarget(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if let Self::SimpleAssignmentTarget(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_assignment_target_pattern(&self) -> Option<&AssignmentTargetPattern<'a>> {
        if let Self::AssignmentTargetPattern(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_array_assignment_target(&self) -> Option<&ArrayAssignmentTarget<'a>> {
        if let Self::ArrayAssignmentTarget(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_object_assignment_target(&self) -> Option<&ObjectAssignmentTarget<'a>> {
        if let Self::ObjectAssignmentTarget(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_assignment_target_with_default(&self) -> Option<&AssignmentTargetWithDefault<'a>> {
        if let Self::AssignmentTargetWithDefault(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_sequence_expression(&self) -> Option<&SequenceExpression<'a>> {
        if let Self::SequenceExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_super(&self) -> Option<&Super> {
        if let Self::Super(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_await_expression(&self) -> Option<&AwaitExpression<'a>> {
        if let Self::AwaitExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_chain_expression(&self) -> Option<&ChainExpression<'a>> {
        if let Self::ChainExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_parenthesized_expression(&self) -> Option<&ParenthesizedExpression<'a>> {
        if let Self::ParenthesizedExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_directive(&self) -> Option<&Directive<'a>> {
        if let Self::Directive(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_hashbang(&self) -> Option<&Hashbang<'a>> {
        if let Self::Hashbang(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_block_statement(&self) -> Option<&BlockStatement<'a>> {
        if let Self::BlockStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_variable_declaration(&self) -> Option<&VariableDeclaration<'a>> {
        if let Self::VariableDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_variable_declarator(&self) -> Option<&VariableDeclarator<'a>> {
        if let Self::VariableDeclarator(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_empty_statement(&self) -> Option<&EmptyStatement> {
        if let Self::EmptyStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_expression_statement(&self) -> Option<&ExpressionStatement<'a>> {
        if let Self::ExpressionStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_if_statement(&self) -> Option<&IfStatement<'a>> {
        if let Self::IfStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_do_while_statement(&self) -> Option<&DoWhileStatement<'a>> {
        if let Self::DoWhileStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_while_statement(&self) -> Option<&WhileStatement<'a>> {
        if let Self::WhileStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_for_statement(&self) -> Option<&ForStatement<'a>> {
        if let Self::ForStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_for_statement_init(&self) -> Option<&ForStatementInit<'a>> {
        if let Self::ForStatementInit(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_for_in_statement(&self) -> Option<&ForInStatement<'a>> {
        if let Self::ForInStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_for_of_statement(&self) -> Option<&ForOfStatement<'a>> {
        if let Self::ForOfStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_continue_statement(&self) -> Option<&ContinueStatement<'a>> {
        if let Self::ContinueStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_break_statement(&self) -> Option<&BreakStatement<'a>> {
        if let Self::BreakStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_return_statement(&self) -> Option<&ReturnStatement<'a>> {
        if let Self::ReturnStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_with_statement(&self) -> Option<&WithStatement<'a>> {
        if let Self::WithStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_switch_statement(&self) -> Option<&SwitchStatement<'a>> {
        if let Self::SwitchStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_switch_case(&self) -> Option<&SwitchCase<'a>> {
        if let Self::SwitchCase(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_labeled_statement(&self) -> Option<&LabeledStatement<'a>> {
        if let Self::LabeledStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_throw_statement(&self) -> Option<&ThrowStatement<'a>> {
        if let Self::ThrowStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_try_statement(&self) -> Option<&TryStatement<'a>> {
        if let Self::TryStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_finally_clause(&self) -> Option<&BlockStatement<'a>> {
        if let Self::FinallyClause(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_catch_clause(&self) -> Option<&CatchClause<'a>> {
        if let Self::CatchClause(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_catch_parameter(&self) -> Option<&CatchParameter<'a>> {
        if let Self::CatchParameter(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_debugger_statement(&self) -> Option<&DebuggerStatement> {
        if let Self::DebuggerStatement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_assignment_pattern(&self) -> Option<&AssignmentPattern<'a>> {
        if let Self::AssignmentPattern(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_object_pattern(&self) -> Option<&ObjectPattern<'a>> {
        if let Self::ObjectPattern(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_array_pattern(&self) -> Option<&ArrayPattern<'a>> {
        if let Self::ArrayPattern(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_binding_rest_element(&self) -> Option<&BindingRestElement<'a>> {
        if let Self::BindingRestElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Function<'a>> {
        if let Self::Function(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_formal_parameters(&self) -> Option<&FormalParameters<'a>> {
        if let Self::FormalParameters(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_formal_parameter(&self) -> Option<&FormalParameter<'a>> {
        if let Self::FormalParameter(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_function_body(&self) -> Option<&FunctionBody<'a>> {
        if let Self::FunctionBody(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_arrow_function_expression(&self) -> Option<&ArrowFunctionExpression<'a>> {
        if let Self::ArrowFunctionExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_yield_expression(&self) -> Option<&YieldExpression<'a>> {
        if let Self::YieldExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_class(&self) -> Option<&Class<'a>> {
        if let Self::Class(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_class_heritage(&self) -> Option<&Expression<'a>> {
        if let Self::ClassHeritage(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_class_body(&self) -> Option<&ClassBody<'a>> {
        if let Self::ClassBody(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_method_definition(&self) -> Option<&MethodDefinition<'a>> {
        if let Self::MethodDefinition(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_property_definition(&self) -> Option<&PropertyDefinition<'a>> {
        if let Self::PropertyDefinition(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_private_identifier(&self) -> Option<&PrivateIdentifier<'a>> {
        if let Self::PrivateIdentifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_static_block(&self) -> Option<&StaticBlock<'a>> {
        if let Self::StaticBlock(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_module_declaration(&self) -> Option<&ModuleDeclaration<'a>> {
        if let Self::ModuleDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_import_expression(&self) -> Option<&ImportExpression<'a>> {
        if let Self::ImportExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_import_declaration(&self) -> Option<&ImportDeclaration<'a>> {
        if let Self::ImportDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_import_specifier(&self) -> Option<&ImportSpecifier<'a>> {
        if let Self::ImportSpecifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_import_default_specifier(&self) -> Option<&ImportDefaultSpecifier<'a>> {
        if let Self::ImportDefaultSpecifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_import_namespace_specifier(&self) -> Option<&ImportNamespaceSpecifier<'a>> {
        if let Self::ImportNamespaceSpecifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_export_named_declaration(&self) -> Option<&ExportNamedDeclaration<'a>> {
        if let Self::ExportNamedDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_export_default_declaration(&self) -> Option<&ExportDefaultDeclaration<'a>> {
        if let Self::ExportDefaultDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_export_all_declaration(&self) -> Option<&ExportAllDeclaration<'a>> {
        if let Self::ExportAllDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_export_specifier(&self) -> Option<&ExportSpecifier<'a>> {
        if let Self::ExportSpecifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_this_parameter(&self) -> Option<&TSThisParameter<'a>> {
        if let Self::TSThisParameter(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_enum_declaration(&self) -> Option<&TSEnumDeclaration<'a>> {
        if let Self::TSEnumDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_enum_member(&self) -> Option<&TSEnumMember<'a>> {
        if let Self::TSEnumMember(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_annotation(&self) -> Option<&TSTypeAnnotation<'a>> {
        if let Self::TSTypeAnnotation(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_literal_type(&self) -> Option<&TSLiteralType<'a>> {
        if let Self::TSLiteralType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_conditional_type(&self) -> Option<&TSConditionalType<'a>> {
        if let Self::TSConditionalType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_union_type(&self) -> Option<&TSUnionType<'a>> {
        if let Self::TSUnionType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_intersection_type(&self) -> Option<&TSIntersectionType<'a>> {
        if let Self::TSIntersectionType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_parenthesized_type(&self) -> Option<&TSParenthesizedType<'a>> {
        if let Self::TSParenthesizedType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_indexed_access_type(&self) -> Option<&TSIndexedAccessType<'a>> {
        if let Self::TSIndexedAccessType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_named_tuple_member(&self) -> Option<&TSNamedTupleMember<'a>> {
        if let Self::TSNamedTupleMember(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_any_keyword(&self) -> Option<&TSAnyKeyword> {
        if let Self::TSAnyKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_string_keyword(&self) -> Option<&TSStringKeyword> {
        if let Self::TSStringKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_boolean_keyword(&self) -> Option<&TSBooleanKeyword> {
        if let Self::TSBooleanKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_number_keyword(&self) -> Option<&TSNumberKeyword> {
        if let Self::TSNumberKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_never_keyword(&self) -> Option<&TSNeverKeyword> {
        if let Self::TSNeverKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_intrinsic_keyword(&self) -> Option<&TSIntrinsicKeyword> {
        if let Self::TSIntrinsicKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_unknown_keyword(&self) -> Option<&TSUnknownKeyword> {
        if let Self::TSUnknownKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_null_keyword(&self) -> Option<&TSNullKeyword> {
        if let Self::TSNullKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_undefined_keyword(&self) -> Option<&TSUndefinedKeyword> {
        if let Self::TSUndefinedKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_void_keyword(&self) -> Option<&TSVoidKeyword> {
        if let Self::TSVoidKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_symbol_keyword(&self) -> Option<&TSSymbolKeyword> {
        if let Self::TSSymbolKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_this_type(&self) -> Option<&TSThisType> {
        if let Self::TSThisType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_object_keyword(&self) -> Option<&TSObjectKeyword> {
        if let Self::TSObjectKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_big_int_keyword(&self) -> Option<&TSBigIntKeyword> {
        if let Self::TSBigIntKeyword(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_reference(&self) -> Option<&TSTypeReference<'a>> {
        if let Self::TSTypeReference(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_name(&self) -> Option<&TSTypeName<'a>> {
        if let Self::TSTypeName(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_qualified_name(&self) -> Option<&TSQualifiedName<'a>> {
        if let Self::TSQualifiedName(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_parameter_instantiation(&self) -> Option<&TSTypeParameterInstantiation<'a>> {
        if let Self::TSTypeParameterInstantiation(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_parameter(&self) -> Option<&TSTypeParameter<'a>> {
        if let Self::TSTypeParameter(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_parameter_declaration(&self) -> Option<&TSTypeParameterDeclaration<'a>> {
        if let Self::TSTypeParameterDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_alias_declaration(&self) -> Option<&TSTypeAliasDeclaration<'a>> {
        if let Self::TSTypeAliasDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_class_implements(&self) -> Option<&TSClassImplements<'a>> {
        if let Self::TSClassImplements(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_interface_declaration(&self) -> Option<&TSInterfaceDeclaration<'a>> {
        if let Self::TSInterfaceDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_property_signature(&self) -> Option<&TSPropertySignature<'a>> {
        if let Self::TSPropertySignature(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_method_signature(&self) -> Option<&TSMethodSignature<'a>> {
        if let Self::TSMethodSignature(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_construct_signature_declaration(
        &self,
    ) -> Option<&TSConstructSignatureDeclaration<'a>> {
        if let Self::TSConstructSignatureDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_interface_heritage(&self) -> Option<&TSInterfaceHeritage<'a>> {
        if let Self::TSInterfaceHeritage(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_module_declaration(&self) -> Option<&TSModuleDeclaration<'a>> {
        if let Self::TSModuleDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_module_block(&self) -> Option<&TSModuleBlock<'a>> {
        if let Self::TSModuleBlock(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_literal(&self) -> Option<&TSTypeLiteral<'a>> {
        if let Self::TSTypeLiteral(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_infer_type(&self) -> Option<&TSInferType<'a>> {
        if let Self::TSInferType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_query(&self) -> Option<&TSTypeQuery<'a>> {
        if let Self::TSTypeQuery(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_import_type(&self) -> Option<&TSImportType<'a>> {
        if let Self::TSImportType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_mapped_type(&self) -> Option<&TSMappedType<'a>> {
        if let Self::TSMappedType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_template_literal_type(&self) -> Option<&TSTemplateLiteralType<'a>> {
        if let Self::TSTemplateLiteralType(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_as_expression(&self) -> Option<&TSAsExpression<'a>> {
        if let Self::TSAsExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_satisfies_expression(&self) -> Option<&TSSatisfiesExpression<'a>> {
        if let Self::TSSatisfiesExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_type_assertion(&self) -> Option<&TSTypeAssertion<'a>> {
        if let Self::TSTypeAssertion(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_import_equals_declaration(&self) -> Option<&TSImportEqualsDeclaration<'a>> {
        if let Self::TSImportEqualsDeclaration(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_module_reference(&self) -> Option<&TSModuleReference<'a>> {
        if let Self::TSModuleReference(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_external_module_reference(&self) -> Option<&TSExternalModuleReference<'a>> {
        if let Self::TSExternalModuleReference(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_non_null_expression(&self) -> Option<&TSNonNullExpression<'a>> {
        if let Self::TSNonNullExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_decorator(&self) -> Option<&Decorator<'a>> {
        if let Self::Decorator(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_export_assignment(&self) -> Option<&TSExportAssignment<'a>> {
        if let Self::TSExportAssignment(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_ts_instantiation_expression(&self) -> Option<&TSInstantiationExpression<'a>> {
        if let Self::TSInstantiationExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_element(&self) -> Option<&JSXElement<'a>> {
        if let Self::JSXElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_opening_element(&self) -> Option<&JSXOpeningElement<'a>> {
        if let Self::JSXOpeningElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_closing_element(&self) -> Option<&JSXClosingElement<'a>> {
        if let Self::JSXClosingElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_fragment(&self) -> Option<&JSXFragment<'a>> {
        if let Self::JSXFragment(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_element_name(&self) -> Option<&JSXElementName<'a>> {
        if let Self::JSXElementName(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_namespaced_name(&self) -> Option<&JSXNamespacedName<'a>> {
        if let Self::JSXNamespacedName(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_member_expression(&self) -> Option<&JSXMemberExpression<'a>> {
        if let Self::JSXMemberExpression(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_member_expression_object(&self) -> Option<&JSXMemberExpressionObject<'a>> {
        if let Self::JSXMemberExpressionObject(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_expression_container(&self) -> Option<&JSXExpressionContainer<'a>> {
        if let Self::JSXExpressionContainer(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_attribute_item(&self) -> Option<&JSXAttributeItem<'a>> {
        if let Self::JSXAttributeItem(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_spread_attribute(&self) -> Option<&JSXSpreadAttribute<'a>> {
        if let Self::JSXSpreadAttribute(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_identifier(&self) -> Option<&JSXIdentifier<'a>> {
        if let Self::JSXIdentifier(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_jsx_text(&self) -> Option<&JSXText<'a>> {
        if let Self::JSXText(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_expression_array_element(&self) -> Option<&Expression<'a>> {
        if let Self::ExpressionArrayElement(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}
