#![allow(clippy::print_stdout)]
use std::{env, path::Path, sync::Arc};

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_cfg::{
    graph::{
        dot::{Config, Dot},
        visit::EdgeRef,
    },
    DisplayDot, EdgeType,
};
use oxc_parser::Parser;
use oxc_semantic::{dot::DebugDot, SemanticBuilder};
use oxc_span::SourceType;
use rustc_hash::FxHashMap;

// Instruction:
// 1. create a `test.js`,
// 2. run `cargo run -p oxc_semantic --example cfg`
//    or `just watch "run -p oxc_semantic --example cfg"`
// 3. observe visualizations of:
//    - AST (test.ast.txt)
//    - CFG blocks (test.cfg.txt)
//    - CFG graph (test.dot)

fn main() -> std::io::Result<()> {
    let test_file_name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let ast_file_name = env::args().nth(1).unwrap_or_else(|| "test.ast.txt".to_string());
    let cfg_file_name = env::args().nth(1).unwrap_or_else(|| "test.cfg.txt".to_string());
    let dot_file_name = env::args().nth(1).unwrap_or_else(|| "test.dot".to_string());

    let test_file_path = Path::new(&test_file_name);
    let ast_file_path = Path::new(&ast_file_name);
    let cfg_file_path = Path::new(&cfg_file_name);
    let dot_file_path = Path::new(&dot_file_name);

    let source_text = Arc::new(std::fs::read_to_string(test_file_path)?);
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(test_file_path).unwrap();
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !parser_ret.errors.is_empty() {
        let error_message: String = parser_ret
            .errors
            .into_iter()
            .map(|error| error.with_source_code(Arc::clone(&source_text)).to_string())
            .join("\n\n");

        println!("Parsing failed:\n\n{error_message}",);
        return Ok(());
    }

    let program = allocator.alloc(parser_ret.program);
    std::fs::write(ast_file_path, format!("{:#?}", &program))?;
    println!("Wrote AST to: {}", &ast_file_name);

    let semantic = SemanticBuilder::new(&source_text)
        .with_check_syntax_error(true)
        .with_trivias(parser_ret.trivias)
        .with_cfg(true)
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

    let cfg = semantic
        .semantic
        .cfg()
        .expect("we set semantic to build the control flow (`with_cfg`) for us so it should always be `Some`");

    let mut ast_nodes_by_block = FxHashMap::<_, Vec<_>>::default();
    for node in semantic.semantic.nodes() {
        let block = node.cfg_id();
        let block_ix = cfg.graph.node_weight(block).unwrap();
        ast_nodes_by_block.entry(*block_ix).or_default().push(node);
    }

    let basic_blocks_printed = cfg
        .basic_blocks
        .iter_enumerated()
        .map(|(i, it)| {
            let it = it.display_dot();
            format!(
                "bb{i}: {{\n{}\n---\n{}\n}}",
                it.lines().map(|x| format!("\t{}", x.trim())).join("\n"),
                ast_nodes_by_block
                    .get(&i)
                    .map(|nodes| {
                        nodes.iter().map(|node| format!("{}", node.kind().debug_name())).join("\n")
                    })
                    .unwrap_or_default()
            )
        })
        .join("\n\n");

    std::fs::write(cfg_file_path, basic_blocks_printed)?;
    println!("Wrote CFG blocks to: {}", &cfg_file_name);

    let cfg_dot_diagram = format!(
        "{:?}",
        Dot::with_attr_getters(
            cfg.graph(),
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_graph, edge| {
                let weight = edge.weight();
                let label = format!("label = \"{weight:?}\"");
                if matches!(weight, EdgeType::Unreachable)
                    || cfg.basic_block(edge.source()).is_unreachable()
                {
                    format!("{label}, style = \"dotted\" ")
                } else {
                    label
                }
            },
            &|_graph, node| {
                let nodes = ast_nodes_by_block.get(node.1).map_or("None".to_string(), |nodes| {
                    let nodes: Vec<_> =
                        nodes.iter().map(|node| format!("{}", node.kind().debug_name())).collect();
                    if nodes.len() > 1 {
                        format!(
                            "{}\\l",
                            nodes.into_iter().map(|it| format!("\\l    {it}")).join("")
                        )
                    } else {
                        nodes.into_iter().join("")
                    }
                });
                format!(
                    "xlabel = \"nodes{} [{}]\\l\", label = \"bb{}\n{}\"",
                    node.1,
                    nodes,
                    node.1,
                    cfg.basic_blocks[*node.1].debug_dot(semantic.semantic.nodes().into()).trim()
                )
            }
        )
    );
    std::fs::write(dot_file_path, cfg_dot_diagram)?;
    println!("Wrote CFG dot diagram to: {}", &dot_file_name);

    Ok(())
}
