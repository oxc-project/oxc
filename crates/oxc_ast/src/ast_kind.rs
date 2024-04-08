use oxc_span::{Atom, GetSpan, Span};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

macro_rules! ast_kinds {
    { $($ident:ident($type:ty),)* } => (
        #[derive(Debug, Clone, Copy)]
        pub enum AstType {
            Elision,
            $($ident,)*
        }

        /// Untyped AST Node Kind
        #[derive(Debug, Clone, Copy)]
        pub enum AstKind<'a> {
            Elision(Span),
            $($ident(&'a $type),)*
        }

        /// Untyped AST Node Kind without references
        #[derive(Debug)]
        pub enum AstOwnedKind<'a> {
            Elision(Span),
            $($ident($type),)*
        }
    )
}

ast_kinds! {
    Program(Program<'a>),
    Directive(Directive<'a>),
    Hashbang(Hashbang<'a>),

    BlockStatement(BlockStatement<'a>),
    BreakStatement(BreakStatement<'a>),
    ContinueStatement(ContinueStatement<'a>),
    DebuggerStatement(DebuggerStatement),
    DoWhileStatement(DoWhileStatement<'a>),
    EmptyStatement(EmptyStatement),
    ExpressionStatement(ExpressionStatement<'a>),
    ForInStatement(ForInStatement<'a>),
    ForOfStatement(ForOfStatement<'a>),
    ForStatement(ForStatement<'a>),
    ForStatementInit(ForStatementInit<'a>),
    IfStatement(IfStatement<'a>),
    LabeledStatement(LabeledStatement<'a>),
    ReturnStatement(ReturnStatement<'a>),
    SwitchStatement(SwitchStatement<'a>),
    ThrowStatement(ThrowStatement<'a>),
    TryStatement(TryStatement<'a>),
    WhileStatement(WhileStatement<'a>),
    WithStatement(WithStatement<'a>),

    SwitchCase(SwitchCase<'a>),
    CatchClause(CatchClause<'a>),
    FinallyClause(BlockStatement<'a>),

    VariableDeclaration(VariableDeclaration<'a>),
    VariableDeclarator(VariableDeclarator<'a>),

    UsingDeclaration(UsingDeclaration<'a>),

    IdentifierName(IdentifierName<'a>),
    IdentifierReference(IdentifierReference<'a>),
    BindingIdentifier(BindingIdentifier<'a>),
    LabelIdentifier(LabelIdentifier<'a>),
    PrivateIdentifier(PrivateIdentifier<'a>),

    NumericLiteral(NumericLiteral<'a>),
    StringLiteral(StringLiteral<'a>),
    BooleanLiteral(BooleanLiteral),
    NullLiteral(NullLiteral),
    BigintLiteral(BigIntLiteral<'a>),
    RegExpLiteral(RegExpLiteral<'a>),
    TemplateLiteral(TemplateLiteral<'a>),

    MetaProperty(MetaProperty<'a>),
    Super(Super),

    ArrayExpression(ArrayExpression<'a>),
    ArrowFunctionExpression(ArrowFunctionExpression<'a>),
    AssignmentExpression(AssignmentExpression<'a>),
    AwaitExpression(AwaitExpression<'a>),
    BinaryExpression(BinaryExpression<'a>),
    CallExpression(CallExpression<'a>),
    ChainExpression(ChainExpression<'a>),
    ConditionalExpression(ConditionalExpression<'a>),
    LogicalExpression(LogicalExpression<'a>),
    MemberExpression(MemberExpression<'a>),
    NewExpression(NewExpression<'a>),
    ObjectExpression(ObjectExpression<'a>),
    ParenthesizedExpression(ParenthesizedExpression<'a>),
    SequenceExpression(SequenceExpression<'a>),
    TaggedTemplateExpression(TaggedTemplateExpression<'a>),
    ThisExpression(ThisExpression),
    UnaryExpression(UnaryExpression<'a>),
    UpdateExpression(UpdateExpression<'a>),
    YieldExpression(YieldExpression<'a>),
    ImportExpression(ImportExpression<'a>),
    PrivateInExpression(PrivateInExpression<'a>),

    ObjectProperty(ObjectProperty<'a>),
    PropertyKey(PropertyKey<'a>),
    Argument(Argument<'a>),
    AssignmentTarget(AssignmentTarget<'a>),
    SimpleAssignmentTarget(SimpleAssignmentTarget<'a>),
    AssignmentTargetWithDefault(AssignmentTargetWithDefault<'a>),
    ArrayExpressionElement(ArrayExpressionElement<'a>),
    // Elision(Span),
    ExpressionArrayElement(Expression<'a>),
    SpreadElement(SpreadElement<'a>),
    BindingRestElement(BindingRestElement<'a>),

    Function(Function<'a>),
    FunctionBody(FunctionBody<'a>),
    FormalParameters(FormalParameters<'a>),
    FormalParameter(FormalParameter<'a>),

    Class(Class<'a>),
    ClassBody(ClassBody<'a>),
    ClassHeritage(Expression<'a>),
    StaticBlock(StaticBlock<'a>),
    PropertyDefinition(PropertyDefinition<'a>),
    MethodDefinition(MethodDefinition<'a>),

    ArrayPattern(ArrayPattern<'a>),
    ObjectPattern(ObjectPattern<'a>),
    AssignmentPattern(AssignmentPattern<'a>),

    Decorator(Decorator<'a>),

    ModuleDeclaration(ModuleDeclaration<'a>),
    ImportDeclaration(ImportDeclaration<'a>),
    ImportSpecifier(ImportSpecifier<'a>),
    ImportDefaultSpecifier(ImportDefaultSpecifier<'a>),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier<'a>),
    ExportDefaultDeclaration(ExportDefaultDeclaration<'a>),
    ExportNamedDeclaration(ExportNamedDeclaration<'a>),
    ExportAllDeclaration(ExportAllDeclaration<'a>),

    // JSX
    // Please make sure to add these to `is_jsx` below.
    JSXElement(JSXElement<'a>),
    JSXFragment(JSXFragment<'a>),
    JSXOpeningElement(JSXOpeningElement<'a>),
    JSXClosingElement(JSXClosingElement<'a>),
    JSXElementName(JSXElementName<'a>),
    JSXExpressionContainer(JSXExpressionContainer<'a>),
    JSXAttributeItem(JSXAttributeItem<'a>),
    JSXSpreadAttribute(JSXSpreadAttribute<'a>),
    JSXText(JSXText<'a>),
    JSXIdentifier(JSXIdentifier<'a>),
    JSXMemberExpression(JSXMemberExpression<'a>),
    JSXMemberExpressionObject(JSXMemberExpressionObject<'a>),
    JSXNamespacedName(JSXNamespacedName<'a>),

    // TypeScript
    TSModuleBlock(TSModuleBlock<'a>),

    // NOTE: make sure add these to AstKind::is_type below
    TSAnyKeyword(TSAnyKeyword),
    TSIntersectionType(TSIntersectionType<'a>),
    TSLiteralType(TSLiteralType<'a>),
    TSMethodSignature(TSMethodSignature<'a>),
    TSNullKeyword(TSNullKeyword),
    TSTypeLiteral(TSTypeLiteral<'a>),
    TSTypeReference(TSTypeReference<'a>),
    TSUnionType(TSUnionType<'a>),
    TSVoidKeyword(TSVoidKeyword),

    TSIndexedAccessType(TSIndexedAccessType<'a>),

    TSAsExpression(TSAsExpression<'a>),
    TSSatisfiesExpression(TSSatisfiesExpression<'a>),
    TSNonNullExpression(TSNonNullExpression<'a>),
    TSInstantiationExpression(TSInstantiationExpression<'a>),

    TSEnumDeclaration(TSEnumDeclaration<'a>),
    TSEnumMember(TSEnumMember<'a>),

    TSImportEqualsDeclaration(TSImportEqualsDeclaration<'a>),
    TSTypeName(TSTypeName<'a>),
    TSExternalModuleReference(TSExternalModuleReference<'a>),
    TSQualifiedName(TSQualifiedName<'a>),

    TSInterfaceDeclaration(TSInterfaceDeclaration<'a>),
    TSModuleDeclaration(TSModuleDeclaration<'a>),
    TSTypeAliasDeclaration(TSTypeAliasDeclaration<'a>),
    TSTypeAnnotation(TSTypeAnnotation<'a>),
    TSTypeQuery(TSTypeQuery<'a>),
    TSTypeAssertion(TSTypeAssertion<'a>),
    TSTypeParameter(TSTypeParameter<'a>),
    TSTypeParameterDeclaration(TSTypeParameterDeclaration<'a>),
    TSTypeParameterInstantiation(TSTypeParameterInstantiation<'a>),
    TSImportType(TSImportType<'a>),

    TSPropertySignature(TSPropertySignature<'a>),
}

// SAFETY:
// The AST is part of the bump allocator,
// it is our responsibility to never simultaneously mutate across threads.
unsafe impl<'a> Send for AstKind<'a> {}
// SAFETY:
// The AST is part of the bump allocator,
// it is our responsibility to never simultaneously mutate across threads.
unsafe impl<'a> Sync for AstKind<'a> {}

impl<'a> AstKind<'a> {
    #[rustfmt::skip]
    pub fn is_statement(self) -> bool {
        self.is_iteration_statement()
            || matches!(self, Self::BlockStatement(_) | Self::BreakStatement(_) | Self::ContinueStatement(_)
                    | Self::DebuggerStatement(_) | Self::EmptyStatement(_) | Self::ExpressionStatement(_)
                    | Self::LabeledStatement(_) | Self::ReturnStatement(_) | Self::SwitchStatement(_)
                    | Self::ThrowStatement(_) | Self::TryStatement(_) | Self::WithStatement(_)
                    | Self::IfStatement(_) | Self::VariableDeclaration(_))
    }

    #[rustfmt::skip]
    pub fn is_declaration(self) -> bool {
        matches!(self, Self::Function(func) if func.is_declaration())
        || matches!(self, Self::Class(class) if class.is_declaration())
        || matches!(self, Self::ModuleDeclaration(_) | Self::TSEnumDeclaration(_) | Self::TSModuleDeclaration(_)
            | Self::VariableDeclaration(_) | Self::TSInterfaceDeclaration(_)
            | Self::TSTypeAliasDeclaration(_) | Self::TSImportEqualsDeclaration(_) | Self::PropertyDefinition(_)
        )
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

    pub fn is_type(self) -> bool {
        matches!(
            self,
            Self::TSIntersectionType(_)
                | Self::TSLiteralType(_)
                | Self::TSTypeReference(_)
                | Self::TSMethodSignature(_)
        )
    }

    pub fn is_literal(self) -> bool {
        matches!(
            self,
            Self::NumericLiteral(_)
                | Self::StringLiteral(_)
                | Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::BigintLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::TemplateLiteral(_)
        )
    }

    pub fn is_function_like(self) -> bool {
        matches!(self, Self::Function(_) | Self::ArrowFunctionExpression(_))
    }

    pub fn identifier_name(self) -> Option<Atom<'a>> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierReference(ident) => Some(ident.name.clone()),
            Self::LabelIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierName(ident) => Some(ident.name.clone()),
            _ => None,
        }
    }

    pub fn is_jsx(self) -> bool {
        matches!(
            self,
            Self::JSXElement(_)
                | Self::JSXOpeningElement(_)
                | Self::JSXElementName(_)
                | Self::JSXFragment(_)
                | Self::JSXAttributeItem(_)
                | Self::JSXText(_)
                | Self::JSXExpressionContainer(_)
        )
    }

    pub fn is_specific_id_reference(&self, name: &str) -> bool {
        match self {
            Self::IdentifierReference(ident) => ident.name == name,
            _ => false,
        }
    }

    pub fn from_expression(e: &'a Expression<'a>) -> Self {
        match e {
            Expression::None => unreachable!(),
            Expression::BooleanLiteral(e) => Self::BooleanLiteral(e),
            Expression::NullLiteral(e) => Self::NullLiteral(e),
            Expression::NumericLiteral(e) => Self::NumericLiteral(e),
            Expression::BigintLiteral(e) => Self::BigintLiteral(e),
            Expression::RegExpLiteral(e) => Self::RegExpLiteral(e),
            Expression::StringLiteral(e) => Self::StringLiteral(e),
            Expression::TemplateLiteral(e) => Self::TemplateLiteral(e),
            Expression::Identifier(e) => Self::IdentifierReference(e),
            Expression::MetaProperty(e) => Self::MetaProperty(e),
            Expression::Super(e) => Self::Super(e),
            Expression::ArrayExpression(e) => Self::ArrayExpression(e),
            Expression::ArrowFunctionExpression(e) => Self::ArrowFunctionExpression(e),
            Expression::AssignmentExpression(e) => Self::AssignmentExpression(e),
            Expression::AwaitExpression(e) => Self::AwaitExpression(e),
            Expression::BinaryExpression(e) => Self::BinaryExpression(e),
            Expression::CallExpression(e) => Self::CallExpression(e),
            Expression::ChainExpression(e) => Self::ChainExpression(e),
            Expression::ClassExpression(e) => Self::Class(e),
            Expression::ConditionalExpression(e) => Self::ConditionalExpression(e),
            Expression::FunctionExpression(e) => Self::Function(e),
            Expression::ImportExpression(e) => Self::ImportExpression(e),
            Expression::LogicalExpression(e) => Self::LogicalExpression(e),
            Expression::MemberExpression(e) => Self::MemberExpression(e),
            Expression::NewExpression(e) => Self::NewExpression(e),
            Expression::ObjectExpression(e) => Self::ObjectExpression(e),
            Expression::ParenthesizedExpression(e) => Self::ParenthesizedExpression(e),
            Expression::SequenceExpression(e) => Self::SequenceExpression(e),
            Expression::TaggedTemplateExpression(e) => Self::TaggedTemplateExpression(e),
            Expression::ThisExpression(e) => Self::ThisExpression(e),
            Expression::UnaryExpression(e) => Self::UnaryExpression(e),
            Expression::UpdateExpression(e) => Self::UpdateExpression(e),
            Expression::YieldExpression(e) => Self::YieldExpression(e),
            Expression::PrivateInExpression(e) => Self::PrivateInExpression(e),
            Expression::JSXElement(e) => Self::JSXElement(e),
            Expression::JSXFragment(e) => Self::JSXFragment(e),
            Expression::TSAsExpression(e) => Self::TSAsExpression(e),
            Expression::TSSatisfiesExpression(e) => Self::TSSatisfiesExpression(e),
            Expression::TSTypeAssertion(e) => Self::TSTypeAssertion(e),
            Expression::TSNonNullExpression(e) => Self::TSNonNullExpression(e),
            Expression::TSInstantiationExpression(e) => Self::TSInstantiationExpression(e),
        }
    }
}

impl<'a> GetSpan for AstKind<'a> {
    #[allow(clippy::match_same_arms)]
    fn span(&self) -> Span {
        match self {
            Self::Program(x) => x.span,
            Self::Directive(x) => x.span,
            Self::Hashbang(x) => x.span,

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

            Self::UsingDeclaration(x) => x.span,

            Self::IdentifierName(x) => x.span,
            Self::IdentifierReference(x) => x.span,
            Self::BindingIdentifier(x) => x.span,
            Self::LabelIdentifier(x) => x.span,
            Self::PrivateIdentifier(x) => x.span,

            Self::NumericLiteral(x) => x.span,
            Self::StringLiteral(x) => x.span,
            Self::BooleanLiteral(x) => x.span,
            Self::NullLiteral(x) => x.span,
            Self::BigintLiteral(x) => x.span,
            Self::RegExpLiteral(x) => x.span,
            Self::TemplateLiteral(x) => x.span,

            Self::MetaProperty(x) => x.span,
            Self::Super(x) => x.span,

            Self::ArrayExpression(x) => x.span,
            Self::ArrowFunctionExpression(x) => x.span,
            Self::AssignmentExpression(x) => x.span,
            Self::AwaitExpression(x) => x.span,
            Self::BinaryExpression(x) => x.span,
            Self::CallExpression(x) => x.span,
            Self::ChainExpression(x) => x.span,
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
            Self::ImportExpression(x) => x.span,
            Self::PrivateInExpression(x) => x.span,

            Self::ObjectProperty(x) => x.span,
            Self::PropertyKey(x) => x.span(),
            Self::Argument(x) => x.span(),
            Self::ArrayExpressionElement(x) => x.span(),
            Self::AssignmentTarget(x) => x.span(),
            Self::SimpleAssignmentTarget(x) => x.span(),
            Self::AssignmentTargetWithDefault(x) => x.span,
            Self::SpreadElement(x) => x.span,
            Self::Elision(span) => *span,
            Self::ExpressionArrayElement(x) => x.span(),
            Self::BindingRestElement(x) => x.span,

            Self::Function(x) => x.span,
            Self::FunctionBody(x) => x.span,
            Self::FormalParameters(x) => x.span,
            Self::FormalParameter(x) => x.span,

            Self::Class(x) => x.span,
            Self::ClassBody(x) => x.span,
            Self::ClassHeritage(x) => x.span(),
            Self::StaticBlock(x) => x.span,
            Self::PropertyDefinition(x) => x.span,
            Self::MethodDefinition(x) => x.span,

            Self::ArrayPattern(x) => x.span,
            Self::ObjectPattern(x) => x.span,
            Self::AssignmentPattern(x) => x.span,

            Self::Decorator(x) => x.span,

            Self::ModuleDeclaration(x) => x.span(),
            Self::ImportDeclaration(x) => x.span,
            Self::ImportSpecifier(x) => x.span,
            Self::ImportDefaultSpecifier(x) => x.span,
            Self::ImportNamespaceSpecifier(x) => x.span,
            Self::ExportDefaultDeclaration(x) => x.span,
            Self::ExportNamedDeclaration(x) => x.span,
            Self::ExportAllDeclaration(x) => x.span,

            Self::JSXOpeningElement(x) => x.span,
            Self::JSXClosingElement(x) => x.span,
            Self::JSXElementName(x) => x.span(),
            Self::JSXElement(x) => x.span,
            Self::JSXFragment(x) => x.span,
            Self::JSXAttributeItem(x) => x.span(),
            Self::JSXSpreadAttribute(x) => x.span,
            Self::JSXText(x) => x.span,
            Self::JSXExpressionContainer(x) => x.span,
            Self::JSXIdentifier(x) => x.span,
            Self::JSXMemberExpression(x) => x.span,
            Self::JSXMemberExpressionObject(x) => x.span(),
            Self::JSXNamespacedName(x) => x.span,

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
            Self::TSInstantiationExpression(x) => x.span,

            Self::TSEnumDeclaration(x) => x.span,
            Self::TSEnumMember(x) => x.span,

            Self::TSImportEqualsDeclaration(x) => x.span,
            Self::TSTypeName(x) => x.span(),
            Self::TSExternalModuleReference(x) => x.span,
            Self::TSQualifiedName(x) => x.span,
            Self::TSInterfaceDeclaration(x) => x.span,
            Self::TSModuleDeclaration(x) => x.span,
            Self::TSTypeAliasDeclaration(x) => x.span,
            Self::TSTypeAnnotation(x) => x.span,
            Self::TSTypeQuery(x) => x.span,
            Self::TSTypeAssertion(x) => x.span,
            Self::TSTypeParameter(x) => x.span,
            Self::TSTypeParameterDeclaration(x) => x.span,
            Self::TSTypeParameterInstantiation(x) => x.span,
            Self::TSImportType(x) => x.span,

            Self::TSPropertySignature(x) => x.span,
        }
    }
}

impl<'a> AstKind<'a> {
    #[allow(clippy::match_same_arms)]
    /// Get the AST kind name with minimal details. Particularly useful for
    /// when debugging an iteration over an AST.
    ///
    /// Note that this method does not exist in release builds. Do not include
    /// usage of this method within your code.
    pub fn debug_name(&self) -> std::borrow::Cow<str> {
        match self {
            Self::Program(_) => "Program".into(),
            Self::Directive(d) => d.directive.as_ref().into(),
            Self::Hashbang(_) => "Hashbang".into(),

            Self::BlockStatement(_) => "BlockStatement".into(),
            Self::BreakStatement(_) => "BreakStatement".into(),
            Self::ContinueStatement(_) => "ContinueStatement".into(),
            Self::DebuggerStatement(_) => "DebuggerStatement".into(),
            Self::DoWhileStatement(_) => "DoWhileStatement".into(),
            Self::EmptyStatement(_) => "EmptyStatement".into(),
            Self::ExpressionStatement(_) => "ExpressionStatement".into(),
            Self::ForInStatement(_) => "ForInStatement".into(),
            Self::ForOfStatement(_) => "ForOfStatement".into(),
            Self::ForStatement(_) => "ForStatement".into(),
            Self::ForStatementInit(_) => "ForStatementInit".into(),
            Self::IfStatement(_) => "IfStatement".into(),
            Self::LabeledStatement(_) => "LabeledStatement".into(),
            Self::ReturnStatement(_) => "ReturnStatement".into(),
            Self::SwitchStatement(_) => "SwitchStatement".into(),
            Self::ThrowStatement(_) => "ThrowStatement".into(),
            Self::TryStatement(_) => "TryStatement".into(),
            Self::WhileStatement(_) => "WhileStatement".into(),
            Self::WithStatement(_) => "WithStatement".into(),

            Self::SwitchCase(_) => "SwitchCase".into(),
            Self::CatchClause(_) => "CatchClause".into(),
            Self::FinallyClause(_) => "FinallyClause".into(),

            Self::VariableDeclaration(_) => "VariableDeclaration".into(),
            Self::VariableDeclarator(_) => "VariableDeclarator".into(),

            Self::UsingDeclaration(_) => "UsingDeclaration".into(),

            Self::IdentifierName(x) => format!("IdentifierName({})", x.name).into(),
            Self::IdentifierReference(x) => format!("IdentifierReference({})", x.name).into(),
            Self::BindingIdentifier(x) => format!("BindingIdentifier({})", x.name).into(),
            Self::LabelIdentifier(x) => format!("LabelIdentifier({})", x.name).into(),
            Self::PrivateIdentifier(x) => format!("PrivateIdentifier({})", x.name).into(),

            Self::NumericLiteral(n) => format!("NumericLiteral({})", n.value).into(),
            Self::StringLiteral(s) => format!("NumericLiteral({})", s.value).into(),
            Self::BooleanLiteral(b) => format!("BooleanLiteral({})", b.value).into(),
            Self::NullLiteral(_) => "NullLiteral".into(),
            Self::BigintLiteral(b) => format!("BigintLiteral({})", b.raw).into(),
            Self::RegExpLiteral(r) => format!("RegExpLiteral({})", r.regex).into(),
            Self::TemplateLiteral(t) => format!(
                "TemplateLiteral({})",
                t.quasi().map_or_else(|| "None".into(), |q| format!("Some({q})"))
            )
            .into(),

            Self::MetaProperty(_) => "MetaProperty".into(),
            Self::Super(_) => "Super".into(),

            Self::ArrayExpression(_) => "ArrayExpression".into(),
            Self::ArrowFunctionExpression(_) => "ArrowFunctionExpression".into(),
            Self::AssignmentExpression(_) => "AssignmentExpression".into(),
            Self::AwaitExpression(_) => "AwaitExpression".into(),
            Self::BinaryExpression(b) => format!("BinaryExpression{}", b.operator.as_str()).into(),
            Self::CallExpression(_) => "CallExpression".into(),
            Self::ChainExpression(_) => "ChainExpression".into(),
            Self::ConditionalExpression(_) => "ConditionalExpression".into(),
            Self::LogicalExpression(_) => "LogicalExpression".into(),
            Self::MemberExpression(_) => "MemberExpression".into(),
            Self::NewExpression(_) => "NewExpression".into(),
            Self::ObjectExpression(_) => "ObjectExpression".into(),
            Self::ParenthesizedExpression(_) => "ParenthesizedExpression".into(),
            Self::SequenceExpression(_) => "SequenceExpression".into(),
            Self::TaggedTemplateExpression(_) => "TaggedTemplateExpression".into(),
            Self::ThisExpression(_) => "ThisExpression".into(),
            Self::UnaryExpression(expr) => format!("UnaryExpression({:?})", expr.operator).into(),
            Self::UpdateExpression(_) => "UpdateExpression".into(),
            Self::YieldExpression(_) => "YieldExpression".into(),
            Self::ImportExpression(_) => "ImportExpression".into(),
            Self::PrivateInExpression(_) => "PrivateInExpression".into(),

            Self::ObjectProperty(_) => "ObjectProperty".into(),
            Self::PropertyKey(_) => "PropertyKey".into(),
            Self::Argument(_) => "Argument".into(),
            Self::ArrayExpressionElement(_) => "ArrayExpressionElement".into(),
            Self::AssignmentTarget(_) => "AssignmentTarget".into(),
            Self::SimpleAssignmentTarget(_) => "SimpleAssignmentTarget".into(),
            Self::AssignmentTargetWithDefault(_) => "AssignmentTargetWithDefault".into(),
            Self::SpreadElement(_) => "SpreadElement".into(),
            Self::Elision(_) => "Elision".into(),
            Self::ExpressionArrayElement(_) => "ExpressionArrayElement".into(),
            Self::BindingRestElement(_) => "BindingRestElement".into(),

            Self::Function(x) => format!(
                "Function({})",
                x.id.as_ref().map_or_else(|| "<anonymous>", |id| id.name.as_str())
            )
            .into(),
            Self::FunctionBody(_) => "FunctionBody".into(),
            Self::FormalParameters(_) => "FormalParameters".into(),
            Self::FormalParameter(_) => "FormalParameter".into(),

            Self::Class(c) => format!(
                "Class({})",
                c.id.as_ref().map_or_else(|| "<anonymous>", |id| id.name.as_str())
            )
            .into(),
            Self::ClassBody(_) => "ClassBody".into(),
            Self::ClassHeritage(_) => "ClassHeritage".into(),
            Self::StaticBlock(_) => "StaticBlock".into(),
            Self::PropertyDefinition(_) => "PropertyDefinition".into(),
            Self::MethodDefinition(_) => "MethodDefinition".into(),

            Self::ArrayPattern(_) => "ArrayPattern".into(),
            Self::ObjectPattern(_) => "ObjectPattern".into(),
            Self::AssignmentPattern(_) => "AssignmentPattern".into(),

            Self::Decorator(_) => "Decorator".into(),

            Self::ModuleDeclaration(_) => "ModuleDeclaration".into(),
            Self::ImportDeclaration(_) => "ImportDeclaration".into(),
            Self::ImportSpecifier(_) => "ImportSpecifier".into(),
            Self::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier".into(),
            Self::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier".into(),
            Self::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration".into(),
            Self::ExportNamedDeclaration(_) => "ExportNamedDeclaration".into(),
            Self::ExportAllDeclaration(_) => "ExportAllDeclaration".into(),
            Self::JSXOpeningElement(_) => "JSXOpeningElement".into(),
            Self::JSXClosingElement(_) => "JSXClosingElement".into(),
            Self::JSXElementName(_) => "JSXElementName".into(),
            Self::JSXElement(_) => "JSXElement".into(),
            Self::JSXFragment(_) => "JSXFragment".into(),
            Self::JSXAttributeItem(_) => "JSXAttributeItem".into(),
            Self::JSXSpreadAttribute(_) => "JSXSpreadAttribute".into(),
            Self::JSXText(_) => "JSXText".into(),
            Self::JSXExpressionContainer(_) => "JSXExpressionContainer".into(),
            Self::JSXIdentifier(_) => "JSXIdentifier".into(),
            Self::JSXMemberExpression(_) => "JSXMemberExpression".into(),
            Self::JSXMemberExpressionObject(_) => "JSXMemberExpressionObject".into(),
            Self::JSXNamespacedName(_) => "JSXNamespacedName".into(),

            Self::TSModuleBlock(_) => "TSModuleBlock".into(),

            Self::TSAnyKeyword(_) => "TSAnyKeyword".into(),
            Self::TSIntersectionType(_) => "TSIntersectionType".into(),
            Self::TSLiteralType(_) => "TSLiteralType".into(),
            Self::TSMethodSignature(_) => "TSMethodSignature".into(),
            Self::TSNullKeyword(_) => "TSNullKeyword".into(),
            Self::TSTypeLiteral(_) => "TSTypeLiteral".into(),
            Self::TSTypeReference(_) => "TSTypeReference".into(),
            Self::TSUnionType(_) => "TSUnionType".into(),
            Self::TSVoidKeyword(_) => "TSVoidKeyword".into(),

            Self::TSIndexedAccessType(_) => "TSIndexedAccessType".into(),

            Self::TSAsExpression(_) => "TSAsExpression".into(),
            Self::TSSatisfiesExpression(_) => "TSSatisfiesExpression".into(),
            Self::TSNonNullExpression(_) => "TSNonNullExpression".into(),
            Self::TSInstantiationExpression(_) => "TSInstantiationExpression".into(),

            Self::TSEnumDeclaration(decl) => format!("TSEnumDeclaration({})", &decl.id.name).into(),

            Self::TSEnumMember(_) => "TSEnumMember".into(),

            Self::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration".into(),
            Self::TSTypeName(_) => "TSTypeName".into(),
            Self::TSExternalModuleReference(_) => "TSExternalModuleReference".into(),
            Self::TSQualifiedName(_) => "TSQualifiedName".into(),
            Self::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration".into(),
            Self::TSModuleDeclaration(_) => "TSModuleDeclaration".into(),
            Self::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration".into(),
            Self::TSTypeAnnotation(_) => "TSTypeAnnotation".into(),
            Self::TSTypeQuery(_) => "TSTypeQuery".into(),
            Self::TSTypeAssertion(_) => "TSTypeAssertion".into(),
            Self::TSTypeParameter(_) => "TSTypeParameter".into(),
            Self::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration".into(),
            Self::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation".into(),
            Self::TSImportType(_) => "TSImportType".into(),

            Self::TSPropertySignature(_) => "TSPropertySignature".into(),
        }
    }
}
