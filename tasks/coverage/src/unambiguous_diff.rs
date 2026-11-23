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
    ast_visit::Visit,
    diagnostics::{Diagnostics, LabeledSpan, OxcDiagnostic},
    parser::{Parser, ParserReturn, config::TokensParserConfig},
    span::{ContentEq, SourceType, Span},
    syntax::module_record::ModuleRecord,
};
use rayon::prelude::*;

use crate::{BabelFile, CoverageResult, MiscFile, Test262File, TestResult, TypeScriptFile};

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

/// [`ContentEq`] ignores spans, so compare every node's span separately.
fn program_eq(actual: &Program, expected: &Program) -> bool {
    actual.content_eq(expected)
        && SpanCollector::collect(actual) == SpanCollector::collect(expected)
}

#[derive(Default)]
struct SpanCollector(Vec<Span>);

impl SpanCollector {
    fn collect(program: &Program) -> Vec<Span> {
        let mut collector = Self::default();
        collector.visit_program(program);
        collector.0
    }
}

impl Visit<'_> for SpanCollector {
    fn visit_span(&mut self, span: &Span) {
        self.0.push(*span);
    }
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
