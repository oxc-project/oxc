use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_codegen(criterion: &mut Criterion) {
    for file in TestFiles::minimal().files() {
        let source_text = &file.source_text;
        let source_type = file.source_type;
        let allocator = Allocator::default();

        let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(parser_ret.diagnostics.is_empty());
        let mut program = parser_ret.program;

        let scoping =
            SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();

        let transform_options = TransformOptions::enable_all();
        let transformer_ret =
            Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                .build_with_scoping(scoping, &mut program);
        assert!(transformer_ret.diagnostics.is_empty());

        for ascii_only in [false, true] {
            for sourcemap_enabled in [false, true] {
                let name = format!(
                    "codegen{}{}",
                    if ascii_only { "_ascii_only" } else { "" },
                    if sourcemap_enabled { "" } else { "_no_sourcemap" }
                );

                let id = BenchmarkId::from_parameter(&file.file_name);
                let mut group = criterion.benchmark_group(name);
                group.bench_function(id, |b| {
                    b.iter_with_large_drop(|| {
                        Codegen::new()
                            .with_options(CodegenOptions {
                                ascii_only,
                                source_map_path: sourcemap_enabled
                                    .then(|| PathBuf::from(&file.file_name)),
                                ..CodegenOptions::default()
                            })
                            .build(&program)
                    });
                });
                group.finish();
            }
        }
    }
}

fn bench_attached_comments(criterion: &mut Criterion) {
    let mut inputs: Vec<_> = TestFiles::minimal()
        .files()
        .iter()
        .map(|file| (file.file_name.clone(), file.source_text.clone(), file.source_type))
        .collect();
    inputs.extend([
        (
            "comment_free".into(),
            "value = left + right;\n".repeat(1024),
            oxc_span::SourceType::mjs(),
        ),
        (
            "sparse".into(),
            ("value = left + right;\n".repeat(127) + "value = /* sparse */ left + right;\n")
                .repeat(8),
            oxc_span::SourceType::mjs(),
        ),
        (
            "dense".into(),
            "/* leading */ value = /* operand */ left + right; // trailing\n".repeat(1024),
            oxc_span::SourceType::mjs(),
        ),
    ]);
    for (name, source, source_type) in inputs {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, source_type).parse();
        assert!(parsed.diagnostics.is_empty(), "{name}: {:?}", parsed.diagnostics);
        let program = parsed.program;
        let mut group = criterion.benchmark_group(format!("codegen_attached/{name}"));
        for (name, comments) in [
            ("without_comments", CommentOptions::disabled()),
            ("with_comments", CommentOptions::default()),
        ] {
            group.bench_function(name, |b| {
                b.iter_with_large_drop(|| {
                    Codegen::new()
                        .with_options(CodegenOptions {
                            comments: comments.clone(),
                            ..CodegenOptions::default()
                        })
                        .build(&program)
                });
            });
        }
        group.finish();
    }
}

criterion_group!(codegen, bench_codegen, bench_attached_comments);
criterion_main!(codegen);
