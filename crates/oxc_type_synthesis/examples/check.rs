use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_type_synthesis::synthesise_program;

const PRELUDE: &str = "
type StringOrNumber = string | number;

interface Operators {
    Add<T extends StringOrNumber, U extends StringOrNumber>(a: T, b: U): (T extends string ? string : U extends string ? string: number) & Ezno.ConstantFunction<'add'>;

    Mul(a: number, b: number): number & Ezno.ConstantFunction<'mul'>;

    StrictEqual(a: any, b: any): boolean & Ezno.ConstantFunction<'equal'>;
}

interface Math {
    sin(x: number): number & Ezno.ConstantFunction<'sin'>;
}

interface string {
    toUppercase(): string & Ezno.ConstantFunction<'uppercase'>
}

interface Console {
    log(msg: any): void;
}

declare var Math: Math;
declare var console: Console;
";

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "examples/demo.ts".to_string());
    let path = Path::new(&name);
    let source_text = PRELUDE.to_owned()
        + &std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if ret.errors.is_empty() {
        println!("Program parsed");
        // println!("{}", serde_json::to_string_pretty(&ret.program).unwrap());

        let (diagnostics, events, types, ..) =
            synthesise_program(&ret.program, |_: &std::path::Path| None);

        let args: Vec<_> = env::args().collect();

        if args.iter().any(|arg| arg == "--types") {
            eprintln!("Types:");
            for item in types.into_vec_temp() {
                eprintln!("\t{item:?}");
            }
        }
        if args.iter().any(|arg| arg == "--events") {
            eprintln!("Events:");
            for item in events {
                eprintln!("\t{item:?}");
            }
        }

        eprintln!("Diagnostics:");
        for diag in diagnostics.into_iter() {
            eprintln!("\t{}", diag.reason());
        }
    } else {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
    }
}
