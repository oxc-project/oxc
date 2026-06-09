use std::path::Path;

use oxc::{
    CompilerInterface,
    codegen::CodegenOptions,
    mangler::MangleOptions,
    minifier::CompressOptions,
    transformer::TransformOptions,
    transformer_plugins::{InjectGlobalVariablesConfig, InjectImport, ReplaceGlobalDefinesConfig},
};
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_tasks_common::TestFiles;

/// A [`CompilerInterface`] that runs the complete compilation pipeline:
/// parse -> semantic -> transform -> define -> inject -> minify -> mangle -> codegen.
struct PipelineCompiler {
    transform_options: TransformOptions,
    define_options: ReplaceGlobalDefinesConfig,
    inject_options: InjectGlobalVariablesConfig,
    compress_options: CompressOptions,
    mangle_options: MangleOptions,
    codegen_options: CodegenOptions,
}

impl CompilerInterface for PipelineCompiler {
    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.transform_options)
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        Some(self.define_options.clone())
    }

    fn inject_options(&self) -> Option<InjectGlobalVariablesConfig> {
        Some(self.inject_options.clone())
    }

    fn compress_options(&self) -> Option<CompressOptions> {
        Some(self.compress_options.clone())
    }

    fn mangle_options(&self) -> Option<MangleOptions> {
        Some(self.mangle_options)
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(self.codegen_options.clone())
    }

    fn check_semantic_error(&self) -> bool {
        false
    }
}

fn bench_pipeline(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("pipeline");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;
        let source_path = Path::new(&file.file_name);

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Create options inside the closure to avoid move issues
                let mut compiler = PipelineCompiler {
                    transform_options: TransformOptions::from_target("esnext").unwrap(),
                    define_options: ReplaceGlobalDefinesConfig::new(&[(
                        "process.env.NODE_ENV",
                        "'production'",
                    )])
                    .unwrap(),
                    inject_options: InjectGlobalVariablesConfig::new(vec![
                        InjectImport::named_specifier("node:buffer", Some("Buffer"), "Buffer"),
                    ]),
                    compress_options: CompressOptions::smallest(),
                    mangle_options: MangleOptions::default(),
                    codegen_options: CodegenOptions::default(),
                };

                runner.run(|| {
                    compiler.compile(source_text, source_type, source_path);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(pipeline, bench_pipeline);
criterion_main!(pipeline);
