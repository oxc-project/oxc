//! Tool runner functions for coverage testing

use std::{borrow::Cow, fs, path::{Path, PathBuf}, sync::Arc};

use oxc::{
    allocator::Allocator,
    ast_visit::utf8_to_utf16::Utf8ToUtf16,
    diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource, OxcDiagnostic},
    minifier::CompressOptions,
    parser::{ParseOptions, Parser, ParserReturn, config::RuntimeParserConfig},
    span::{ModuleKind, SourceType, Span},
    transformer::{JsxOptions, JsxRuntime, TransformOptions},
};
use oxc_estree_tokens::{EstreeTokenOptions, to_estree_tokens_pretty_json};
use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, Expand, FormatOptions,
    Formatter, IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteProperties, QuoteStyle,
    Semicolons, TrailingCommas, get_parse_options,
};
use rayon::prelude::*;

use crate::{
    AcornJsxFile, BabelFile, CoverageResult, Driver, MiscFile, Test262File, TestResult,
    TypeScriptFile, test262::TestFlag, typescript::constants::TS_IGNORE_SUPPRESSIBLE_ERRORS,
    workspace_root,
};

// ================================
// Parser
// ================================

fn run_parser(
    path: &Path,
    code: &str,
    source_type: SourceType,
    always_strict: bool,
    allow_return_outside_function: bool,
) -> TestResult {
    let source_text: Cow<str> = if always_strict {
        Cow::Owned(format!("'use strict';\n{code}"))
    } else {
        Cow::Borrowed(code)
    };

    let mut driver = Driver { allow_return_outside_function, ..Driver::default() };
    driver.run(&source_text, source_type);

    let errors = driver.errors();
    if errors.is_empty() {
        TestResult::Passed
    } else {
        let handler = GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
        let mut output = String::new();
        // Create Arc once and share across all errors to avoid cloning source for each error
        let source_arc: Arc<String> = Arc::new(source_text.into_owned());
        // Extract path string before loop to avoid repeated conversions
        let path_str = path.to_string_lossy();
        for error in &errors {
            let error = error
                .clone()
                .with_source_code(NamedSource::new(path_str.clone(), Arc::clone(&source_arc)));
            handler.render_report(&mut output, error.as_ref()).unwrap();
        }
        TestResult::ParseError(output, driver.panicked)
    }
}

fn evaluate_result(result: TestResult, should_fail: bool) -> TestResult {
    match (result, should_fail) {
        (TestResult::ParseError(err, panicked), true) => TestResult::CorrectError(err, panicked),
        (TestResult::Passed, true) => TestResult::IncorrectlyPassed,
        (result, _) => result,
    }
}

pub fn run_parser_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let flags = &f.meta.flags;
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            let source_type = SourceType::cjs().with_script(true);

            let result = if flags.contains(&TestFlag::OnlyStrict) {
                let r = run_parser(&f.path, &f.code, source_type, true, false);
                evaluate_result(r, should_fail)
            } else if flags.contains(&TestFlag::Module) {
                let r = run_parser(&f.path, &f.code, source_type.with_module(true), false, false);
                evaluate_result(r, should_fail)
            } else if flags.contains(&TestFlag::NoStrict) || flags.contains(&TestFlag::Raw) {
                let r = run_parser(&f.path, &f.code, source_type, false, false);
                evaluate_result(r, should_fail)
            } else {
                // Run both non-strict and strict
                let r = run_parser(&f.path, &f.code, source_type, false, false);
                let r = evaluate_result(r, should_fail);
                if matches!(r, TestResult::Passed | TestResult::CorrectError(..)) {
                    let r2 = run_parser(&f.path, &f.code, source_type, true, false);
                    evaluate_result(r2, should_fail)
                } else {
                    r
                }
            };

            CoverageResult { path: f.path.clone(), should_fail, result }
        })
        .collect()
}

pub fn run_parser_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let result = run_parser(
                &f.path,
                &f.code,
                f.source_type,
                false,
                f.options.allow_return_outside_function,
            );
            let result = evaluate_result(result, f.should_fail);
            CoverageResult { path: f.path.clone(), should_fail: f.should_fail, result }
        })
        .collect()
}

/// Check if a diagnostic error is suppressed by a `@ts-ignore` or `@ts-expect-error`
/// comment on the preceding line.
fn is_error_suppressed_by_ts_ignore(
    error: &OxcDiagnostic,
    source_text: &str,
    ts_ignore_spans: &[Span],
) -> bool {
    // Check if this error message is suppressible
    let error_message = error.to_string();
    if !TS_IGNORE_SUPPRESSIBLE_ERRORS.contains(error_message.as_str()) {
        return false;
    }

    // Get the error's byte offset from the first label
    let Some(labels) = &error.labels else {
        return false;
    };
    let Some(first_label) = labels.first() else {
        return false;
    };
    let error_offset = first_label.offset();

    // Check if any ts-ignore span covers the line before this error
    for ts_ignore_span in ts_ignore_spans {
        let after_comment = &source_text[ts_ignore_span.end as usize..];

        // Find the first newline (end of the comment line)
        let Some(first_newline_pos) = after_comment.find('\n') else {
            continue;
        };

        // The next line starts after the first newline
        let next_line_start = ts_ignore_span.end as usize + first_newline_pos + 1;

        // Find the end of the next line (second newline or end of string)
        let next_line_end = source_text[next_line_start..]
            .find('\n')
            .map_or(source_text.len(), |pos| next_line_start + pos);

        // Check if the error offset falls within the next line
        if error_offset >= next_line_start && error_offset < next_line_end {
            return true;
        }
    }

    false
}

/// Run parser for TypeScript with ts-ignore error suppression
fn run_parser_typescript_unit(
    path: &Path,
    code: &str,
    source_type: SourceType,
    always_strict: bool,
    ts_ignore_spans: &[Span],
) -> TestResult {
    let source_text: Cow<str> = if always_strict {
        Cow::Owned(format!("'use strict';\n{code}"))
    } else {
        Cow::Borrowed(code)
    };

    let mut driver = Driver { allow_return_outside_function: false, ..Driver::default() };
    driver.run(&source_text, source_type);

    let errors = driver.errors();
    if errors.is_empty() {
        return TestResult::Passed;
    }

    // Filter out errors that are suppressed by ts-ignore
    let has_unsuppressed_errors = errors
        .iter()
        .any(|error| !is_error_suppressed_by_ts_ignore(error, &source_text, ts_ignore_spans));

    if !has_unsuppressed_errors {
        return TestResult::Passed;
    }

    // Format errors for output
    let handler = GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
    let mut output = String::new();
    // Create Arc once and share across all errors to avoid cloning source for each error
    let source_arc: Arc<String> = Arc::new(source_text.into_owned());
    // Extract path string before loop to avoid repeated conversions
    let path_str = path.to_string_lossy();
    for error in &errors {
        let error = error
            .clone()
            .with_source_code(NamedSource::new(path_str.clone(), Arc::clone(&source_arc)));
        handler.render_report(&mut output, error.as_ref()).unwrap();
    }
    TestResult::ParseError(output, driver.panicked)
}

pub fn run_parser_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let mut final_result = TestResult::Passed;
            for unit in &f.units {
                let result = run_parser_typescript_unit(
                    &f.path,
                    &unit.content,
                    unit.source_type,
                    f.settings.always_strict,
                    &unit.ts_ignore_spans,
                );
                if !matches!(result, TestResult::Passed) {
                    final_result = result;
                    break;
                }
            }
            let result = evaluate_result(final_result, f.should_fail);
            CoverageResult { path: f.path.clone(), should_fail: f.should_fail, result }
        })
        .collect()
}

pub fn run_parser_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let result = run_parser(&f.path, &f.code, f.source_type, false, false);
            let result = evaluate_result(result, f.should_fail);
            CoverageResult { path: f.path.clone(), should_fail: f.should_fail, result }
        })
        .collect()
}

// ================================
// Semantic
// ================================

fn get_default_transformer_options() -> TransformOptions {
    TransformOptions {
        jsx: JsxOptions {
            jsx_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            ..Default::default()
        },
        ..TransformOptions::enable_all()
    }
}

fn run_semantic(
    code: &str,
    source_type: SourceType,
    path: &Path,
    options: Option<TransformOptions>,
) -> TestResult {
    let mut driver = Driver {
        path: path.to_path_buf(),
        transform: Some(options.unwrap_or_else(get_default_transformer_options)),
        check_semantic: true,
        ..Driver::default()
    };

    driver.run(code, source_type);
    let errors = driver.errors();
    if errors.is_empty() {
        TestResult::Passed
    } else {
        let messages =
            errors.into_iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("\n");
        TestResult::GenericError("semantic", messages)
    }
}

