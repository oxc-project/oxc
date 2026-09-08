use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_allocator::{Allocator, ArenaStringBuilder};
use oxc_str::{JSStr, JSStrBuilder, Str};

fn strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_str");
    for (name, source) in [
        ("short_ascii", String::from("__vite_ssr_import_0__")),
        ("long_ascii", "a JavaScript string value ".repeat(128)),
        ("unicode", "aé日本語😀".repeat(128)),
    ] {
        let source = source.as_str();
        group.bench_function(BenchmarkId::new("borrow_str", name), |b| {
            b.iter(|| black_box(Str::from(black_box(source))));
        });
        group.bench_function(BenchmarkId::new("borrow_js_str", name), |b| {
            b.iter(|| black_box(JSStr::from(black_box(source))));
        });

        let mut allocator = Allocator::new();
        group.bench_function(BenchmarkId::new("append_str", name), |b| {
            b.iter(|| {
                allocator.reset();
                let mut builder =
                    ArenaStringBuilder::with_capacity_in(source.len() + 2, &allocator);
                for part in black_box(["[", source, "]"]) {
                    builder.push_str(part);
                }
                black_box(builder.into_str());
            });
        });
        group.bench_function(BenchmarkId::new("append_js_str", name), |b| {
            b.iter(|| {
                allocator.reset();
                let mut builder = JSStrBuilder::with_capacity_in(source.len() + 2, &allocator);
                for part in black_box(["[", source, "]"]) {
                    builder.push_str(part);
                }
                black_box(builder.finish());
            });
        });

        let parts = [JSStr::from("["), JSStr::from(source), JSStr::from("]")];
        group.bench_function(BenchmarkId::new("append_js_values", name), |b| {
            b.iter(|| {
                allocator.reset();
                let mut builder = JSStrBuilder::with_capacity_in(source.len() + 2, &allocator);
                for part in black_box(parts) {
                    builder.push_js_str(part);
                }
                black_box(builder.finish());
            });
        });

        group.bench_function(BenchmarkId::new("chars_str", name), |b| {
            b.iter(|| black_box(source).chars().fold(0, |sum, c| sum ^ c as u32));
        });
        let value = JSStr::from(source);
        group.bench_function(BenchmarkId::new("chars_js_str", name), |b| {
            b.iter(|| black_box(value).chars().fold(0, |sum, c| sum ^ c.to_u32()));
        });
        group.bench_function(BenchmarkId::new("utf16_str", name), |b| {
            b.iter(|| black_box(source).encode_utf16().fold(0, |sum, c| sum ^ c));
        });
        group.bench_function(BenchmarkId::new("utf16_js_str", name), |b| {
            b.iter(|| black_box(value).encode_utf16().fold(0, |sum, c| sum ^ c));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("js_str_surrogates");
    let mut allocator = Allocator::new();
    let inputs = Allocator::new();
    for (name, units) in [
        ("pairs", [0xD800, 0xDC00].repeat(128)),
        ("lone", [0xD800, 0x61, 0xDC00].repeat(128)),
        ("edges", [vec![0xDC00], vec![0x61; 1024], vec![0xD800]].concat()),
    ] {
        group.bench_function(BenchmarkId::new("from_utf16", name), |b| {
            b.iter(|| {
                allocator.reset();
                black_box(JSStr::from_utf16_in(black_box(&units), &&allocator));
            });
        });
        let input = JSStr::from_utf16_in(&units, &&inputs);
        group.bench_function(BenchmarkId::new("append", name), |b| {
            b.iter(|| {
                allocator.reset();
                let mut builder = JSStrBuilder::with_capacity_in(input.len() * 2, &allocator);
                builder.push_js_str(black_box(input));
                builder.push_js_str(black_box(input));
                black_box(builder.finish());
            });
        });
        if name == "edges" {
            // Pair away both edge surrogates. The builder must establish that
            // the copied interior contains no other lone surrogates.
            group.bench_function("repair_edges", |b| {
                b.iter(|| {
                    allocator.reset();
                    let mut builder = JSStrBuilder::with_capacity_in(input.len() + 6, &allocator);
                    builder.push_code_unit(0xD800);
                    builder.push_js_str(black_box(input));
                    builder.push_code_unit(0xDC00);
                    black_box(builder.finish());
                });
            });
        }
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(500))
        .sample_size(30);
    targets = strings
}
criterion_main!(benches);
