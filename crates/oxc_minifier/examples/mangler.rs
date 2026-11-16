#![expect(clippy::print_stdout)]
//! # Variable Name Mangling Example
//!
//! This example demonstrates variable name mangling using the Oxc mangler.
//! It shortens variable names to reduce file size.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_minifier --example mangler [filename] [options]
//! ```
//!
//! ## Options
//!
//! - `--keep-names`: Preserve function and class names
//! - `--reserve-exports`: Reserve `exports` / `module` binding names (cjs-module-lexer)
//! - `--debug`: Enable debug output
//! - `--twice`: Test idempotency by running twice

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_parser::Parser;
use oxc_span::{SourceCode, SourceCodeBuf, SourceType};
use pico_args::Arguments;
use rustc_hash::FxHashSet;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example mangler`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let keep_names = args.contains("--keep-names");
    let reserve_exports = args.contains("--reserve-exports");
    let debug = args.contains("--debug");
    let twice = args.contains("--twice");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    // Read the file straight into a buffer which ends with the padding the parser reads,
    // so the parser reads the source text in place
    let source = SourceCodeBuf::read(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let options = MangleOptions {
        top_level: None,
        keep_names: MangleOptionsKeepNames { function: keep_names, class: keep_names },
        reserved: if reserve_exports {
            ["exports", "module"].into_iter().map(Into::into).collect()
        } else {
            FxHashSet::default()
        },
        debug,
    };
    let printed = mangler(source.as_source_code(), source_type, options.clone());
    println!("{printed}");

    if twice {
        // Add the padding the parser reads to the printed code, so it is parsed in place too
        let printed = SourceCodeBuf::from_string(printed);
        let printed2 = mangler(printed.as_source_code(), source_type, options);
        println!("{printed2}");
        println!("same = {}", printed.as_str() == printed2);
    }

    Ok(())
}

fn mangler(source: SourceCode<'_>, source_type: SourceType, options: MangleOptions) -> String {
    let allocator = Allocator::default();
    let ret = Parser::from_source(&allocator, source, source_type).parse();
    assert!(ret.diagnostics.is_empty());
    let mangler_return = Mangler::new().with_options(options).build(&ret.program);
    Codegen::new()
        .with_scoping(Some(mangler_return.scoping))
        .with_private_member_mappings(Some(mangler_return.class_private_mappings))
        .build(&ret.program)
        .code
}