pub fn run_semantic_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            !should_fail
        })
        .map(|f| {
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let result = run_semantic(&f.code, source_type, &f.path, None);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_semantic_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_semantic(&f.code, f.source_type, &f.path, None);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_semantic_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        // Skip files that have any error codes (not just the supported ones)
        .filter(|f| f.error_codes.is_empty())
        .map(|f| {
            let mut final_result = TestResult::Passed;
            for unit in &f.units {
                let mut options = get_default_transformer_options();
                let mut source_type = unit.source_type;
                if f.settings.jsx.last().is_some_and(|jsx| jsx == "react") {
                    source_type = source_type.with_module(true);
                    options.jsx.runtime = JsxRuntime::Classic;
                }
                let result = run_semantic(&unit.content, source_type, &f.path, Some(options));
                if result != TestResult::Passed {
                    final_result = result;
                    break;
                }
            }
            CoverageResult { path: f.path.clone(), should_fail: false, result: final_result }
        })
        .collect()
}

pub fn run_semantic_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_semantic(&f.code, f.source_type, &f.path, None);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

// ================================
// Codegen
// ================================

fn run_codegen(code: &str, source_type: SourceType) -> TestResult {
    // Normal idempotency
    let result =
        Driver { codegen: true, ..Driver::default() }.idempotency("Normal", code, source_type);
    if result != TestResult::Passed {
        return result;
    }
    // Minified idempotency
    Driver { codegen: true, remove_whitespace: true, ..Driver::default() }.idempotency(
        "Minify",
        code,
        source_type,
    )
}

pub fn run_codegen_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            !should_fail
        })
        .map(|f| {
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let result = run_codegen(&f.code, source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_codegen_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_codegen(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_codegen_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let mut final_result = TestResult::Passed;
            for unit in &f.units {
                let result = run_codegen(&unit.content, unit.source_type);
                if result != TestResult::Passed {
                    final_result = result;
                    break;
                }
            }
            CoverageResult { path: f.path.clone(), should_fail: false, result: final_result }
        })
        .collect()
}

pub fn run_codegen_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_codegen(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

// ================================
// Formatter
// ================================

fn get_formatter_options_list() -> [FormatOptions; 3] {
    [
        FormatOptions::default(),
        FormatOptions {
            indent_style: IndentStyle::Tab,
            indent_width: IndentWidth::try_from(4).unwrap(),
            line_ending: LineEnding::Crlf,
            line_width: LineWidth::try_from(80).unwrap(),
            quote_style: QuoteStyle::Single,
            jsx_quote_style: QuoteStyle::Single,
            quote_properties: QuoteProperties::Consistent,
            trailing_commas: TrailingCommas::Es5,
            semicolons: Semicolons::AsNeeded,
            arrow_parentheses: ArrowParentheses::AsNeeded,
            bracket_spacing: BracketSpacing::from(false),
            bracket_same_line: BracketSameLine::from(false),
            attribute_position: AttributePosition::Multiline,
            expand: Expand::Never,
            ..Default::default()
        },
        FormatOptions {
            indent_width: IndentWidth::try_from(8).unwrap(),
            line_width: LineWidth::try_from(120).unwrap(),
            line_ending: LineEnding::Lf,
            quote_properties: QuoteProperties::Preserve,
            trailing_commas: TrailingCommas::None,
            ..Default::default()
        },
    ]
}

fn run_formatter(code: &str, source_type: SourceType) -> TestResult {
    let allocator = Allocator::default();
    let ParserReturn { program, errors, .. } =
        Parser::new(&allocator, code, source_type).with_options(get_parse_options()).parse();

    if !errors.is_empty() {
        return TestResult::Passed; // Skip if parse error
    }

    for options in get_formatter_options_list() {
        let text1 = Formatter::new(&allocator, options.clone()).build(&program);

        let allocator2 = Allocator::default();
        let ParserReturn { program: program2, errors, .. } =
            Parser::new(&allocator2, &text1, source_type).with_options(get_parse_options()).parse();

        if !errors.is_empty() {
            return TestResult::ParseError(
                errors.iter().map(std::string::ToString::to_string).collect(),
                false,
            );
        }

        let text2 = Formatter::new(&allocator2, options).build(&program2);

        if text1 != text2 {
            return TestResult::Mismatch("Mismatch", text1, text2);
        }
    }

    TestResult::Passed
}

pub fn run_formatter_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            !should_fail
        })
        .map(|f| {
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let result = run_formatter(&f.code, source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_formatter_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_formatter(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_formatter_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let mut final_result = TestResult::Passed;
            for unit in &f.units {
                let result = run_formatter(&unit.content, unit.source_type);
                if result != TestResult::Passed {
                    final_result = result;
                    break;
                }
            }
            CoverageResult { path: f.path.clone(), should_fail: false, result: final_result }
        })
        .collect()
}

pub fn run_formatter_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_formatter(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

// ================================
// Transformer
// ================================

fn run_transformer(
    code: &str,
    source_type: SourceType,
    path: &Path,
    options: Option<TransformOptions>,
) -> TestResult {
    let mut driver = Driver {
        path: path.to_path_buf(),
        transform: Some(options.unwrap_or_else(get_default_transformer_options)),
        codegen: true,
        ..Driver::default()
    };

    driver.run(code, source_type);
    let transformed1 = std::mem::take(&mut driver.printed);

    // Second pass with only JavaScript syntax
    let second_pass_source_type = match source_type.module_kind() {
        ModuleKind::Module => SourceType::mjs(),
        ModuleKind::Script => SourceType::script(),
        ModuleKind::Unambiguous => SourceType::unambiguous(),
        ModuleKind::CommonJS => SourceType::cjs(),
    };
    driver.run(&transformed1, second_pass_source_type);

    if transformed1 == driver.printed {
        TestResult::Passed
    } else {
        TestResult::Mismatch("Mismatch", transformed1, std::mem::take(&mut driver.printed))
    }
}

pub fn run_transformer_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            !should_fail
        })
        .map(|f| {
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let result = run_transformer(&f.code, source_type, &f.path, None);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_transformer_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_transformer(&f.code, f.source_type, &f.path, None);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_transformer_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let mut final_result = TestResult::Passed;
            for unit in &f.units {
                let mut options = get_default_transformer_options();
                let mut source_type = unit.source_type;
                if f.settings.jsx.last().is_some_and(|jsx| jsx == "react") {
                    source_type = source_type.with_module(true);
                    options.jsx.runtime = JsxRuntime::Classic;
                }
                let result = run_transformer(&unit.content, source_type, &f.path, Some(options));
                if result != TestResult::Passed {
                    final_result = result;
                    break;
                }
            }
            CoverageResult { path: f.path.clone(), should_fail: false, result: final_result }
        })
        .collect()
}

