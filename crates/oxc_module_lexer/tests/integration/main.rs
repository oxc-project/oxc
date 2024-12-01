pub mod esm;
pub mod typescript;

use oxc_allocator::Allocator;
use oxc_module_lexer::ImportType;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[non_exhaustive]
pub struct ModuleLexer {
    pub imports: Vec<ImportSpecifier>,
    pub exports: Vec<ExportSpecifier>,
    pub has_module_syntax: bool,
    pub facade: bool,
}

#[derive(Debug, Clone)]
pub struct ImportSpecifier {
    pub n: Option<String>,
    pub s: u32,
    pub e: u32,
    pub ss: u32,
    pub se: u32,
    pub d: ImportType,
    pub a: Option<u32>,
    pub t: bool,
}

impl From<oxc_module_lexer::ImportSpecifier<'_>> for ImportSpecifier {
    fn from(value: oxc_module_lexer::ImportSpecifier) -> Self {
        Self {
            n: value.n.map(|n| n.to_string()),
            s: value.s,
            e: value.e,
            ss: value.ss,
            se: value.se,
            d: value.d,
            a: value.a,
            t: value.t,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportSpecifier {
    pub n: String,
    pub ln: Option<String>,
    pub s: u32,
    pub e: u32,
    pub ls: Option<u32>,
    pub le: Option<u32>,
    pub t: bool,
}

impl From<oxc_module_lexer::ExportSpecifier<'_>> for ExportSpecifier {
    fn from(value: oxc_module_lexer::ExportSpecifier) -> Self {
        Self {
            n: value.n.to_string(),
            ln: value.ln.map(|ln| ln.to_string()),
            s: value.s,
            e: value.e,
            ls: value.ls,
            le: value.le,
            t: value.t,
        }
    }
}

/// # Panics
pub fn parse(source: &str) -> ModuleLexer {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let module_lexer = oxc_module_lexer::ModuleLexer::new().build(&ret.program);
    // Copy data over because `ModuleLexer<'a>` can't be returned
    ModuleLexer {
        imports: module_lexer.imports.into_iter().map(Into::into).collect(),
        exports: module_lexer.exports.into_iter().map(Into::into).collect(),
        has_module_syntax: module_lexer.has_module_syntax,
        facade: module_lexer.facade,
    }
}
