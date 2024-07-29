use oxc_allocator::Allocator;
use oxc_codegen::{CodegenOptions, WhitespaceRemover};
use oxc_minifier::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub(crate) fn test(source_text: &str, expected: &str, config: ReplaceGlobalDefinesConfig) {
    let minified = {
        let source_type = SourceType::default();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        ReplaceGlobalDefines::new(&allocator, config).build(program);
        WhitespaceRemover::new()
            .with_options(CodegenOptions { single_quote: true })
            .build(program)
            .source_text
    };
    assert_eq!(minified, expected, "for source {source_text}");
}

#[test]
fn replace_global_definitions() {
    let config = ReplaceGlobalDefinesConfig::new(&[("id", "text"), ("str", "'text'")]).unwrap();
    test("id, str", "text,'text';", config);
}
