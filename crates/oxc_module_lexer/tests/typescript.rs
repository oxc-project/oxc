use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[non_exhaustive]
struct ModuleLexer {
    imports: Vec<bool>,
    exports: Vec<bool>,
}

fn parse(source: &str) -> ModuleLexer {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true).with_typescript_definition(true);
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(ret.errors.is_empty(), "{source} should not produce errors.\n{:?}", ret.errors);
    let module_lexer = oxc_module_lexer::ModuleLexer::new().build(&ret.program);
    ModuleLexer {
        imports: module_lexer.imports.into_iter().map(|i| i.t).collect(),
        exports: module_lexer.exports.into_iter().map(|e| e.t).collect(),
    }
}

#[test]
fn import_type_named() {
    let source = "import type { foo } from 'foo'";
    let impt = &parse(source).imports[0];
    assert!(impt);
}

#[test]
fn import_type_namespace() {
    let source = "import type * as foo from 'foo'";
    let impt = &parse(source).imports[0];
    assert!(impt);
}

#[test]
fn import_type_default() {
    let source = "import type foo from 'foo'";
    let impt = &parse(source).imports[0];
    assert!(impt);
}

#[test]
fn export_type_named() {
    let source = "export type { foo } from 'foo'";
    let expt = &parse(source).exports[0];
    assert!(expt);
}

#[test]
fn export_type_namespace() {
    let source = "export type * as foo from 'foo'";
    let expt = &parse(source).exports[0];
    assert!(expt);
}
