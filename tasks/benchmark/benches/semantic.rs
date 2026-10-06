use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

fn bench_semantic(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("semantic");
    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Create `Allocator` outside of `bench_function`, so same allocator is used for
        // both the warmup and measurement phases
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration
                allocator.reset();

                // Create fresh AST for each iteration, as `SemanticBuilder` alters the AST
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = black_box(program);

                runner.run(|| {
                    // We drop `Semantic` inside this closure as drop time is part of cost of using this API.
                    // We return `errors` to be dropped outside of the measured section, as usually
                    // real-world code has no errors, so allocating and dropping them is atypical and
                    // we don't want to include it in benchmark time.
                    let ret = SemanticBuilder::new_compiler().build(&program);
                    let ret = black_box(ret);
                    ret.diagnostics
                });
            });
        });
    }
    group.finish();
}

#[cfg(feature = "comment_assignment")]
fn bench_comment_input(
    criterion: &mut Criterion,
    name: &str,
    source: &str,
    source_type: oxc_span::SourceType,
) {
    use oxc_comment_assignment::CommentAssignment;

    let mut group = criterion.benchmark_group(format!("semantic_comments/{name}"));
    let mut allocator = Allocator::default();
    for mode in ["semantic_only", "standalone_then_semantic", "fused"] {
        group.bench_function(mode, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let parsed = Parser::new(&allocator, source, source_type).parse();
                assert!(parsed.diagnostics.is_empty(), "{name}: {:?}", parsed.diagnostics);
                let mut program = black_box(parsed.program);
                runner.run(|| {
                    let builder = SemanticBuilder::new_compiler();
                    let result = match mode {
                        "semantic_only" => builder.build(&program),
                        "standalone_then_semantic" => {
                            CommentAssignment::new().assign(&mut program);
                            builder.build(&program)
                        }
                        "fused" => builder.build_with_comments(&mut program),
                        _ => unreachable!(),
                    };
                    // Include Semantic destruction, matching the existing group.
                    black_box(result).diagnostics
                });
            });
        });
    }
    group.finish();
}

#[cfg(feature = "comment_assignment")]
fn bench_semantic_comments(criterion: &mut Criterion) {
    use oxc_span::SourceType;

    let statement = "value = left + right;\n";
    let mut sparse = String::new();
    for index in 0..1024 {
        if index % 128 == 0 {
            sparse.push_str("/* statement */\n");
        }
        sparse.push_str(statement);
    }
    for (name, source) in [
        ("comment_free", statement.repeat(1024)),
        ("sparse", sparse),
        (
            "dense",
            "/* statement */ value = /* left */ left + /* right */ right; // tail\n".repeat(1024),
        ),
    ] {
        bench_comment_input(criterion, name, &source, SourceType::mjs());
    }
    for file in TestFiles::minimal().files() {
        bench_comment_input(criterion, &file.file_name, &file.source_text, file.source_type);
    }
}

#[cfg(feature = "comment_assignment")]
criterion_group!(semantic, bench_semantic, bench_semantic_comments);
#[cfg(not(feature = "comment_assignment"))]
criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
