#[allow(clippy::wildcard_imports)]
use crate::{ast::*, Atom, GetSpan, Span};

/// Untyped AST Node Kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AstKind<'a> {
    Root,

    Program(&'a Program<'a>),
    Directive(&'a Directive<'a>),

    BlockStatement(&'a BlockStatement<'a>),
    BreakStatement(&'a BreakStatement),
    ContinueStatement(&'a ContinueStatement),
    DebuggerStatement(&'a DebuggerStatement),
    DoWhileStatement(&'a DoWhileStatement<'a>),
    EmptyStatement(&'a EmptyStatement),
    ExpressionStatement(&'a ExpressionStatement<'a>),
    ForInStatement(&'a ForInStatement<'a>),
    ForOfStatement(&'a ForOfStatement<'a>),
    ForStatement(&'a ForStatement<'a>),
    ForStatementInit(&'a ForStatementInit<'a>),
    IfStatement(&'a IfStatement<'a>),
    LabeledStatement(&'a LabeledStatement<'a>),
    ReturnStatement(&'a ReturnStatement<'a>),
    SwitchStatement(&'a SwitchStatement<'a>),
    ThrowStatement(&'a ThrowStatement<'a>),
    TryStatement(&'a TryStatement<'a>),
    WhileStatement(&'a WhileStatement<'a>),
    WithStatement(&'a WithStatement<'a>),

    SwitchCase(&'a SwitchCase<'a>),
    CatchClause(&'a CatchClause<'a>),
    FinallyClause(&'a BlockStatement<'a>),

    VariableDeclaration(&'a VariableDeclaration<'a>),
    VariableDeclarator(&'a VariableDeclarator<'a>),

    IdentifierName(&'a IdentifierName),
    IdentifierReference(&'a IdentifierReference),
    BindingIdentifier(&'a BindingIdentifier),
    LabelIdentifier(&'a LabelIdentifier),
    PrivateIdentifier(&'a PrivateIdentifier),

    NumberLiteral(&'a NumberLiteral<'a>),
    StringLiteral(&'a StringLiteral),
    BooleanLiteral(&'a BooleanLiteral),
    NullLiteral(&'a NullLiteral),
    BigintLiteral(&'a BigintLiteral),
    RegExpLiteral(&'a RegExpLiteral),
    TemplateLiteral(&'a TemplateLiteral<'a>),

    MetaProperty(&'a MetaProperty),
    Super(&'a Super),

    ArrayExpression(&'a ArrayExpression<'a>),
    ArrowExpression(&'a ArrowExpression<'a>),
    AssignmentExpression(&'a AssignmentExpression<'a>),
    AwaitExpression(&'a AwaitExpression<'a>),
    BinaryExpression(&'a BinaryExpression<'a>),
    CallExpression(&'a CallExpression<'a>),
    ConditionalExpression(&'a ConditionalExpression<'a>),
    LogicalExpression(&'a LogicalExpression<'a>),
    MemberExpression(&'a MemberExpression<'a>),
    NewExpression(&'a NewExpression<'a>),
    ObjectExpression(&'a ObjectExpression<'a>),
    ParenthesizedExpression(&'a ParenthesizedExpression<'a>),
    SequenceExpression(&'a SequenceExpression<'a>),
    TaggedTemplateExpression(&'a TaggedTemplateExpression<'a>),
    ThisExpression(&'a ThisExpression),
    UnaryExpression(&'a UnaryExpression<'a>),
    UpdateExpression(&'a UpdateExpression<'a>),
    YieldExpression(&'a YieldExpression<'a>),

    Property(&'a Property<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    PropertyValue(&'a PropertyValue<'a>),
    Argument(&'a Argument<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>),
    SpreadElement(&'a SpreadElement<'a>),
    RestElement(&'a RestElement<'a>),

    Function(&'a Function<'a>),
    FunctionBody(&'a FunctionBody<'a>),
    FormalParameters(&'a FormalParameters<'a>),
    FormalParameter(&'a FormalParameter<'a>),

    Class(&'a Class<'a>),
    ClassHeritage(&'a Expression<'a>),
    StaticBlock(&'a StaticBlock<'a>),
    PropertyDefinition(&'a PropertyDefinition<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),

    ArrayPattern(&'a ArrayPattern<'a>),
    ObjectPattern(&'a ObjectPattern<'a>),
    AssignmentPattern(&'a AssignmentPattern<'a>),

    Decorator(&'a Decorator<'a>),

    ModuleDeclaration(&'a ModuleDeclaration<'a>),

    // JSX
    // Please make sure to add these to `is_jsx` below.
    JSXOpeningElement(&'a JSXOpeningElement<'a>),
    JSXElementName(&'a JSXElementName<'a>),

    // TypeScript
    TSModuleBlock(&'a TSModuleBlock<'a>),

    // NOTE: make sure add these to AstKind::is_type below
    TSAnyKeyword(&'a TSAnyKeyword),
    TSIntersectionType(&'a TSIntersectionType<'a>),
    TSLiteralType(&'a TSLiteralType<'a>),
    TSMethodSignature(&'a TSMethodSignature<'a>),
    TSNullKeyword(&'a TSNullKeyword),
    TSTypeLiteral(&'a TSTypeLiteral<'a>),
    TSTypeReference(&'a TSTypeReference<'a>),
    TSUnionType(&'a TSUnionType<'a>),
    TSVoidKeyword(&'a TSVoidKeyword),

    TSIndexedAccessType(&'a TSIndexedAccessType<'a>),

    TSAsExpression(&'a TSAsExpression<'a>),
    TSSatisfiesExpression(&'a TSSatisfiesExpression<'a>),
    TSNonNullExpression(&'a TSNonNullExpression<'a>),

    TSEnumDeclaration(&'a TSEnumDeclaration<'a>),
    TSEnumMember(&'a TSEnumMember<'a>),
    TSImportEqualsDeclaration(&'a TSImportEqualsDeclaration<'a>),
    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>),
    TSModuleDeclaration(&'a TSModuleDeclaration<'a>),
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>),
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>),
    TSTypeAssertion(&'a TSTypeAssertion<'a>),
    TSTypeParameter(&'a TSTypeParameter<'a>),
    TSTypeParameterDeclaration(&'a TSTypeParameterDeclaration<'a>),
    TSTypeParameterInstantiation(&'a TSTypeParameterInstantiation<'a>),

    TSPropertySignature(&'a TSPropertySignature<'a>),
}

// SAFETY: The AST is part of the bump allocator,
// it is our responsibility to never simultaneously mutate across threads.
unsafe impl<'a> Send for AstKind<'a> {}
unsafe impl<'a> Sync for AstKind<'a> {}

impl<'a> AstKind<'a> {
    #[must_use]
    #[rustfmt::skip]
    pub fn is_statement(self) -> bool {
        self.is_iteration_statement()
            || matches!(self, Self::BlockStatement(_) | Self::BreakStatement(_) | Self::ContinueStatement(_)
                    | Self::DebuggerStatement(_) | Self::EmptyStatement(_) | Self::ExpressionStatement(_)
                    | Self::LabeledStatement(_) | Self::ReturnStatement(_) | Self::SwitchStatement(_)
                    | Self::ThrowStatement(_) | Self::TryStatement(_) | Self::WithStatement(_)
                    | Self::IfStatement(_) | Self::VariableDeclaration(_))
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_declaration(self) -> bool {
        matches!(
            self,
            Self::ModuleDeclaration(_) | Self::TSEnumDeclaration(_) | Self::TSModuleDeclaration(_)
                | Self::VariableDeclaration(_) | Self::TSInterfaceDeclaration(_)
                | Self::TSTypeAliasDeclaration(_) | Self::TSImportEqualsDeclaration(_)
        )
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_iteration_statement(self) -> bool {
        matches!(self, Self::DoWhileStatement(_) | Self::WhileStatement(_) | Self::ForInStatement(_)
                | Self::ForOfStatement(_) | Self::ForStatement(_))
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_identifier(self) -> bool {
        matches!(self, Self::BindingIdentifier(_)
                | Self::IdentifierReference(_)
                | Self::LabelIdentifier(_))
    }

    #[must_use]
    pub fn is_type(self) -> bool {
        matches!(
            self,
            Self::TSIntersectionType(_)
                | Self::TSLiteralType(_)
                | Self::TSTypeReference(_)
                | Self::TSMethodSignature(_)
        )
    }

    #[must_use]
    pub fn is_literal(self) -> bool {
        matches!(
            self,
            Self::NumberLiteral(_)
                | Self::StringLiteral(_)
                | Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::BigintLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::TemplateLiteral(_)
        )
    }

    #[must_use]
    pub fn is_function_like(self) -> bool {
        matches!(self, Self::Function(_) | Self::ArrowExpression(_))
    }

    #[must_use]
    pub fn identifier_name(self) -> Option<Atom> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierReference(ident) => Some(ident.name.clone()),
            Self::LabelIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierName(ident) => Some(ident.name.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_jsx(self) -> bool {
        matches!(self, Self::JSXOpeningElement(_) | Self::JSXElementName(_))
    }
}

impl<'a> GetSpan for AstKind<'a> {
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    fn span(&self) -> Span {
        match self {
            Self::Root => Span::default(),

            Self::Program(x) => x.span,
            Self::Directive(x) => x.span,

            Self::BlockStatement(x) => x.span,
            Self::BreakStatement(x) => x.span,
            Self::ContinueStatement(x) => x.span,
            Self::DebuggerStatement(x) => x.span,
            Self::DoWhileStatement(x) => x.span,
            Self::EmptyStatement(x) => x.span,
            Self::ExpressionStatement(x) => x.span,
            Self::ForInStatement(x) => x.span,
            Self::ForOfStatement(x) => x.span,
            Self::ForStatement(x) => x.span,
            Self::ForStatementInit(x) => x.span(),
            Self::IfStatement(x) => x.span,
            Self::LabeledStatement(x) => x.span,
            Self::ReturnStatement(x) => x.span,
            Self::SwitchStatement(x) => x.span,
            Self::ThrowStatement(x) => x.span,
            Self::TryStatement(x) => x.span,
            Self::WhileStatement(x) => x.span,
            Self::WithStatement(x) => x.span,

            Self::SwitchCase(x) => x.span,
            Self::CatchClause(x) => x.span,
            Self::FinallyClause(x) => x.span,

            Self::VariableDeclaration(x) => x.span,
            Self::VariableDeclarator(x) => x.span,

            Self::IdentifierName(x) => x.span,
            Self::IdentifierReference(x) => x.span,
            Self::BindingIdentifier(x) => x.span,
            Self::LabelIdentifier(x) => x.span,
            Self::PrivateIdentifier(x) => x.span,

            Self::NumberLiteral(x) => x.span,
            Self::StringLiteral(x) => x.span,
            Self::BooleanLiteral(x) => x.span,
            Self::NullLiteral(x) => x.span,
            Self::BigintLiteral(x) => x.span,
            Self::RegExpLiteral(x) => x.span,
            Self::TemplateLiteral(x) => x.span,

            Self::MetaProperty(x) => x.span,
            Self::Super(x) => x.span,

            Self::ArrayExpression(x) => x.span,
            Self::ArrowExpression(x) => x.span,
            Self::AssignmentExpression(x) => x.span,
            Self::AwaitExpression(x) => x.span,
            Self::BinaryExpression(x) => x.span,
            Self::CallExpression(x) => x.span,
            Self::ConditionalExpression(x) => x.span,
            Self::LogicalExpression(x) => x.span,
            Self::MemberExpression(x) => x.span(),
            Self::NewExpression(x) => x.span,
            Self::ObjectExpression(x) => x.span,
            Self::ParenthesizedExpression(x) => x.span,
            Self::SequenceExpression(x) => x.span,
            Self::TaggedTemplateExpression(x) => x.span,
            Self::ThisExpression(x) => x.span,
            Self::UnaryExpression(x) => x.span,
            Self::UpdateExpression(x) => x.span,
            Self::YieldExpression(x) => x.span,

            Self::Property(x) => x.span,
            Self::PropertyKey(x) => x.span(),
            Self::PropertyValue(x) => x.span(),
            Self::Argument(x) => x.span(),
            Self::AssignmentTarget(x) => x.span(),
            Self::SimpleAssignmentTarget(x) => x.span(),
            Self::AssignmentTargetWithDefault(x) => x.span,
            Self::SpreadElement(x) => x.span,
            Self::RestElement(x) => x.span,

            Self::Function(x) => x.span,
            Self::FunctionBody(x) => x.span,
            Self::FormalParameters(x) => x.span,
            Self::FormalParameter(x) => x.span,

            Self::Class(x) => x.span,
            Self::ClassHeritage(x) => x.span(),
            Self::StaticBlock(x) => x.span,
            Self::PropertyDefinition(x) => x.span,
            Self::MethodDefinition(x) => x.span,

            Self::ArrayPattern(x) => x.span,
            Self::ObjectPattern(x) => x.span,
            Self::AssignmentPattern(x) => x.span,

            Self::Decorator(x) => x.span,

            Self::ModuleDeclaration(x) => x.span,

            Self::JSXOpeningElement(x) => x.span,
            Self::JSXElementName(x) => x.span(),

            Self::TSModuleBlock(x) => x.span,

            Self::TSAnyKeyword(x) => x.span,
            Self::TSIntersectionType(x) => x.span,
            Self::TSLiteralType(x) => x.span,
            Self::TSMethodSignature(x) => x.span,
            Self::TSNullKeyword(x) => x.span,
            Self::TSTypeLiteral(x) => x.span,
            Self::TSTypeReference(x) => x.span,
            Self::TSUnionType(x) => x.span,
            Self::TSVoidKeyword(x) => x.span,

            Self::TSIndexedAccessType(x) => x.span,

            Self::TSAsExpression(x) => x.span,
            Self::TSSatisfiesExpression(x) => x.span,
            Self::TSNonNullExpression(x) => x.span,

            Self::TSEnumDeclaration(x) => x.span,
            Self::TSEnumMember(x) => x.span,
            Self::TSImportEqualsDeclaration(x) => x.span,
            Self::TSInterfaceDeclaration(x) => x.span,
            Self::TSModuleDeclaration(x) => x.span,
            Self::TSTypeAliasDeclaration(x) => x.span,
            Self::TSTypeAnnotation(x) => x.span,
            Self::TSTypeAssertion(x) => x.span,
            Self::TSTypeParameter(x) => x.span,
            Self::TSTypeParameterDeclaration(x) => x.span,
            Self::TSTypeParameterInstantiation(x) => x.span,

            Self::TSPropertySignature(x) => x.span,
        }
    }
}
