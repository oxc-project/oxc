use oxc_span::{Atom, Span};

#[allow(clippy::wildcard_imports)]
use crate::hir::*;

/// Untyped AST Node Kind
#[derive(Debug, Clone, Copy)]
pub enum HirKind<'a> {
    Root,

    Program(&'a Program<'a>),
    Directive(&'a Directive<'a>),

    BlockStatement(&'a BlockStatement<'a>),
    BreakStatement(&'a BreakStatement),
    ContinueStatement(&'a ContinueStatement),
    DebuggerStatement(&'a DebuggerStatement),
    DoWhileStatement(&'a DoWhileStatement<'a>),
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
    SequenceExpression(&'a SequenceExpression<'a>),
    TaggedTemplateExpression(&'a TaggedTemplateExpression<'a>),
    ThisExpression(&'a ThisExpression),
    UnaryExpression(&'a UnaryExpression<'a>),
    UpdateExpression(&'a UpdateExpression<'a>),
    YieldExpression(&'a YieldExpression<'a>),

    ObjectProperty(&'a ObjectProperty<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    Argument(&'a Argument<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>),
    ArrayExpressionElement(&'a ArrayExpressionElement<'a>),
    Elision(Span),
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

    TSEnumDeclaration(&'a TSEnumDeclaration<'a>),
    TSEnumMember(&'a TSEnumMember<'a>),
}

impl<'a> HirKind<'a> {
    #[rustfmt::skip]
    pub fn is_statement(self) -> bool {
        self.is_iteration_statement()
            || matches!(self, Self::BlockStatement(_) | Self::BreakStatement(_) | Self::ContinueStatement(_)
                    | Self::DebuggerStatement(_) | Self::ExpressionStatement(_)
                    | Self::LabeledStatement(_) | Self::ReturnStatement(_) | Self::SwitchStatement(_)
                    | Self::ThrowStatement(_) | Self::TryStatement(_) | Self::WithStatement(_)
                    | Self::IfStatement(_) | Self::VariableDeclaration(_))
    }

    #[rustfmt::skip]
    pub fn is_declaration(self) -> bool {
        matches!(self, Self::Function(func) if func.is_declaration())
        || matches!(self, Self::Class(class) if class.is_declaration())
        || matches!(self, Self::ModuleDeclaration(_) | Self::VariableDeclaration(_))
    }

    #[rustfmt::skip]
    pub fn is_iteration_statement(self) -> bool {
        matches!(self, Self::DoWhileStatement(_) | Self::WhileStatement(_) | Self::ForInStatement(_)
                | Self::ForOfStatement(_) | Self::ForStatement(_))
    }

    #[rustfmt::skip]
    pub fn is_identifier(self) -> bool {
        matches!(self, Self::BindingIdentifier(_)
                | Self::IdentifierReference(_)
                | Self::LabelIdentifier(_))
    }

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

    pub fn is_function_like(self) -> bool {
        matches!(self, Self::Function(_) | Self::ArrowExpression(_))
    }

    pub fn identifier_name(self) -> Option<Atom> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierReference(ident) => Some(ident.name.clone()),
            Self::LabelIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierName(ident) => Some(ident.name.clone()),
            _ => None,
        }
    }

    pub fn is_jsx(self) -> bool {
        matches!(self, Self::JSXOpeningElement(_) | Self::JSXElementName(_))
    }
}
