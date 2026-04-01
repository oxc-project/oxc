//! Tool runner functions for coverage testing

use std::{borrow::Cow, fs, path::Path, sync::Arc};

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
// Checker (.types baseline conformance)
// ================================

pub fn run_checker_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    let baselines_dir = workspace_root().join("typescript/tests/baselines/reference");

    files
        .par_iter()
        .filter_map(|f| {
            // Skip files with expected errors (they may not parse)
            if !f.error_codes.is_empty() {
                return None;
            }
            // Only handle single-unit files for now
            if f.units.len() != 1 {
                return None;
            }

            // Derive .types baseline path from source path
            let stem = f.path.file_stem()?.to_str()?;
            let baseline_path = baselines_dir.join(format!("{stem}.types"));
            let baseline_content = fs::read_to_string(&baseline_path).ok()?;

            let source = &f.units[0].content;
            let source_type = f.units[0].source_type;
            let result = run_checker_single(source, source_type, &baseline_content);
            Some(CoverageResult { path: f.path.clone(), should_fail: false, result })
        })
        .collect()
}

fn run_checker_single(
    source: &str,
    source_type: SourceType,
    baseline_content: &str,
) -> TestResult {
    use oxc::semantic::SemanticBuilder;
    use oxc_checker::Checker;

    // Parse the .types baseline
    let assertions = parse_types_baseline(baseline_content);
    if assertions.is_empty() {
        return TestResult::Passed;
    }

    // Parse source → semantic → checker
    let type_arena = oxc_types::TypeArena::with_capacity(64);
    let project = oxc_project::Project::new(&type_arena);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if !parsed.errors.is_empty() {
        return TestResult::ParseError(
            parsed.errors.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("\n"),
            false,
        );
    }
    let program = &parsed.program;
    let semantic = SemanticBuilder::new().build(program).semantic;
    let mut checker = Checker::new_with_host(&semantic, &type_arena, &project, String::new(), 1);
    checker.check_program(program);

    // Collect computed types from AST (uses cached types from check_program)
    let actual = collect_checker_types(&mut checker, program, source);

    // Match assertions against actual
    let mut actual_iter = actual.iter();
    for (expr_text, expected_type) in &assertions {
        let mut found = false;
        for (act_text, act_type) in actual_iter.by_ref() {
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

/// Walk the AST collecting `(source_text, type_string)` pairs for nodes
/// that tsc reports types for: expression nodes and declaration binding names.
fn collect_checker_types<'a>(
    checker: &mut oxc_checker::Checker<'a>,
    program: &oxc::ast::ast::Program<'a>,
    source: &str,
) -> Vec<(String, String)> {
    use oxc::ast_visit::Visit;

    let mut walker = TypeCollectorVisitor { checker, source, results: Vec::new(), last_expression_type: None };
    walker.visit_program(program);
    walker.results
}

struct TypeCollectorVisitor<'a, 'b> {
    checker: &'b mut oxc_checker::Checker<'a>,
    source: &'b str,
    results: Vec<(String, String)>,
    /// Stashed type string from the last visit_expression, so
    /// visit_static_member_expression can emit the property name
    /// with the same type as the parent member expression.
    last_expression_type: Option<String>,
}

impl<'a> oxc::ast_visit::Visit<'a> for TypeCollectorVisitor<'a, '_> {
    fn visit_expression(&mut self, expr: &oxc::ast::ast::Expression<'a>) {
        use oxc::span::GetSpan as _;

        // Record the type for this expression before recursing
        let span = expr.span();
        if (span.start as usize) < self.source.len() && (span.end as usize) <= self.source.len() {
            let expr_text = &self.source[span.start as usize..span.end as usize];
            let type_id = self.checker.get_type_at_location(expr);
            let type_str = self.checker.type_to_string(type_id);
            self.results.push((expr_text.to_string(), type_str.clone()));
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
                self.results.push((prop_text.to_string(), parent_type));
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
                    let prop_type = self.checker.get_type_of_expression(&prop.value, None);
                    let widened = self.checker.get_widened_literal_type(prop_type);
                    let key_text =
                        &self.source[key_span.start as usize..key_span.end as usize];
                    self.results
                        .push((key_text.to_string(), self.checker.type_to_string(widened)));
                }
            }
        }

        oxc::ast_visit::walk::walk_object_property(self, prop);
    }

    fn visit_property_definition(&mut self, prop: &oxc::ast::ast::PropertyDefinition<'a>) {
        use oxc::span::GetSpan as _;
        // Emit the property key's type for class property definitions
        if let Some(_name) = prop.key.static_name() {
            let key_span = prop.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let prop_type = if let Some(ann) = &prop.type_annotation {
                    self.checker.get_type_from_type_node(&ann.type_annotation)
                } else if let Some(init) = &prop.value {
                    self.checker.get_type_of_expression(init, None)
                } else {
                    self.checker.any_type
                };
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(prop_type)));
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
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(method_type)));
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
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(prop_type)));
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
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(method_type)));
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
                self.checker.get_type_of_expression(init, None)
            } else {
                self.checker.any_type
            };
            let name_text =
                &self.source[name_span.start as usize..name_span.end as usize];
            self.results
                .push((name_text.to_string(), self.checker.type_to_string(member_type)));
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
            self.results.push((id.name.to_string(), self.checker.type_to_string(type_id)));
        }

        oxc::ast_visit::walk::walk_binding_identifier(self, id);
    }

}

