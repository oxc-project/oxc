#[allow(clippy::wildcard_imports)]
use crate::{ast::*, Atom, GetNode, Node};

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
    pub const fn is_statement(self) -> bool {
        self.is_iteration_statement()
            || matches!(self, Self::BlockStatement(_) | Self::BreakStatement(_) | Self::ContinueStatement(_)
                    | Self::DebuggerStatement(_) | Self::EmptyStatement(_) | Self::ExpressionStatement(_)
                    | Self::LabeledStatement(_) | Self::ReturnStatement(_) | Self::SwitchStatement(_)
                    | Self::ThrowStatement(_) | Self::TryStatement(_) | Self::WithStatement(_)
                    | Self::IfStatement(_) | Self::VariableDeclaration(_))
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_declaration(self) -> bool {
        matches!(
            self,
            Self::ModuleDeclaration(_) | Self::TSEnumDeclaration(_) | Self::TSModuleDeclaration(_)
                | Self::VariableDeclaration(_) | Self::TSInterfaceDeclaration(_)
                | Self::TSTypeAliasDeclaration(_) | Self::TSImportEqualsDeclaration(_)
        )
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_iteration_statement(self) -> bool {
        matches!(self, Self::DoWhileStatement(_) | Self::WhileStatement(_) | Self::ForInStatement(_)
                | Self::ForOfStatement(_) | Self::ForStatement(_))
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_identifier(self) -> bool {
        matches!(self, Self::BindingIdentifier(_)
                | Self::IdentifierReference(_)
                | Self::LabelIdentifier(_))
    }

    #[must_use]
    pub const fn is_type(self) -> bool {
        matches!(
            self,
            Self::TSIntersectionType(_)
                | Self::TSLiteralType(_)
                | Self::TSTypeReference(_)
                | Self::TSMethodSignature(_)
        )
    }

    #[must_use]
    pub const fn is_literal(self) -> bool {
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
    pub const fn is_function_like(self) -> bool {
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
    pub const fn is_jsx(self) -> bool {
        matches!(self, Self::JSXOpeningElement(_) | Self::JSXElementName(_))
    }
}

impl<'a> GetNode for AstKind<'a> {
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    fn node(&self) -> Node {
        match self {
            Self::Root => Node::default(),

            Self::Program(x) => x.node,
            Self::Directive(x) => x.node,

            Self::BlockStatement(x) => x.node,
            Self::BreakStatement(x) => x.node,
            Self::ContinueStatement(x) => x.node,
            Self::DebuggerStatement(x) => x.node,
            Self::DoWhileStatement(x) => x.node,
            Self::EmptyStatement(x) => x.node,
            Self::ExpressionStatement(x) => x.node,
            Self::ForInStatement(x) => x.node,
            Self::ForOfStatement(x) => x.node,
            Self::ForStatement(x) => x.node,
            Self::ForStatementInit(x) => x.node(),
            Self::IfStatement(x) => x.node,
            Self::LabeledStatement(x) => x.node,
            Self::ReturnStatement(x) => x.node,
            Self::SwitchStatement(x) => x.node,
            Self::ThrowStatement(x) => x.node,
            Self::TryStatement(x) => x.node,
            Self::WhileStatement(x) => x.node,
            Self::WithStatement(x) => x.node,

            Self::SwitchCase(x) => x.node,
            Self::CatchClause(x) => x.node,
            Self::FinallyClause(x) => x.node,

            Self::VariableDeclaration(x) => x.node,
            Self::VariableDeclarator(x) => x.node,

            Self::IdentifierName(x) => x.node,
            Self::IdentifierReference(x) => x.node,
            Self::BindingIdentifier(x) => x.node,
            Self::LabelIdentifier(x) => x.node,
            Self::PrivateIdentifier(x) => x.node,

            Self::NumberLiteral(x) => x.node,
            Self::StringLiteral(x) => x.node,
            Self::BooleanLiteral(x) => x.node,
            Self::NullLiteral(x) => x.node,
            Self::BigintLiteral(x) => x.node,
            Self::RegExpLiteral(x) => x.node,
            Self::TemplateLiteral(x) => x.node,

            Self::MetaProperty(x) => x.node,
            Self::Super(x) => x.node,

            Self::ArrayExpression(x) => x.node,
            Self::ArrowExpression(x) => x.node,
            Self::AssignmentExpression(x) => x.node,
            Self::AwaitExpression(x) => x.node,
            Self::BinaryExpression(x) => x.node,
            Self::CallExpression(x) => x.node,
            Self::ConditionalExpression(x) => x.node,
            Self::LogicalExpression(x) => x.node,
            Self::MemberExpression(x) => x.node(),
            Self::NewExpression(x) => x.node,
            Self::ObjectExpression(x) => x.node,
            Self::ParenthesizedExpression(x) => x.node,
            Self::SequenceExpression(x) => x.node,
            Self::TaggedTemplateExpression(x) => x.node,
            Self::ThisExpression(x) => x.node,
            Self::UnaryExpression(x) => x.node,
            Self::UpdateExpression(x) => x.node,
            Self::YieldExpression(x) => x.node,

            Self::Property(x) => x.node,
            Self::PropertyKey(x) => x.node(),
            Self::PropertyValue(x) => x.node(),
            Self::Argument(x) => x.node(),
            Self::AssignmentTarget(x) => x.node(),
            Self::SimpleAssignmentTarget(x) => x.node(),
            Self::AssignmentTargetWithDefault(x) => x.node,
            Self::SpreadElement(x) => x.node,
            Self::RestElement(x) => x.node,

            Self::Function(x) => x.node,
            Self::FunctionBody(x) => x.node,
            Self::FormalParameters(x) => x.node,
            Self::FormalParameter(x) => x.node,

            Self::Class(x) => x.node,
            Self::ClassHeritage(x) => x.node(),
            Self::StaticBlock(x) => x.node,
            Self::PropertyDefinition(x) => x.node,
            Self::MethodDefinition(x) => x.node,

            Self::ArrayPattern(x) => x.node,
            Self::ObjectPattern(x) => x.node,
            Self::AssignmentPattern(x) => x.node,

            Self::Decorator(x) => x.node,

            Self::ModuleDeclaration(x) => x.node,

            Self::JSXOpeningElement(x) => x.node,
            Self::JSXElementName(x) => x.node(),

            Self::TSModuleBlock(x) => x.node,

            Self::TSAnyKeyword(x) => x.node,
            Self::TSIntersectionType(x) => x.node,
            Self::TSLiteralType(x) => x.node,
            Self::TSMethodSignature(x) => x.node,
            Self::TSNullKeyword(x) => x.node,
            Self::TSTypeLiteral(x) => x.node,
            Self::TSTypeReference(x) => x.node,
            Self::TSUnionType(x) => x.node,
            Self::TSVoidKeyword(x) => x.node,

            Self::TSIndexedAccessType(x) => x.node,

            Self::TSAsExpression(x) => x.node,
            Self::TSNonNullExpression(x) => x.node,

            Self::TSEnumDeclaration(x) => x.node,
            Self::TSEnumMember(x) => x.node,
            Self::TSImportEqualsDeclaration(x) => x.node,
            Self::TSInterfaceDeclaration(x) => x.node,
            Self::TSModuleDeclaration(x) => x.node,
            Self::TSTypeAliasDeclaration(x) => x.node,
            Self::TSTypeAnnotation(x) => x.node,
            Self::TSTypeAssertion(x) => x.node,
            Self::TSTypeParameter(x) => x.node,
            Self::TSTypeParameterDeclaration(x) => x.node,
            Self::TSTypeParameterInstantiation(x) => x.node,

            Self::TSPropertySignature(x) => x.node,
        }
    }
}
