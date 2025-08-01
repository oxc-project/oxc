#![expect(clippy::print_stdout)]
//! # Control Flow Graph (CFG) Example
//!
//! This example demonstrates how to build and visualize control flow graphs from JavaScript code
//! using Oxc's semantic analysis. Control flow graphs are essential for program analysis,
//! optimization, and understanding code execution paths.
//!
//! ## Features
//!
//! - Parse JavaScript/TypeScript source code into AST
//! - Perform semantic analysis with CFG construction
//! - Generate multiple output formats for analysis:
//!   - AST dump for understanding program structure
//!   - CFG basic blocks with contained AST nodes
//!   - Graphviz DOT diagram for visualization
//! - Handle both reachable and unreachable code paths
//! - Comprehensive error reporting
//!
//! ## Control Flow Graphs
//!
//! A control flow graph represents the flow of execution through a program:
//! - **Basic Blocks**: Sequences of statements with single entry/exit points
//! - **Edges**: Represent possible control flow between blocks
//! - **Edge Types**: Normal flow, conditional branches, unreachable paths
//! - **Analysis**: Enables dead code detection, optimization opportunities
//!
//! ## Generated Files
//!
//! The example creates three output files:
//! 1. `test.ast.txt` - Complete AST structure dump
//! 2. `test.cfg.txt` - Basic blocks with their AST nodes
//! 3. `test.dot` - Graphviz diagram for visualization
//!
//! ## Visualization
//!
//! To visualize the generated DOT file:
//! ```bash
//! # Install Graphviz (if not already installed)
//! # Ubuntu/Debian: sudo apt install graphviz
//! # macOS: brew install graphviz
//! # Windows: choco install graphviz
//!
//! # Generate visualization
//! dot -Tpng test.dot -o cfg.png
//! dot -Tsvg test.dot -o cfg.svg
//! ```
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_semantic --example cfg [filename]
//!    ```
//!    Or with cargo watch:
//!    ```bash
//!    just watch "run -p oxc_semantic --example cfg"
//!    ```

use std::{env, path::Path, sync::Arc};

use itertools::Itertools;
use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_cfg::{
    DisplayDot, EdgeType,
    graph::{
        dot::{Config, Dot},
        visit::EdgeRef,
    },
};
use oxc_parser::Parser;
use oxc_semantic::{SemanticBuilder, dot::DebugDot};
use oxc_span::SourceType;

