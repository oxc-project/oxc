//! Tool runner functions for coverage testing

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc::{
    allocator::Allocator,
    ast_visit::utf8_to_utf16::Utf8ToUtf16,
    diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource, OxcDiagnostic},
    minifier::CompressOptions,
    parser::{ParseOptions, Parser, ParserReturn, config::RuntimeParserConfig},
    span::{ModuleKind, SourceType, Span},
    transformer::{JsxOptions, JsxRuntime, TransformOptions},
};
use oxc_estree_tokens::{ESTreeTokenOptions, to_estree_tokens_pretty_json};
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
    run_estree_test262_impl(
        files,
        |test_file| {
            workspace_root()
                .join("estree-conformance/tests")
                .join(&test_file.path)
                .with_extension("json")
        },
        RuntimeParserConfig::default(),
        |parser_ret| {
            let mut program = parser_ret.program;
            let source_text = program.source_text;
            Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);
            program.to_pretty_estree_js_json(false)
        },
    )
}

pub fn run_estree_test262_tokens(files: &[Test262File]) -> Vec<CoverageResult> {
    run_estree_test262_impl(
        files,
        |test_file| {
            workspace_root()
                .join("estree-conformance/tests/test262-tokens")
                .join(test_file.path.strip_prefix("test262/").unwrap_or(&test_file.path))
                .with_extension("json")
        },
        RuntimeParserConfig::new(true),
        |parser_ret| {
            let ParserReturn { program, tokens, .. } = parser_ret;
            let source_text = program.source_text;
            let span_converter = Utf8ToUtf16::new(source_text);
            to_estree_tokens_pretty_json(
                &tokens,
                &program,
                source_text,
                &span_converter,
                ESTreeTokenOptions::new(false),
            )
        },
    )
}

fn run_estree_test262_impl(
    files: &[Test262File],
    get_snapshot_path: impl Fn(&Test262File) -> PathBuf + Sync,
    parser_config: RuntimeParserConfig,
    get_json: impl for<'a> Fn(ParserReturn<'a>) -> String + Sync,
) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter(|test_file| {
            !test_file
                .meta
                .negative
                .as_ref()
                .is_some_and(|n| n.phase == crate::test262::Phase::Parse)
        })
        .filter_map(|test_file| {
            let snapshot_path = get_snapshot_path(test_file);
            let expected_json = fs::read_to_string(&snapshot_path).ok()?;
            Some((test_file, expected_json))
        })
        .map(|(test_file, expected_json)| {
            let is_module = test_file.meta.flags.contains(&TestFlag::Module);
            let source_type = SourceType::script().with_module(is_module);
            let source_text = test_file.code.as_str();

            let allocator = Allocator::new();
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_config(parser_config)
                .parse();

            if ret.panicked || !ret.errors.is_empty() {
                let error =
                    ret.errors.first().map_or_else(|| "Panicked".to_string(), ToString::to_string);
                return CoverageResult {
                    path: test_file.path.clone(),
                    should_fail: false,
                    result: TestResult::ParseError(error, ret.panicked),
                };
            }

            let actual_json = get_json(ret);
            let result = if actual_json == expected_json {
                TestResult::Passed
            } else {
                TestResult::Mismatch("Mismatch", actual_json, expected_json)
            };
            CoverageResult { path: test_file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_estree_acorn_jsx(files: &[AcornJsxFile]) -> Vec<CoverageResult> {
    run_estree_acorn_jsx_impl(
        files,
        |test_file| workspace_root().join(&test_file.path).with_extension("json"),
        RuntimeParserConfig::default(),
        |parser_ret| {
            let mut program = parser_ret.program;
            let source_text = program.source_text;
            Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);
            program.to_pretty_estree_js_json(false)
        },
    )
}

pub fn run_estree_acorn_jsx_tokens(files: &[AcornJsxFile]) -> Vec<CoverageResult> {
    run_estree_acorn_jsx_impl(
        files,
        |test_file| workspace_root().join(test_file.path.with_extension("tokens.json")),
        RuntimeParserConfig::new(true),
        |parser_ret| {
            let ParserReturn { program, tokens, .. } = parser_ret;
            let source_text = program.source_text;
            let span_converter = Utf8ToUtf16::new(source_text);
            to_estree_tokens_pretty_json(
                &tokens,
                &program,
                source_text,
                &span_converter,
                ESTreeTokenOptions::new(false),
            )
        },
    )
}

fn run_estree_acorn_jsx_impl(
    files: &[AcornJsxFile],
    get_snapshot_path: impl Fn(&AcornJsxFile) -> PathBuf + Sync,
    parser_config: RuntimeParserConfig,
    get_json: impl for<'a> Fn(ParserReturn<'a>) -> String + Sync,
) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|test_file| {
            let source_type = SourceType::default().with_module(true).with_jsx(true);
            let source_text = test_file.code.as_str();

            let allocator = Allocator::new();
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_config(parser_config)
                .parse();

            if ret.panicked || !ret.errors.is_empty() {
                let error =
                    ret.errors.first().map_or_else(|| "Panicked".to_string(), ToString::to_string);
                let result = if test_file.should_fail {
                    TestResult::CorrectError(error, ret.panicked)
                } else {
                    TestResult::ParseError(error, ret.panicked)
                };
                return CoverageResult {
                    path: test_file.path.clone(),
                    should_fail: test_file.should_fail,
                    result,
                };
            }

            if test_file.should_fail {
                return CoverageResult {
                    path: test_file.path.clone(),
                    should_fail: true,
                    result: TestResult::IncorrectlyPassed,
                };
            }

            let snapshot_path = get_snapshot_path(test_file);
            let expected_json = match fs::read_to_string(&snapshot_path) {
                Ok(json) => json,
                Err(error) => {
                    return CoverageResult {
                        path: test_file.path.clone(),
                        should_fail: false,
                        result: TestResult::GenericError(
                            "Error reading Acorn JSON",
                            error.to_string(),
                        ),
                    };
                }
            };

            let actual_json = get_json(ret);
            let result = if actual_json == expected_json {
                TestResult::Passed
            } else {
                TestResult::Mismatch("Mismatch", actual_json, expected_json)
            };
            CoverageResult { path: test_file.path.clone(), should_fail: false, result }
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
    run_estree_typescript_impl(files, "AST", RuntimeParserConfig::default(), |ret| {
        let mut program = ret.program;
        let source_text = program.source_text;
        Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);
        program.to_pretty_estree_ts_json(false)
    })
}

