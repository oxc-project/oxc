use std::{collections::HashMap, env, fs, io, path::Path, sync::Arc};

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

// Instruction:
// 1. create a `test.js`,
// 2. run `cargo run -p oxc_semantic --example cfg`
//    or `just watch "run -p oxc_semantic --example cfg"`
// 3. observe visualizations of:
//    - AST (test.ast.txt)
//    - CFG blocks (test.cfg.txt)
//    - CFG graph (test.dot)

fn main() -> io::Result<()> {
    let test_file_name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let ast_file_name = env::args().nth(1).unwrap_or_else(|| "test.ast.txt".to_string());
    let cfg_file_name = env::args().nth(1).unwrap_or_else(|| "test.cfg.txt".to_string());
    let dot_file_name = env::args().nth(1).unwrap_or_else(|| "test.dot".to_string());

    let test_file_path = Path::new(&test_file_name);
    let ast_file_path = Path::new(&ast_file_name);
    let cfg_file_path = Path::new(&cfg_file_name);
    let dot_file_path = Path::new(&dot_file_name);

    let source_text = Arc::new(fs::read_to_string(test_file_path)?);
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(test_file_path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    fs::write(ast_file_path, format!("{:#?}", &program))?;
    println!("Wrote AST to: {}", &ast_file_name);

    let semantic = SemanticBuilder::new(&source_text, source_type)
        .with_check_syntax_error(true)
        .with_trivias(ret.trivias)
        .build(program);

    if !semantic.errors.is_empty() {
        let error_message: String = semantic
            .errors
            .into_iter()
            .map(|error| error.with_source_code(Arc::clone(&source_text)).to_string())
            .join("\n\n");

        println!("Semantic analysis failed:\n\n{error_message}",);
        return Ok(());
    }

    let mut ast_nodes_by_block = HashMap::<_, Vec<_>>::new();
    for node in semantic.semantic.nodes().iter() {
        let block = node.cfg_id();
        let block_ix = semantic.semantic.cfg().graph.node_weight(block).unwrap();
        ast_nodes_by_block.entry(*block_ix).or_default().push(node);
    }

    let basic_blocks_printed = semantic
        .semantic
        .cfg()
        .basic_blocks
        .iter()
        .enumerate()
        .map(|(i, basic_block)| {
            format!(
                "bb{i}: {{\n{}\n---\n{}\n}}",
                basic_block.iter().map(|el| format!("\t{el}")).join("\n"),
                ast_nodes_by_block
                    .get(&i)
                    .map(|nodes| {
                        nodes.iter().map(|node| format!("{}", node.kind().debug_name())).join("\n")
                    })
                    .unwrap_or_default()
            )
        })
        .join("\n\n");

    fs::write(cfg_file_path, basic_blocks_printed)?;
    println!("Wrote CFG blocks to: {}", &cfg_file_name);

    let cfg_dot_diagram = format!("{:?}", semantic.semantic.cfg().dot());
    fs::write(dot_file_path, cfg_dot_diagram)?;
    println!("Wrote CFG dot diagram to: {}", &dot_file_name);

    Ok(())
}
