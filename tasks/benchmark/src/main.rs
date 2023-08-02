#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput};
use oxc_allocator::Allocator;
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::{TestFile, TestFiles};
use pico_args::Arguments;

/// # Errors
/// # Panics
pub fn main() -> Result<(), String> {
    let files = TestFiles::new();
    let files = files
        .files()
        .iter()
        .filter(|file| {
            ["react", "vue", "antd", "typescript"].iter().any(|f| file.file_name.contains(f))
        })
        .collect::<Vec<_>>();
    let mut args = Arguments::from_env();

    let baseline: Option<String> = args.opt_value_from_str("--save-baseline").unwrap();
    let measurement_time = Duration::new(/* seconds */ 15, 0);
    let mut criterion = Criterion::default().without_plots().measurement_time(measurement_time);
    if let Some(ref baseline) = baseline {
        criterion = criterion.save_baseline(baseline.to_string());
    }

    // Check files
    for file in &files {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &file.source_text, SourceType::default()).parse();
        if !ret.errors.is_empty() {
            println!("{} failed", &file.file_name);
            for error in &ret.errors {
                println!("{error:?}");
            }
            return Err("Parse Failed.".to_string());
        }
    }

    bench_parser(&mut criterion, &files);
    bench_semantic(&mut criterion, &files);
    bench_minifier(&mut criterion, &files);
    drop(criterion);

    Ok(())
}

fn bench_parser(criterion: &mut Criterion, files: &[&TestFile]) {
    let mut group = criterion.benchmark_group("parser");
    for file in files {
        group.throughput(Throughput::Bytes(file.source_text.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                b.iter_with_large_drop(|| {
                    // Include the allocator drop time to make time measurement consistent.
                    // Otherwise the allocator will allocate huge memory chunks (by power of two) from the
                    // system allocator, which makes time measurement unequal during long runs.
                    let allocator = Allocator::default();
                    _ = Parser::new(&allocator, source_text, SourceType::default()).parse();
                    allocator
                });
            },
        );
    }
    group.finish();
}

fn bench_minifier(criterion: &mut Criterion, files: &[&TestFile]) {
    let mut group = criterion.benchmark_group("minifier");
    for file in files {
        group.throughput(Throughput::Bytes(file.source_text.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let source_type = SourceType::from_path(&file.file_name).unwrap();
                let options = MinifierOptions::default();
                b.iter_with_large_drop(|| Minifier::new(source_text, source_type, options).build());
            },
        );
    }
    group.finish();
}

fn bench_semantic(criterion: &mut Criterion, files: &[&TestFile]) {
    let mut group = criterion.benchmark_group("semantic");
    for file in files {
        group.throughput(Throughput::Bytes(file.source_text.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let source_type = SourceType::from_path(&file.file_name).unwrap();
                let allocator = Allocator::default();
                let ret = Parser::new(&allocator, source_text, SourceType::default()).parse();
                let program = allocator.alloc(ret.program);
                b.iter_with_large_drop(|| {
                    SemanticBuilder::new(source_text, source_type)
                        .with_module_record_builder(true)
                        .build(program)
                });
            },
        );
    }
    group.finish();
}
