use serde::Serialize;
use serde::Serializer;

use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::common::from_json_str_unbounded;
use crate::react_compiler_ast::declarations::DeclareClass;
use crate::react_compiler_ast::declarations::DeclareExportAllDeclaration;
use crate::react_compiler_ast::declarations::DeclareExportDeclaration;
use crate::react_compiler_ast::declarations::DeclareFunction;
use crate::react_compiler_ast::declarations::DeclareInterface;
use crate::react_compiler_ast::declarations::DeclareModule;
use crate::react_compiler_ast::declarations::DeclareModuleExports;
use crate::react_compiler_ast::declarations::DeclareOpaqueType;
use crate::react_compiler_ast::declarations::DeclareTypeAlias;
use crate::react_compiler_ast::declarations::DeclareVariable;
use crate::react_compiler_ast::declarations::EnumDeclaration;
use crate::react_compiler_ast::declarations::ExportAllDeclaration;
use crate::react_compiler_ast::declarations::ExportDefaultDeclaration;
use crate::react_compiler_ast::declarations::ExportNamedDeclaration;
use crate::react_compiler_ast::declarations::ImportDeclaration;
use crate::react_compiler_ast::declarations::InterfaceDeclaration;
use crate::react_compiler_ast::declarations::OpaqueType;
use crate::react_compiler_ast::declarations::TSDeclareFunction;
use crate::react_compiler_ast::declarations::TSEnumDeclaration;
use crate::react_compiler_ast::declarations::TSInterfaceDeclaration;
use crate::react_compiler_ast::declarations::TSModuleDeclaration;
use crate::react_compiler_ast::declarations::TSTypeAliasDeclaration;
use crate::react_compiler_ast::declarations::TypeAlias;
use crate::react_compiler_ast::expressions::ClassBody;
use crate::react_compiler_ast::expressions::Expression;
use crate::react_compiler_ast::expressions::Identifier;
use crate::react_compiler_ast::patterns::PatternLike;

fn is_false(v: &bool) -> bool {
    !v
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
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
    ImportDeclaration(ImportDeclaration),
    ExportNamedDeclaration(ExportNamedDeclaration),
    ExportDefaultDeclaration(ExportDefaultDeclaration),
    ExportAllDeclaration(ExportAllDeclaration),
    // TypeScript declarations
    TSTypeAliasDeclaration(TSTypeAliasDeclaration),
    TSInterfaceDeclaration(TSInterfaceDeclaration),
    TSEnumDeclaration(TSEnumDeclaration),
    TSModuleDeclaration(TSModuleDeclaration),
    TSDeclareFunction(TSDeclareFunction),
    // Flow declarations
    TypeAlias(TypeAlias),
    OpaqueType(OpaqueType),
    InterfaceDeclaration(InterfaceDeclaration),
    DeclareVariable(DeclareVariable),
    DeclareFunction(DeclareFunction),
    DeclareClass(DeclareClass),
    DeclareModule(DeclareModule),
    DeclareModuleExports(DeclareModuleExports),
    DeclareExportDeclaration(DeclareExportDeclaration),
    DeclareExportAllDeclaration(DeclareExportAllDeclaration),
    DeclareInterface(DeclareInterface),
    DeclareTypeAlias(DeclareTypeAlias),
    DeclareOpaqueType(DeclareOpaqueType),
    EnumDeclaration(EnumDeclaration),
    /// Catch-all for statement `type`s the typed AST does not model, e.g. the
    /// TypeScript module-interop statements `import x = require(...)`,
    /// `export = x`, and `export as namespace X`. Carries the complete raw
    /// Babel node so the Babel path can preserve unmodeled top-level
    /// statements verbatim instead of failing the whole file.
    ///
    /// Deserialization dispatches through [`KnownStatement`]: a modeled `type`
    /// whose body is malformed errors with the typed variant's precise message
    /// rather than degrading to `Unknown`. Adding a variant to this enum
    /// requires adding it to the `known_statements!` list below, which is the
    /// single source for the dispatch enum, its `From` mapping, and
    /// [`KNOWN_STATEMENT_TYPES`]. A variant added here but not there degrades
    /// to `Unknown` silently; that is the one drift case structure cannot
    /// catch.
    #[serde(untagged)]
    Unknown(UnknownStatement),
}

// NOTE: `Deserialize` for `Statement` is hand-written below; the
// `#[serde(tag = "type")]` and `#[serde(untagged)]` attributes on the enum
// configure only the derived `Serialize`.

#[derive(Debug, Clone)]
pub struct UnknownStatement {
    raw: RawNode,
    base: BaseNode,
}

impl UnknownStatement {
    pub fn from_raw(raw: RawNode) -> Result<Self, String> {
        match raw.type_name() {
            Some(_) => {
                // Parsing into BaseNode reads only the fields BaseNode declares,
                // not the whole (arbitrarily large) unknown subtree.
                let base = from_json_str_unbounded::<BaseNode>(raw.get())
                    .map_err(|err| format!("failed to read unknown statement base: {err}"))?;
                Ok(Self { raw, base })
            }
            None => Err("unknown statement is missing a string `type` field".to_string()),
        }
    }