pub fn run_transformer_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail)
        .map(|f| {
            let result = run_transformer(&f.code, f.source_type, &f.path, None);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

// ================================
// Minifier
// ================================

fn run_minifier(code: &str, source_type: SourceType) -> TestResult {
    Driver { compress: Some(CompressOptions::smallest()), codegen: true, ..Driver::default() }
        .idempotency("Compress", code, source_type)
}

pub fn run_minifier_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            let is_no_strict = f.meta.flags.contains(&TestFlag::NoStrict);
            !should_fail && !is_no_strict
        })
        .map(|f| {
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let result = run_minifier(&f.code, source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_minifier_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| !f.should_fail && !f.source_type.is_typescript())
        .map(|f| {
            let result = run_minifier(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

// ================================
// ESTree
// ================================

pub fn run_estree_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            if should_fail {
                return false;
            }
            // Skip tests where no Acorn JSON file
            let acorn_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&f.path)
                .with_extension("json");
            acorn_path.exists()
        })
        .map(|f| {
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let allocator = Allocator::new();
            let ret = Parser::new(&allocator, &f.code, source_type).parse();

            if ret.panicked || !ret.errors.is_empty() {
                let error =
                    ret.errors.first().map_or_else(|| "Panicked".to_string(), ToString::to_string);
                return CoverageResult {
                    path: f.path.clone(),
                    should_fail: false,
                    result: TestResult::ParseError(error, ret.panicked),
                };
            }

            let mut program = ret.program;
            Utf8ToUtf16::new(&f.code).convert_program_with_ascending_order_checks(&mut program);

            let acorn_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&f.path)
                .with_extension("json");
            let acorn_json = fs::read_to_string(&acorn_path).unwrap_or_default();
            let oxc_json = program.to_pretty_estree_js_json(false);

            let result = if oxc_json == acorn_json {
                TestResult::Passed
            } else {
                TestResult::Mismatch("Mismatch", oxc_json, acorn_json)
            };

            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_estree_acorn_jsx(files: &[AcornJsxFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let source_type = SourceType::default().with_module(true).with_jsx(true);
            let allocator = Allocator::new();
            let ret = Parser::new(&allocator, &f.code, source_type).parse();
            let is_parse_error = ret.panicked || !ret.errors.is_empty();

            if is_parse_error {
                let error =
                    ret.errors.first().map_or_else(|| "Panicked".to_string(), ToString::to_string);
                let result = if f.should_fail {
                    TestResult::CorrectError(error, ret.panicked)
                } else {
                    TestResult::ParseError(error, ret.panicked)
                };
                return CoverageResult { path: f.path.clone(), should_fail: f.should_fail, result };
            }

            if f.should_fail {
                return CoverageResult {
                    path: f.path.clone(),
                    should_fail: true,
                    result: TestResult::IncorrectlyPassed,
                };
            }

            let mut program = ret.program;
            Utf8ToUtf16::new(&f.code).convert_program_with_ascending_order_checks(&mut program);

            let acorn_json_path = workspace_root().join(&f.path).with_extension("json");
            let acorn_json = match fs::read_to_string(&acorn_json_path) {
                Ok(acorn_json) => acorn_json,
                Err(error) => {
                    return CoverageResult {
                        path: f.path.clone(),
                        should_fail: false,
                        result: TestResult::GenericError(
                            "Error reading Acorn JSON",
                            error.to_string(),
                        ),
                    };
                }
            };
            let oxc_json = program.to_pretty_estree_js_json(false);

            let result = if oxc_json == acorn_json {
                TestResult::Passed
            } else {
                TestResult::Mismatch("Mismatch", oxc_json, acorn_json)
            };

            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_estree_test262_tokens(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            let should_fail =
                f.meta.negative.as_ref().is_some_and(|n| n.phase == crate::test262::Phase::Parse);
            if should_fail {
                return false;
            }
            workspace_root()
                .join("estree-conformance/tests/test262-tokens")
                .join(f.path.strip_prefix("test262/").unwrap_or(&f.path))
                .with_extension("json")
                .exists()
        })
        .map(|f| {
            let allocator = Allocator::new();
            let source_text = f.code.as_str();
            let is_module = f.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let ret = Parser::new(&allocator, &f.code, source_type)
                .with_config(RuntimeParserConfig::new(true))
                .parse();

            if ret.panicked || !ret.errors.is_empty() {
                let error =
                    ret.errors.first().map_or_else(|| "Panicked".to_string(), ToString::to_string);
                return CoverageResult {
                    path: f.path.clone(),
                    should_fail: false,
                    result: TestResult::ParseError(error, ret.panicked),
                };
            }

            let ParserReturn { mut program, tokens, .. } = ret;
            let span_converter = Utf8ToUtf16::new(source_text);

            let oxc_tokens_json = to_estree_tokens_pretty_json(
                &tokens,
                &program,
                source_text,
                &span_converter,
                EstreeTokenOptions::test262(),
            );

            span_converter.convert_program_with_ascending_order_checks(&mut program);

            let token_path = workspace_root()
                .join("estree-conformance/tests/test262-tokens")
                .join(f.path.strip_prefix("test262/").unwrap_or(&f.path))
                .with_extension("json");
            let expected_tokens_json = fs::read_to_string(&token_path).unwrap_or_default();

            let result = if oxc_tokens_json == expected_tokens_json {
                TestResult::Passed
            } else {
                TestResult::Mismatch("Token mismatch", oxc_tokens_json, expected_tokens_json)
            };

            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_estree_acorn_jsx_tokens(files: &[AcornJsxFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let allocator = Allocator::new();
            let source_text = f.code.as_str();
            let source_type = SourceType::script().with_module(true).with_jsx(true);
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_config(RuntimeParserConfig::new(true))
                .parse();
            if ret.panicked || !ret.errors.is_empty() {
                let error =
                    ret.errors.first().map_or_else(|| "Panicked".to_string(), ToString::to_string);
                return CoverageResult {
                    path: f.path.clone(),
                    should_fail: false,
                    result: TestResult::ParseError(error, ret.panicked),
                };
            }

            let ParserReturn { mut program, tokens, .. } = ret;
            let span_converter = Utf8ToUtf16::new(source_text);

            let oxc_tokens_json = to_estree_tokens_pretty_json(
                &tokens,
                &program,
                source_text,
                &span_converter,
                EstreeTokenOptions::test262(),
            );

            span_converter.convert_program_with_ascending_order_checks(&mut program);

            let token_path = workspace_root().join(f.path.with_extension("tokens.json"));
            let expected_tokens_json = fs::read_to_string(&token_path).unwrap_or_default();

            let result = if oxc_tokens_json == expected_tokens_json {
                TestResult::Passed
            } else {
                TestResult::Mismatch("Token mismatch", oxc_tokens_json, expected_tokens_json)
            };

            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

// Skip paths for TypeScript ESTree tests
static TS_SKIP_PATHS: &[&str] = &[
    // Skip cases which are failing in parser conformance tests
    "typescript/tests/cases/compiler/arrayFromAsync.ts",
    "typescript/tests/cases/conformance/classes/propertyMemberDeclarations/staticPropertyNameConflicts.ts",
    "typescript/tests/cases/conformance/es2019/importMeta/importMeta.ts",
    "typescript/tests/cases/compiler/sourceMapValidationDecorators.ts",
    "typescript/tests/cases/conformance/esDecorators/esDecorators-decoratorExpression.1.ts",
    // Skip tests where TS-ESLint is incorrect
    "typescript/tests/cases/conformance/es6/templates/templateStringMultiline3.ts",
];

pub fn run_estree_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            if f.should_fail {
                return false;
            }
            // Skip ignored paths
            if f.path.to_str().is_some_and(|p| TS_SKIP_PATHS.contains(&p)) {
                return false;
            }
            // Skip tests where no expected ESTree file exists
            let ext = f.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let estree_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&f.path)
                .with_extension(format!("{ext}.md"));
            estree_path.exists()
        })
        .map(|f| {
            let ext = f.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let estree_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&f.path)
                .with_extension(format!("{ext}.md"));

            let estree_content = fs::read_to_string(&estree_path).unwrap_or_default();
            let estree_units = parse_estree_json_blocks(&estree_content, "AST");

            if estree_units.len() != f.units.len() {
                return CoverageResult {
                    path: f.path.clone(),
                    should_fail: false,
                    result: TestResult::GenericError(
                        "Unexpected estree file",
                        format!("{} != {}", estree_units.len(), f.units.len()),
                    ),
                };
            }

            for (unit, expected) in f.units.iter().zip(estree_units.iter()) {
                let allocator = Allocator::new();
                let options = ParseOptions { preserve_parens: false, ..Default::default() };
                let ret = Parser::new(&allocator, &unit.content, unit.source_type)
                    .with_options(options)
                    .parse();

                if ret.panicked || !ret.errors.is_empty() {
                    let error = ret
                        .errors
                        .first()
                        .map_or_else(|| "Panicked".to_string(), ToString::to_string);
                    return CoverageResult {
                        path: f.path.clone(),
                        should_fail: false,
                        result: TestResult::ParseError(error, ret.panicked),
                    };
                }

                let mut program = ret.program;
                Utf8ToUtf16::new(&unit.content)
                    .convert_program_with_ascending_order_checks(&mut program);
                let oxc_json = program.to_pretty_estree_ts_json(false);

                if oxc_json != *expected {
                    return CoverageResult {
                        path: f.path.clone(),
                        should_fail: false,
                        result: TestResult::Mismatch("Mismatch", oxc_json, expected.to_string()),
                    };
                }
            }

            CoverageResult { path: f.path.clone(), should_fail: false, result: TestResult::Passed }
        })
        .collect()
}

pub fn run_estree_typescript_tokens(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|f| {
            if f.should_fail {
                return false;
            }
            if f.path.to_str().is_some_and(|p| TS_SKIP_PATHS.contains(&p)) {
                return false;
            }
            let ext = f.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let estree_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&f.path)
                .with_extension(format!("{ext}.md"));
            estree_path.exists()
        })
        .map(|f| {
            let ext = f.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let estree_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&f.path)
                .with_extension(format!("{ext}.md"));

            let estree_content = fs::read_to_string(&estree_path).unwrap_or_default();
            let estree_token_units = parse_estree_json_blocks(&estree_content, "TOKENS");

            if estree_token_units.len() != f.units.len() {
                return CoverageResult {
                    path: f.path.clone(),
                    should_fail: false,
                    result: TestResult::GenericError(
                        "Unexpected estree file",
                        format!("TOKENS {} != {}", estree_token_units.len(), f.units.len()),
                    ),
                };
            }

            for (unit, expected_tokens) in f.units.iter().zip(estree_token_units.iter()) {
                let allocator = Allocator::new();
                let source_text = unit.content.as_str();
                let source_type = unit.source_type;
                let ret = Parser::new(&allocator, source_text, source_type)
                    .with_options(ParseOptions { preserve_parens: false, ..Default::default() })
                    .with_config(RuntimeParserConfig::new(true))
                    .parse();

                if ret.panicked || !ret.errors.is_empty() {
                    let error = ret
                        .errors
                        .first()
                        .map_or_else(|| "Panicked".to_string(), ToString::to_string);
                    return CoverageResult {
                        path: f.path.clone(),
                        should_fail: false,
                        result: TestResult::ParseError(error, ret.panicked),
                    };
                }

                let ParserReturn { mut program, tokens, .. } = ret;
                let span_converter = Utf8ToUtf16::new(source_text);

                let oxc_tokens_json = to_estree_tokens_pretty_json(
                    &tokens,
                    &program,
                    source_text,
                    &span_converter,
                    EstreeTokenOptions::typescript(),
                );

                span_converter.convert_program_with_ascending_order_checks(&mut program);

                if oxc_tokens_json != *expected_tokens {
                    return CoverageResult {
                        path: f.path.clone(),
                        should_fail: false,
                        result: TestResult::Mismatch(
                            "Token mismatch",
                            oxc_tokens_json,
                            expected_tokens.to_string(),
                        ),
                    };
                }
            }

            CoverageResult { path: f.path.clone(), should_fail: false, result: TestResult::Passed }
        })
        .collect()
}

// ================================
// Checker (shared helpers)
// ================================

/// Convert test runner `CompilerSettings` into `CheckerOptions`.
fn checker_options_from_settings(
    settings: &crate::typescript::meta::CompilerSettings,
) -> oxc_checker::CheckerOptions {
    oxc_checker::CheckerOptions {
        strict: settings.strict,
        strict_null_checks: settings.strict_null_checks,
        strict_property_initialization: settings.strict_property_initialization,
        strict_function_types: settings.strict_function_types,
        no_implicit_any: settings.no_implicit_any,
        no_implicit_this: settings.no_implicit_this,
        allow_unreachable_code: settings.allow_unreachable_code,
        allow_unused_labels: settings.allow_unused_labels,
        no_fallthrough_cases_in_switch: settings.no_fallthrough_cases_in_switch,
        no_implicit_returns: settings.no_implicit_returns,
    }
}

