//! Example: Convert ESTree JSON to oxc AST
//!
//! This example demonstrates the end-to-end conversion from ESTree AST (JSON format)
//! to oxc AST. This is useful for integrating custom ESLint parsers with oxc.
//!
//! Usage:
//! ```bash
//! cargo run -p oxc_linter --example estree_converter
//! ```

use oxc_allocator::Allocator;
use oxc_linter::estree_converter::convert_estree_json_to_oxc_program;

fn main() {
    let allocator = Allocator::default();
    let source_text = r#"
const x = 42;
const y = "hello";
let z = true;
"#;

    // Example ESTree JSON (as would be produced by a custom ESLint parser)
    let estree_json = r#"
{
    "type": "Program",
    "body": [
        {
            "type": "VariableDeclaration",
            "kind": "const",
            "declarations": [
                {
                    "type": "VariableDeclarator",
                    "id": {
                        "type": "Identifier",
                        "name": "x",
                        "range": [7, 8]
                    },
                    "init": {
                        "type": "Literal",
                        "value": 42,
                        "raw": "42",
                        "range": [11, 13]
                    },
                    "range": [7, 13]
                }
            ],
            "range": [1, 14]
        },
        {
            "type": "VariableDeclaration",
            "kind": "const",
            "declarations": [
                {
                    "type": "VariableDeclarator",
                    "id": {
                        "type": "Identifier",
                        "name": "y",
                        "range": [20, 21]
                    },
                    "init": {
                        "type": "Literal",
                        "value": "hello",
                        "raw": "\"hello\"",
                        "range": [24, 31]
                    },
                    "range": [20, 31]
                }
            ],
            "range": [15, 32]
        },
        {
            "type": "VariableDeclaration",
            "kind": "let",
            "declarations": [
                {
                    "type": "VariableDeclarator",
                    "id": {
                        "type": "Identifier",
                        "name": "z",
                        "range": [37, 38]
                    },
                    "init": {
                        "type": "Literal",
                        "value": true,
                        "raw": "true",
                        "range": [41, 45]
                    },
                    "range": [37, 45]
                }
            ],
            "range": [33, 46]
        }
    ],
    "range": [0, 46]
}
"#;

    println!("Converting ESTree JSON to oxc AST...");
    println!("Source text: {}", source_text);

    match convert_estree_json_to_oxc_program(estree_json, source_text, &allocator) {
        Ok(program) => {
            println!("\n✅ Conversion successful!");
            println!("Program has {} statement(s)", program.body.len());
            
            use oxc_ast::ast::Statement;
            for (i, stmt) in program.body.iter().enumerate() {
                match stmt {
                    Statement::VariableDeclaration(var_decl) => {
                        println!("\nStatement {}: VariableDeclaration", i + 1);
                        println!("  Kind: {:?}", var_decl.kind);
                        println!("  Declarations: {}", var_decl.declarations.len());
                        
                        for (j, decl) in var_decl.declarations.iter().enumerate() {
                            use oxc_ast::ast::BindingPatternKind;
                            match &decl.id.kind {
                                BindingPatternKind::BindingIdentifier(binding_id) => {
                                    println!("    Declarator {}: {}", j + 1, binding_id.name.as_str());
                                }
                                _ => println!("    Declarator {}: (pattern)", j + 1),
                            }
                            
                            if let Some(init) = &decl.init {
                                use oxc_ast::ast::Expression;
                                match init {
                                    Expression::NumericLiteral(n) => {
                                        println!("      Init: {} (number)", n.value);
                                    }
                                    Expression::StringLiteral(s) => {
                                        println!("      Init: \"{}\" (string)", s.value.as_str());
                                    }
                                    Expression::BooleanLiteral(b) => {
                                        println!("      Init: {} (boolean)", b.value);
                                    }
                                    _ => println!("      Init: (expression)"),
                                }
                            }
                        }
                    }
                    _ => println!("\nStatement {}: {:?}", i + 1, stmt),
                }
            }
        }
        Err(e) => {
            println!("\n❌ Conversion failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
