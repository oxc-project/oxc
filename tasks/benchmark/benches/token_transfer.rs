use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use codspeed_criterion_compat::{BatchSize, Criterion, criterion_group, criterion_main};
use oxc_allocator::Allocator;
use oxc_benchmark as _;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::{TestFile, TestFiles};
use oxc_transformer::{TransformOptions, Transformer};

fn fingerprint(output: &CodegenReturn<'_>) -> u64 {
    let mut hash = DefaultHasher::new();
    output.code.hash(&mut hash);
    if let Some(map) = &output.map {
        for token in map.get_source_view_tokens() {
            (
                token.get_dst_line(),
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
                token.get_name(),
            )
                .hash(&mut hash);
        }
    }
    hash.finish()
}

fn bench_file(criterion: &mut Criterion, file: &TestFile, source_map: bool) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &file.source_text, file.source_type).parse();
    assert!(ret.diagnostics.is_empty());
    let mut program = ret.program;
    let scoping =
        SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();
    let options = TransformOptions::enable_all();
    let ret = Transformer::new(&allocator, Path::new(&file.file_name), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty());

    let generate = || {
        Codegen::new()
            .with_options(CodegenOptions {
                source_map_path: source_map.then(|| PathBuf::from(&file.file_name)),
                ..CodegenOptions::default()
            })
            .build(&program)
    };
    let control = if source_map { "map" } else { "no_map" };
    println!("CHECK {} {control} {:016x}", file.file_name, fingerprint(&generate()));
    let variant = std::env::var("TOKEN_VARIANT").unwrap_or_else(|_| "default".into());
    let allocator_name = if cfg!(feature = "system_allocator") { "system" } else { "never_grow" };
    let mut group = criterion.benchmark_group(format!("tokens_{allocator_name}_{variant}"));
    group.bench_function(format!("{}_{control}", file.file_name), |b| {
        // Keep the output alive until measurement ends. In memory mode, allocations
        // minus frees therefore measures retained output bytes as well as peak usage.
        b.iter_batched(|| (), |()| generate(), BatchSize::PerIteration);
    });
    if file.file_name == "lightly_mapped.js" {
        group.bench_function("released_control", |b| {
            b.iter_batched(|| (), |()| drop(generate()), BatchSize::PerIteration);
        });
    }
    group.finish();
}

fn bench_token_transfer(criterion: &mut Criterion) {
    for file in TestFiles::minimal().files() {
        bench_file(criterion, file, true);
        if file.file_name == "App.tsx" {
            bench_file(criterion, file, false);
        }
    }
    for url in [
        "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs",
        "https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js",
    ] {
        bench_file(criterion, &TestFile::new(url), true);
    }
    for (name, source) in [
        ("comment_heavy.js", format!("{}const value = 1;", "/* comment */\n".repeat(40_000))),
        ("lightly_mapped.js", format!("const value = \"{}\";", "a".repeat(524_288))),
    ] {
        let file = TestFile {
            file_name: name.into(),
            url: String::new(),
            source_text: source,
            source_type: SourceType::mjs(),
        };
        bench_file(criterion, &file, true);
    }
}

criterion_group!(token_transfer, bench_token_transfer);
criterion_main!(token_transfer);