/// Build project-level `CompilerOptions` from test runner settings.
///
/// For multi-target tests (e.g. `@target: es5,es2015`), the test runner aggregates
/// error codes from all variant baselines into a single expected set. We collect
/// all parsed targets so that config validation can emit diagnostics (e.g. TS5107)
/// for any deprecated target in the list, matching the aggregation.
fn compiler_options_from_settings(
    settings: &crate::typescript::meta::CompilerSettings,
) -> Vec<oxc_project::CompilerOptions> {
    let targets: Vec<_> = settings
        .targets
        .iter()
        .filter_map(|t| oxc_project::ScriptTarget::from_str_option(t))
        .collect();
    if targets.is_empty() {
        vec![oxc_project::CompilerOptions::default()]
    } else {
        targets.into_iter().map(|t| oxc_project::CompilerOptions { target: Some(t) }).collect()
    }
}

/// Extract the first target from test settings for lib file loading.
fn target_from_settings(
    settings: &crate::typescript::meta::CompilerSettings,
) -> Option<oxc_project::ScriptTarget> {
    settings
        .targets
        .first()
        .and_then(|t| oxc_project::ScriptTarget::from_str_option(t))
}

// ================================
// Checker (.types baseline conformance)
// ================================

/// Check if a test should be skipped because ALL its compiler option combos
/// use features tsgo doesn't support. When a test has multiple targets/modules,
/// tsgo runs each combo separately and skips only the unsupported ones.
/// We skip the whole test only when every combo would be skipped.
fn should_skip_tsgo(settings: &crate::typescript::meta::CompilerSettings) -> bool {
    // If all modules are unsupported, skip
    let all_modules_unsupported = !settings.modules.is_empty() && settings.modules.iter().all(|m| {
        let lower = m.to_lowercase();
        lower == "amd" || lower == "umd" || lower == "system"
    });
    if all_modules_unsupported {
        return true;
    }

    // If all targets are ES5, skip
    let all_targets_unsupported = !settings.targets.is_empty() && settings.targets.iter().all(|t| {
        t.to_lowercase() == "es5"
    });
    if all_targets_unsupported {
        return true;
    }

    false
}

/// Get the tsgo baselines root directory.
fn tsgo_baselines_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("TSGO_BASELINES")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                format!("{home}/dev/typescript-go/testdata/baselines/reference/submodule")
            }),
    )
}

/// Extract the relative path after `tests/cases/` without extension.
/// e.g. `typescript/tests/cases/compiler/foo.ts` -> `compiler/foo`
fn test_rel_stem(test_path: &Path) -> Option<String> {
    let path_str = test_path.to_string_lossy();
    let after_cases = path_str.split("tests/cases/").nth(1)?;
    let without_ext = after_cases.strip_suffix(".ts")
        .or_else(|| after_cases.strip_suffix(".tsx"))?;
    Some(without_ext.to_string())
}

/// Resolve the tsgo baseline content for a given extension (e.g. ".types", ".errors.txt").
/// If tsgo has a full baseline file, returns its content.
/// If tsgo has a .diff file, applies it to the tsc baseline to reconstruct tsgo's expected output.
/// Returns None if tsgo has no baseline for this test.
fn resolve_tsgo_baseline_content(test_path: &Path, extension: &str, tsc_content: Option<&str>) -> Option<String> {
    let tsgo_dir = tsgo_baselines_dir();
    let rel_stem = test_rel_stem(test_path)?;

    // Try full baseline first
    let full_path = tsgo_dir.join(format!("{rel_stem}{extension}"));
    if let Ok(content) = fs::read_to_string(&full_path) {
        return Some(content);
    }

    // Try .diff file — apply it to the tsc baseline
    let diff_path = tsgo_dir.join(format!("{rel_stem}{extension}.diff"));
    if let Ok(diff_content) = fs::read_to_string(&diff_path) {
        if let Some(tsc) = tsc_content {
            return Some(apply_tsgo_diff(tsc, &diff_content));
        }
    }

    None
}

/// Apply a tsgo-format diff to a base string.
///
/// The diff format uses:
///   `@@= skipped -N, +N lines =@@` to skip N identical lines
///   `-line` for lines only in old (removed)
///   `+line` for lines only in new (added)
///   ` line` (leading space) for context lines (unchanged)
fn apply_tsgo_diff(base: &str, diff: &str) -> String {
    use lazy_regex::{Lazy, Regex, lazy_regex};

    static SKIP_RE: Lazy<Regex> = lazy_regex!(r"^@@= skipped -(\d+), \+(\d+) lines =@@$");

    let base_lines: Vec<&str> = base.lines().collect();
    let mut result: Vec<&str> = Vec::new();
    let mut base_idx: usize = 0;

    let diff_lines: Vec<&str> = diff.lines().collect();
    let mut di = 0;

    // Skip the header lines (--- old, +++ new)
    while di < diff_lines.len() {
        let line = diff_lines[di];
        if line.starts_with("@@=") {
            break;
        }
        di += 1;
    }

    while di < diff_lines.len() {
        let line = diff_lines[di];

        if let Some(cap) = SKIP_RE.captures(line) {
            let skip_old: usize = cap[1].parse().unwrap();
            // Copy skipped lines from base
            let end = (base_idx + skip_old).min(base_lines.len());
            for &bl in &base_lines[base_idx..end] {
                result.push(bl);
            }
            base_idx = end;
            di += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix('-') {
            // Removed line — skip it in the base
            base_idx += 1;
            let _ = rest; // consumed
            di += 1;
        } else if let Some(rest) = line.strip_prefix('+') {
            // Added line — add to result (rest is owned by diff_lines which lives long enough)
            result.push(rest);
            di += 1;
        } else if let Some(rest) = line.strip_prefix(' ') {
            // Context line — copy from base, advance both
            result.push(rest);
            base_idx += 1;
            di += 1;
        } else {
            // Empty line or unexpected — treat as context
            if base_idx < base_lines.len() {
                result.push(base_lines[base_idx]);
                base_idx += 1;
            }
            di += 1;
        }
    }

    // Copy remaining base lines
    for &bl in &base_lines[base_idx..] {
        result.push(bl);
    }

    result.join("\n")
}

/// Generate the full .types baseline text from collected type entries,
/// matching tsgo's format: file header, source lines interleaved with
/// `>expr_text : type_string` entries.
fn generate_types_baseline(
    filename: &str,
    source: &str,
    types: &[(u32, String, String)],
) -> String {
    let mut out = String::new();

    // File header
    out.push_str(&format!("//// [tests/cases/{filename}] ////\n"));
    out.push('\n');

    // Get just the filename part for the === header
    let basename = Path::new(filename).file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| filename.to_string());
    out.push_str(&format!("=== {basename} ===\n"));

    // Interleave source lines with type entries
    let lines: Vec<&str> = source.split('\n').collect();
    let mut type_idx = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        out.push_str(line);
        out.push('\n');

        // Emit all type entries for this line
        while type_idx < types.len() && types[type_idx].0 == line_idx as u32 {
            let (_, ref expr_text, ref type_str) = types[type_idx];
            out.push_str(&format!(">{expr_text} : {type_str}\n"));
            type_idx += 1;
        }
    }

    out
}

/// Generate the full .types baseline text for a multi-file test.
fn generate_types_baseline_multi(
    units: &[(&str, &str, &[(u32, String, String)])],
    test_path: &str,
) -> String {
    let mut out = String::new();

    // File header referencing the test source
    out.push_str(&format!("//// [tests/cases/{test_path}] ////\n"));

    for (filename, source, types) in units {
        out.push('\n');
        out.push_str(&format!("=== {filename} ===\n"));

        let lines: Vec<&str> = source.split('\n').collect();
        let mut type_idx = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            out.push_str(line);
            out.push('\n');

            while type_idx < types.len() && types[type_idx].0 == line_idx as u32 {
                let (_, ref expr_text, ref type_str) = types[type_idx];
                out.push_str(&format!(">{expr_text} : {type_str}\n"));
                type_idx += 1;
            }
        }
    }

    out
}

