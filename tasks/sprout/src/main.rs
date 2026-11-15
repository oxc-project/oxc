//! JSONL bridge between the Rust compiler and the JavaScript Sprout evaluator.

use std::io::{self, BufRead, Write};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::{CompileResult, compile};
use oxc_semantic::SemanticBuilder;
use serde::{Deserialize, Serialize};

#[path = "../../../crates/oxc_react_compiler/tests/common/mod.rs"]
mod fixture;

#[derive(Deserialize, Serialize)]
struct Input {
    path: String,
    source: String,
}

#[derive(Serialize)]
struct Output {
    #[serde(flatten)]
    input: Input,
    code: Option<String>,
    diagnostics: Vec<String>,
}

fn transform(input: Input) -> Output {
    let allocator = Allocator::default();
    let (source_type, options) = fixture::parse_pragma(&input.source);
    let parsed = Parser::new(&allocator, &input.source, source_type).parse();
    let mut program = parsed.program;
    let mut diagnostics = parsed.diagnostics;
    let mut code = None;
    if diagnostics.is_empty() {
        let result = {
            let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
            compile(&program, &semantic, &allocator, options)
        };
        match result {
            CompileResult::Success { output, diagnostics: compiler_diagnostics } => {
                diagnostics.extend(compiler_diagnostics);
                if let Some(output) = output {
                    output.transform(&mut program);
                }
                code = Some(Codegen::new().build(&program).code);
            }
            CompileResult::Fatal { diagnostics: compiler_diagnostics } => {
                diagnostics.extend(compiler_diagnostics);
            }
        }
    }
    Output {
        input,
        code,
        diagnostics: diagnostics.iter().map(|diagnostic| format!("{diagnostic:?}")).collect(),
    }
}

fn run() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for line in io::stdin().lock().lines() {
        let input = serde_json::from_str(&line?)?;
        serde_json::to_writer(&mut stdout, &transform(input))?;
        writeln!(stdout)?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(run)?.join().unwrap()
}
