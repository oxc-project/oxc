//! Differential test of unambiguous parsing.
//!
//! Each fixture is parsed as unambiguous, then again as whichever of module or script
//! the first parse resolved to. Both parses must produce identical output.

use std::{
    fmt::Debug,
    panic::{AssertUnwindSafe, catch_unwind},
};

use oxc::{
    allocator::Allocator,
    ast::ast::Program,
    diagnostics::{Diagnostics, LabeledSpan, OxcDiagnostic},
    parser::{Parser, ParserReturn, config::TokensParserConfig},
    span::SourceType,
    syntax::module_record::ModuleRecord,
};
use rayon::prelude::*;

use crate::{
    BabelFile, CoverageResult, MiscFile, Test262File, TestResult, TypeScriptFile, full_eq::FullEq,
};

pub fn run_unambiguous_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let result = unambiguous_diff(&file.code, SourceType::default());
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_unambiguous_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let result = unambiguous_diff(&file.code, file.source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_unambiguous_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .flat_map_iter(|file| {
            let multiple_units = file.units.len() > 1;
            file.units.iter().map(move |unit| {
                let path =
                    if multiple_units { file.path.join(&unit.name) } else { file.path.clone() };
                let result = unambiguous_diff(&unit.content, unit.source_type);
                CoverageResult { path, should_fail: false, result }
            })
        })
        .collect()
}

pub fn run_unambiguous_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let result = unambiguous_diff(&file.code, file.source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

fn unambiguous_diff(code: &str, source_type: SourceType) -> TestResult {
    let source_type = source_type.with_unambiguous(true);
    let allocator = Allocator::default();
    let Ok(actual) = catch_unwind(AssertUnwindSafe(|| parse(&allocator, code, source_type))) else {
        return TestResult::GenericError("Panic", "unambiguous parse panicked".to_string());
    };
    let resolved = actual.program.source_type;
    let allocator = Allocator::default();
    let Ok(expected) = catch_unwind(AssertUnwindSafe(|| parse(&allocator, code, resolved))) else {
        return TestResult::GenericError("Panic", format!("{resolved:?} parse panicked"));
    };

    if !program_eq(&actual.program, &expected.program) {
        return mismatch("AST", &actual.program, &expected.program);
    }
    if actual.program.comments != expected.program.comments {
        return mismatch("Comments", &actual.program.comments, &expected.program.comments);
    }
    if actual.tokens != expected.tokens {
        return mismatch("Tokens", &actual.tokens, &expected.tokens);
    }
    if actual.irregular_whitespaces != expected.irregular_whitespaces {
        return mismatch(
            "Irregular whitespaces",
            &actual.irregular_whitespaces,
            &expected.irregular_whitespaces,
        );
    }
    let actual_diagnostics = sorted_diagnostics(&actual.diagnostics);
    let expected_diagnostics = sorted_diagnostics(&expected.diagnostics);
    if actual.fatal_error != expected.fatal_error || actual_diagnostics != expected_diagnostics {
        return mismatch("Diagnostics", &actual_diagnostics, &expected_diagnostics);
    }
    if !module_record_eq(&actual.module_record, &expected.module_record) {
        return mismatch("Module record", &actual.module_record, &expected.module_record);
    }
    TestResult::Passed
}

fn parse<'a>(allocator: &'a Allocator, code: &'a str, source_type: SourceType) -> ParserReturn<'a> {
    Parser::new(allocator, code, source_type).with_config(TokensParserConfig).parse()
}

fn mismatch(case: &'static str, actual: &impl Debug, expected: &impl Debug) -> TestResult {
    TestResult::Mismatch(case, format!("{actual:#?}"), format!("{expected:#?}"))
}

/// Deferred errors are reported last, so sort by position before comparing.
fn sorted_diagnostics(diagnostics: &Diagnostics) -> Vec<&OxcDiagnostic> {
    let mut diagnostics = diagnostics.iter().collect::<Vec<_>>();
    diagnostics.sort_by(|a, b| {
        let span = |label: &LabeledSpan| (label.offset(), label.len());
        a.labels
            .iter()
            .map(span)
            .cmp(b.labels.iter().map(span))
            .then_with(|| a.message.cmp(&b.message))
    });
    diagnostics
}

/// Compare every field of 2 [`Program`]s with [`FullEq`], except:
/// - `source_text`, which is the same for both.
/// - `comments`, which are compared separately, so they get their own mismatch.
///
/// `Program` is `#[non_exhaustive]`, so can't be destructured here to check that every field
/// is listed. Update this function if `Program` gains a field.
fn program_eq(actual: &Program, expected: &Program) -> bool {
    actual.span.full_eq(&expected.span)
        && actual.source_type.full_eq(&expected.source_type)
        && actual.hashbang.full_eq(&expected.hashbang)
        && actual.directives.full_eq(&expected.directives)
        && actual.body.full_eq(&expected.body)
        && actual.scope_id.full_eq(&expected.scope_id)
}

/// Skip `has_module_syntax`. Unambiguous mode also sets it for top-level `await`,
/// to resolve the file as a module, but parsing as a module only sets it for `import` and `export`.
fn module_record_eq(actual: &ModuleRecord, expected: &ModuleRecord) -> bool {
    actual.requested_modules == expected.requested_modules
        && actual.import_entries == expected.import_entries
        && actual.local_export_entries == expected.local_export_entries
        && actual.indirect_export_entries == expected.indirect_export_entries
        && actual.star_export_entries == expected.star_export_entries
        && actual.exported_bindings == expected.exported_bindings
        && actual.dynamic_imports == expected.dynamic_imports
        && actual.import_metas == expected.import_metas
}
