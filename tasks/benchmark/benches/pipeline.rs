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

/// A representative Vite-style define config. Unlike a single dot define, this exercises every
/// `ReplaceGlobalDefines` dispatch path — identifier, dot, specific `import.meta.env.*`, and
/// trailing wildcards — so the benchmark reflects the per-node lookup cost of a real config.
const DEFINES: &[(&str, &str)] = &[
    // identifier defines
    ("__DEV__", "false"),
    ("__PROD__", "true"),
    ("__VERSION__", "'1.2.3'"),
    ("__TEST__", "false"),
    ("DEBUG", "false"),
    // dot defines
    ("process.env.NODE_ENV", "'production'"),
    ("process.env.PLATFORM", "'browser'"),
    ("process.env.VERSION", "'1.2.3'"),
    ("globalThis.__DEV__", "false"),
    // specific `import.meta.env` entries
    ("import.meta.env.MODE", "'production'"),
    ("import.meta.env.BASE_URL", "'/'"),
    ("import.meta.env.PROD", "true"),
    ("import.meta.env.DEV", "false"),
    ("import.meta.env.SSR", "false"),
    ("import.meta.env.VITE_API_URL", "'https://example.com'"),
    ("import.meta.env.VITE_APP_TITLE", "'app'"),
    // trailing wildcards
    ("import.meta.env.*", "undefined"),
    ("import.meta.*", "undefined"),
];

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
        Some(self.mangle.clone())
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
            define: ReplaceGlobalDefinesConfig::new(DEFINES).unwrap(),
            // Common global-variable injections: Node.js polyfills (when bundling for the
            // browser) plus a classic-runtime JSX auto-import. Like the defines, these exercise
            // the per-reference inject matching rather than a single specifier.
            inject: InjectGlobalVariablesConfig::new(vec![
                InjectImport::named_specifier("node:buffer", Some("Buffer"), "Buffer"),
                InjectImport::default_specifier("node:process", "process"),
                InjectImport::named_specifier("node:timers", Some("setImmediate"), "setImmediate"),
                InjectImport::named_specifier(
                    "node:timers",
                    Some("clearImmediate"),
                    "clearImmediate",
                ),
                InjectImport::default_specifier("react", "React"),
            ]),
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
