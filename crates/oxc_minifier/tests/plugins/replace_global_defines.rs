use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_minifier::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::run;

pub(crate) fn test(source_text: &str, expected: &str, config: ReplaceGlobalDefinesConfig) {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    ReplaceGlobalDefines::new(&allocator, config).build(program);
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(program)
        .source_text;
    let expected = run(expected, source_type, None);
    assert_eq!(result, expected, "for source {source_text}");
}

#[test]
fn replace_global_definitions() {
    let config = ReplaceGlobalDefinesConfig::new(&[("id", "text"), ("str", "'text'")]).unwrap();
    test("id, str", "text, 'text'", config);
}

#[test]
fn replace_global_definitions_dot() {
    {
        let config =
            ReplaceGlobalDefinesConfig::new(&[("process.env.NODE_ENV", "production")]).unwrap();
        test("process.env.NODE_ENV", "production", config.clone());
        test("process.env", "process.env", config.clone());
        test("process.env.foo.bar", "process.env.foo.bar", config.clone());
        test("process", "process", config);
    }

    {
        let config = ReplaceGlobalDefinesConfig::new(&[("process", "production")]).unwrap();
        test("foo.process.NODE_ENV", "foo.process.NODE_ENV", config);
    }
}
