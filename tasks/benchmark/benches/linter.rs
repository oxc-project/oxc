use std::{env, path::Path, rc::Rc};

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_linter::{FixKind, LinterBuilder};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter");

    // If `FIXTURE` env is set, only run the specified benchmark. This is used for sharding in CI.
    let test_files = if let Ok(fixture_index) = env::var("FIXTURE") {
        let fixture_index = fixture_index.parse::<usize>().unwrap();
        TestFiles::complicated_one(fixture_index)
    } else {
        TestFiles::complicated()
    };

    for file in test_files.files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                let ret = Parser::new(&allocator, source_text, source_type).parse();
                let program = allocator.alloc(ret.program);
                let semantic_ret = SemanticBuilder::new()
                    .with_build_jsdoc(true)
                    .with_cfg(true)
                    .build_module_record(Path::new(""), program)
                    .build(program);
                let linter = LinterBuilder::all().with_fix(FixKind::All).build();
                let semantic = Rc::new(semantic_ret.semantic);
                b.iter(|| linter.run(Path::new(std::ffi::OsStr::new("")), Rc::clone(&semantic)));
            },
        );
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
