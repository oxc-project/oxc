#![expect(clippy::print_stdout)]
//! # Isolated Declarations Example
//!
//! This example demonstrates how to use Oxc's isolated declarations feature to generate
//! TypeScript declaration files (.d.ts) from TypeScript source code. Isolated declarations
//! allow for faster and more reliable generation of type definitions.
//!
//! ## Features
//!
//! - Parse TypeScript source code into AST
//! - Generate isolated TypeScript declarations
//! - Strip internal types and declarations when configured
//! - Preserve JSDoc comments in generated declarations
//! - Comprehensive error reporting for invalid declarations
//!
//! ## Usage
//!
//! 1. Create a TypeScript test file (e.g., `test.ts`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_isolated_declarations --example isolated_declarations [filename]
//!    ```
//!    Or with just:
//!    ```bash
//!    just example isolated_declarations
//!    ```
//!
//! ## Example Input/Output
//!
//! For a TypeScript file containing:
//! ```typescript
//! export function greet(name: string): string {
//!   return `Hello, ${name}!`;
//! }
//! ```
//!
//! This will generate the corresponding declaration:
//! ```typescript
//! export declare function greet(name: string): string;
//! ```

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Main entry point for the isolated declarations example
fn main() -> std::io::Result<()> {
    // Parse command line arguments - default to TypeScript file
    let name = env::args().nth(1).unwrap_or_else(|| "test.ts".to_string());
    let path = Path::new(&name);

    println!("Oxc Isolated Declarations Example");
    println!("=================================");
    println!("Input file: {}", name);

    // Read the source file
    let source_text = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", name, e);
            eprintln!("💡 Make sure the file exists and is readable");
            return Err(e);
        }
    };

    // Determine source type from file extension
    let source_type = match SourceType::from_path(path) {
        Ok(st) => st,
        Err(e) => {
            eprintln!("❌ Error determining source type: {}", e);
            eprintln!("💡 Isolated declarations work best with TypeScript files (.ts, .tsx)");
            return Ok(());
        }
    };

    let allocator = Allocator::default();

    println!("Source type: {:?}", source_type);
    println!("Source length: {} bytes", source_text.len());

    // Warn if not TypeScript
    if !source_type.is_typescript() {
        println!(
            "⚠️  Warning: This is not a TypeScript file. Isolated declarations work best with TypeScript."
        );
    }

    println!();

    // Parse the TypeScript source code
    println!("Parsing source code...");
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parsing errors
    if !ret.errors.is_empty() {
        println!("❌ Parsing failed with {} error(s):", ret.errors.len());
        println!();

        for (i, error) in ret.errors.iter().enumerate() {
            println!("Error #{}: ", i + 1);
            let error = error.clone().with_source_code(source_text.clone());
            println!("{:?}", error);
            println!();
        }

        eprintln!("💡 Fix the parsing errors above and try again");
        return Ok(());
    }

    println!("✅ Parsing successful!");
    println!();

    println!("Original TypeScript code:");
    println!("{}", "─".repeat(60));
    println!("{}", source_text);
    println!("{}", "─".repeat(60));
    println!();

    // Generate isolated declarations
    println!("Generating isolated declarations...");
    let id_options =
        IsolatedDeclarationsOptions { 
        strip_internal: true  // Strip @internal declarations
    };

    let id_ret = IsolatedDeclarations::new(&allocator, id_options).build(&ret.program);

    // Generate the declaration code with JSDoc comments preserved
    let codegen_options = CodegenOptions {
        comments: CommentOptions {
            jsdoc: true, // Preserve JSDoc comments in declarations
            ..CommentOptions::disabled()
        },
        ..CodegenOptions::default()
    };

    let declaration_code = Codegen::new().with_options(codegen_options).build(&id_ret.program).code;

    println!("✅ Declaration generation completed!");
    println!();
    println!("Generated TypeScript declarations ({} bytes):", declaration_code.len());
    println!("{}", "─".repeat(60));
    println!("{}", declaration_code);
    println!("{}", "─".repeat(60));

    // Report any declaration generation errors
    if !id_ret.errors.is_empty() {
        println!();
        println!(
            "⚠️  Declaration generation encountered {} warning(s)/error(s):",
            id_ret.errors.len()
        );
        println!();

        for (i, error) in id_ret.errors.iter().enumerate() {
            println!("Issue #{}: ", i + 1);
            let error = error.clone().with_source_code(source_text.clone());
            println!("{:?}", error);
            println!();
        }

        println!("💡 Some TypeScript patterns may not be suitable for isolated declarations");
        println!("   Consider refactoring the code to use more explicit type annotations");
    } else {
        println!();
        println!("🎉 No issues found during declaration generation!");
    }

    // Show size comparison
    let size_ratio = (declaration_code.len() as f64 / source_text.len() as f64) * 100.0;
    println!();
    println!("📊 Statistics:");
    println!("   Original source: {} bytes", source_text.len());
    println!(
        "   Generated declarations: {} bytes ({:.1}% of original)",
        declaration_code.len(),
        size_ratio
    );

    Ok(())
}
