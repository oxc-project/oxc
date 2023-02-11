use std::hint::black_box; // See: `https://rust-lang.github.io/rfcs/2360-bench-black-box.html`
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput};
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_benchmark::get_code;
use oxc_parser::Parser;
use pico_args::Arguments;

/// # Panics
pub fn main() {
    let mut args = Arguments::from_env();
    let baseline: Option<String> = args.opt_value_from_str("--save-baseline").unwrap();

    let mut criterion = Criterion::default().without_plots().measurement_time(Duration::new(20, 0));

    if let Some(ref baseline) = baseline {
        criterion = criterion.save_baseline(baseline.to_string());
    }

    let codes =
        include_str!("./libs.txt").lines().map(|lib| get_code(lib).unwrap()).collect::<Vec<_>>();

    // Bench Parser
    let mut group = criterion.benchmark_group("parser");
    for (id, code) in &codes {
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(id), &code, |b, code| {
            let allocator = Allocator::default();
            b.iter(|| {
                let _drop = Parser::new(&allocator, black_box(code), SourceType::default()).parse();
            });
        });
    }

    group.finish();
}
