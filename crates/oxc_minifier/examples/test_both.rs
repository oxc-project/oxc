use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn main() {
    let allocator = Allocator::default();
    
    // Test unquoted constructor
    let source_text1 = r#"class C {
  constructor() {
    console.log("constructor!");
  }
}"#;
    
    // Test quoted constructor  
    let source_text2 = r#"class C {
  "constructor"() {
    console.log("constructor!");
  }
}"#;
    
    let source_type = SourceType::default().with_module(true);
    
    for (i, source_text) in [source_text1, source_text2].iter().enumerate() {
        println!("=== Test {} ===", i + 1);
        println!("Original:\n{}", source_text);
        
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let mut program = ret.program;
        
        let options = MinifierOptions {
            mangle: None,
            compress: Some(CompressOptions::smallest()),
        };
        
        let minify_ret = Minifier::new(options.clone()).build(&allocator, &mut program);
        
        let code_ret = Codegen::new()
            .with_options(CodegenOptions {
                minify: true,
                comments: CommentOptions::disabled(),
                ..CodegenOptions::default()
            })
            .with_scoping(minify_ret.scoping)
            .build(&program);
        
        println!("Minified:\n{}", code_ret.code);
        println!();
    }
}