/// Normalize a .types baseline for comparison: strip \r, underline lines,
/// and trailing whitespace.
fn normalize_types_baseline(s: &str) -> String {
    s.lines()
        // Strip underline lines: lines like ">  : ^^^^^^"
        .filter(|l| {
            if let Some(rest) = l.strip_prefix('>') {
                let trimmed = rest.trim_start();
                if let Some(after_colon) = trimmed.strip_prefix(": ") {
                    // If everything after ": " is just ^ and spaces, it's an underline
                    if !after_colon.is_empty() && after_colon.chars().all(|c| c == '^' || c == ' ') {
                        return false;
                    }
                }
            }
            true
        })
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// Partial matching: sequential (expr, type) comparison.
/// Returns Passed if all assertions match, or the first Mismatch.
fn partial_match_types(
    assertions: &[(String, String)],
    actual: &[(u32, String, String)],
) -> TestResult {
    let mut actual_iter = actual.iter();
    for (expr_text, expected_type) in assertions {
        let mut found = false;
        for (_, act_text, act_type) in actual_iter.by_ref() {
            if act_text == expr_text {
                if act_type != expected_type {
                    return TestResult::Mismatch(
                        "checker",
                        format!(">{expr_text} : {act_type}"),
                        format!(">{expr_text} : {expected_type}"),
                    );
                }
                found = true;
                break;
            }
        }
        if !found {
            return TestResult::Mismatch(
                "checker",
                String::new(),
                format!(">{expr_text} : {expected_type}"),
            );
        }
    }

    TestResult::Passed
}

pub fn run_checker_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    let tsc_baselines_dir = workspace_root().join("typescript/tests/baselines/reference");

    files
        .par_iter()
        .filter_map(|f| {
            // Skip files with expected errors (they may not parse)
            if !f.error_codes.is_empty() {
                return None;
            }

            // Skip tests with features tsgo doesn't support
            if should_skip_tsgo(&f.settings) {
                return None;
            }

            let stem = f.path.file_stem()?.to_str()?;

            // Use tsgo baseline when available, fall back to tsc baseline
            // (tsgo stores no separate baseline when its output matches tsc)
            let tsc_path = tsc_baselines_dir.join(format!("{stem}.types"));
            let tsc_content = fs::read_to_string(&tsc_path).ok();
            let baseline_content = resolve_tsgo_baseline_content(
                &f.path,
                ".types",
                tsc_content.as_deref(),
            ).or(tsc_content)?;

            // Extract relative path for baseline generation (e.g. "compiler/foo.ts")
            let test_rel_path = f.path.to_string_lossy()
                .split("tests/cases/")
                .nth(1)
                .unwrap_or(&f.path.to_string_lossy())
                .to_string();

            let options = checker_options_from_settings(&f.settings);
            let target = target_from_settings(&f.settings);

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if f.units.len() == 1 {
                    run_checker_single(
                        &f.units[0].content,
                        f.units[0].source_type,
                        &baseline_content,
                        &test_rel_path,
                        options,
                        target,
                    )
                } else {
                    run_checker_multi(
                        &f.units,
                        &baseline_content,
                        &test_rel_path,
                        options,
                    )
                }
            }));

            let result = match result {
                Ok(r) => r,
                Err(_) => TestResult::ParseError("panic during checking".to_string(), true),
            };

            Some(CoverageResult { path: f.path.clone(), should_fail: false, result })
        })
        .collect()
}

fn run_checker_single(
    source: &str,
    source_type: SourceType,
    baseline_content: &str,
    test_rel_path: &str,
    options: oxc_checker::CheckerOptions,
    target: Option<oxc_project::ScriptTarget>,
) -> TestResult {
    let assertions = parse_types_baseline(baseline_content);
    if assertions.is_empty() {
        return TestResult::Passed;
    }

    let type_arena = oxc_types::TypeArena::with_capacity(64);
    let files = vec![(
        PathBuf::from("/virtual/test.ts"),
        source.to_string(),
        source_type,
    )];
    let mut project =
        oxc_project::Project::new_multi_from_sources_with_target(&type_arena, files, options, target);
    project.check_all();

    let file_idx = project.lib_file_count();
    let actual = project.with_checker(file_idx, |checker, program| {
        collect_checker_types(checker, program, source)
    });
    let Some(actual) = actual else {
        return TestResult::ParseError("file not checked".to_string(), false);
    };

    // Tier 1: Full text comparison
    let actual_text = generate_types_baseline(test_rel_path, source, &actual);
    if normalize_types_baseline(&actual_text) == normalize_types_baseline(baseline_content) {
        return TestResult::Passed;
    }

    // Tier 2: Partial (expr, type) matching
    let partial = partial_match_types(&assertions, &actual);
    if partial == TestResult::Passed {
        // Partial matched but full text didn't — flag it
        return TestResult::Mismatch(
            "checker_text",
            actual_text,
            baseline_content.to_string(),
        );
    }

    // Tier 3: Fail
    partial
}

fn run_checker_multi(
    units: &[crate::typescript::meta::TestUnitData],
    baseline_content: &str,
    test_rel_path: &str,
    options: oxc_checker::CheckerOptions,
) -> TestResult {
    let assertions = parse_types_baseline(baseline_content);
    if assertions.is_empty() {
        return TestResult::Passed;
    }

    let ts_units: Vec<_> = units
        .iter()
        .filter(|u| SourceType::from_path(Path::new(&u.name)).is_ok())
        .collect();

    if ts_units.is_empty() {
        return TestResult::Passed;
    }

    let unit_sources: Vec<&str> = ts_units.iter().map(|u| u.content.as_str()).collect();

    let files: Vec<(PathBuf, String, SourceType)> = ts_units
        .iter()
        .map(|u| (virtual_path_from_unit_name(&u.name), u.content.clone(), u.source_type))
        .collect();

    let type_arena = oxc_types::TypeArena::with_capacity(64);
    let mut project = oxc_project::Project::new_multi_from_sources(&type_arena, files, options);
    project.check_all();

    // Collect types per file for baseline generation
    let mut per_file_types: Vec<Vec<(u32, String, String)>> = Vec::new();
    let mut all_types: Vec<(u32, String, String)> = Vec::new();
    for (i, file_idx) in project.user_file_range().enumerate() {
        let source = unit_sources[i];
        let types = project.with_checker(file_idx, |checker, program| {
            collect_checker_types(checker, program, source)
        });
        let types = types.unwrap_or_default();
        all_types.extend(types.iter().cloned());
        per_file_types.push(types);
    }

    // Tier 1: Full text comparison
    let unit_names: Vec<String> = ts_units.iter()
        .map(|u| {
            Path::new(&u.name).file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| u.name.clone())
        })
        .collect();
    let baseline_units: Vec<(&str, &str, &[(u32, String, String)])> = unit_names.iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), unit_sources[i], per_file_types[i].as_slice()))
        .collect();

    let actual_text = generate_types_baseline_multi(&baseline_units, test_rel_path);
    if normalize_types_baseline(&actual_text) == normalize_types_baseline(baseline_content) {
        return TestResult::Passed;
    }

    // Tier 2: Partial matching
    let partial = partial_match_types(&assertions, &all_types);
    if partial == TestResult::Passed {
        return TestResult::Mismatch(
            "checker_text",
            actual_text,
            baseline_content.to_string(),
        );
    }

    // Tier 3: Fail
    partial
}

/// Parse `.types` baseline content into `(expression_text, expected_type)` pairs.
fn parse_types_baseline(content: &str) -> Vec<(String, String)> {
    let mut assertions = Vec::new();
    let mut in_source = false;

    for line in content.lines() {
        if line.starts_with("=== ") && line.ends_with(" ===") {
            in_source = true;
            continue;
        }
        if line.starts_with("//// [") || !in_source {
            continue;
        }
        if let Some(rest) = line.strip_prefix('>') {
            // Skip underline lines like ">  : ^^^^^^" or ">   : ^ ^^  ^^^^^^^^^"
            let trimmed = rest.trim_start();
            if trimmed.starts_with(": ") && trimmed[2..].chars().all(|c| c == '^' || c == ' ') {
                continue;
            }
            if let Some((expr, typ)) = rest.split_once(" : ") {
                assertions.push((expr.to_string(), typ.to_string()));
            }
        }
    }

    assertions
}

/// Walk the AST collecting `(line, source_text, type_string)` tuples for nodes
/// that tsc reports types for: expression nodes and declaration binding names.
/// Line is 0-based.
fn collect_checker_types<'a>(
    checker: &mut oxc_checker::Checker<'a>,
    program: &oxc::ast::ast::Program<'a>,
    source: &str,
) -> Vec<(u32, String, String)> {
    use oxc::ast_visit::Visit;

    let line_starts = compute_line_starts(source);
    let mut walker = TypeCollectorVisitor {
        checker,
        source,
        line_starts: &line_starts,
        results: Vec::new(),
        last_expression_type: None,
        super_class_types: rustc_hash::FxHashMap::default(),
        class_type_stack: Vec::new(),
    };
    walker.visit_program(program);
    walker.results
}

struct TypeCollectorVisitor<'a, 'b> {
    checker: &'b mut oxc_checker::Checker<'a>,
    source: &'b str,
    line_starts: &'b [usize],
    results: Vec<(u32, String, String)>,
    /// Stashed type string from the last visit_expression, so
    /// visit_static_member_expression can emit the property name
    /// with the same type as the parent member expression.
    last_expression_type: Option<String>,
    /// Pre-computed base types for class super_class expressions.
    /// Keyed by span key `(start << 32) | end`. tsc's .types walker
    /// reports the instance type (not `typeof X`) for heritage expressions;
    /// this map provides the override for visit_expression.
    super_class_types: rustc_hash::FxHashMap<u64, oxc_types::TypeId>,
    /// Stack of class instance types for resolving property types from the
    /// class type rather than re-inferring from the AST. Matches tsgo's
    /// `GetTypeAtLocation` behavior which goes through the unified widening path.
    class_type_stack: Vec<Option<oxc_types::TypeId>>,
}

impl TypeCollectorVisitor<'_, '_> {
    /// Compute 0-based line index for a byte offset.
    fn line_of(&self, offset: u32) -> u32 {
        line_of_offset(self.line_starts, offset as usize) as u32
    }
}