pub fn run_estree_typescript_tokens(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    run_estree_typescript_impl(files, "TOKENS", RuntimeParserConfig::new(true), |ret| {
        let ParserReturn { program, tokens, .. } = ret;
        let source_text = program.source_text;
        let span_converter = Utf8ToUtf16::new(source_text);
        to_estree_tokens_pretty_json(
            &tokens,
            &program,
            source_text,
            &span_converter,
            ESTreeTokenOptions::new(true),
        )
    })
}

fn run_estree_typescript_impl(
    files: &[TypeScriptFile],
    section_key: &'static str,
    parser_config: RuntimeParserConfig,
    get_json: impl for<'a> Fn(ParserReturn<'a>) -> String + Sync,
) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter_map(|test_file| {
            if test_file.should_fail {
                return None;
            }
            if test_file.path.to_str().is_some_and(|p| TS_SKIP_PATHS.contains(&p)) {
                return None;
            }

            let ext = test_file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let estree_path = workspace_root()
                .join("estree-conformance/tests")
                .join(&test_file.path)
                .with_extension(format!("{ext}.md"));
            let estree_content = fs::read_to_string(&estree_path).ok()?;
            Some((test_file, estree_content))
        })
        .map(|(test_file, estree_content)| {
            let snapshot_units = parse_estree_json_blocks(&estree_content, section_key);

            if snapshot_units.len() != test_file.units.len() {
                return CoverageResult {
                    path: test_file.path.clone(),
                    should_fail: false,
                    result: TestResult::GenericError(
                        "Unexpected ESTree file",
                        format!(
                            "{section_key} {} != {}",
                            snapshot_units.len(),
                            test_file.units.len()
                        ),
                    ),
                };
            }

            for (unit, &expected_json) in test_file.units.iter().zip(snapshot_units.iter()) {
                let source_type = unit.source_type;
                let source_text = unit.content.as_str();
                let parse_options = ParseOptions { preserve_parens: false, ..Default::default() };

                let allocator = Allocator::new();
                let ret = Parser::new(&allocator, source_text, source_type)
                    .with_options(parse_options)
                    .with_config(parser_config)
                    .parse();

                if ret.panicked || !ret.errors.is_empty() {
                    let error = ret
                        .errors
                        .first()
                        .map_or_else(|| "Panicked".to_string(), ToString::to_string);
                    return CoverageResult {
                        path: test_file.path.clone(),
                        should_fail: false,
                        result: TestResult::ParseError(error, ret.panicked),
                    };
                }

                let actual_json = get_json(ret);
                if actual_json != expected_json {
                    return CoverageResult {
                        path: test_file.path.clone(),
                        should_fail: false,
                        result: TestResult::Mismatch(
                            "Mismatch",
                            actual_json,
                            expected_json.to_string(),
                        ),
                    };
                }
            }

            CoverageResult {
                path: test_file.path.clone(),
                should_fail: false,
                result: TestResult::Passed,
            }
        })
        .collect()
}

fn parse_estree_json_blocks<'a>(content: &'a str, section_key: &str) -> Vec<&'a str> {
    content
        .split("__ESTREE_TEST__")
        .skip(1)
        .filter_map(|section| {
            let (key, json) = section
                .strip_prefix(':')
                .and_then(|s| s.split_once(":\n```json\n"))
                .and_then(|(key, rest)| rest.strip_suffix("\n```\n").map(|json| (key, json)))
                .expect(r"each TS snapshot section must have the form '__ESTREE_TEST__:<key>:\n```json\n<JSON>\n```\n'");
            if key == section_key { Some(json) } else { None }
        })
        .collect()
}
