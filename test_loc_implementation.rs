use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::Program};
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_span::{SourceType, Span};

fn main() {
    // Test loc functionality with actual line breaks and Unicode
    let source = "hello\nworld;\n🤨 = 123;";
    println!("Source text: {:?}", source);
    println!("Source length: {} bytes", source.len());
    
    let allocator = Allocator::new();
    let ast = AstBuilder::new(&allocator);
    
    // Create a simple program AST
    let program = ast.program(
        Span::new(0, source.len() as u32),
        SourceType::default(),
        source,
        ast.vec(),
        None,
        ast.vec(),
        ast.vec1(
            // Simple expression statement: "🤨 = 123;"
            ast.statement_expression(
                Span::new(13, source.len() as u32), // Position after "hello\nworld;\n"
                ast.expression_assignment(
                    Span::new(13, source.len() as u32),
                    oxc_ast::ast::AssignmentOperator::Assign,
                    ast.assignment_target_identifier_reference(
                        Span::new(13, 17), // Unicode char span
                        "🤨"
                    ),
                    ast.expression_numeric_literal(
                        Span::new(20, 23), // "123" span  
                        123.0,
                        None,
                        oxc_syntax::number::NumberBase::Decimal
                    ),
                )
            )
        )
    );
    
    println!("\n=== Testing regular serialization ===");
    let regular_json = program.to_estree_ts_json_with_loc(false, true);
    println!("Regular JSON with loc: {}", regular_json);
    
    println!("\n=== Testing with translation table ===");
    let table = Utf8ToUtf16::new_with_lines(source, true);
    
    // Test offset to line column conversion
    println!("Offset 0 -> {:?}", table.offset_to_line_column(0)); // start of "hello"
    println!("Offset 6 -> {:?}", table.offset_to_line_column(6)); // start of "world"  
    println!("Offset 13 -> {:?}", table.offset_to_line_column(13)); // start of "🤨"
    println!("Offset 17 -> {:?}", table.offset_to_line_column(17)); // after "🤨"
    
    let table_json = program.to_estree_ts_json_with_table(false, &table);
    println!("JSON with table: {}", table_json);
    
    println!("\n=== Comparing outputs ===");
    if regular_json != table_json {
        println!("✅ Different outputs - loc translation working!");
    } else {
        println!("❌ Same outputs - loc translation not working");
    }
}