impl<'a> oxc::ast_visit::Visit<'a> for TypeCollectorVisitor<'a, '_> {
    fn visit_class(&mut self, class: &oxc::ast::ast::Class<'a>) {
        use oxc::span::GetSpan as _;

        // Resolve the class instance type for property type lookups.
        let class_instance_type = class.id.as_ref().and_then(|id| {
            id.symbol_id.get().map(|sid| self.checker.get_declared_type_of_symbol(sid))
        });

        // Pre-compute the base type for the super_class expression.
        // tsc's .types walker reports the instance type (e.g., "Base")
        // for heritage expressions, not the constructor type ("typeof Base").
        // See tsgo type_symbol_baseline.go:369-375.
        if let (Some(class_type), Some(super_class)) = (class_instance_type, &class.super_class) {
            if let oxc_types::TypeData::Structured(s) =
                self.checker.type_arena().get_data(class_type)
            {
                if let oxc_types::StructuredTypeKind::Interface {
                    resolved_base_types, ..
                } = &s.kind
                {
                    if let Some(&base_type) = resolved_base_types.first() {
                        let span = super_class.span();
                        let key = (span.start as u64) << 32 | span.end as u64;
                        self.super_class_types.insert(key, base_type);
                    }
                }
            }
        }

        self.class_type_stack.push(class_instance_type);
        oxc::ast_visit::walk::walk_class(self, class);
        self.class_type_stack.pop();
    }

    fn visit_expression(&mut self, expr: &oxc::ast::ast::Expression<'a>) {
        use oxc::span::GetSpan as _;

        // Record the type for this expression before recursing
        let span = expr.span();
        if (span.start as usize) < self.source.len() && (span.end as usize) <= self.source.len() {
            let expr_text = &self.source[span.start as usize..span.end as usize];
            // For class extends expressions, use the pre-computed base type
            let key = (span.start as u64) << 32 | span.end as u64;
            let type_id = if let Some(&base_type) = self.super_class_types.get(&key) {
                base_type
            } else {
                self.checker.get_type_at_location(expr)
            };
            let type_str = self.checker.type_to_string(type_id);
            let line = self.line_of(span.start);
            self.results.push((line, expr_text.to_string(), type_str.clone()));
            // Stash for visit_static_member_expression to pick up
            self.last_expression_type = Some(type_str);
        }

        // Continue walking into sub-expressions
        oxc::ast_visit::walk::walk_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &oxc::ast::ast::StaticMemberExpression<'a>) {
        use oxc::span::GetSpan as _;

        // tsc emits the property name with the type of the member access.
        // e.g., for `this.ships`, baseline has both:
        //   >this.ships : Ship[]     (from visit_expression on the parent)
        //   >ships : Ship[]          (this handler — the property name)
        // The type was stashed by visit_expression just before walk_expression
        // recursed into this node.
        if let Some(parent_type) = self.last_expression_type.take() {
            let prop_span = expr.property.span();
            if (prop_span.start as usize) < self.source.len()
                && (prop_span.end as usize) <= self.source.len()
            {
                let prop_text = &self.source[prop_span.start as usize..prop_span.end as usize];
                let line = self.line_of(prop_span.start);
                self.results.push((line, prop_text.to_string(), parent_type));
            }
        }

        oxc::ast_visit::walk::walk_static_member_expression(self, expr);
    }

    fn visit_object_property(&mut self, prop: &oxc::ast::ast::ObjectProperty<'a>) {
        use oxc::span::GetSpan as _;

        // Emit the property key's type (matching tsc's walker).
        // The property key text (e.g., "a") gets the property's value type.
        if prop.kind == oxc::ast::ast::PropertyKind::Init {
            if prop.key.static_name().is_some() {
                let key_span = prop.key.span();
                if (key_span.start as usize) < self.source.len()
                    && (key_span.end as usize) <= self.source.len()
                {
                    let prop_type = self.checker.get_type_of_expression(&prop.value, None, oxc_checker::CheckMode::TYPE_ONLY);
                    let widened = self.checker.get_widened_literal_type(prop_type);
                    let key_text =
                        &self.source[key_span.start as usize..key_span.end as usize];
                    let line = self.line_of(key_span.start);
                    self.results
                        .push((line, key_text.to_string(), self.checker.type_to_string(widened)));
                }
            }
        }

        oxc::ast_visit::walk::walk_object_property(self, prop);
    }

    fn visit_property_definition(&mut self, prop: &oxc::ast::ast::PropertyDefinition<'a>) {
        use oxc::span::GetSpan as _;
        // Emit the property key's type for class property definitions.
        // Look up the property type from the class instance/constructor type
        // so that widening applied during type construction is reflected.
        // This matches tsgo's GetTypeAtLocation which goes through the
        // unified getWidenedTypeForVariableLikeDeclaration path.
        if let Some(name) = prop.key.static_name() {
            let key_span = prop.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                // Try to look up property from the class type (instance or constructor)
                let prop_type = if !prop.r#static {
                    // Instance property: look up from the class instance type
                    self.class_type_stack
                        .last()
                        .copied()
                        .flatten()
                        .and_then(|ct| self.checker.get_property_of_type(ct, &name))
                } else {
                    // Static property: look up from the constructor (value) type
                    // via get_type_of_symbol on the class symbol
                    self.class_type_stack
                        .last()
                        .copied()
                        .flatten()
                        .and_then(|ct| {
                            let sym = self.checker.type_arena().get_symbol(ct);
                            sym.map(|(_, sid)| self.checker.get_type_of_symbol(sid))
                        })
                        .and_then(|ctor_type| {
                            self.checker.get_property_of_type(ctor_type, &name)
                        })
                };

                // Fall back to re-inferring from the AST if lookup failed
                let prop_type = prop_type.unwrap_or_else(|| {
                    if let Some(ann) = &prop.type_annotation {
                        self.checker.get_type_from_type_node(&ann.type_annotation)
                    } else if let Some(init) = &prop.value {
                        self.checker.get_type_of_expression(
                            init,
                            None,
                            oxc_checker::CheckMode::TYPE_ONLY,
                        )
                    } else {
                        self.checker.any_type
                    }
                });

                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                let line = self.line_of(key_span.start);
                self.results
                    .push((line, key_text.to_string(), self.checker.type_to_string(prop_type)));
            }
        }
        oxc::ast_visit::walk::walk_property_definition(self, prop);
    }

    fn visit_method_definition(&mut self, method: &oxc::ast::ast::MethodDefinition<'a>) {
        use oxc::span::GetSpan as _;
        // Emit the method key's type for class method definitions
        if let Some(_name) = method.key.static_name() {
            let key_span = method.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let sig = self.checker.build_signature_from_function(&method.value);
                let method_type = self.checker.create_function_type(sig);
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                let line = self.line_of(key_span.start);
                self.results
                    .push((line, key_text.to_string(), self.checker.type_to_string(method_type)));
            }
        }
        oxc::ast_visit::walk::walk_method_definition(self, method);
    }

    fn visit_ts_property_signature(&mut self, prop: &oxc::ast::ast::TSPropertySignature<'a>) {
        use oxc::span::GetSpan as _;
        // Emit the property key's type for interface/type literal property signatures
        if let Some(_name) = prop.key.static_name() {
            let key_span = prop.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let prop_type = if let Some(ann) = &prop.type_annotation {
                    self.checker.get_type_from_type_node(&ann.type_annotation)
                } else {
                    self.checker.any_type
                };
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                let line = self.line_of(key_span.start);
                self.results
                    .push((line, key_text.to_string(), self.checker.type_to_string(prop_type)));
            }
        }
        oxc::ast_visit::walk::walk_ts_property_signature(self, prop);
    }

    fn visit_ts_method_signature(&mut self, method: &oxc::ast::ast::TSMethodSignature<'a>) {
        use oxc::span::GetSpan as _;
        // Emit the method key's type for interface/type literal method signatures
        if let Some(_name) = method.key.static_name() {
            let key_span = method.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let sig = self.checker.build_signature_from_params(
                    &method.params,
                    method.return_type.as_deref(),
                );
                let method_type = self.checker.create_function_type(sig);
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                let line = self.line_of(key_span.start);
                self.results
                    .push((line, key_text.to_string(), self.checker.type_to_string(method_type)));
            }
        }
        oxc::ast_visit::walk::walk_ts_method_signature(self, method);
    }

    fn visit_ts_enum_member(&mut self, member: &oxc::ast::ast::TSEnumMember<'a>) {
        use oxc::span::GetSpan as _;
        // Emit the enum member name's type
        let name_span = member.id.span();
        if (name_span.start as usize) < self.source.len()
            && (name_span.end as usize) <= self.source.len()
        {
            let member_type = if let Some(init) = &member.initializer {
                self.checker.get_type_of_expression(init, None, oxc_checker::CheckMode::TYPE_ONLY)
            } else {
                self.checker.any_type
            };
            let name_text =
                &self.source[name_span.start as usize..name_span.end as usize];
            let line = self.line_of(name_span.start);
            self.results
                .push((line, name_text.to_string(), self.checker.type_to_string(member_type)));
        }
        oxc::ast_visit::walk::walk_ts_enum_member(self, member);
    }

    fn visit_binding_identifier(&mut self, id: &oxc::ast::ast::BindingIdentifier<'a>) {
        // Record the type for declaration binding names.
        // For type declarations (class, interface, enum, type alias), use the
        // declared type, matching tsc's getTypeOfNode which calls
        // getDeclaredTypeOfSymbol for type declaration names.
        if let Some(symbol_id) = id.symbol_id.get() {
            use oxc::ast::AstKind;
            let node_id = self.checker.semantic().scoping().symbol_declaration(symbol_id);
            let node = self.checker.semantic().nodes().get_node(node_id);
            let type_id = match node.kind() {
                AstKind::Class(_)
                | AstKind::TSInterfaceDeclaration(_)
                | AstKind::TSTypeAliasDeclaration(_)
                | AstKind::TSEnumDeclaration(_) => {
                    self.checker.get_declared_type_of_symbol(symbol_id)
                }
                _ => self.checker.get_type_of_symbol(symbol_id),
            };
            let line = self.line_of(id.span.start);
            self.results.push((line, id.name.to_string(), self.checker.type_to_string(type_id)));
        }

        oxc::ast_visit::walk::walk_binding_identifier(self, id);
    }

}

