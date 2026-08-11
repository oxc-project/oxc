#![expect(clippy::print_stdout)]

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_ast_generator::AstGenerator;
use oxc_codegen::Codegen;
use oxc_span::SourceType;
use rand::{SeedableRng, rngs::StdRng};

fn main() {
    let seed = std::env::args().nth(1).map_or(0, |seed| seed.parse().expect("invalid seed"));
    let allocator = Allocator::default();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut generator = AstGenerator::new(&allocator, &mut rng, SourceType::mjs());
    let program = generator.generate::<Program<'_>>();

    println!("{}", Codegen::new().build(&program).code);
}
