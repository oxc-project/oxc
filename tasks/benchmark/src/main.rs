#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// See: `https://rust-lang.github.io/rfcs/2360-bench-black-box.html`
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput};
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_benchmark::Code;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use pico_args::Arguments;

/// # Errors
/// # Panics
pub fn main() -> Result<(), String> {
    let root = project_root::get_project_root().unwrap();
    let codes = std::fs::read_to_string(root.join("./tasks/benchmark/libs.txt"))
        .unwrap()
        .lines()
        .map(|file| Code::new(10, file).unwrap())
        .collect::<Vec<_>>();

    let mut args = Arguments::from_env();

    let baseline: Option<String> = args.opt_value_from_str("--save-baseline").unwrap();

    let mut criterion = Criterion::default().without_plots();
    if let Some(ref baseline) = baseline {
        criterion = criterion.save_baseline(baseline.to_string());
    }

    // Check files
    for code in &codes {
        let allocator = Allocator::default();
        let ret =
            Parser::new(&allocator, black_box(&code.source_text), SourceType::default()).parse();
        if !ret.errors.is_empty() {
            println!("{} failed", &code.file_name);
            for error in &ret.errors {
                println!("{error:?}");
            }
            return Err("Parse Failed.".to_string());
        }
    }

    bench_parser(&mut criterion, &codes);
    bench_semantic(&mut criterion, &codes);
    drop(criterion);

    Ok(())
}

fn bench_parser(criterion: &mut Criterion, codes: &[Code]) {
    let mut group = criterion.benchmark_group("parser");
    for code in codes {
        group.throughput(Throughput::Bytes(code.source_text.len() as u64));
        group.measurement_time(code.measurement_time);
        group.bench_with_input(
            BenchmarkId::from_parameter(&code.file_name),
            &code.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                b.iter(|| {
                    let _drop =
                        Parser::new(&allocator, black_box(source_text), SourceType::default())
                            .parse();
                });
            },
        );
    }
    group.finish();
}

fn bench_semantic(criterion: &mut Criterion, codes: &[Code]) {
    let mut group = criterion.benchmark_group("semantic");
    for code in codes {
        group.throughput(Throughput::Bytes(code.source_text.len() as u64));
        group.measurement_time(code.measurement_time);
        group.bench_with_input(
            BenchmarkId::from_parameter(&code.file_name),
            &code.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                let source_type = SourceType::from_path(&code.file_name).unwrap();
                let ret = Parser::new(&allocator, source_text, source_type).parse();
                let program = allocator.alloc(ret.program);
                b.iter(|| {
                    let _semantic = SemanticBuilder::new(source_text, source_type, &ret.trivias)
                        .build(black_box(program));
                });
            },
        );
    }
    group.finish();
}
