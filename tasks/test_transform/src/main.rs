use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

fn main() {
    // display-name/nested
    transform(
        "foo.js",
        "var foo = qux(createReactClass({}));\nvar bar = qux(React.createClass({}));\n",
    );

    // imports/elision-locations
    transform(
        "foo.ts",
        "import { A, B, C, D, E, F, G, H } from \"m\";
class Class extends A<B> implements C<D> {}
interface Iface extends E<F> {}
const x: G = 0;
const y: H.T = 0;
",
    );
}

fn transform(filename: &str, source_text: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(filename).unwrap();
    let ParserReturn { trivias, program, .. } =
        Parser::new(&allocator, source_text, source_type).parse();

    let transform_options = TransformOptions::default();
    let program = allocator.alloc(program);
    Transformer::new(
        &allocator,
        Path::new(filename),
        source_type,
        source_text,
        &trivias,
        transform_options,
    )
    .build(program)
    .unwrap();

    let codegen_options = CodegenOptions::default();
    let transformed_code =
        Codegen::<false>::new("", source_text, codegen_options.clone()).build(program).source_text;
    println!("{transformed_code}");
}
