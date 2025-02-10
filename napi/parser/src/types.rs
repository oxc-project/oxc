use std::mem;

use napi_derive::napi;

use oxc_napi::OxcError;

use crate::magic_string::MagicString;

#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,

    /// Treat the source text as `js`, `jsx`, `ts`, or `tsx`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx'")]
    pub lang: Option<String>,

    /// Emit `ParenthesizedExpression` in AST.
    ///
    /// If this option is true, parenthesized expressions are represented by
    /// (non-standard) `ParenthesizedExpression` nodes that have a single `expression` property
    /// containing the expression inside parentheses.
    ///
    /// Default: true
    pub preserve_parens: Option<bool>,
}

#[napi]
pub struct ParseResult {
    pub(crate) source_text: String,
    pub(crate) program: String,
    pub(crate) module: EcmaScriptModule,
    pub(crate) comments: Vec<Comment>,
    pub(crate) errors: Vec<OxcError>,
}

#[napi]
impl ParseResult {
    #[napi(getter, ts_return_type = "import(\"@oxc-project/types\").Program")]
    pub fn get_program(&mut self) -> String {
        mem::take(&mut self.program)
    }

    #[napi(getter)]
    pub fn module(&mut self) -> EcmaScriptModule {
        mem::take(&mut self.module)
    }

    #[napi(getter)]
    pub fn comments(&mut self) -> Vec<Comment> {
        mem::take(&mut self.comments)
    }

    #[napi(getter)]
    pub fn errors(&mut self) -> Vec<OxcError> {
        mem::take(&mut self.errors)
    }

    #[napi(getter)]
    pub fn magic_string(&mut self) -> MagicString {
        MagicString::new(mem::take(&mut self.source_text))
    }
}

#[napi(object)]
pub struct Comment {
    #[napi(ts_type = "'Line' | 'Block'")]
    pub r#type: String,
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
#[derive(Default)]
pub struct EcmaScriptModule {
    /// Has ESM syntax.
    ///
    /// i.e. `import` and `export` statements, and `import.meta`.
    ///
    /// Dynamic imports `import('foo')` are ignored since they can be used in non-ESM files.
    pub has_module_syntax: bool,
    /// Import statements.
    pub static_imports: Vec<StaticImport>,
    /// Export statements.
    pub static_exports: Vec<StaticExport>,
    /// Dynamic import expressions.
    pub dynamic_imports: Vec<DynamicImport>,
    /// Span positions` of `import.meta`
    pub import_metas: Vec<Span>,
}

#[napi(object)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct ValueSpan {
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct StaticImport {
    /// Start of import statement.
    pub start: u32,
    /// End of import statement.
    pub end: u32,
    /// Import source.
    ///
    /// ```js
    /// import { foo } from "mod";
    /// //                   ^^^
    /// ```
    pub module_request: ValueSpan,
    /// Import specifiers.
    ///
    /// Empty for `import "mod"`.
    pub entries: Vec<StaticImportEntry>,
}

#[napi(object)]
pub struct StaticImportEntry {
    /// The name under which the desired binding is exported by the module.
    ///
    /// ```js
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //       ^^^
    /// ```
    pub import_name: ImportName,
    /// The name that is used to locally access the imported value from within the importing module.
    /// ```js
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //              ^^^
    /// ```
    pub local_name: ValueSpan,

    /// Whether this binding is for a TypeScript type-only import.
    ///
    /// `true` for the following imports:
    /// ```ts
    /// import type { foo } from "mod";
    /// import { type foo } from "mod";
    /// ```
    pub is_type: bool,
}

#[napi(string_enum)]
pub enum ImportNameKind {
    /// `import { x } from "mod"`
    Name,
    /// `import * as ns from "mod"`
    NamespaceObject,
    /// `import defaultExport from "mod"`
    Default,
}

#[napi(object)]
pub struct ImportName {
    pub kind: ImportNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(object)]
pub struct StaticExportEntry {
    pub start: u32,
    pub end: u32,
    pub module_request: Option<ValueSpan>,
    /// The name under which the desired binding is exported by the module`.
    pub import_name: ExportImportName,
    /// The name used to export this binding by this module.
    pub export_name: ExportExportName,
    /// The name that is used to locally access the exported value from within the importing module.
    pub local_name: ExportLocalName,
}

#[napi(object)]
pub struct StaticExport {
    pub start: u32,
    pub end: u32,
    pub entries: Vec<StaticExportEntry>,
}

#[napi(string_enum)]
pub enum ExportImportNameKind {
    /// `export { name }
    Name,
    /// `export * as ns from "mod"`
    All,
    /// `export * from "mod"`
    AllButDefault,
    /// Does not have a specifier.
    None,
}

#[napi(object)]
pub struct ExportImportName {
    pub kind: ExportImportNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(string_enum)]
pub enum ExportExportNameKind {
    /// `export { name }
    Name,
    /// `export default expression`
    Default,
    /// `export * from "mod"
    None,
}

#[napi(object)]
pub struct ExportExportName {
    pub kind: ExportExportNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(object)]
pub struct ExportLocalName {
    pub kind: ExportLocalNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(string_enum)]
pub enum ExportLocalNameKind {
    /// `export { name }
    Name,
    /// `export default expression`
    Default,
    /// If the exported value is not locally accessible from within the module.
    /// `export default function () {}`
    None,
}

#[napi(object)]
pub struct DynamicImport {
    pub start: u32,
    pub end: u32,
    pub module_request: Span,
}