    /// The node's `type` discriminant, read from the captured [`BaseNode`].
    /// Falls back to `"Unknown"` rather than panicking if the raw node was
    /// mutated out from under it.
    pub fn node_type(&self) -> &str {
        self.base.node_type.as_deref().unwrap_or("Unknown")
    }

    pub fn raw(&self) -> &RawNode {
        &self.raw
    }

    /// Mutate the raw node, then refresh the cached [`BaseNode`] so `base()`
    /// and `node_type()` cannot drift from `raw`. Mutations that remove the
    /// string `type` field are rejected and rolled back.
    pub fn with_raw_mut<R>(&mut self, f: impl FnOnce(&mut RawNode) -> R) -> Result<R, String> {
        let saved = self.raw.clone();
        let result = f(&mut self.raw);
        if self.raw.type_name().is_none() {
            self.raw = saved;
            return Err("unknown statement mutation removed the string `type` field".to_string());
        }
        match from_json_str_unbounded::<BaseNode>(self.raw.get()) {
            Ok(base) => {
                self.base = base;
                Ok(result)
            }
            Err(err) => {
                self.raw = saved;
                Err(format!("failed to refresh unknown statement base: {err}"))
            }
        }
    }

    pub fn base(&self) -> &BaseNode {
        &self.base
    }
}

impl Serialize for UnknownStatement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub body: Vec<Statement>,
    #[serde(default)]
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Directive {
    #[serde(flatten)]
    pub base: BaseNode,
    pub value: DirectiveLiteral,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectiveLiteral {
    #[serde(flatten)]
    pub base: BaseNode,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReturnStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub argument: Option<Box<Expression>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpressionStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IfStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub consequent: Box<Statement>,
    pub alternate: Option<Box<Statement>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub init: Option<Box<ForInit>>,
    pub test: Option<Box<Expression>>,
    pub update: Option<Box<Expression>>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ForInit {
    VariableDeclaration(VariableDeclaration),
    #[serde(untagged)]
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, Serialize)]
pub struct WhileStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoWhileStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForInStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub left: Box<ForInOfLeft>,
    pub right: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForOfStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub left: Box<ForInOfLeft>,
    pub right: Box<Expression>,
    pub body: Box<Statement>,
    #[serde(default, rename = "await")]
    pub is_await: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ForInOfLeft {
    VariableDeclaration(VariableDeclaration),
    #[serde(untagged)]
    Pattern(Box<PatternLike>),
}

#[derive(Debug, Clone, Serialize)]
pub struct SwitchStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub discriminant: Box<Expression>,
    pub cases: Vec<SwitchCase>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwitchCase {
    #[serde(flatten)]
    pub base: BaseNode,
    pub test: Option<Box<Expression>>,
    pub consequent: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThrowStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TryStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub block: BlockStatement,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStatement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatchClause {
    #[serde(flatten)]
    pub base: BaseNode,
    pub param: Option<PatternLike>,
    pub body: BlockStatement,
}

#[derive(Debug, Clone, Serialize)]
pub struct BreakStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub label: Option<Identifier>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContinueStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub label: Option<Identifier>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabeledStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub label: Identifier,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmptyStatement {
    #[serde(flatten)]
    pub base: BaseNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebuggerStatement {
    #[serde(flatten)]
    pub base: BaseNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct WithStatement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub object: Box<Expression>,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableDeclaration {
    #[serde(flatten)]
    pub base: BaseNode,
    pub declarations: Vec<VariableDeclarator>,
    pub kind: VariableDeclarationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declare: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
    Using,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableDeclarator {
    #[serde(flatten)]
    pub base: BaseNode,
    pub id: PatternLike,
    pub init: Option<Box<Expression>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definite: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDeclaration {
    #[serde(flatten)]
    pub base: BaseNode,
    pub id: Option<Identifier>,
    pub params: Vec<PatternLike>,
    pub body: BlockStatement,
    #[serde(default)]
    pub generator: bool,
    #[serde(default, rename = "async")]
    pub is_async: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declare: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "returnType")]
    pub return_type: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "predicate")]
    pub predicate: Option<RawNode>,
    /// Set by the Hermes parser for Flow `component Foo(...) { ... }` syntax
    #[serde(default, skip_serializing_if = "is_false", rename = "__componentDeclaration")]
    pub component_declaration: bool,
    /// Set by the Hermes parser for Flow `hook useFoo(...) { ... }` syntax
    #[serde(default, skip_serializing_if = "is_false", rename = "__hookDeclaration")]
    pub hook_declaration: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassDeclaration {
    #[serde(flatten)]
    pub base: BaseNode,
    pub id: Option<Identifier>,
    #[serde(rename = "superClass")]
    pub super_class: Option<Box<Expression>>,
    pub body: ClassBody,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "abstract")]
    pub is_abstract: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declare: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "implements")]
    pub implements: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "superTypeParameters")]
    pub super_type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mixins: Option<Vec<RawNode>>,
}
