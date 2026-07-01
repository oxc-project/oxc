use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::expressions::Expression;
use crate::react_compiler_ast::expressions::Identifier;
use crate::react_compiler_ast::patterns::PatternLike;

#[derive(Debug, Clone)]
pub enum Statement {
    // Statements
    BlockStatement(BlockStatement),
    ReturnStatement(ReturnStatement),
    IfStatement(IfStatement),
    ForStatement(ForStatement),
    WhileStatement(WhileStatement),
    DoWhileStatement(DoWhileStatement),
    ForInStatement(ForInStatement),
    ForOfStatement(ForOfStatement),
    SwitchStatement(SwitchStatement),
    ThrowStatement(ThrowStatement),
    TryStatement(TryStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    LabeledStatement(LabeledStatement),
    ExpressionStatement(ExpressionStatement),
    EmptyStatement(EmptyStatement),
    DebuggerStatement(DebuggerStatement),
    WithStatement(WithStatement),
    // Declarations are also statements
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
    // Import/export declarations
    ImportDeclaration(crate::react_compiler_ast::declarations::ImportDeclaration),
    ExportNamedDeclaration(crate::react_compiler_ast::declarations::ExportNamedDeclaration),
    ExportDefaultDeclaration(crate::react_compiler_ast::declarations::ExportDefaultDeclaration),
    ExportAllDeclaration(crate::react_compiler_ast::declarations::ExportAllDeclaration),
    // TypeScript declarations
    TSTypeAliasDeclaration(crate::react_compiler_ast::declarations::TSTypeAliasDeclaration),
    TSInterfaceDeclaration(crate::react_compiler_ast::declarations::TSInterfaceDeclaration),
    TSEnumDeclaration(crate::react_compiler_ast::declarations::TSEnumDeclaration),
    TSModuleDeclaration(crate::react_compiler_ast::declarations::TSModuleDeclaration),
    TSDeclareFunction(crate::react_compiler_ast::declarations::TSDeclareFunction),
    // Flow declarations
    TypeAlias(crate::react_compiler_ast::declarations::TypeAlias),
    OpaqueType(crate::react_compiler_ast::declarations::OpaqueType),
    InterfaceDeclaration(crate::react_compiler_ast::declarations::InterfaceDeclaration),
    DeclareVariable(crate::react_compiler_ast::declarations::DeclareVariable),
    DeclareFunction(crate::react_compiler_ast::declarations::DeclareFunction),
    DeclareClass(crate::react_compiler_ast::declarations::DeclareClass),
    DeclareModule(crate::react_compiler_ast::declarations::DeclareModule),
    DeclareModuleExports(crate::react_compiler_ast::declarations::DeclareModuleExports),
    DeclareExportDeclaration(crate::react_compiler_ast::declarations::DeclareExportDeclaration),
    DeclareExportAllDeclaration(
        crate::react_compiler_ast::declarations::DeclareExportAllDeclaration,
    ),
    DeclareInterface(crate::react_compiler_ast::declarations::DeclareInterface),
    DeclareTypeAlias(crate::react_compiler_ast::declarations::DeclareTypeAlias),
    DeclareOpaqueType(crate::react_compiler_ast::declarations::DeclareOpaqueType),
    EnumDeclaration(crate::react_compiler_ast::declarations::EnumDeclaration),
    /// Catch-all for statement `type`s the typed AST does not model, e.g. the
    /// TypeScript module-interop statements `import x = require(...)`,
    /// `export = x`, and `export as namespace X`. In the oxc integration these
    /// are re-emitted verbatim from their source span; the variant is retained
    /// for the upstream Babel path.
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone)]
pub struct UnknownStatement {
    raw: RawNode,
    base: BaseNode,
}

impl UnknownStatement {
    pub fn from_raw(raw: RawNode) -> Result<Self, String> {
        let base = BaseNode { node_type: raw.node_type.clone(), ..BaseNode::default() };
        Ok(Self { raw, base })
    }

