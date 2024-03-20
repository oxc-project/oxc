use oxc_span::Span;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

#[derive(Clone, Copy)]
pub struct AstKind2<'a> {
    r#ref: AstRef<'a>,
    r#type: AstType,
}

impl<'a> AstKind2<'a> {
    pub fn ast_type(&self) -> AstType {
        self.r#type
    }
}

macro_rules! def_ast_types {
    { $($ident:ident($type:ty),)* } => (
        #[derive(Clone, Copy)]
        pub enum AstType {
            $($ident,)*
        }

        #[allow(non_snake_case)]
        #[derive(Clone, Copy)]
        pub union AstRef<'a> {
            $($ident: $type,)*
        }

        $(
            impl<'a> AstKind2<'a> {
                #[allow(non_snake_case)]
                pub fn $ident(value: $type) -> Self {
                    Self { r#ref: AstRef { $ident: value }, r#type: AstType::$ident }
                }
            }
        )*
    )
}

def_ast_types! {
    Program(&'a Program<'a>),
    Directive(&'a Directive<'a>),
    Hashbang(&'a Hashbang<'a>),

    BlockStatement(&'a BlockStatement<'a>),
    BreakStatement(&'a BreakStatement<'a>),
    ContinueStatement(&'a ContinueStatement<'a>),
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

    UsingDeclaration(&'a UsingDeclaration<'a>),

    IdentifierName(&'a IdentifierName<'a>),
    IdentifierReference(&'a IdentifierReference<'a>),
    BindingIdentifier(&'a BindingIdentifier<'a>),
    LabelIdentifier(&'a LabelIdentifier<'a>),
    PrivateIdentifier(&'a PrivateIdentifier<'a>),

    NumericLiteral(&'a NumericLiteral<'a>),
    StringLiteral(&'a StringLiteral<'a>),
    BooleanLiteral(&'a BooleanLiteral),
    NullLiteral(&'a NullLiteral),
    BigintLiteral(&'a BigIntLiteral<'a>),
    RegExpLiteral(&'a RegExpLiteral<'a>),
    TemplateLiteral(&'a TemplateLiteral<'a>),

    MetaProperty(&'a MetaProperty<'a>),
    Super(&'a Super),

    ArrayExpression(&'a ArrayExpression<'a>),
    ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
    AssignmentExpression(&'a AssignmentExpression<'a>),
    AwaitExpression(&'a AwaitExpression<'a>),
    BinaryExpression(&'a BinaryExpression<'a>),
    CallExpression(&'a CallExpression<'a>),
    ChainExpression(&'a ChainExpression<'a>),
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
    ImportExpression(&'a ImportExpression<'a>),
    PrivateInExpression(&'a PrivateInExpression<'a>),

    ObjectProperty(&'a ObjectProperty<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    Argument(&'a Argument<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>),
    ArrayExpressionElement(&'a ArrayExpressionElement<'a>),
    Elision(Span),
    ExpressionArrayElement(&'a Expression<'a>),
    SpreadElement(&'a SpreadElement<'a>),
    BindingRestElement(&'a BindingRestElement<'a>),

    Function(&'a Function<'a>),
    FunctionBody(&'a FunctionBody<'a>),
    FormalParameters(&'a FormalParameters<'a>),
    FormalParameter(&'a FormalParameter<'a>),

    Class(&'a Class<'a>),
    ClassBody(&'a ClassBody<'a>),
    ClassHeritage(&'a Expression<'a>),
    StaticBlock(&'a StaticBlock<'a>),
    PropertyDefinition(&'a PropertyDefinition<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),

    ArrayPattern(&'a ArrayPattern<'a>),
    ObjectPattern(&'a ObjectPattern<'a>),
    AssignmentPattern(&'a AssignmentPattern<'a>),

    Decorator(&'a Decorator<'a>),

    ModuleDeclaration(&'a ModuleDeclaration<'a>),
    ImportDeclaration(&'a ImportDeclaration<'a>),
    ImportSpecifier(&'a ImportSpecifier<'a>),
    ImportDefaultSpecifier(&'a ImportDefaultSpecifier<'a>),
    ImportNamespaceSpecifier(&'a ImportNamespaceSpecifier<'a>),
    ExportDefaultDeclaration(&'a ExportDefaultDeclaration<'a>),
    ExportNamedDeclaration(&'a ExportNamedDeclaration<'a>),
    ExportAllDeclaration(&'a ExportAllDeclaration<'a>),

    // JSX
    // Please make sure to add these to `is_jsx` below.
    JSXElement(&'a JSXElement<'a>),
    JSXFragment(&'a JSXFragment<'a>),
    JSXOpeningElement(&'a JSXOpeningElement<'a>),
    JSXClosingElement(&'a JSXClosingElement<'a>),
    JSXElementName(&'a JSXElementName<'a>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXAttributeItem(&'a JSXAttributeItem<'a>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXText(&'a JSXText<'a>),
    JSXIdentifier(&'a JSXIdentifier<'a>),
    JSXMemberExpression(&'a JSXMemberExpression<'a>),
    JSXMemberExpressionObject(&'a JSXMemberExpressionObject<'a>),
    JSXNamespacedName(&'a JSXNamespacedName<'a>),

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
    TSInstantiationExpression(&'a TSInstantiationExpression<'a>),

    TSEnumDeclaration(&'a TSEnumDeclaration<'a>),
    TSEnumMember(&'a TSEnumMember<'a>),

    TSImportEqualsDeclaration(&'a TSImportEqualsDeclaration<'a>),
    TSTypeName(&'a TSTypeName<'a>),
    TSExternalModuleReference(&'a TSExternalModuleReference<'a>),
    TSQualifiedName(&'a TSQualifiedName<'a>),

    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>),
    TSModuleDeclaration(&'a TSModuleDeclaration<'a>),
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>),
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>),
    TSTypeQuery(&'a TSTypeQuery<'a>),
    TSTypeAssertion(&'a TSTypeAssertion<'a>),
    TSTypeParameter(&'a TSTypeParameter<'a>),
    TSTypeParameterDeclaration(&'a TSTypeParameterDeclaration<'a>),
    TSTypeParameterInstantiation(&'a TSTypeParameterInstantiation<'a>),
    TSImportType(&'a TSImportType<'a>),

    TSPropertySignature(&'a TSPropertySignature<'a>),

}