// ================================
// Checker (.errors.txt error code conformance)
// ================================

/// A located error: (line, col, code). Line and col are 1-based to match tsc output.
/// For file-less errors (e.g. compiler option validation), line=0, col=0.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LocatedError {
    line: u32,
    col: u32,
    code: String,
}

/// Parse (line, col, code) tuples from .errors.txt header lines.
///
/// Header lines look like:
///   `filename(line,col): error TSxxxx: message`
/// File-less errors look like:
///   `error TSxxxx: message`
fn parse_expected_locations(error_files: &[String]) -> Vec<LocatedError> {
    use lazy_regex::{Lazy, Regex, lazy_regex};

    // Matches header lines with location: "filename(line,col): error TSxxxx:"
    static LOCATED: Lazy<Regex> =
        lazy_regex!(r"^[^\s(]+\((\d+),(\d+)\):\s+error\s+TS(\d+):");
    // Matches file-less errors: "error TSxxxx:" at start of line (no filename prefix)
    static FILELESS: Lazy<Regex> =
        lazy_regex!(r"^error\s+TS(\d+):");

    let mut locations = Vec::new();
    for error_file in error_files {
        for line in error_file.lines() {
            if let Some(cap) = LOCATED.captures(line) {
                locations.push(LocatedError {
                    line: cap[1].parse().unwrap(),
                    col: cap[2].parse().unwrap(),
                    code: cap[3].to_string(),
                });
            } else if let Some(cap) = FILELESS.captures(line) {
                locations.push(LocatedError {
                    line: 0,
                    col: 0,
                    code: cap[1].to_string(),
                });
            }
        }
    }
    locations.sort();
    locations
}

/// Compute line starts (byte offsets of each line's first character) for a source string.
fn compute_line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// Find the 0-based line index for a byte offset using the line-starts table.
fn line_of_offset(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i - 1,
    }
}

/// Count UTF-16 code units in a string slice. This matches tsc's column counting
/// (which uses UTF-16 offsets, not byte offsets or Unicode scalar counts).
fn utf16_len(s: &str) -> u32 {
    s.chars().map(|c| if c as u32 > 0xFFFF { 2u32 } else { 1u32 }).sum()
}

/// Extract (line, col, code) from our diagnostics. Line and col are 1-based.
/// `sources` maps virtual file paths to their source text.
fn extract_actual_locations(
    diagnostics: &[(PathBuf, Vec<OxcDiagnostic>)],
    sources: &[(PathBuf, String)],
    option_diagnostics: &[OxcDiagnostic],
) -> Vec<LocatedError> {
    // Build a lookup from path to (source, line_starts)
    let source_map: Vec<(&Path, &str, Vec<usize>)> = sources
        .iter()
        .map(|(path, src)| (path.as_path(), src.as_str(), compute_line_starts(src)))
        .collect();

    let mut locations = Vec::new();

    // File-less diagnostics from compiler option validation
    for d in option_diagnostics {
        if let Some(code) = d.code.number.as_ref() {
            locations.push(LocatedError {
                line: 0,
                col: 0,
                code: code.to_string(),
            });
        }
    }

    for (path, diags) in diagnostics {
        // Find the source for this file
        let source_info = source_map.iter().find(|(p, _, _)| *p == path.as_path());

        for d in diags {
            let code = match d.code.number.as_ref() {
                Some(c) => c.to_string(),
                None => continue,
            };

            if let Some((_, src, line_starts)) = source_info {
                if let Some(offset) = d.labels.as_ref().and_then(|labels| {
                    labels.first().map(|l| l.offset())
                }) {
                    if offset < src.len() {
                        let line_idx = line_of_offset(line_starts, offset);
                        let line_start = line_starts[line_idx];
                        // UTF-16 column, 1-based
                        let col = utf16_len(&src[line_start..offset]) + 1;
                        locations.push(LocatedError {
                            line: (line_idx as u32) + 1,
                            col,
                            code,
                        });
                        continue;
                    }
                }
            }

            // No label or offset out of range — treat as file-less
            locations.push(LocatedError { line: 0, col: 0, code });
        }
    }

    locations.sort();
    locations
}

/// Generate the full .errors.txt baseline text from diagnostics, matching tsc's format.
///
/// Format:
/// 1. Header lines: `filename(line,col): error TSxxxx: message` (sorted by file, then offset)
/// 2. Blank line separator
/// 3. For each file: `==== filename (N errors) ====`
///    followed by interleaved source lines + squiggle/error annotations
fn generate_error_baseline(
    diagnostics: &[(PathBuf, Vec<OxcDiagnostic>)],
    sources: &[(PathBuf, String)],
    option_diagnostics: &[OxcDiagnostic],
) -> String {
    let mut out = String::new();

    // Collect all diagnostics with their file info for header lines
    struct DiagEntry<'a> {
        filename: String,
        line: u32,       // 1-based
        col: u32,        // 1-based, UTF-16
        code: &'a str,
        message: String,
        offset: usize,   // byte offset in source (for squiggle generation)
        len: usize,      // byte length of span
    }

    // Build source lookup
    let source_map: Vec<(&Path, &str, Vec<usize>)> = sources
        .iter()
        .map(|(path, src)| (path.as_path(), src.as_str(), compute_line_starts(src)))
        .collect();

    let mut file_less_entries: Vec<String> = Vec::new();
    // Map from source index -> list of diagnostics for that file
    let mut per_file_entries: Vec<Vec<DiagEntry<'_>>> = (0..sources.len()).map(|_| Vec::new()).collect();

    // File-less diagnostics from compiler option validation
    for d in option_diagnostics {
        if let Some(code) = d.code.number.as_ref() {
            file_less_entries.push(format!("error TS{}: {}", code, d.message));
        }
    }

    for (path, diags) in diagnostics {
        let file_idx = source_map.iter().position(|(p, _, _)| *p == path.as_path());

        for d in diags {
            let code = match d.code.number.as_ref() {
                Some(c) => &**c,
                None => continue,
            };

            let msg = d.message.to_string();

            if let Some(fi) = file_idx {
                let (_, src, line_starts) = &source_map[fi];
                if let Some((offset, len)) = d.labels.as_ref().and_then(|labels| {
                    labels.first().map(|l| (l.offset(), l.len()))
                }) {
                    if offset < src.len() {
                        let line_idx = line_of_offset(line_starts, offset);
                        let line_start = line_starts[line_idx];
                        let col = utf16_len(&src[line_start..offset]) + 1;

                        // Get just the filename from the path
                        let filename = path.file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string());

                        per_file_entries[fi].push(DiagEntry {
                            filename,
                            line: (line_idx as u32) + 1,
                            col,
                            code,
                            message: msg,
                            offset,
                            len,
                        });
                        continue;
                    }
                }
            }

            // No source location — file-less
            file_less_entries.push(format!("error TS{code}: {msg}"));
        }
    }

    // Sort per-file entries by offset
    for entries in &mut per_file_entries {
        entries.sort_by_key(|e| (e.offset, e.len));
    }

    // === Part 1: Header lines ===
    // File-less errors first
    for entry in &file_less_entries {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(entry);
    }

    // Then located errors, sorted by file order then offset
    for entries in &per_file_entries {
        for e in entries {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("{}({},{}): error TS{}: {}", e.filename, e.line, e.col, e.code, e.message));
        }
    }

    // === Part 2: Blank line separator + interleaved source ===
    // tsc emits two blank lines between the header block and the interleaved block
    if !out.is_empty() {
        out.push_str("\n\n\n");
    }

    // File-less errors appear as `!!! error` lines before any file sections
    for entry in &file_less_entries {
        out.push_str(&format!("!!! {entry}\n"));
    }

    for (fi, (_, src, line_starts)) in source_map.iter().enumerate() {
        let entries = &per_file_entries[fi];
        let error_count = entries.len();
        let filename = sources[fi].0.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| sources[fi].0.to_string_lossy().to_string());

        // File section header
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("==== {filename} ({error_count} errors) ===="));

        // Use split('\n') instead of lines() to match tsc's behavior:
        // if source ends with '\n', tsc emits an extra indented empty line.
        let lines: Vec<&str> = src.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let this_line_start = line_starts[line_idx];
            let next_line_start = if line_idx + 1 < line_starts.len() {
                line_starts[line_idx + 1]
            } else {
                src.len()
            };

            // Emit source line
            out.push('\n');
            out.push_str("    ");
            out.push_str(line);

            // Find all diagnostics that overlap this line
            for e in entries {
                let err_end = e.offset + e.len;
                if err_end >= this_line_start
                    && (e.offset < next_line_start || line_idx == lines.len() - 1)
                {
                    let relative_offset = e.offset as isize - this_line_start as isize;
                    let squiggle_start = relative_offset.max(0) as usize;
                    let length = e.len - (this_line_start.saturating_sub(e.offset));
                    let squiggle_end = squiggle_start + length;
                    let squiggle_end = squiggle_end.min(line.len()).max(squiggle_start);

                    // Preserve leading whitespace, replace non-whitespace with spaces
                    let prefix: String = line[..squiggle_start]
                        .chars()
                        .map(|c| if c.is_whitespace() { c } else { ' ' })
                        .collect();

                    out.push('\n');
                    out.push_str("    ");
                    out.push_str(&prefix);
                    // Use rune count for squiggle width (matching tsc)
                    let squiggle_text = &line[squiggle_start..squiggle_end];
                    let squiggle_count = squiggle_text.chars().count().max(1);
                    out.push_str(&"~".repeat(squiggle_count));

                    // Emit message if the error ends on this line
                    if line_idx == lines.len() - 1 || next_line_start > err_end {
                        out.push('\n');
                        out.push_str(&format!("!!! error TS{}: {}", e.code, e.message));
                    }
                }
            }
        }

        // Trailing newline after each file section
        out.push('\n');
    }

    out
}



