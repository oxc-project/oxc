//! Parse files in parallel and then `Send` them to the main thread for processing.

#![allow(clippy::future_not_send)] // clippy warns `Allocator` is not `Send`
#![allow(clippy::redundant_pub_crate)] // comes from  `ouroboros`'s macro

// Instruction:
// run `cargo run -p oxc_parser --example multi-thread`
// or `cargo watch -x "run -p oxc_parser --example multi-thread"`

use std::sync::mpsc;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, thread};

/// Wrap the AST for unsafe `Send` and `Sync`
struct BumpaloProgram<'a>(Program<'a>);

#[allow(clippy::non_send_fields_in_send_ty)]
// SAFETY: It is now our responsibility to never simultaneously mutate the AST across threads.
unsafe impl<'a> Send for BumpaloProgram<'a> {}
// SAFETY: It is now our responsibility to never simultaneously mutate the AST across threads.
unsafe impl<'a> Sync for BumpaloProgram<'a> {}

/// `ouroboros` is used to "bind" the allocator and AST together to remove the lifetime.
#[ouroboros::self_referencing]
struct AST {
    index: usize,
    allocator: Allocator,
    source_text: Arc<str>,
    #[borrows(allocator, source_text)]
    #[not_covariant]
    ast: &'this BumpaloProgram<'this>,
}

/// Example output:
/// ```
/// sent ast(0) in ThreadId(2) at 1691652865800
/// sent ast(1) in ThreadId(3) at 1691652865801
/// sent ast(2) in ThreadId(4) at 1691652865801
/// received ast(0) in ThreadId(1) at 1691652865801
/// received ast(1) in ThreadId(1) at 1691652865801
/// received ast(2) in ThreadId(1) at 1691652865801
/// ```
fn main() {
    let (ast_tx, ast_rx) = mpsc::channel::<AST>();
    let sources = (0..3).map(|i| Arc::from(format!("const a = {i};"))).collect::<Vec<_>>();

    // Construct AST from different threads
    for (index, source_text) in sources.iter().enumerate() {
        let ast_tx = ast_tx.clone();
        let source_text = Arc::clone(source_text);

        _ = thread::spawn(move || {
            let ast = ASTBuilder {
                index,
                allocator: Allocator::default(),
                source_text,
                ast_builder: |allocator, source_text| {
                    let ret = Parser::new(allocator, source_text, SourceType::default()).parse();
                    allocator.alloc(BumpaloProgram(ret.program))
                },
            }
            .build();

            ast_tx.send(ast).unwrap();
            println!("sent ast({index}) in {:?} at {}", thread::current().id(), timestamp());
        })
        .join();
    }

    // Collect all ASTs on the main thread
    for _ in 0..sources.len() {
        let ast = ast_rx.recv().unwrap();
        let index = ast.borrow_index();
        println!("received ast({index}) in {:?} at {}", thread::current().id(), timestamp());
        ast.with_ast(|bumpalo_program| {
            println!("AST span: {:?}", bumpalo_program.0.span);
        });
    }
}

fn timestamp() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