/// Main entry point for the control flow graph example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let test_file_name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());

    // Generate output file names based on input file
    let base_name =
        Path::new(&test_file_name).file_stem().and_then(|s| s.to_str()).unwrap_or("test");

    let ast_file_name = format!("{}.ast.txt", base_name);
    let cfg_file_name = format!("{}.cfg.txt", base_name);
    let dot_file_name = format!("{}.dot", base_name);

    println!("Oxc Control Flow Graph Example");
    println!("==============================");
    println!("Input file: {}", test_file_name);
    println!("Output files:");
    println!("  • AST dump: {}", ast_file_name);
    println!("  • CFG blocks: {}", cfg_file_name);
    println!("  • DOT diagram: {}", dot_file_name);
    println!();

    // Read and validate the source file
    let test_file_path = Path::new(&test_file_name);
    let source_text = match std::fs::read_to_string(test_file_path) {
        Ok(content) => Arc::new(content),
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", test_file_name, e);
            eprintln!("💡 Make sure the file exists and is readable");
            return Err(e);
        }
    };

    let source_type = match SourceType::from_path(test_file_path) {
        Ok(st) => st,
        Err(e) => {
            eprintln!("❌ Error determining source type: {}", e);
            return Ok(());
        }
    };

    println!("Source type: {:?}", source_type);
    println!("Source length: {} bytes", source_text.len());
    println!();

    println!("Source code:");
    println!("{}", "─".repeat(60));
    println!("{}", source_text.as_ref());
    println!("{}", "─".repeat(60));
    println!();

    // Parse the source code
    println!("🔍 Parsing source code...");
    let allocator = Allocator::default();
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parsing errors
    if !parser_ret.errors.is_empty() {
        let error_message: String = parser_ret
            .errors
            .into_iter()
            .map(|error| error.with_source_code(Arc::clone(&source_text)).to_string())
            .join("\n\n");

        println!("❌ Parsing failed with {} error(s):", parser_ret.errors.len());
        println!();
        println!("{}", error_message);

        eprintln!("💡 Fix the parsing errors above and try again");
        return Ok(());
    }

    println!("✅ Parsing successful!");

    // Write AST to file
    let program = parser_ret.program;
    let ast_content = format!("{:#?}", &program);

    match std::fs::write(&ast_file_name, &ast_content) {
        Ok(_) => println!("📄 AST written to: {}", ast_file_name),
        Err(e) => eprintln!("⚠️  Failed to write AST file: {}", e),
    };

    // Perform semantic analysis with CFG construction
    println!("🧠 Performing semantic analysis with CFG construction...");
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_cfg(true) // Enable control flow graph construction
        .build(&program);

    // Handle semantic analysis errors
    if !semantic.errors.is_empty() {
        let error_message: String = semantic
            .errors
            .into_iter()
            .map(|error| error.with_source_code(Arc::clone(&source_text)).to_string())
            .join("\n\n");

        println!("❌ Semantic analysis failed with {} error(s):", semantic.errors.len());
        println!();
        println!("{}", error_message);

        eprintln!("💡 Fix the semantic errors above and try again");
        return Ok(());
    }

    println!("✅ Semantic analysis completed!");

    // Extract the control flow graph
    let cfg = semantic
        .semantic
        .cfg()
        .expect("CFG should be available since we enabled it with .with_cfg(true)");

    println!("📊 CFG Statistics:");
    println!("   • Basic blocks: {}", cfg.basic_blocks.len());
    println!("   • Graph nodes: {}", cfg.graph.node_count());
    println!("   • Graph edges: {}", cfg.graph.edge_count());

    // Group AST nodes by their basic block
    let mut ast_nodes_by_block = FxHashMap::<_, Vec<_>>::default();
    for node in semantic.semantic.nodes() {
        let cfg_id = node.cfg_id();
        let block_index = cfg.graph.node_weight(cfg_id).unwrap();
        ast_nodes_by_block.entry(*block_index).or_default().push(node);
    }

    // Generate basic blocks text representation
    println!();
    println!("📝 Generating CFG basic blocks representation...");

    let basic_blocks_content = cfg
        .basic_blocks
        .iter_enumerated()
        .map(|(block_index, basic_block)| {
            let block_display = basic_block.display_dot();
            let block_instructions =
                block_display.lines().map(|line| format!("    {}", line.trim())).join("\n");

            let ast_nodes_info = ast_nodes_by_block
                .get(&block_index)
                .map(|nodes| {
                    let node_names: Vec<String> =
                        nodes.iter().map(|node| node.kind().debug_name().to_string()).collect();
                    if node_names.is_empty() {
                        "    (no AST nodes)".to_string()
                    } else {
                        node_names.into_iter().map(|name| format!("    📍 {}", name)).join("\n")
                    }
                })
                .unwrap_or_else(|| "    (no AST nodes)".to_string());

            format!(
                "Basic Block bb{}:\n{}\n{}\n  AST Nodes:\n{}",
                block_index, "  Instructions:", block_instructions, ast_nodes_info
            )
        })
        .join("\n\n");

    match std::fs::write(&cfg_file_name, &basic_blocks_content) {
        Ok(_) => println!("📄 CFG blocks written to: {}", cfg_file_name),
        Err(e) => eprintln!("⚠️  Failed to write CFG blocks file: {}", e),
    };

    // Generate Graphviz DOT diagram
    println!("🎨 Generating Graphviz DOT diagram...");

    let dot_content = format!(
        "{:?}",
        Dot::with_attr_getters(
            cfg.graph(),
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_graph, edge| {
                let weight = edge.weight();
                let label = format!("label = \"{:?}\"", weight);

                // Style unreachable edges with dotted lines
                if matches!(weight, EdgeType::Unreachable)
                    || cfg.basic_block(edge.source()).is_unreachable()
                {
                    format!("{}, style = \"dotted\", color = \"red\"", label)
                } else {
                    format!("{}, color = \"blue\"", label)
                }
            },
            &|_graph, node| {
                let block_index = node.1;
                let basic_block = &cfg.basic_blocks[*block_index];

                // Get AST nodes for this block
                let ast_nodes_summary =
                    ast_nodes_by_block.get(block_index).map_or("None".to_string(), |nodes| {
                        let node_names: Vec<_> =
                            nodes.iter().map(|node| node.kind().debug_name().to_string()).collect();
                        if node_names.len() > 3 {
                            format!("{} nodes", node_names.len())
                        } else if node_names.is_empty() {
                            "empty".to_string()
                        } else {
                            node_names.join(", ")
                        }
                    });

                // Create a clean block display for the DOT format
                let block_content = basic_block
                    .debug_dot(semantic.semantic.nodes().into())
                    .replace('\n', "\\l")
                    .trim()
                    .to_string();

                let color = if basic_block.is_unreachable() { "lightcoral" } else { "lightblue" };

                format!(
                    "label = \"bb{}\\l{}\\l\", xlabel = \"AST: {}\\l\", style = \"filled\", fillcolor = \"{}\"",
                    block_index, block_content, ast_nodes_summary, color
                )
            }
        )
    );

    match std::fs::write(&dot_file_name, &dot_content) {
        Ok(_) => {
            println!("📄 DOT diagram written to: {}", dot_file_name);
            println!();
            println!("🎯 To visualize the control flow graph:");
            println!("   dot -Tpng {} -o cfg.png", dot_file_name);
            println!("   dot -Tsvg {} -o cfg.svg", dot_file_name);
        }
        Err(e) => eprintln!("⚠️  Failed to write DOT file: {}", e),
    };

    println!();
    println!("✅ Control flow analysis completed successfully!");
    println!();
    println!("💡 Understanding the output:");
    println!("   • Basic blocks are sequences of statements with single entry/exit");
    println!("   • Edges show possible control flow between blocks");
    println!("   • Unreachable blocks/edges are marked in red (dotted in DOT)");
    println!("   • AST nodes show which program elements belong to each block");

    Ok(())
}