pub fn run_checker_errors_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter_map(|f| {
            // Only test files that have expected error codes
            if f.error_codes.is_empty() {
                return None;
            }

            // Skip tests with features tsgo doesn't support
            if should_skip_tsgo(&f.settings) {
                return None;
            }

            // Use tsgo baseline when available, fall back to tsc baselines
            // (tsgo stores no separate baseline when its output matches tsc)
            let tsgo_content = resolve_tsgo_baseline_content(&f.path, ".errors.txt", None)
                .or_else(|| {
                    f.error_files.iter().find_map(|tsc_ef| {
                        resolve_tsgo_baseline_content(&f.path, ".errors.txt", Some(tsc_ef))
                    })
                });

            let error_files: Cow<'_, [String]> = match tsgo_content {
                Some(content) => Cow::Owned(vec![content]),
                None => Cow::Borrowed(f.error_files.as_slice()),
            };

            if error_files.is_empty() {
                return None;
            }

            let options = checker_options_from_settings(&f.settings);
            let compiler_options_list = compiler_options_from_settings(&f.settings);
            let target = target_from_settings(&f.settings);

            // Extract the test filename (e.g. "accessorWithLineTerminator.ts")
            // for use as the virtual path — must match baseline filenames.
            let test_filename = f.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "test.ts".to_string());

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if f.units.len() == 1 {
                    run_checker_errors_single(
                        &f.units[0].content,
                        f.units[0].source_type,
                        &error_files,
                        &test_filename,
                        options,
                        &compiler_options_list,
                        target,
                    )
                } else {
                    run_checker_errors_multi(
                        &f.units,
                        &error_files,
                        options,
                        &compiler_options_list,
                    )
                }
            }));

            let result = match result {
                Ok(r) => r,
                Err(_) => TestResult::ParseError("panic during checking".to_string(), true),
            };

            Some(CoverageResult { path: f.path.clone(), should_fail: false, result })
        })
        .collect()
}

fn run_checker_errors_single(
    source: &str,
    source_type: SourceType,
    error_files: &[String],
    test_filename: &str,
    options: oxc_checker::CheckerOptions,
    compiler_options_list: &[oxc_project::CompilerOptions],
    target: Option<oxc_project::ScriptTarget>,
) -> TestResult {
    let mut option_diagnostics: Vec<OxcDiagnostic> = Vec::new();

    // Validate compiler options (emits e.g. TS5107 for deprecated target=ES5).
    for compiler_options in compiler_options_list {
        option_diagnostics.extend(oxc_project::validate_compiler_options(compiler_options));
    }

    // Use the original test filename as the virtual path so it matches
    // the filenames in .errors.txt baselines.
    let type_arena = oxc_types::TypeArena::with_capacity(64);
    let path = PathBuf::from(format!("/virtual/{test_filename}"));
    let files = vec![(path.clone(), source.to_string(), source_type)];
    let mut project =
        oxc_project::Project::new_multi_from_sources_with_target(&type_arena, files, options, target);
    let result = project.check_all();

    let sources = vec![(path, source.to_string())];

    compare_errors(
        error_files,
        &result.diagnostics,
        &sources,
        &option_diagnostics,
    )
}

/// Convert a test unit filename to a virtual path under /virtual/.
///
/// Normalizes leading `./` or `/` so all units share a common parent
/// directory, enabling relative imports to resolve correctly.
fn virtual_path_from_unit_name(name: &str) -> PathBuf {
    let normalized = name
        .strip_prefix("./")
        .or_else(|| name.strip_prefix('/'))
        .unwrap_or(name);
    PathBuf::from(format!("/virtual/{normalized}"))
}

fn run_checker_errors_multi(
    units: &[crate::typescript::meta::TestUnitData],
    error_files: &[String],
    options: oxc_checker::CheckerOptions,
    compiler_options_list: &[oxc_project::CompilerOptions],
) -> TestResult {
    let mut option_diagnostics: Vec<OxcDiagnostic> = Vec::new();

    // Validate compiler options (same as single-unit path)
    for compiler_options in compiler_options_list {
        option_diagnostics.extend(oxc_project::validate_compiler_options(compiler_options));
    }

    // Build virtual file list, filtering out non-TS/JS files (package.json, etc.)
    let files: Vec<(PathBuf, String, SourceType)> = units
        .iter()
        .filter(|unit| SourceType::from_path(Path::new(&unit.name)).is_ok())
        .map(|unit| {
            let path = virtual_path_from_unit_name(&unit.name);
            (path, unit.content.clone(), unit.source_type)
        })
        .collect();

    if files.is_empty() {
        return TestResult::Passed;
    }

    // Match tsc's baseline file ordering. When the last unit contains require()
    // or triple-slash references, tsc treats it as the primary file (toBeCompiled)
    // and lists it first, with other files following. Otherwise, all files stay
    // in their original order.
    let mut sources: Vec<(PathBuf, String)> = files.iter()
        .map(|(path, src, _)| (path.clone(), src.clone()))
        .collect();
    if sources.len() > 1 {
        let last_content = &units.last().map(|u| u.content.as_str()).unwrap_or("");
        let has_require = last_content.contains("require(");
        let has_reference = last_content.contains("/// <reference");
        if has_require || has_reference {
            let last = sources.pop().unwrap();
            sources.insert(0, last);
        }
    }

    let type_arena = oxc_types::TypeArena::with_capacity(64);
    let mut project = oxc_project::Project::new_multi_from_sources(
        &type_arena,
        files,
        options,
    );
    let result = project.check_all();

    compare_errors(
        error_files,
        &result.diagnostics,
        &sources,
        &option_diagnostics,
    )
}

/// Three-tier comparison of actual vs expected errors:
/// 1. Location match: sorted multiset of (line, col, code)
/// 2. Full text match: verbatim .errors.txt comparison
///
/// When multiple .errors.txt files exist (different compiler option combos),
/// we compare against each independently — pass if any matches.
///
/// Returns Passed if location match passes, but uses "checker_errors_text"
/// tag for Mismatch when location matches but full text doesn't.
fn compare_errors(
    error_files: &[String],
    diagnostics: &[(PathBuf, Vec<OxcDiagnostic>)],
    sources: &[(PathBuf, String)],
    option_diagnostics: &[OxcDiagnostic],
) -> TestResult {
    let actual_locations = extract_actual_locations(diagnostics, sources, option_diagnostics);

    // Check location match against each error file independently.
    // Track which baseline(s) matched locations for the text comparison.
    let mut matched_baseline_idx: Option<usize> = None;
    for (i, ef) in error_files.iter().enumerate() {
        let expected = parse_expected_locations(std::slice::from_ref(ef));
        if expected == actual_locations {
            matched_baseline_idx = Some(i);
            break;
        }
    }

    if matched_baseline_idx.is_none() {
        // Location mismatch — show diff against the first error file
        let expected_locations = parse_expected_locations(std::slice::from_ref(
            error_files.first().unwrap_or(&String::new()),
        ));
        let actual_str = actual_locations.iter()
            .map(|e| format!("({},{}) TS{}", e.line, e.col, e.code))
            .collect::<Vec<_>>()
            .join("\n");
        let expected_str = expected_locations.iter()
            .map(|e| format!("({},{}) TS{}", e.line, e.col, e.code))
            .collect::<Vec<_>>()
            .join("\n");
        return TestResult::Mismatch("checker_errors", actual_str, expected_str);
    }

    let matched_idx = matched_baseline_idx.unwrap();

    // Location match passed — now check full text
    let actual_text = generate_error_baseline(diagnostics, sources, option_diagnostics);

    // Compare text against the baseline whose locations matched
    let matched_baseline = &error_files[matched_idx];
    if normalize_baseline(matched_baseline) == normalize_baseline(&actual_text) {
        return TestResult::Passed;
    }

    // Also check other baselines in case one matches text exactly
    let text_matches = error_files.iter().any(|ef| {
        normalize_baseline(ef) == normalize_baseline(&actual_text)
    });

    if text_matches {
        TestResult::Passed
    } else {
        // Location matched but full text didn't — show the matched baseline
        TestResult::Mismatch(
            "checker_errors_text",
            actual_text,
            matched_baseline.clone(),
        )
    }
}

/// Normalize a baseline string for comparison: trim trailing whitespace per line,
/// normalize line endings, trim trailing blank lines.
fn normalize_baseline(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

fn parse_estree_json_blocks<'a>(content: &'a str, section_kind: &str) -> Vec<&'a str> {
    let prefix = format!(":{section_kind}:\n```json\n");
    content
        .split("__ESTREE_TEST__")
        .skip(1)
        .filter_map(|section| {
            let json = section.strip_prefix(&prefix)?;
            json.strip_suffix("\n```\n").or_else(|| json.strip_suffix("\n```"))
        })
        .collect()
}
