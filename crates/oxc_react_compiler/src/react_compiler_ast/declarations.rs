use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::expressions::Expression;
use crate::react_compiler_ast::expressions::Identifier;
use crate::react_compiler_ast::literals::StringLiteral;

/// Union of Declaration types that can appear in export declarations
#[derive(Debug, Clone)]
pub enum Declaration {
    FunctionDeclaration(crate::react_compiler_ast::statements::FunctionDeclaration),
    ClassDeclaration(crate::react_compiler_ast::statements::ClassDeclaration),
    VariableDeclaration(crate::react_compiler_ast::statements::VariableDeclaration),
    TSTypeAliasDeclaration(TSTypeAliasDeclaration),
    TSInterfaceDeclaration(TSInterfaceDeclaration),
    TSEnumDeclaration(TSEnumDeclaration),
    TSModuleDeclaration(TSModuleDeclaration),
    TSDeclareFunction(TSDeclareFunction),
    TypeAlias(TypeAlias),
    OpaqueType(OpaqueType),
    InterfaceDeclaration(InterfaceDeclaration),
    EnumDeclaration(EnumDeclaration),
}

/// The declaration/expression that can appear in `export default <decl>`
#[derive(Debug, Clone)]
pub enum ExportDefaultDecl {
    FunctionDeclaration(crate::react_compiler_ast::statements::FunctionDeclaration),
    ClassDeclaration(crate::react_compiler_ast::statements::ClassDeclaration),
    EnumDeclaration(EnumDeclaration),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct ImportDeclaration {
    pub base: BaseNode,
    pub specifiers: Vec<ImportSpecifier>,
    pub source: StringLiteral,
    pub import_kind: Option<ImportKind>,
    pub assertions: Option<Vec<ImportAttribute>>,
    pub attributes: Option<Vec<ImportAttribute>>,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Value,
    Type,
    Typeof,
}

#[derive(Debug, Clone)]
pub enum ImportSpecifier {
    ImportSpecifier(ImportSpecifierData),
    ImportDefaultSpecifier(ImportDefaultSpecifierData),
    ImportNamespaceSpecifier(ImportNamespaceSpecifierData),
}

#[derive(Debug, Clone)]
pub struct ImportSpecifierData {
    pub base: BaseNode,
    pub local: Identifier,
    pub imported: ModuleExportName,
    pub import_kind: Option<ImportKind>,
}

#[derive(Debug, Clone)]
pub struct ImportDefaultSpecifierData {
    pub base: BaseNode,
    pub local: Identifier,
}

#[derive(Debug, Clone)]
pub struct ImportNamespaceSpecifierData {
    pub base: BaseNode,
    pub local: Identifier,
}

#[derive(Debug, Clone)]
pub struct ImportAttribute {
    pub base: BaseNode,
    pub key: Identifier,
    pub value: StringLiteral,
}

/// Identifier or StringLiteral used as module export names
#[derive(Debug, Clone)]
pub enum ModuleExportName {
    Identifier(Identifier),
    StringLiteral(StringLiteral),
}

#[derive(Debug, Clone)]
pub struct ExportNamedDeclaration {
    pub base: BaseNode,
    pub declaration: Option<Box<Declaration>>,
    pub specifiers: Vec<ExportSpecifier>,
    pub source: Option<StringLiteral>,
    pub export_kind: Option<ExportKind>,
    pub assertions: Option<Vec<ImportAttribute>>,
    pub attributes: Option<Vec<ImportAttribute>>,
}

#[derive(Debug, Clone)]
pub enum ExportKind {
    Value,
    Type,
}

#[derive(Debug, Clone)]
pub enum ExportSpecifier {
    ExportSpecifier(ExportSpecifierData),
    ExportDefaultSpecifier(ExportDefaultSpecifierData),
    ExportNamespaceSpecifier(ExportNamespaceSpecifierData),
}

#[derive(Debug, Clone)]
pub struct ExportSpecifierData {
    pub base: BaseNode,
    pub local: ModuleExportName,
    pub exported: ModuleExportName,
    pub export_kind: Option<ExportKind>,
}

#[derive(Debug, Clone)]
pub struct ExportDefaultSpecifierData {
    pub base: BaseNode,
    pub exported: Identifier,
}

#[derive(Debug, Clone)]
pub struct ExportNamespaceSpecifierData {
    pub base: BaseNode,
    pub exported: ModuleExportName,
}

#[derive(Debug, Clone)]
pub struct ExportDefaultDeclaration {
    pub base: BaseNode,
    pub declaration: Box<ExportDefaultDecl>,
    pub export_kind: Option<ExportKind>,
}

#[derive(Debug, Clone)]
pub struct ExportAllDeclaration {
    pub base: BaseNode,
    pub source: StringLiteral,
    pub export_kind: Option<ExportKind>,
    pub assertions: Option<Vec<ImportAttribute>>,
    pub attributes: Option<Vec<ImportAttribute>>,
}

// TypeScript declarations (pass-through via RawNode for bodies)
#[derive(Debug, Clone)]
pub struct TSTypeAliasDeclaration {
    pub base: BaseNode,
    pub id: Identifier,
    pub type_annotation: RawNode,
    pub type_parameters: Option<RawNode>,
    pub declare: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TSInterfaceDeclaration {
    pub base: BaseNode,
    pub id: Identifier,
    pub body: RawNode,
    pub type_parameters: Option<RawNode>,
    pub extends: Option<Vec<RawNode>>,
    pub declare: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TSEnumDeclaration {
    pub base: BaseNode,
    pub id: Identifier,
    pub members: Vec<RawNode>,
    pub declare: Option<bool>,
    pub is_const: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TSModuleDeclaration {
    pub base: BaseNode,
    pub id: RawNode,
    pub body: RawNode,
    pub declare: Option<bool>,
    pub global: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TSDeclareFunction {
    pub base: BaseNode,
    pub id: Option<Identifier>,
    pub params: Vec<RawNode>,
    pub is_async: Option<bool>,
    pub declare: Option<bool>,
    pub generator: Option<bool>,
    pub return_type: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
}

// Flow declarations (pass-through)
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub base: BaseNode,
    pub id: Identifier,
    pub right: RawNode,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct OpaqueType {
    pub base: BaseNode,
    pub id: Identifier,
    pub supertype: Option<RawNode>,
    pub impltype: RawNode,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct InterfaceDeclaration {
    pub base: BaseNode,
    pub id: Identifier,
    pub body: RawNode,
    pub type_parameters: Option<RawNode>,
    pub extends: Option<Vec<RawNode>>,
    pub mixins: Option<Vec<RawNode>>,
    pub implements: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone)]
pub struct DeclareVariable {
    pub base: BaseNode,
    pub id: Identifier,
}

#[derive(Debug, Clone)]
pub struct DeclareFunction {
    pub base: BaseNode,
    pub id: Identifier,
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct DeclareClass {
    pub base: BaseNode,
    pub id: Identifier,
    pub body: RawNode,
    pub type_parameters: Option<RawNode>,
    pub extends: Option<Vec<RawNode>>,
    pub mixins: Option<Vec<RawNode>>,
    pub implements: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone)]
pub struct DeclareModule {
    pub base: BaseNode,
    pub id: RawNode,
    pub body: RawNode,
    pub kind: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeclareModuleExports {
    pub base: BaseNode,
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone)]
pub struct DeclareExportDeclaration {
    pub base: BaseNode,
    pub declaration: Option<RawNode>,
    pub specifiers: Option<Vec<RawNode>>,
    pub source: Option<StringLiteral>,
    pub default: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DeclareExportAllDeclaration {
    pub base: BaseNode,
    pub source: StringLiteral,
}

#[derive(Debug, Clone)]
pub struct DeclareInterface {
    pub base: BaseNode,
    pub id: Identifier,
    pub body: RawNode,
    pub type_parameters: Option<RawNode>,
    pub extends: Option<Vec<RawNode>>,
    pub mixins: Option<Vec<RawNode>>,
    pub implements: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone)]
pub struct DeclareTypeAlias {
    pub base: BaseNode,
    pub id: Identifier,
    pub right: RawNode,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct DeclareOpaqueType {
    pub base: BaseNode,
    pub id: Identifier,
    pub supertype: Option<RawNode>,
    pub impltype: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub base: BaseNode,
    pub id: Identifier,
    pub body: RawNode,
}
