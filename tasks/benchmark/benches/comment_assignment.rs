//! Compare comment assignment with a plain AST traversal.
//!
//! Parsing and node counting happen outside the measured loop. Throughput is AST
//! nodes per second; `assign` includes writing attachments into comment storage.

mod comment_inputs;

use std::hint::black_box;

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast_visit::Visit;
use oxc_benchmark::{Criterion, Throughput, criterion_group, criterion_main};
use oxc_comment_assignment::CommentAssignment;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

#[derive(Default)]
struct NodeCounter(u64);

impl<'a> Visit<'a> for NodeCounter {
    fn enter_node(&mut self, _kind: AstKind<'a>) {
        self.0 += 1;
    }
}

fn bench_input(criterion: &mut Criterion, name: &str, source: &str, source_type: SourceType) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{name}: {:?}", parsed.diagnostics);
    let mut program = parsed.program;

    let mut counter = NodeCounter::default();
    counter.visit_program(&program);
    let mut group = criterion.benchmark_group(format!("comment_assignment/{name}"));
    group.throughput(Throughput::Elements(counter.0));

    group.bench_function("traversal", |b| {
        b.iter(|| {
            let mut counter = NodeCounter::default();
            counter.visit_program(black_box(&program));
            black_box(counter.0)
        });
    });
    group.bench_function("assign", |b| {
        b.iter(|| CommentAssignment::new().assign(black_box(&mut program)));
    });
    group.finish();
}

fn bench_assignment(criterion: &mut Criterion) {
    for (name, source) in comment_inputs::inputs() {
        bench_input(criterion, name, &source, SourceType::mjs());
    }
    for file in TestFiles::minimal().files() {
        bench_input(criterion, &file.file_name, &file.source_text, file.source_type);
    }
}

criterion_group!(assignment, bench_assignment);
criterion_main!(assignment);
