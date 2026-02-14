#![expect(clippy::print_stdout)]

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{
    FormatOptions, Formatter, SortImportsOptions, SortOrder, default_groups,
    default_internal_patterns, get_parse_options,
};
use oxc_parser::Parser;
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
    let newlines_between = !args.contains("--no_newlines_between");

    let sort_imports_options = SortImportsOptions {
        order,
        partition_by_newline,
        partition_by_comment,
        sort_side_effects,
        ignore_case,
        newlines_between,
        internal_pattern: default_internal_patterns(),
        groups: default_groups(),
        custom_groups: vec![],
        newline_boundary_overrides: vec![],
    };

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::new();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(get_parse_options())
        .parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    // Format the parsed code
    let options = FormatOptions {
        experimental_sort_imports: Some(sort_imports_options.clone()),
        ..Default::default()
    };

    let formatter = Formatter::new(&allocator, options);
    let formatted = formatter.format(&ret.program);
    if show_ir {
        // Do not rely on `Display` of `Document` here, as it shows Prettier IR
        println!("[");
        for el in formatted.document().iter() {
            println!("  {el:?},");
        }
        println!("]");
    } else {
        let code = formatted.print().map_err(|e| e.to_string())?.into_code();
        println!("{code}");
    }

    println!("=======================");
    println!("Formatted with {sort_imports_options:#?}",);

    Ok(())
}
