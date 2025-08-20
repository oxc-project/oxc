mod collapse_variable_declarations;
mod dead_code_elimination;
mod esbuild;
mod minimize_exit_points;
mod obscure_edge_cases;
mod oxc;
mod real_world_patterns;
mod statement_fusion;

use oxc_minifier::{CompressOptions, CompressOptionsUnused};

pub fn default_options() -> CompressOptions {
    CompressOptions {
        drop_debugger: false,
        unused: CompressOptionsUnused::Keep,
        ..CompressOptions::smallest()
    }
}

#[track_caller]
fn test(source_text: &str, expected: &str) {
    crate::test(source_text, expected, default_options());
}

#[track_caller]
fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
fn test_output(source_text: &str, expected: &str) {
    let options = default_options();
    let actual = crate::run(source_text, oxc_span::SourceType::default(), Some(options))
        .trim_end()
        .to_string();
    assert_eq!(actual, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{actual}");
}
