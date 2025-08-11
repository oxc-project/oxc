use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn main() {
    let allocator = Allocator::default();
    let source_text = r#"class C {
  "constructor"() {
    console.log("constructor!");
  }
}"#;
    let source_type = SourceType::default().with_module(true);
    
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    
    let options = MinifierOptions {
        mangle: None,
        compress: Some(CompressOptions::smallest()),
    };
    
    let minify_ret = Minifier::new(options).build(&allocator, &mut program);
    
    let code_ret = Codegen::new()
        .with_options(CodegenOptions {
            minify: true,
            comments: CommentOptions::disabled(),
            ..CodegenOptions::default()
        })
        .with_scoping(minify_ret.scoping)
        .build(&program);
    
    println!("Original:\n{}", source_text);
    println!("Minified:\n{}", code_ret.code);
}