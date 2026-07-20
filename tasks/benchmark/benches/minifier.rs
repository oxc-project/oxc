use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_minifier::{
    CompressOptions, Compressor, ManglePropertiesOptions, Minifier, MinifierOptions,
    PropertyMangler,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_minifier(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("minifier");

    for file in TestFiles::minimal().files().iter().skip(1) {
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

                // Create fresh AST + semantic data for each iteration
                let mut program = Parser::new(&allocator, source_text, source_type).parse().program;
                let scoping = SemanticBuilder::new()
                    .with_enum_eval(true)
                    .build(&program)
                    .semantic
                    .into_scoping();

                // Minifier only works on esnext.
                let transform_options = TransformOptions::from_target("esnext").unwrap();
                let transformer_ret =
                    Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                        .build_with_scoping(scoping, &mut program);
                assert!(transformer_ret.diagnostics.is_empty());
                let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

                let options = CompressOptions::smallest();
                runner.run(|| {
                    Compressor::new(&allocator).build_with_scoping(&mut program, scoping, options);
                });
            });
        });
    }

    group.finish();
}

/// Transform a parsed program to plain JavaScript (targeting esnext), the way the real pipeline
/// feeds the mangler: TypeScript is stripped before minification, so the mangler only ever runs
/// on JS - mangling an untransformed `.ts`/`.tsx` AST would not be representative. Returns the
/// transformed program; the caller builds semantic data from it and mangles.
fn transform_to_js<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    path: &Path,
) -> Program<'a> {
    let mut program = Parser::new(allocator, source_text, source_type).parse().program;
    let scoping =
        SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();
    let transform_options = TransformOptions::from_target("esnext").unwrap();
    let transformer_ret = Transformer::new(allocator, path, &transform_options)
        .build_with_scoping(scoping, &mut program);
    assert!(transformer_ret.diagnostics.is_empty());
    program
}

fn bench_mangler(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("mangler");
    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let source_text = file.source_text.as_str();
        let path = Path::new(&file.file_name);
        let mut allocator = Allocator::default();
        let mut temp_allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                temp_allocator.reset();
                let program = transform_to_js(&allocator, source_text, source_type, path);
                let mut semantic =
                    SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
                runner.run(|| {
                    Mangler::new_with_temp_allocator(&temp_allocator)
                        .build_with_semantic(&mut semantic, &program);
                });
            });
        });
    }

    {
        let files = TestFiles::minimal();
        let first_file = files.files().first().unwrap();
        let id = BenchmarkId::from_parameter(format!("{}_keep_names", first_file.file_name));
        let source_type = SourceType::from_path(&first_file.file_name).unwrap();
        let source_text = first_file.source_text.as_str();
        let path = Path::new(&first_file.file_name);
        let mut allocator = Allocator::default();
        let mut temp_allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                temp_allocator.reset();
                let program = transform_to_js(&allocator, source_text, source_type, path);
                let mut semantic =
                    SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
                runner.run(|| {
                    Mangler::new_with_temp_allocator(&temp_allocator)
                        .with_options(MangleOptions {
                            top_level: None,
                            keep_names: MangleOptionsKeepNames::all_true(),
                            ..MangleOptions::default()
                        })
                        .build_with_semantic(&mut semantic, &program);
                });
            });
        });
    }

    group.finish();
}

fn bench_property_mangler(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("property_mangler");
    let options = ManglePropertiesOptions::from_pattern("(^_|_$)").expect("valid benchmark regex");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let source_text = file.source_text.as_str();
        let path = Path::new(&file.file_name);
        let mut allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let mut program = transform_to_js(&allocator, source_text, source_type, path);
                runner.run(|| {
                    let mut mangler = PropertyMangler::new(options.clone());
                    mangler.collect(&program, None);
                    mangler.assign();
                    mangler.rewrite(&mut program, &allocator, None);
                    std::hint::black_box(mangler.into_cache());
                });
            });
        });
    }

    group.finish();
}

fn bench_minifier_property_mangling(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("minifier_property_mangling");
    let property_options =
        ManglePropertiesOptions::from_pattern("(^_|_$)").expect("valid benchmark regex");
    let files = TestFiles::minimal();

    for file in files.files().iter().filter(|file| {
        matches!(file.file_name.as_str(), "react.development.js" | "kitchen-sink.tsx")
    }) {
        for enabled in [false, true] {
            let id =
                BenchmarkId::new(&file.file_name, if enabled { "enabled" } else { "disabled" });
            let source_type = SourceType::from_path(&file.file_name).unwrap();
            let source_text = file.source_text.as_str();
            let path = Path::new(&file.file_name);
            let mut allocator = Allocator::default();
            let mangle_properties = enabled.then(|| property_options.clone());

            group.bench_function(id, |b| {
                b.iter_with_setup_wrapper(|runner| {
                    allocator.reset();
                    let mut program = transform_to_js(&allocator, source_text, source_type, path);
                    runner.run(|| {
                        let result = Minifier::new(MinifierOptions {
                            mangle: None,
                            mangle_properties: mangle_properties.clone(),
                            compress: Some(CompressOptions::smallest()),
                        })
                        .minify(&allocator, &mut program);
                        std::hint::black_box(result.property_mangle_cache);
                    });
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    minifier,
    bench_minifier,
    bench_mangler,
    bench_property_mangler,
    bench_minifier_property_mangling
);
criterion_main!(minifier);
