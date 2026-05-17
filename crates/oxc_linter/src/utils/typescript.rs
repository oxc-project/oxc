//! Shared TypeScript utilities and types for linter rules

use oxc_ast::AstKind;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext};

/// Type or value specifier for matching specific declarations
///
/// Supports four types of specifiers:
///
/// 1. **String specifier** (deprecated): Universal match by name
///    ```json
///    "Promise"
///    ```
///
/// 2. **File specifier**: Match types/values declared in local files
///    ```json
///    { "from": "file", "name": "MyType" }
///    { "from": "file", "name": ["Type1", "Type2"] }
///    { "from": "file", "name": "MyType", "path": "./types.ts" }
///    ```
///
/// 3. **Lib specifier**: Match TypeScript built-in lib types
///    ```json
///    { "from": "lib", "name": "Promise" }
///    { "from": "lib", "name": ["Promise", "PromiseLike"] }
///    ```
///
/// 4. **Package specifier**: Match types/values from npm packages
///    ```json
///    { "from": "package", "name": "Observable", "package": "rxjs" }
///    { "from": "package", "name": ["Observable", "Subject"], "package": "rxjs" }
///    ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TypeOrValueSpecifier {
    /// Universal string specifier - matches all types and values with this name regardless of declaration source.
    /// Not recommended - will be removed in a future major version.
    String(String),
    /// Describes specific types or values declared in local files.
    File(FileSpecifier),
    /// Describes specific types or values declared in TypeScript's built-in lib.*.d.ts types.
    Lib(LibSpecifier),
    /// Describes specific types or values imported from packages.
    Package(PackageSpecifier),
}

/// Describes specific types or values declared in local files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileSpecifier {
    /// Must be "file"
    pub from: FileFrom,
    /// The name(s) of the type or value to match
    pub name: NameSpecifier,
    /// Optional file path to specify where the types or values must be declared.
    /// If omitted, all files will be matched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FileFrom {
    File,
}

/// Describes specific types or values declared in TypeScript's built-in lib.*.d.ts types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibSpecifier {
    /// Must be "lib"
    pub from: LibFrom,
    /// The name(s) of the lib type or value to match
    pub name: NameSpecifier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LibFrom {
    Lib,
}

/// Describes specific types or values imported from packages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PackageSpecifier {
    /// Must be "package"
    pub from: PackageFrom,
    /// The name(s) of the type or value to match
    pub name: NameSpecifier,
    /// The package name to match
    pub package: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PackageFrom {
    Package,
}

/// Name specifier that can be a single string or array of strings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum NameSpecifier {
    /// Single name
    Single(String),
    /// Multiple names
    Multiple(Vec<String>),
}

/// Returns `true` when `node` has a TypeScript ambient declaration ancestor
/// such as `declare module`, `declare namespace`, or `declare global`,
/// including `global {}` nested inside ambient modules or namespaces.
pub fn has_ambient_typescript_ancestor(node: &AstNode, ctx: &LintContext) -> bool {
    ctx.nodes().ancestors(node.id()).any(|ancestor| match ancestor.kind() {
        AstKind::TSModuleDeclaration(module) => module.declare,
        AstKind::TSGlobalDeclaration(global) => global.declare,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use std::{rc::Rc, sync::Arc};

    use oxc_allocator::Allocator;
    use oxc_ast::AstKind;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use crate::{
        ContextHost, ModuleRecord,
        context::{ContextSubHost, ContextSubHostOptions},
        options::LintOptions,
    };

    use super::has_ambient_typescript_ancestor;

    fn helper_result(source: &str) -> bool {
        let allocator = Allocator::default();
        let source_type = SourceType::ts();
        let parser_ret = Parser::new(&allocator, source, source_type).parse();
        assert!(parser_ret.errors.is_empty(), "Parse error in: {source}");

        let program = allocator.alloc(parser_ret.program);
        let semantic = SemanticBuilder::new().with_cfg(true).build(program).semantic;
        let ctx = Rc::new(ContextHost::new(
            "test.ts",
            vec![ContextSubHost::new(
                semantic,
                Arc::new(ModuleRecord::default()),
                0,
                ContextSubHostOptions::default(),
            )],
            LintOptions::default(),
            Arc::default(),
        ))
        .spawn_for_test();

        let decl =
            ctx.nodes().iter().find(|node| matches!(node.kind(), AstKind::VariableDeclaration(_)));
        let decl = decl.expect("expected test source to contain a variable declaration");
        has_ambient_typescript_ancestor(decl, &ctx)
    }

    #[test]
    fn test_has_ambient_typescript_ancestor() {
        assert!(helper_result("declare module 'pkg' { var x: string; }"));
        assert!(helper_result("declare global { var x: string; }"));
        assert!(helper_result("declare module 'pkg' { global { var x: string; } }"));
        assert!(!helper_result("namespace Foo { var x: string; }"));
    }
}
