use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_mangler::ManglerBuilder;
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn mangle(source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = ret.program;
    let mangler = ManglerBuilder::default().build(&program);
    CodeGenerator::new().with_mangler(Some(mangler)).build(&program).source_text
}
