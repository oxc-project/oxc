mod collapse_variable_declarations;
mod dead_code_elimination;
mod esbuild;
mod minimize_exit_points;
mod oxc;
mod statement_fusion;

use oxc_minifier::{CompressOptions, CompressOptionsUnused};

#[track_caller]
fn test(source_text: &str, expected: &str) {
    let options = CompressOptions {
        drop_debugger: false,
        drop_console: false,
        unused: CompressOptionsUnused::Keep,
        ..CompressOptions::smallest()
    };
    crate::test(source_text, expected, options);
}

#[track_caller]
fn test_same(source_text: &str) {
    test(source_text, source_text);
}