// ================================
// Checker (.errors.txt error code conformance)
// ================================

pub fn run_checker_errors_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter_map(|f| {
            // Only test files that have expected error codes
            if f.error_codes.is_empty() {
                return None;
            }
            // Only handle single-unit files for now
            if f.units.len() != 1 {
                return None;
            }

            let source = &f.units[0].content;
            let source_type = f.units[0].source_type;
            let result = run_checker_errors_single(source, source_type, &f.error_codes);
            Some(CoverageResult { path: f.path.clone(), should_fail: false, result })
        })
        .collect()
}

fn run_checker_errors_single(
    source: &str,
    source_type: SourceType,
    expected_codes: &[String],
) -> TestResult {
    use oxc::semantic::SemanticBuilder;
    use oxc_checker::Checker;

    // Collect error codes from all phases: parser, semantic, and checker.
    // tsc's .errors.txt baselines include errors from all compiler phases,
    // so we must do the same to get accurate conformance results.
    let mut actual_codes: Vec<String> = Vec::new();

    let type_arena = oxc_types::TypeArena::with_capacity(64);
    let project = oxc_project::Project::new(&type_arena);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    // Collect TS error codes from parser diagnostics
    for d in &parsed.errors {
        if let Some(code) = d.code.number.as_ref() {
            actual_codes.push(code.to_string());
        }
    }

    // If the parser panicked (unrecoverable), we can't build an AST to continue
    if parsed.panicked {
        return TestResult::ParseError(
            parsed.errors.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("\n"),
            false,
        );
    }

    // Run semantic and checker even if there were (recoverable) parser errors,
    // since tsc continues analysis and may emit additional errors
    let program = &parsed.program;
    let semantic_ret = SemanticBuilder::new().build(program);

    // Collect TS error codes from semantic diagnostics
    for d in &semantic_ret.errors {
        if let Some(code) = d.code.number.as_ref() {
            actual_codes.push(code.to_string());
        }
    }

    let mut checker =
        Checker::new_with_host(&semantic_ret.semantic, &type_arena, &project, String::new(), 1);
    checker.check_program(program);

    // Collect TS error codes from checker diagnostics
    for d in &checker.take_diagnostics() {
        if let Some(code) = d.code.number.as_ref() {
            actual_codes.push(code.to_string());
        }
    }

    actual_codes.sort();
    actual_codes.dedup();

    // Check if our actual codes match expected codes
    let mut expected_sorted: Vec<&str> = expected_codes.iter().map(|s| s.as_str()).collect();
    expected_sorted.sort();

    if actual_codes.iter().map(|s| s.as_str()).collect::<Vec<_>>() == expected_sorted {
        TestResult::Passed
    } else {
        TestResult::Mismatch(
            "checker_errors",
            format!("actual: [{}]", actual_codes.join(", ")),
            format!("expected: [{}]", expected_sorted.join(", ")),
        )
    }
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