    /// The node's `type` discriminant, read from the captured [`BaseNode`].
    pub fn node_type(&self) -> &str {
        self.base.node_type.as_deref().unwrap_or("Unknown")
    }

    pub fn raw(&self) -> &RawNode {
        &self.raw
    }

    /// Mutate the raw node in place.
    pub fn with_raw_mut<R>(&mut self, f: impl FnOnce(&mut RawNode) -> R) -> Result<R, String> {
        Ok(f(&mut self.raw))
    }

    pub fn base(&self) -> &BaseNode {
        &self.base
    }
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub base: BaseNode,
    pub body: Vec<Statement>,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub base: BaseNode,
    pub value: DirectiveLiteral,
}

#[derive(Debug, Clone)]
pub struct DirectiveLiteral {
    pub base: BaseNode,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub base: BaseNode,
    pub argument: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub consequent: Box<Statement>,
    pub alternate: Option<Box<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub base: BaseNode,
    pub init: Option<Box<ForInit>>,
    pub test: Option<Box<Expression>>,
    pub update: Option<Box<Expression>>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub enum ForInit {
    VariableDeclaration(VariableDeclaration),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub struct DoWhileStatement {
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub struct ForInStatement {
    pub base: BaseNode,
    pub left: Box<ForInOfLeft>,
    pub right: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub struct ForOfStatement {
    pub base: BaseNode,
    pub left: Box<ForInOfLeft>,
    pub right: Box<Expression>,
    pub body: Box<Statement>,
    pub is_await: bool,
}

#[derive(Debug, Clone)]
pub enum ForInOfLeft {
    VariableDeclaration(VariableDeclaration),
    Pattern(Box<PatternLike>),
}

#[derive(Debug, Clone)]
pub struct SwitchStatement {
    pub base: BaseNode,
    pub discriminant: Box<Expression>,
    pub cases: Vec<SwitchCase>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub base: BaseNode,
    pub test: Option<Box<Expression>>,
    pub consequent: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ThrowStatement {
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct TryStatement {
    pub base: BaseNode,
    pub block: BlockStatement,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStatement>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub base: BaseNode,
    pub param: Option<PatternLike>,
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct BreakStatement {
    pub base: BaseNode,
    pub label: Option<Identifier>,
}

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub base: BaseNode,
    pub label: Option<Identifier>,
}

#[derive(Debug, Clone)]
pub struct LabeledStatement {
    pub base: BaseNode,
    pub label: Identifier,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub struct EmptyStatement {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct DebuggerStatement {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct WithStatement {
    pub base: BaseNode,
    pub object: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub base: BaseNode,
    pub declarations: Vec<VariableDeclarator>,
    pub kind: VariableDeclarationKind,
    pub declare: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
    Using,
}

#[derive(Debug, Clone)]
pub struct VariableDeclarator {
    pub base: BaseNode,
    pub id: PatternLike,
    pub init: Option<Box<Expression>>,
    pub definite: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub base: BaseNode,
    pub id: Option<Identifier>,
    pub params: Vec<PatternLike>,
    pub body: BlockStatement,
    pub generator: bool,
    pub is_async: bool,
    pub declare: Option<bool>,
    pub return_type: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
    pub predicate: Option<RawNode>,
    /// Set by the Hermes parser for Flow `component Foo(...) { ... }` syntax
    pub component_declaration: bool,
    /// Set by the Hermes parser for Flow `hook useFoo(...) { ... }` syntax
    pub hook_declaration: bool,
}

#[derive(Debug, Clone)]
pub struct ClassDeclaration {
    pub base: BaseNode,
    pub id: Option<Identifier>,
    pub super_class: Option<Box<Expression>>,
    pub body: crate::react_compiler_ast::expressions::ClassBody,
    pub decorators: Option<Vec<RawNode>>,
    pub is_abstract: Option<bool>,
    pub declare: Option<bool>,
    pub implements: Option<Vec<RawNode>>,
    pub super_type_parameters: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
    pub mixins: Option<Vec<RawNode>>,
}
