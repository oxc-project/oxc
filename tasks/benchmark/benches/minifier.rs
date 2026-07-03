use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_minifier::{CompressOptions, Compressor, ManglePropertiesOptions, PropertyMangler};
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
        let id = BenchmarkId::from_parameter(format!("{}_keep_names", &first_file.file_name));
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
    // The measured region is small relative to the per-iteration transform setup; cut the
    // sample count so local `cargo bench` stays fast. CodSpeed ignores sampling config.
    group.sample_size(10);

    // Mangle properties prefixed with `_` (esbuild's conventional opt-in); `^_` is intentional.
    #[expect(clippy::trivial_regex)]
    let prop_regex = lazy_regex::Regex::new("^_").unwrap();

    // Only fixtures with a real `^_` property workload are benchmarked — on a file with no
    // matching properties this would measure just an empty collect pass. Probed over
    // `TestFiles::minimal()` (mangle candidates / minified bytes saved with `^_` + mangleQuoted):
    // react.development.js 20 / 631 B (real workload); App.tsx 2 / 48 B (415 kB file — dominated
    // by the collect walk); RadixUIAdoptionSection.jsx and binder.ts 0 / 0 B; kitchen-sink.tsx
    // 2 / 4 B.
    let files = TestFiles::minimal();
    let selected = files
        .files()
        .iter()
        .filter(|file| matches!(file.file_name.as_str(), "react.development.js" | "App.tsx"));

    for file in selected {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = file.source_type;
        let path = Path::new(&file.file_name);
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let mut program = transform_to_js(&allocator, source_text, source_type, path);
                let options = ManglePropertiesOptions {
                    regex: Some(prop_regex.clone()),
                    mangle_quoted: true,
                    ..Default::default()
                };
                // The feature's added work, mirroring the driver sequence in `Minifier::build`:
                // collect + annotated-literal rename run before compress, rewrite after variable
                // mangling. Measured standalone so the `minifier` group stays compressor-only.
                runner.run(|| {
                    let mut mangler = PropertyMangler::new(options);
                    mangler.collect(&program);
                    mangler.rename_annotated_literals(&mut program, &allocator);
                    mangler.rewrite(&mut program, &allocator);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(minifier, bench_minifier, bench_mangler, bench_property_mangler);
criterion_main!(minifier);
