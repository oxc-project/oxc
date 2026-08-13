pub mod campaign;
pub mod generator;
pub mod oracle;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug, Clone)]
pub struct Minified {
    pub code: String,
    pub iterations: u8,
}

/// Compress `source` with `CompressOptions::smallest()` and print it back out.
///
/// With `mangle`, bindings are renamed as well, which also exercises the
/// mangler's scope analysis. Generated programs only observe behavior through
/// their final `console.log`, so renaming must not change the comparison.
///
/// # Errors
///
/// Returns a message when the generated source fails to parse or fails semantic analysis.
pub fn minify(source: &str, mangle: bool) -> Result<Minified, String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::script()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Err(format!("parser rejected generated input: {:?}", parsed.diagnostics));
    }

    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        mangle: mangle.then(|| MangleOptions { top_level: Some(true), ..MangleOptions::default() }),
        compress: Some(CompressOptions::smallest()),
    })
    .minify(&allocator, &mut program);
    let code = Codegen::new()
        .with_options(CodegenOptions::minify())
        .with_scoping(result.scoping)
        .build(&program)
        .code;
    Ok(Minified { code, iterations: result.iterations })
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use crate::{
        generator::generate,
        minify,
        oracle::{Comparison, Oracle},
    };

    #[test]
    fn same_seed_generates_same_program() {
        assert_eq!(generate(42), generate(42));
        assert_ne!(generate(42), generate(43));
    }

    #[test]
    fn generated_programs_parse_and_have_valid_semantics() {
        for seed in 0..100 {
            let source = generate(seed);
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &source, SourceType::script()).parse();
            assert!(!parsed.panicked, "seed {seed}\n{source}");
            assert!(parsed.diagnostics.is_empty(), "seed {seed}\n{source}");

            let semantic = SemanticBuilder::new().build(&parsed.program);
            assert!(semantic.diagnostics.is_empty(), "seed {seed}\n{source}");
        }
    }

    #[test]
    fn oracle_accepts_equivalent_programs_and_rejects_changed_output() {
        let oracle = Oracle::new(100);
        assert!(matches!(
            oracle.compare("console.log(1)", "console.log(1)"),
            Comparison::Equivalent { .. }
        ));
        assert!(matches!(
            oracle.compare("console.log(1)", "console.log(2)"),
            Comparison::Mismatch { .. }
        ));
    }

    #[test]
    fn oracle_skips_inputs_that_do_not_complete() {
        let oracle = Oracle::new(25);
        assert!(matches!(
            oracle.compare("for (;;) {}", "console.log(1)"),
            Comparison::Skipped { .. }
        ));
    }

    #[test]
    fn generated_programs_keep_their_observable_behavior_after_minification() {
        check_generated_programs(false);
    }

    #[test]
    fn generated_programs_keep_their_observable_behavior_after_mangling() {
        check_generated_programs(true);
    }

    fn check_generated_programs(mangle: bool) {
        let programs: Vec<_> = (0..25)
            .map(|seed| {
                let original = generate(seed);
                let minified = minify(&original, mangle)
                    .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
                (seed, original, minified.code)
            })
            .collect();
        let cases: Vec<_> = programs
            .iter()
            .map(|(_, original, minified)| (original.as_str(), minified.as_str()))
            .collect();

        for ((seed, original, minified), comparison) in
            programs.iter().zip(Oracle::new(100).compare_many(&cases))
        {
            assert!(
                matches!(comparison, Comparison::Equivalent { .. }),
                "seed {seed} (mangle={mangle}): {comparison:#?}\noriginal:\n{original}\nminified:\n{minified}"
            );
        }
    }
}
