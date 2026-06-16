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
    transform: TransformOptions,
    define: ReplaceGlobalDefinesConfig,
    inject: InjectGlobalVariablesConfig,
    compress: CompressOptions,
    mangle: MangleOptions,
    codegen: CodegenOptions,
}

impl CompilerInterface for PipelineCompiler {
    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.transform)
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        Some(self.define.clone())
    }

    fn inject_options(&self) -> Option<InjectGlobalVariablesConfig> {
        Some(self.inject.clone())
    }

    fn compress_options(&self) -> Option<CompressOptions> {
        Some(self.compress.clone())
    }

    fn mangle_options(&self) -> Option<MangleOptions> {
        Some(self.mangle)
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(self.codegen.clone())
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

        // `compile` creates its own `Allocator` and keeps no state between runs,
        // so the compiler and its options can be built once and reused.
        let mut compiler = PipelineCompiler {
            transform: TransformOptions::from_target("esnext").unwrap(),
            define: ReplaceGlobalDefinesConfig::new(&[("process.env.NODE_ENV", "'production'")])
                .unwrap(),
            inject: InjectGlobalVariablesConfig::new(vec![InjectImport::named_specifier(
                "node:buffer",
                Some("Buffer"),
                "Buffer",
            )]),
            compress: CompressOptions::smallest(),
            mangle: MangleOptions::default(),
            codegen: CodegenOptions::default(),
        };

        group.bench_function(id, |b| {
            b.iter(|| {
                compiler.compile(source_text, source_type, source_path);
            });
        });
    }

    group.finish();
}

criterion_group!(pipeline, bench_pipeline);
criterion_main!(pipeline);
