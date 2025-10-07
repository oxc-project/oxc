#![expect(clippy::print_stdout)]

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter, SortImports, SortOrder};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

/// Format a JavaScript or TypeScript file
fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let show_ir = args.contains("--ir");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let partition_by_newline = args.contains("--partition_by_newline");
    let partition_by_comment = args.contains("--partition_by_comment");
    let sort_side_effects = args.contains("--sort_side_effects");
    let order = args.opt_value_from_str("--order").unwrap_or(None).unwrap_or(SortOrder::Asc);
    let ignore_case = !args.contains("--no_ignore_case");

    let sort_imports_options = SortImports {
        order,
        partition_by_newline,
        partition_by_comment,
        sort_side_effects,
        ignore_case,
    };

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::new();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: false,
            // Enable all syntax features
            allow_v8_intrinsics: true,
            allow_return_outside_function: true,
            // `oxc_formatter` expects this to be false
            preserve_parens: false,
        })
        .parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    // Format the parsed code
    let options = FormatOptions {
        experimental_sort_imports: Some(sort_imports_options),
        ..Default::default()
    };

    let formatter = Formatter::new(&allocator, options);
    if show_ir {
        let doc = formatter.doc(&ret.program);
        println!("[");
        for el in doc.iter() {
            println!("  {el:?},");
        }
        println!("]");
    } else {
        let code = formatter.build(&ret.program);
        println!("{code}");
    }

    println!("=======================");
    println!("Formatted with {sort_imports_options:#?}",);

    Ok(())
}
