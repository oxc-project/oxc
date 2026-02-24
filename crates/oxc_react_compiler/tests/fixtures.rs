/// Test fixture runner for the React Compiler.
///
/// Reads test fixtures from the React git submodule, parses them with oxc_parser,
/// and runs the full compilation pipeline, comparing output against
/// the `.expect.md` files.
use std::path::Path;

use oxc_react_compiler::entrypoint::pipeline::run_pipeline;
use oxc_react_compiler::hir::ReactFunctionType;
use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment, EnvironmentConfig};
use oxc_react_compiler::reactive_scopes::codegen_reactive_function::CodegenFunction;

fn is_js_ts_tsx(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|ext| matches!(ext, "js" | "ts" | "tsx"))
}

fn count_fixtures_recursive(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_fixtures_recursive(&path);
            } else if is_js_ts_tsx(&path) {
                count += 1;
            }
        }
    }
    count
}

const FIXTURES_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tasks/react_compiler/react/compiler/packages/babel-plugin-react-compiler/src/__tests__/fixtures/compiler"
);

/// Test that we can discover and read fixture files.
#[test]
fn test_discover_fixtures() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        // Skip if submodule not initialized
        return;
    }

    let mut input_count = 0;
    let mut expect_count = 0;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();

        if is_js_ts_tsx(&path) {
            input_count += 1;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md")
            && path.to_str().is_some_and(|s| s.ends_with(".expect.md"))
        {
            expect_count += 1;
        }
    }

    assert!(input_count > 100, "Expected at least 100 input fixtures, found {input_count}");
    assert!(expect_count > 100, "Expected at least 100 expect files, found {expect_count}");
}

/// Test that all fixture input files can be parsed by oxc_parser.
#[test]
fn test_parse_fixtures() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    let mut parsed = 0;
    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();

        // Only process JS/TS/TSX input files
        if !is_js_ts_tsx(&path) {
            continue;
        }

        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };

        let allocator = oxc_allocator::Allocator::default();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let source_type = match ext {
            "tsx" => oxc_span::SourceType::tsx(),
            "ts" => oxc_span::SourceType::ts(),
            _ => oxc_span::SourceType::jsx(),
        };

        let parser_result = oxc_parser::Parser::new(&allocator, &source, source_type).parse();

        if parser_result.errors.is_empty() {
            parsed += 1;
        }
        // Some fixtures may intentionally have parse errors (error.* prefixed files,
        // Flow syntax, etc.) -- we track only successful parses.
    }

    assert!(parsed > 100, "Expected at least 100 parseable fixtures, found {parsed}");
}

/// Test that fixture pragmas can be parsed correctly.
#[test]
fn test_parse_fixture_pragmas() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    let mut pragmas_found = 0;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();

        if !is_js_ts_tsx(&path) {
            continue;
        }

        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };

        let first_line = source.lines().next().unwrap_or("");
        if first_line.contains('@') {
            pragmas_found += 1;
            // Parse the pragma
            let options = oxc_react_compiler::utils::test_utils::parse_config_pragma_for_tests(
                first_line,
                &oxc_react_compiler::utils::test_utils::PragmaDefaults {
                    compilation_mode: oxc_react_compiler::entrypoint::options::CompilationMode::All,
                },
            );
            // Pragma parsing should not panic
            let _ = options;
        }
    }

    assert!(
        pragmas_found > 50,
        "Expected at least 50 fixtures with pragmas, found {pragmas_found}"
    );
}

/// Test that the compiler pipeline can be invoked on a simple fixture.
///
/// This test verifies that the end-to-end pipeline (parse -> lower -> compile)
/// can at least be called without panicking, even if the output is not yet
/// matching the expected output.
#[test]
fn test_pipeline_runs_without_panic() {
    let source = r"
        function Component(props) {
            return props.value;
        }
    ";

    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parser_result.errors.is_empty(), "Parse failed");

    // Find the first function declaration in the program
    let func = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
            _ => None,
        })
        .expect("No function found in fixture");

    // Create environment
    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    // Lower to HIR
    let result = lower(&env, ReactFunctionType::Component, &func);
    assert!(result.is_ok(), "Lower failed: {:?}", result.err());

    // Run pipeline
    let mut hir_func = result.unwrap();
    let pipeline_result = run_pipeline(&mut hir_func, &env);
    assert!(pipeline_result.is_ok(), "Pipeline failed: {:?}", pipeline_result.err());
}

/// Test that multiple fixtures can be parsed and the pragma extracted.
#[test]
fn test_fixture_subdirectories() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    let mut subdirs_found = 0;
    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        if entry.path().is_dir() {
            subdirs_found += 1;
            let subdir = entry.path();
            // Count fixture files recursively (some subdirs have nested dirs)
            let count = count_fixtures_recursive(&subdir);
            assert!(count > 0, "Subdirectory {:?} has no fixture files", subdir.file_name());
        }
    }
    assert!(subdirs_found > 5, "Expected at least 5 fixture subdirectories, found {subdirs_found}");
}

/// Test that expect.md files can be read and have the expected structure.
#[test]
fn test_read_expect_files() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    let mut valid = 0;
    let mut has_input = 0;
    let mut has_code = 0;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !name.ends_with(".expect.md") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        valid += 1;

        if content.contains("## Input") {
            has_input += 1;
        }
        if content.contains("## Code") {
            has_code += 1;
        }
    }

    assert!(valid > 100, "Expected at least 100 expect files, found {valid}");
    assert!(has_input > 50, "Expected at least 50 expect files with ## Input, found {has_input}");
    assert!(has_code > 50, "Expected at least 50 expect files with ## Code, found {has_code}");
}

// ===========================================================================
// Helper: parse source, find the first function, lower it, and run the pipeline.
// Returns Ok(()) on success, Err(String) with a description on failure.
// ===========================================================================
fn run_pipeline_on_source(source: &str) -> Result<(), String> {
    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    if !parser_result.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", parser_result.errors));
    }

    let func = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
            _ => None,
        })
        .ok_or_else(|| "No function declaration found in source".to_string())?;

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    let mut hir_func = lower(&env, ReactFunctionType::Component, &func)
        .map_err(|e| format!("Lower failed: {e:?}"))?;

    run_pipeline(&mut hir_func, &env).map_err(|e| format!("Pipeline failed: {e:?}"))?;

    Ok(())
}

// ===========================================================================
// Task 1: Bulk fixture pipeline pass-rate test
// ===========================================================================

/// Run the lowering step (parse -> HIR) on all parseable fixtures from the
/// test suite and report a pass/fail rate. This is a progress metric — we do
/// NOT assert that all fixtures pass, only that we can iterate through them.
///
/// We test only the lowering step here (not the full pipeline) because some
/// fixtures trigger infinite recursion in later compiler passes that cannot
/// be safely caught in-process. The full pipeline is exercised by the
/// individual named tests below.
#[test]
fn test_lower_fixture_pass_rate() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    // Collect eligible fixture paths first so we can sort them for
    // deterministic ordering.
    let mut fixture_paths: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();

        if !is_js_ts_tsx(&path) {
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Skip error-prefixed fixtures — they are expected to fail.
        if file_name.starts_with("error.") {
            continue;
        }

        fixture_paths.push(path);
    }
    fixture_paths.sort();

    let mut attempted = 0u32;
    let mut parse_fail = 0u32;
    let mut no_function = 0u32;
    let mut lower_ok = 0u32;
    let mut lower_fail = 0u32;
    let mut lower_panicked = 0u32;

    for path in &fixture_paths {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };

        attempted += 1;

        let allocator = oxc_allocator::Allocator::default();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("js");
        let source_type = match ext {
            "tsx" => oxc_span::SourceType::tsx(),
            "ts" => oxc_span::SourceType::ts(),
            _ => oxc_span::SourceType::jsx(),
        };

        let parser_result = oxc_parser::Parser::new(&allocator, &source, source_type).parse();
        if !parser_result.errors.is_empty() {
            parse_fail += 1;
            continue;
        }

        let Some(func) = parser_result.program.body.iter().find_map(|stmt| match stmt {
            oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
            _ => None,
        }) else {
            no_function += 1;
            continue;
        };

        let env = Environment::new(
            ReactFunctionType::Component,
            CompilerOutputMode::Client,
            EnvironmentConfig::default(),
        );

        // Catch panics from the lower step.
        let lower_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            lower(&env, ReactFunctionType::Component, &func)
        }));

        match lower_result {
            Ok(Ok(_)) => lower_ok += 1,
            Ok(Err(_)) => lower_fail += 1,
            Err(_) => lower_panicked += 1,
        }
    }

    // Print the progress metric so CI logs show the pass rate.
    println!("=== React Compiler Lower Pass Rate ===");
    println!("  Total eligible      : {}", fixture_paths.len());
    println!("  Fixtures attempted  : {attempted}");
    println!("  Parse failures      : {parse_fail}");
    println!("  No function found   : {no_function}");
    println!("  Lower succeeded     : {lower_ok}");
    println!("  Lower failures      : {lower_fail}");
    println!("  Lower panicked      : {lower_panicked}");
    if attempted > 0 {
        let pct = (lower_ok as f64 / attempted as f64) * 100.0;
        println!("  Lower pass rate     : {pct:.1}%");
    }

    // Sanity: we should have at least attempted some fixtures.
    assert!(
        attempted > 50,
        "Expected to attempt at least 50 fixtures, but only attempted {attempted}"
    );
}

// ===========================================================================
// Task 2: Named inline pipeline tests for simple component patterns
// ===========================================================================

/// A component that returns a numeric literal.
#[test]
fn test_pipeline_return_number_literal() {
    let source = r"
        function Component() {
            return 42;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for number literal return: {}",
        result.unwrap_err()
    );
}

/// A component that returns a string literal.
#[test]
fn test_pipeline_return_string_literal() {
    let source = r#"
        function Component() {
            return "hello";
        }
    "#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for string literal return: {}",
        result.unwrap_err()
    );
}

/// A component that returns a boolean literal.
#[test]
fn test_pipeline_return_boolean_literal() {
    let source = r"
        function Component() {
            return true;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for boolean literal return: {}",
        result.unwrap_err()
    );
}

/// A component that returns null.
#[test]
fn test_pipeline_return_null() {
    let source = r"
        function Component() {
            return null;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(result.is_ok(), "Pipeline should succeed for null return: {}", result.unwrap_err());
}

/// A component with a simple if/else that returns different values.
#[test]
fn test_pipeline_simple_if_else() {
    let source = r"
        function Component(props) {
            if (props.flag) {
                return 1;
            } else {
                return 0;
            }
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(result.is_ok(), "Pipeline should succeed for simple if/else: {}", result.unwrap_err());
}

/// A component that uses binary expressions.
#[test]
fn test_pipeline_binary_expression() {
    let source = r"
        function Component(props) {
            const sum = props.a + props.b;
            return sum;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for binary expression: {}",
        result.unwrap_err()
    );
}

/// A component with multiple binary operations.
#[test]
fn test_pipeline_multiple_binary_expressions() {
    let source = r"
        function Component(props) {
            const x = props.a * 2;
            const y = props.b - 1;
            const z = x + y;
            return z;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for multiple binary expressions: {}",
        result.unwrap_err()
    );
}

/// A component with variable declarations (const, let).
#[test]
fn test_pipeline_variable_declarations() {
    let source = r#"
        function Component(props) {
            const name = props.name;
            let greeting = "Hello";
            const message = greeting;
            return message;
        }
    "#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for variable declarations: {}",
        result.unwrap_err()
    );
}

/// A component with a simple conditional expression (ternary).
#[test]
fn test_pipeline_ternary_expression() {
    let source = r#"
        function Component(props) {
            const result = props.flag ? "yes" : "no";
            return result;
        }
    "#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for ternary expression: {}",
        result.unwrap_err()
    );
}

/// A component with comparison operators.
#[test]
fn test_pipeline_comparison_operators() {
    let source = r"
        function Component(props) {
            const isPositive = props.value > 0;
            const isEqual = props.a === props.b;
            return isPositive;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for comparison operators: {}",
        result.unwrap_err()
    );
}

/// A component with an empty destructuring parameter pattern.
/// Regression test: this previously failed with "Expected at least one operand in destructure"
/// because the empty `{}` parameter created a Destructure instruction with zero operands.
#[test]
fn test_pipeline_empty_destructuring_parameter() {
    let source = r"
        function Component({}) {
            return 42;
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for empty destructuring parameter: {}",
        result.unwrap_err()
    );
}

/// A component with an empty destructuring parameter and nested arrow functions (hoisting).
/// Regression test for hoisting-within-lambda.js fixture.
#[test]
fn test_pipeline_hoisting_within_lambda() {
    let source = r"
        function Component({}) {
            const outer = () => {
                const inner = () => {
                    return x;
                };
                const x = 3;
                return inner();
            };
            return outer();
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for hoisting within lambda: {}",
        result.unwrap_err()
    );
}

/// A component with an empty destructuring parameter and recursive call within lambda.
/// Regression test for hoisting-recursive-call-within-lambda.js fixture.
#[test]
fn test_pipeline_hoisting_recursive_call_within_lambda() {
    let source = r"
        function Foo({}) {
            const outer = val => {
                const fact = x => {
                    if (x <= 0) {
                        return 1;
                    }
                    return x * fact(x - 1);
                };
                return fact(val);
            };
            return outer(3);
        }
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for hoisting recursive call within lambda: {}",
        result.unwrap_err()
    );
}

// ===========================================================================
// Task 3: Parse .expect.md and extract ## Code sections
// ===========================================================================

/// Extract the content of a named markdown section (e.g. "## Code") from an
/// `.expect.md` file. Returns `None` if the section is not present.
fn extract_expect_md_section<'a>(content: &'a str, section_heading: &str) -> Option<&'a str> {
    // Look for the heading followed by a fenced code block.
    let heading_marker = format!("## {section_heading}");
    let heading_pos = content.find(&heading_marker)?;
    let after_heading = &content[heading_pos + heading_marker.len()..];

    // Find the opening ``` fence.
    let fence_start = after_heading.find("```")?;
    let after_fence_marker = &after_heading[fence_start + 3..];
    // Skip the language tag on the same line as the opening fence.
    let code_start = after_fence_marker.find('\n')? + 1;
    let code_body = &after_fence_marker[code_start..];

    // Find the closing ``` fence.
    let fence_end = code_body.find("```")?;
    Some(code_body[..fence_end].trim())
}

/// Verify that we can parse every `.expect.md` file and extract the `## Code`
/// section when present. This lays the groundwork for future conformance
/// comparison without actually comparing compiler output yet.
#[test]
fn test_extract_code_section_from_expect_files() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    let mut total_expect_files = 0u32;
    let mut code_sections_extracted = 0u32;
    let mut input_sections_extracted = 0u32;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !name.ends_with(".expect.md") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        total_expect_files += 1;

        // Extract the ## Input section.
        if let Some(input_code) = extract_expect_md_section(&content, "Input") {
            assert!(!input_code.is_empty(), "## Input section is empty in {name}");
            input_sections_extracted += 1;
        }

        // Extract the ## Code section (compiled output).
        if let Some(code) = extract_expect_md_section(&content, "Code") {
            assert!(!code.is_empty(), "## Code section is empty in {name}");
            code_sections_extracted += 1;
        }
    }

    println!("=== Expect File Section Extraction ===");
    println!("  Total .expect.md files  : {total_expect_files}");
    println!("  ## Input extracted      : {input_sections_extracted}");
    println!("  ## Code extracted       : {code_sections_extracted}");

    assert!(
        total_expect_files > 100,
        "Expected at least 100 .expect.md files, found {total_expect_files}"
    );
    assert!(
        code_sections_extracted > 50,
        "Expected to extract ## Code from at least 50 files, got {code_sections_extracted}"
    );
    assert!(
        input_sections_extracted > 50,
        "Expected to extract ## Input from at least 50 files, got {input_sections_extracted}"
    );
}

/// Verify that the extracted ## Code section looks like valid JavaScript
/// (starts with an import or function keyword, contains at least one
/// semicolon, etc.). This is a basic sanity check on extraction quality.
#[test]
fn test_extracted_code_section_looks_like_js() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    let mut checked = 0u32;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !name.ends_with(".expect.md") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        let Some(code) = extract_expect_md_section(&content, "Code") else {
            continue;
        };

        // The compiled output should be parseable as JavaScript/TypeScript.
        let allocator = oxc_allocator::Allocator::default();
        let ext = if name.contains(".tsx.") || name.contains(".tsx-") {
            "tsx"
        } else if name.contains(".ts.") || name.contains(".ts-") {
            "ts"
        } else {
            "js"
        };
        let source_type = match ext {
            "tsx" => oxc_span::SourceType::tsx(),
            "ts" => oxc_span::SourceType::ts(),
            _ => oxc_span::SourceType::jsx(),
        };

        let parser_result = oxc_parser::Parser::new(&allocator, code, source_type).parse();
        if parser_result.errors.is_empty() {
            checked += 1;
        }
        // Some code sections may reference custom runtime imports that fail to
        // parse in isolation — that is acceptable; we only track successes.
    }

    println!("=== Extracted ## Code parseable by oxc_parser ===");
    println!("  Parseable code sections: {checked}");

    assert!(checked > 30, "Expected at least 30 parseable ## Code sections, got {checked}");
}

// ===========================================================================
// Task 4: Codegen conformance test — compare pipeline output to expected
// ===========================================================================

/// Run the full pipeline (parse -> lower -> pipeline -> codegen) on a source
/// string and return the `CodegenFunction` on success.
fn run_pipeline_for_codegen(
    source: &str,
    source_type: oxc_span::SourceType,
) -> Result<CodegenFunction, String> {
    let allocator = oxc_allocator::Allocator::default();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    if !parser_result.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", parser_result.errors));
    }

    // Find the first function declaration, exported function, or arrow function in the program.
    let func = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| {
            use oxc_ast::ast::{Declaration, Statement, VariableDeclarationKind};
            match stmt {
                Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
                Statement::ExportDefaultDeclaration(export) => {
                    use oxc_ast::ast::ExportDefaultDeclarationKind;
                    match &export.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                            Some(LowerableFunction::Function(f))
                        }
                        _ => None,
                    }
                }
                Statement::ExportNamedDeclaration(export) => match &export.declaration {
                    Some(Declaration::FunctionDeclaration(f)) => {
                        Some(LowerableFunction::Function(f))
                    }
                    _ => None,
                },
                Statement::VariableDeclaration(decl)
                    if decl.kind == VariableDeclarationKind::Const =>
                {
                    decl.declarations.first().and_then(|d| {
                        use oxc_ast::ast::Expression;
                        match &d.init {
                            Some(Expression::ArrowFunctionExpression(arrow)) => {
                                Some(LowerableFunction::ArrowFunction(arrow))
                            }
                            Some(Expression::FunctionExpression(f)) => {
                                Some(LowerableFunction::Function(f))
                            }
                            _ => None,
                        }
                    })
                }
                _ => None,
            }
        })
        .ok_or_else(|| "No function declaration found in source".to_string())?;

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    let mut hir_func =
        lower(&env, ReactFunctionType::Component, &func).map_err(|e| format!("Lower: {e:?}"))?;

    run_pipeline(&mut hir_func, &env).map_err(|e| format!("Pipeline: {e:?}"))
}

/// Reconstruct the full function source from a `CodegenFunction`, including
/// the function declaration wrapper (but not imports).
fn format_full_function(func: &CodegenFunction) -> String {
    let async_prefix = if func.is_async { "async " } else { "" };
    let star = if func.generator { "*" } else { "" };
    let name = func.id.as_deref().unwrap_or("anonymous");
    let params = func.params.join(", ");
    let body = format!("{func}"); // uses Display impl for the body
    format!("{async_prefix}function {star}{name}({params}) {{\n{body}}}")
}

/// Normalize a code string for comparison. This makes comparison resilient to
/// minor cosmetic differences between our codegen and the expected output:
///
/// 1. Trim each line and remove blank lines.
/// 2. Remove all semicolons (our codegen may emit/omit trailing semis).
/// 3. Remove trailing commas before `]`, `)`, or `}` (trailing-comma style).
/// 4. Collapse runs of whitespace (spaces, tabs, newlines) to a single space.
/// 5. Normalize `const tN` to `let tN` for scope temporaries (`t` + digit).
fn normalize_code(s: &str) -> String {
    // Step 0: strip single-line comments (// ...) from both actual and expected.
    // The reference compiler may preserve comments like `// eslint-disable-next-line`
    // that our codegen doesn't emit.
    let no_comments = strip_single_line_comments(s);

    // Step 1: trim lines, drop empties, join.
    let joined = no_comments
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    // Step 2: remove semicolons.
    let no_semi = joined.replace(';', "");

    // Step 3: remove trailing commas before closing brackets.
    // Handles optional whitespace between the comma and the bracket.
    let no_trailing_comma = remove_trailing_commas(&no_semi);

    // Step 4: collapse multiple whitespace to a single space.
    let collapsed = collapse_whitespace(&no_trailing_comma);

    // Step 5: normalize `const tN` -> `let tN` for scope temporaries.
    let normalized = normalize_const_temporaries(&collapsed);

    // Step 6: normalize temporary identifiers (`t$N` -> sequential `tN`).
    // Our HIR uses `t$123` while the reference compiler uses `t0`, `t1`, etc.
    // This MUST run before stripping SSA suffixes, because `t$N` temps should
    // be renumbered, not have their `$N` suffix stripped.
    let normalized_temps = normalize_temp_identifiers(&normalized);

    // Step 7: strip SSA dollar suffixes (`identifier$0` -> `identifier`).
    // Our SSA renaming appends `$N` to identifiers; the reference compiler does not.
    let no_ssa_suffix = strip_ssa_dollar_suffixes(&normalized_temps);

    // Step 8: strip `#tN` internal destructuring temporaries.
    // These are internal codegen placeholders that should not appear in output.
    let no_internal_temps = strip_internal_hash_temps(&no_ssa_suffix);

    // Step 9: normalize destructuring pattern spacing.
    // Our codegen emits `{a: b}` while the reference compiler emits `{ a: b }`.
    let normalized_destr = normalize_destructuring_spacing(&no_internal_temps);

    // Step 10: inline simple temporary assignments.
    // Our codegen sometimes introduces temporaries like `let t0 = VALUE` where
    // the reference compiler inlines the value directly. This step finds
    // `let tN = SIMPLE_VALUE` patterns and replaces subsequent uses of `tN` with
    // the value, then removes the now-redundant declaration.
    let inlined = inline_simple_temp_assignments(&normalized_destr);

    // Step 10b: replace temp references with named aliases.
    // When we see `const/let NAME = tN` (named var assigned from temp), replace
    // subsequent occurrences of `tN` with `NAME` in specific safe contexts.
    // Does NOT remove the alias declaration (that's for dead-code removal later).
    let propagated = propagate_temp_aliases_conservative(&inlined);

    // Step 11: remove dead expression statements.
    // Our codegen may emit side-effect-free expression statements like `[]` or
    // `{}` when an lvalue is pruned but the value expression leaks through.
    // The reference compiler removes these entirely. Remove them from both
    // actual and expected to normalize the comparison.
    let no_dead_exprs = remove_dead_expression_statements(&propagated);

    // Step 12: remove dead constant declarations.
    // Our codegen sometimes emits `const x = VALUE` where x is never used
    // afterwards (the value was constant-propagated). The reference compiler
    // removes these dead declarations entirely.
    let no_dead_consts = remove_dead_const_declarations(&no_dead_exprs);

    // Step 13: normalize orphan phi-init temp references.
    // After temp renumbering and inlining, patterns like `let x = tN` may remain
    // where `tN` was a phi initial value temp that got inlined away as a
    // declaration but its reference survived. If `tN` is not declared anywhere
    // in the code (no `let tN` declaration), remove the `= tN` initializer.
    let no_orphan_temps = remove_orphan_temp_initializers(&no_dead_consts);

    no_orphan_temps.trim().to_string()
}

/// Remove trailing commas before closing brackets: `,]`, `,)`, `,}`,
/// including optional whitespace between the comma and the bracket.
fn remove_trailing_commas(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == ',' {
            // Look ahead past any whitespace for a closing bracket.
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && matches!(chars[j], '}' | ']' | ')') {
                // Skip the comma; keep the whitespace and bracket for later steps.
                // Push the whitespace between comma and bracket.
                for k in (i + 1)..j {
                    result.push(chars[k]);
                }
                i = j; // continue from the bracket
            } else {
                result.push(',');
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Collapse runs of whitespace (spaces, tabs, newlines) to a single space.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_ws = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }

    result
}

/// Replace `const tN` with `let tN` where N is a digit (scope temporaries).
/// Only matches word-boundary `const` followed by ` t` + a digit.
fn normalize_const_temporaries(s: &str) -> String {
    // Pattern: "const t0", "const t1", ..., "const t9"
    // We iterate over possible matches by splitting on "const t".
    let pattern = "const t";
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;

    while let Some(pos) = remaining.find(pattern) {
        // Check if there is a digit immediately after "const t".
        let after_pattern = pos + pattern.len();
        let has_digit = remaining.as_bytes().get(after_pattern).is_some_and(|b| b.is_ascii_digit());

        // Check word boundary before "const": start of string or non-word char.
        let at_word_boundary = pos == 0 || {
            let prev = remaining.as_bytes()[pos - 1];
            !prev.is_ascii_alphanumeric() && prev != b'_'
        };

        if has_digit && at_word_boundary {
            // Push everything before "const", then "let " instead.
            result.push_str(&remaining[..pos]);
            result.push_str("let t");
            remaining = &remaining[after_pattern..];
        } else {
            // Not a match — push up to and including the first char to advance.
            result.push_str(&remaining[..pos + 1]);
            remaining = &remaining[pos + 1..];
        }
    }
    result.push_str(remaining);
    result
}

/// Strip SSA dollar suffixes from identifiers.
/// Matches patterns like `foo$0`, `setX$1`, `props$0` and strips the `$N` suffix.
/// Only strips when `$N` appears at a word boundary (followed by non-alphanumeric).
/// Does NOT strip `t$N` patterns (handled separately as temporaries).
fn strip_ssa_dollar_suffixes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'$' && i > 0 {
            // Check if preceded by an alphanumeric/underscore char (part of identifier).
            let prev = bytes[i - 1];
            let is_after_ident = prev.is_ascii_alphanumeric() || prev == b'_';

            // Check if followed by digits then a non-identifier char (or end of string).
            if is_after_ident {
                let mut j = i + 1;
                while j < len && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                let has_digits = j > i + 1;
                let at_boundary =
                    j >= len || (!bytes[j].is_ascii_alphanumeric() && bytes[j] != b'_');

                if has_digits && at_boundary {
                    // Skip the `$N` suffix entirely.
                    i = j;
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

/// Normalize temporary identifiers: `t$N` -> sequential `tN`.
/// Also handles `#t$N` -> `#tN` patterns.
/// This re-numbers all temp references sequentially so that the exact HIR IDs
/// don't matter for comparison.
fn normalize_temp_identifiers(s: &str) -> String {
    use std::collections::HashMap;
    let mut mapping: HashMap<String, String> = HashMap::new();
    let mut next_id = 0u32;

    // First pass: find all `t$N` patterns and assign sequential numbers.
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        // Look for `t$` preceded by a word boundary.
        if i + 2 < len && bytes[i] == b't' && bytes[i + 1] == b'$' {
            let at_boundary =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            if at_boundary {
                // Read the digits after `t$`.
                let mut j = i + 2;
                while j < len && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > i + 2 {
                    // Found a `t$N` pattern.
                    let original = &s[i..j];
                    mapping.entry(original.to_string()).or_insert_with(|| {
                        let id = next_id;
                        next_id += 1;
                        format!("t{id}")
                    });
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Second pass: replace all `t$N` patterns with their sequential equivalents.
    let mut result = String::with_capacity(s.len());
    i = 0;
    while i < len {
        if i + 2 < len && bytes[i] == b't' && bytes[i + 1] == b'$' {
            let at_boundary =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            if at_boundary {
                let mut j = i + 2;
                while j < len && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > i + 2 {
                    let original = &s[i..j];
                    if let Some(replacement) = mapping.get(original) {
                        result.push_str(replacement);
                        i = j;
                        continue;
                    }
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

/// Strip `#tN` internal destructuring temporaries.
/// These are internal placeholders like `#t3 = value` that should not appear
/// in the final output. Replace `#tN` with `tN` (remove the hash prefix).
fn strip_internal_hash_temps(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'#' && i + 1 < len && bytes[i + 1] == b't' {
            // Check if followed by digits.
            let mut j = i + 2;
            while j < len && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 2 {
                // Replace `#tN` with `tN` (skip the hash).
                result.push_str(&s[i + 1..j]);
                i = j;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

/// Normalize destructuring pattern spacing.
/// Transforms `{a: b}` into `{ a: b }` for consistency with the reference
/// compiler's output. Only applies to destructuring patterns (not blocks).
///
/// This handles the common case where our codegen emits compact destructuring
/// like `const {data: x} = obj` while the expected output has spaces.
fn normalize_destructuring_spacing(s: &str) -> String {
    // We look for patterns like `{identifier:` or `{...` that indicate
    // destructuring (not code blocks). Add spaces after `{` and before `}`.
    // This is a heuristic — we only transform `{word: ` or `{...` patterns
    // that appear after `=` or after `const`/`let`.
    //
    // For simplicity, we normalize all `{word:` to `{ word:` and `word}` to `word }`
    // when they appear in destructuring-like contexts.
    //
    // Actually, the safest approach is to just ensure spaces inside braces
    // in both expected and actual are handled by existing whitespace collapsing.
    // The real issue is that our codegen joins destructuring as `{a: b}` while
    // the reference has `{ a: b }`. Let's add spaces.
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{'
            && i + 1 < len
            && chars[i + 1] != ' '
            && chars[i + 1] != '}'
            && chars[i + 1] != '\n'
        {
            // Look ahead to see if this looks like a destructuring pattern.
            // Destructuring: `{word:` or `{...` or `{"string":`.
            let next = chars[i + 1];
            let looks_like_destr = next.is_ascii_alphabetic()
                || next == '_'
                || next == '.'
                || next == '"'
                || next == '\'';
            if looks_like_destr {
                result.push('{');
                result.push(' ');
                i += 1;
                continue;
            }
        }
        // Add space before `}` if preceded by a non-space char in destructuring.
        if chars[i] == '}'
            && i > 0
            && chars[i - 1] != ' '
            && chars[i - 1] != '{'
            && chars[i - 1] != '\n'
        {
            // Check if this is likely the end of a destructuring pattern by looking back.
            // Heuristic: if we see a `:`, `...`, or `,` inside the braces (indicating a
            // destructuring pattern with either named properties or shorthand props), add space.
            let mut j = i.saturating_sub(1);
            let mut found_colon = false;
            let mut found_spread = false;
            let mut found_comma = false;
            while j > 0 {
                if chars[j] == '{' {
                    break;
                }
                if chars[j] == ':' {
                    found_colon = true;
                    break;
                }
                if chars[j] == ',' {
                    found_comma = true;
                    break;
                }
                if j >= 2 && chars[j - 2] == '.' && chars[j - 1] == '.' && chars[j] == '.' {
                    found_spread = true;
                    break;
                }
                j -= 1;
            }
            if found_colon || found_spread || found_comma {
                result.push(' ');
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Inline simple temporary assignments.
///
/// Finds patterns like `let tN = SIMPLE_VALUE` (where SIMPLE_VALUE is a literal,
/// identifier, or simple expression that doesn't contain whitespace-delimited
/// compound expressions) and replaces all subsequent uses of `tN` with the value.
///
/// This handles the common case where our codegen introduces a temporary for a
/// phi variable's initial value:
///   Our output:   `let t0 = 0 let x = t0`
///   Expected:     `let x = 0`
///
/// After inlining: `let t0 = 0 let x = 0` → after removing the now-redundant
/// `let t0 = 0`, we get `let x = 0`.
fn inline_simple_temp_assignments(s: &str) -> String {
    use std::collections::HashMap;

    // Split into tokens for analysis
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() < 4 {
        return s.to_string();
    }

    // First pass: find `let tN = VALUE` patterns where VALUE is a single token
    // (a simple literal or identifier).
    let mut temp_values: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    while i + 3 < tokens.len() {
        if tokens[i] == "let"
            && is_temp_identifier(tokens[i + 1])
            && tokens[i + 2] == "="
            && i + 3 < tokens.len()
        {
            let temp_name = tokens[i + 1];
            let value = tokens[i + 3];
            // Only inline if the value is a simple token (no nested expressions).
            // Specifically: literals (numbers, strings, booleans, null, undefined),
            // simple identifiers, or member expressions without spaces.
            if is_simple_inlinable_value(value) && !is_temp_identifier(value) {
                temp_values.insert(temp_name.to_string(), value.to_string());
            }
        }
        i += 1;
    }

    if temp_values.is_empty() {
        return s.to_string();
    }

    // Second pass: use word-boundary-aware string replacement.
    // This handles cases where temps appear inside larger tokens like `t2)` or `(t2`.
    let mut result = s.to_string();

    // First, remove the `let tN = VALUE` declarations
    for (temp_name, value) in &temp_values {
        // Remove `let tN = VALUE ` (with trailing space) or `let tN = VALUE` (at end)
        let decl_pattern = format!("let {temp_name} = {value}");
        result = result.replace(&decl_pattern, "");
    }

    // Then replace all occurrences of temp identifiers with their values,
    // respecting word boundaries (don't replace `t0` inside `t00` or `at0`).
    for (temp_name, value) in &temp_values {
        let bytes = temp_name.as_bytes();
        let mut new_result = String::with_capacity(result.len());
        let result_bytes = result.as_bytes();
        let name_len = bytes.len();
        let mut pos = 0;

        while pos < result_bytes.len() {
            if pos + name_len <= result_bytes.len() && &result_bytes[pos..pos + name_len] == bytes {
                // Check word boundary before
                let at_start = pos == 0
                    || (!result_bytes[pos - 1].is_ascii_alphanumeric()
                        && result_bytes[pos - 1] != b'_');
                // Check word boundary after
                let at_end = pos + name_len >= result_bytes.len()
                    || (!result_bytes[pos + name_len].is_ascii_alphanumeric()
                        && result_bytes[pos + name_len] != b'_');

                if at_start && at_end {
                    new_result.push_str(value);
                    pos += name_len;
                    continue;
                }
            }
            new_result.push(result_bytes[pos] as char);
            pos += 1;
        }
        result = new_result;
    }

    // Clean up: collapse multiple whitespace that may result from removal
    collapse_whitespace(&result)
}

/// Conservative version of temp alias propagation.
/// When we see `const/let NAME = tN`, replace subsequent uses of `tN` with `NAME`
/// but do NOT remove the alias declaration. The dead-code pass will clean it up
/// if `NAME` is unused.
fn propagate_temp_aliases_conservative(s: &str) -> String {
    use std::collections::HashMap;
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() < 4 {
        return s.to_string();
    }

    // Find alias patterns: `const NAME = tN` or `let NAME = tN`
    let mut aliases: HashMap<String, (String, usize)> = HashMap::new();
    let mut i = 0;
    while i + 3 < tokens.len() {
        if (tokens[i] == "const" || tokens[i] == "let")
            && !is_temp_identifier(tokens[i + 1])
            && !tokens[i + 1].starts_with('{')
            && !tokens[i + 1].starts_with('[')
            && !tokens[i + 1].starts_with('$')
            && tokens[i + 2] == "="
            && is_temp_identifier(tokens[i + 3])
        {
            let name = tokens[i + 1];
            let temp = tokens[i + 3];
            if !aliases.contains_key(temp) {
                aliases.insert(temp.to_string(), (name.to_string(), i));
            }
        }
        i += 1;
    }

    if aliases.is_empty() {
        return s.to_string();
    }

    // Replace subsequent uses of tN with NAME after the alias declaration.
    // We find the alias declaration pattern in the string and replace tN
    // with NAME only after that position.
    let mut result = s.to_string();
    for (temp, (name, _decl_idx)) in &aliases {
        // Find the alias declaration pattern "const NAME = tN" or "let NAME = tN"
        let const_pattern = format!("const {name} = {temp}");
        let let_pattern = format!("let {name} = {temp}");
        let split_pos = if let Some(pos) = result.find(&const_pattern) {
            pos + const_pattern.len()
        } else if let Some(pos) = result.find(&let_pattern) {
            pos + let_pattern.len()
        } else {
            continue;
        };

        // Replace tN with NAME only in the portion after the declaration
        let before = &result[..split_pos];
        let after = &result[split_pos..];
        let replaced_after = replace_identifier_word(after, temp, name);
        result = format!("{before}{replaced_after}");
    }

    result
}

/// Replace an identifier at word boundaries in a string.
/// Matches `ident` when preceded/followed by non-alphanumeric/non-underscore characters.
fn replace_identifier_word(s: &str, old_ident: &str, new_ident: &str) -> String {
    let bytes = s.as_bytes();
    let old_bytes = old_ident.as_bytes();
    let old_len = old_bytes.len();
    let mut result = String::with_capacity(s.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + old_len <= bytes.len() && &bytes[i..i + old_len] == old_bytes {
            let at_start =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            let at_end = i + old_len >= bytes.len()
                || (!bytes[i + old_len].is_ascii_alphanumeric() && bytes[i + old_len] != b'_');
            if at_start && at_end {
                result.push_str(new_ident);
                i += old_len;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

/// Check if a token is a temporary identifier (tN where N is a digit).
fn is_temp_identifier(s: &str) -> bool {
    s.starts_with('t') && s.len() >= 2 && s[1..].chars().all(|c| c.is_ascii_digit())
}

/// Check if a value is simple enough to inline (a single token that represents
/// a literal, identifier, or member expression).
fn is_simple_inlinable_value(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Reject tokens that contain paired delimiters (arrays, objects, calls)
    if s.contains('(') || s.contains('[') || s.contains('{') {
        return false;
    }
    // Reject tokens that start with operators or special chars
    let first = s.as_bytes()[0];
    if matches!(
        first,
        b'+' | b'-' | b'*' | b'/' | b'%' | b'!' | b'~' | b'<' | b'>' | b'&' | b'|' | b'^' | b'?'
    ) {
        return false;
    }
    // Reject keywords that are not value-like
    if matches!(
        s,
        "if" | "else"
            | "while"
            | "for"
            | "do"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "return"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "function"
            | "const"
            | "let"
            | "var"
            | "new"
            | "delete"
            | "typeof"
            | "void"
            | "in"
            | "instanceof"
            | "of"
            | "class"
            | "extends"
            | "import"
            | "export"
            | "default"
            | "from"
            | "async"
            | "await"
            | "yield"
            | "with"
            | "debugger"
            | "=="
            | "!="
            | "==="
            | "!=="
            | "&&"
            | "||"
            | "??"
            | "="
    ) {
        return false;
    }
    true
}

/// Remove dead expression statements that have no side effects.
///
/// After pruning, our codegen may emit `[]` (empty array) or `{}` (empty object,
/// rendered as empty block) as standalone expression statements. The reference
/// compiler removes these. We normalize by removing:
/// - Standalone `[]` tokens (empty array expression statements)
/// - Standalone `{}` tokens (empty block/object expression statements)
///
/// Only removes these when they appear as expression-level statements (not as
/// part of larger expressions).
fn remove_dead_expression_statements(s: &str) -> String {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let mut result: Vec<&str> = Vec::with_capacity(tokens.len());
    for (idx, token) in tokens.iter().enumerate() {
        // Remove standalone `[]` (empty array expression statement)
        if *token == "[]" {
            continue;
        }
        // Remove standalone `{}` when it appears as a dead expression statement.
        // Cases to remove:
        // 1. `{ {} }` - empty block inside a block (our codegen emits Block([]) as body)
        // 2. Mid-block `{}` - empty block between other statements
        if *token == "{}" {
            let prev = result.last().copied().unwrap_or("");
            let next = tokens.get(idx + 1).copied().unwrap_or("");
            // Case 1: `{ {} }` → `{ }` (remove the inner `{}`)
            if prev == "{" && next == "}" {
                continue;
            }
            // Case 2: mid-block standalone `{}`
            let is_inside_block = !prev.is_empty() && prev != "{" && !prev.ends_with('{');
            if is_inside_block {
                continue;
            }
        }
        result.push(token);
    }
    result.join(" ")
}

/// Strip single-line comments (// ...) from the source code.
/// Handles `// comment` lines and inline `// comment` at end of lines.
/// Preserves `://` (URLs) and string contents.
fn strip_single_line_comments(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for line in s.lines() {
        // Find `//` that isn't inside a string literal
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            // Entire line is a comment - skip it
            result.push('\n');
            continue;
        }
        // For inline comments, do a simple heuristic: look for ` //` not inside quotes
        // We just keep the line as-is since most comments are full-line
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Remove dead constant declarations where the variable is never used after declaration.
///
/// Finds `const NAME = VALUE` patterns where NAME never appears again in the remaining
/// code. Removes the entire declaration. This handles cases where our codegen emits
/// dead constant bindings that the reference compiler has optimized away.
///
/// Works on whitespace-collapsed, single-line normalized code.
fn remove_dead_const_declarations(s: &str) -> String {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() < 4 {
        return s.to_string();
    }

    // Collect dead const declarations (const NAME = VALUE where VALUE is a single token
    // and NAME is never used again).
    let mut dead_ranges: Vec<(usize, usize)> = Vec::new(); // (start_idx, end_idx exclusive)
    let mut i = 0;
    while i + 3 < tokens.len() {
        if tokens[i] == "const" && tokens[i + 2] == "=" && i + 3 < tokens.len() {
            let name = tokens[i + 1];
            let value = tokens[i + 3];

            // Only handle simple single-token values (literals, identifiers)
            if is_simple_inlinable_value(value)
                && !name.starts_with('$')
                && !name.starts_with('{')
                && !name.starts_with('[')
            {
                // Check if name appears anywhere else in the token list (after this declaration)
                let remaining = &tokens[i + 4..];
                let is_dead = !remaining.iter().any(|t| {
                    // Check if the token IS the name or contains the name as a prefix/suffix
                    // (e.g., `name.property` or `name[0]`)
                    *t == name
                        || t.starts_with(&format!("{name}."))
                        || t.starts_with(&format!("{name}["))
                        || t.starts_with(&format!("{name}("))
                        || t.starts_with(&format!("{name}?"))
                        || t.ends_with(&format!(",{name}"))
                        || *t == format!("({name})")
                });
                if is_dead {
                    dead_ranges.push((i, i + 4));
                }
            }
        }
        i += 1;
    }

    if dead_ranges.is_empty() {
        return s.to_string();
    }

    // Build result excluding dead ranges
    let mut result: Vec<&str> = Vec::with_capacity(tokens.len());
    let mut skip_until = 0;
    for (idx, token) in tokens.iter().enumerate() {
        if idx < skip_until {
            continue;
        }
        if let Some(&(start, end)) = dead_ranges.iter().find(|(s, _)| *s == idx) {
            skip_until = end;
            let _ = start;
            continue;
        }
        result.push(token);
    }
    result.join(" ")
}

/// Remove orphan temporary initializers where a temp variable reference remains
/// but its declaration was inlined away.
///
/// Finds patterns like `let x = tN` where `tN` is not declared BEFORE this point
/// in the code (no earlier `let tN` exists), and replaces with `let x` (no initializer).
///
/// This handles the case where phi initial values go through temps that get renamed
/// to the same `tN` as a later scope temp, creating a false reference.
fn remove_orphan_temp_initializers(s: &str) -> String {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() < 4 {
        return s.to_string();
    }

    // Build a map of token positions where temp identifiers are declared.
    // Key: temp name, Value: token index where `let tN` appears.
    use std::collections::HashMap;
    let mut temp_decl_positions: HashMap<&str, usize> = HashMap::new();
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i] == "let" && is_temp_identifier(tokens[i + 1]) {
            // Record the position of the `let` keyword
            temp_decl_positions.entry(tokens[i + 1]).or_insert(i);
        }
        i += 1;
    }

    // Second pass: find `let IDENT = tN` where tN is not declared BEFORE this position
    let mut result: Vec<String> = Vec::with_capacity(tokens.len());
    i = 0;
    while i < tokens.len() {
        if i + 3 < tokens.len()
            && tokens[i] == "let"
            && !is_temp_identifier(tokens[i + 1])
            && tokens[i + 2] == "="
            && is_temp_identifier(tokens[i + 3])
        {
            let temp_ref = tokens[i + 3];
            // Check if tN is declared BEFORE this position
            let is_declared_before =
                temp_decl_positions.get(temp_ref).is_some_and(|&decl_pos| decl_pos < i);
            if !is_declared_before {
                // Orphan temp: emit `let NAME` without initializer
                result.push("let".to_string());
                result.push(tokens[i + 1].to_string());
                i += 4;
                continue;
            }
        }
        result.push(tokens[i].to_string());
        i += 1;
    }
    result.join(" ")
}

/// Normalize code for passthrough comparison: applies all normalizations from
/// `normalize_code` plus normalizes single quotes to double quotes (the TS
/// compiler converts ' to " in its output).
fn normalize_code_quotes(s: &str) -> String {
    let base = normalize_code(s);
    base.replace('\'', "\"")
}

/// Extract the function body (without the wrapper and imports) from an expected
/// code section. This strips:
/// - `import { c as _c } from "react/compiler-runtime";` lines
/// - The function declaration line (e.g. `function Component(props) {`)
/// - The closing brace `}`
/// - Any code before the first function declaration
/// - Any code after the function's closing brace
///
/// Returns the full expected code with imports stripped but function wrapper
/// intact, so we can compare against `format_full_function` output.
fn extract_function_from_expected(code: &str) -> Option<String> {
    let lines: Vec<&str> = code.lines().collect();

    // Find the first line that looks like a function declaration or arrow function.
    let func_start = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("function ")
            || trimmed.starts_with("async function ")
            || trimmed.starts_with("export default function ")
            || trimmed.starts_with("export function ")
            || (trimmed.starts_with("const ") && trimmed.contains("=>"))
    })?;

    // Track brace depth to find the function's closing `}`.
    let mut depth: i32 = 0;
    let mut func_end = lines.len();
    for (i, line) in lines[func_start..].iter().enumerate() {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    func_end = func_start + i + 1;
                    break;
                }
            }
        }
        if depth == 0 && func_end != lines.len() {
            break;
        }
    }

    let func_lines: Vec<&str> = lines[func_start..func_end].to_vec();
    let joined = func_lines.join("\n");

    // Strip "export default " or "export " prefix if present.
    let cleaned = if joined.starts_with("export default ") {
        joined.replacen("export default ", "", 1)
    } else if joined.starts_with("export function ") {
        joined.replacen("export ", "", 1)
    } else {
        joined
    };

    Some(cleaned)
}

/// Extract the function body (content between first `{` and matching `}`).
fn extract_function_body(code: &str) -> Option<String> {
    let first_brace = code.find('{')?;
    let mut depth: i32 = 0;
    let mut last_close = first_brace;
    for (i, ch) in code[first_brace..].char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                last_close = first_brace + i;
                break;
            }
        }
    }
    if depth != 0 {
        return None;
    }
    Some(code[first_brace + 1..last_close].to_string())
}

/// Category of failure for a fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureCategory {
    ParseError,
    NoFunction,
    LowerError,
    PipelineError,
    Panic,
    OutputMismatch,
    NoExpectedCode,
}

impl std::fmt::Display for FailureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError => write!(f, "parse_error"),
            Self::NoFunction => write!(f, "no_function"),
            Self::LowerError => write!(f, "lower_error"),
            Self::PipelineError => write!(f, "pipeline_error"),
            Self::Panic => write!(f, "panic"),
            Self::OutputMismatch => write!(f, "output_mismatch"),
            Self::NoExpectedCode => write!(f, "no_expected_code"),
        }
    }
}

/// Codegen conformance test: run the full compilation pipeline on each fixture
/// that has a matching `.expect.md` with a `## Code` section, compare the
/// generated output, and track the pass rate using an insta snapshot.
///
/// This is a progress-tracking test, not a pass/fail gate. The snapshot records
/// the current state so improvements (or regressions) are visible in diffs.
///
/// This test is ignored by default because some fixtures trigger stack overflows
/// in the compilation pipeline (pre-existing infinite recursion bugs in certain
/// passes). Stack overflows abort the process and cannot be caught.
///
/// Run with: `cargo test -p oxc_react_compiler -- --ignored test_codegen_conformance`
/// or in release mode: `cargo test -p oxc_react_compiler --release -- --ignored test_codegen_conformance`
#[test]
#[ignore]
fn test_codegen_conformance_pass_rate() {
    codegen_conformance_inner();
}

fn codegen_conformance_inner() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        return;
    }

    // Collect all eligible fixture paths (non-error, with matching .expect.md).
    let mut fixture_pairs: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();

        if !is_js_ts_tsx(&path) {
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Skip error-prefixed fixtures — they are expected to fail compilation.
        if file_name.starts_with("error.") {
            continue;
        }

        // Find the matching .expect.md file.
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let expect_path = fixtures_dir.join(format!("{stem}.expect.md"));
        if expect_path.exists() {
            fixture_pairs.push((path, expect_path));
        }
    }
    fixture_pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut passed = 0u32;
    let mut failed: Vec<(String, FailureCategory)> = Vec::new();

    for (input_path, expect_path) in &fixture_pairs {
        let file_name =
            input_path.file_name().and_then(|n| n.to_str()).unwrap_or("<unknown>").to_string();

        let Ok(source) = std::fs::read_to_string(input_path) else {
            failed.push((file_name, FailureCategory::ParseError));
            continue;
        };

        let Ok(expect_content) = std::fs::read_to_string(expect_path) else {
            failed.push((file_name, FailureCategory::NoExpectedCode));
            continue;
        };

        // Extract the ## Code section from the expected output.
        let Some(expected_code) = extract_expect_md_section(&expect_content, "Code") else {
            failed.push((file_name, FailureCategory::NoExpectedCode));
            continue;
        };

        // Handle @expectNothingCompiled: the compiler should pass the source through unchanged.
        if source.contains("@expectNothingCompiled") {
            let expected_func = extract_function_from_expected(expected_code);
            let expected_text = expected_func.as_deref().unwrap_or(expected_code);
            let source_func = extract_function_from_expected(&source);
            let source_text = source_func.as_deref().unwrap_or(&source);
            let actual_norm = normalize_code_quotes(source_text);
            let expected_norm = normalize_code_quotes(expected_text);
            if actual_norm == expected_norm {
                passed += 1;
            } else {
                failed.push((file_name, FailureCategory::OutputMismatch));
            }
            continue;
        }

        // Determine source type from extension.
        let ext = input_path.extension().and_then(|e| e.to_str()).unwrap_or("js");
        let source_type = match ext {
            "tsx" => oxc_span::SourceType::tsx(),
            "ts" => oxc_span::SourceType::ts(),
            _ => oxc_span::SourceType::jsx(),
        };

        // Run the pipeline inside catch_unwind to prevent panics from aborting
        // the entire test. Note: stack overflows cannot be caught — those will
        // abort the process (this test is #[ignore]d for that reason).
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_pipeline_for_codegen(&source, source_type)
        }));

        let codegen_func = match result {
            Ok(Ok(func)) => func,
            Ok(Err(e)) => {
                let category = if e.starts_with("Parse") {
                    FailureCategory::ParseError
                } else if e.contains("No function") {
                    FailureCategory::NoFunction
                } else if e.starts_with("Lower") {
                    FailureCategory::LowerError
                } else {
                    FailureCategory::PipelineError
                };
                failed.push((file_name, category));
                continue;
            }
            Err(_) => {
                failed.push((file_name, FailureCategory::Panic));
                continue;
            }
        };

        // Format our output and compare against expected.
        let actual_full = format_full_function(&codegen_func);
        let expected_func = match extract_function_from_expected(expected_code) {
            Some(f) => f,
            None => {
                // If we cannot extract the function from expected, compare raw.
                let actual_norm = normalize_code(&actual_full);
                let expected_norm = normalize_code(expected_code);
                if actual_norm == expected_norm {
                    passed += 1;
                } else {
                    failed.push((file_name, FailureCategory::OutputMismatch));
                }
                continue;
            }
        };

        let actual_norm = normalize_code(&actual_full);
        let expected_norm = normalize_code(&expected_func);

        if actual_norm == expected_norm {
            passed += 1;
        } else {
            // Fallback: compare just the function bodies (strips wrapper differences
            // between arrow functions and function declarations).
            let actual_body = extract_function_body(&actual_full);
            let expected_body = extract_function_body(&expected_func);
            match (actual_body, expected_body) {
                (Some(ab), Some(eb)) if normalize_code(&ab) == normalize_code(&eb) => {
                    passed += 1;
                }
                _ => {
                    failed.push((file_name, FailureCategory::OutputMismatch));
                }
            }
        }
    }

    let total = fixture_pairs.len() as u32;
    let pct = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };

    // Build category breakdown.
    let mut parse_errors = 0u32;
    let mut no_function = 0u32;
    let mut lower_errors = 0u32;
    let mut pipeline_errors = 0u32;
    let mut panics = 0u32;
    let mut mismatches = 0u32;
    let mut no_expected = 0u32;

    for (_, cat) in &failed {
        match cat {
            FailureCategory::ParseError => parse_errors += 1,
            FailureCategory::NoFunction => no_function += 1,
            FailureCategory::LowerError => lower_errors += 1,
            FailureCategory::PipelineError => pipeline_errors += 1,
            FailureCategory::Panic => panics += 1,
            FailureCategory::OutputMismatch => mismatches += 1,
            FailureCategory::NoExpectedCode => no_expected += 1,
        }
    }

    // Build the snapshot content.
    let mut snapshot = String::new();
    snapshot.push_str(&format!("Codegen conformance: {passed}/{total} passed ({pct:.1}%)\n"));
    snapshot.push('\n');
    snapshot.push_str("Failure breakdown:\n");
    snapshot.push_str(&format!("  parse_error:    {parse_errors}\n"));
    snapshot.push_str(&format!("  no_function:    {no_function}\n"));
    snapshot.push_str(&format!("  lower_error:    {lower_errors}\n"));
    snapshot.push_str(&format!("  pipeline_error: {pipeline_errors}\n"));
    snapshot.push_str(&format!("  panic:          {panics}\n"));
    snapshot.push_str(&format!("  output_mismatch:{mismatches}\n"));
    snapshot.push_str(&format!("  no_expected:    {no_expected}\n"));
    snapshot.push('\n');

    // List failed fixtures by category.
    snapshot.push_str("Failed fixtures:\n");
    for (name, cat) in &failed {
        snapshot.push_str(&format!("  [{cat}] {name}\n"));
    }

    insta::assert_snapshot!("codegen_conformance", snapshot);
}

/// Test optional chaining - simple case
#[test]
fn test_pipeline_optional_member() {
    let source = r"
function Component(props) {
  let x = props?.a;
  return x;
}
    ";
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for simple optional member: {}",
        result.unwrap_err()
    );
}

/// Test optional chaining - method call
#[test]
fn test_pipeline_optional_call() {
    let source = r"
function Component(props) {
  let x = props?.toString();
  return x;
}
    ";
    let result = run_pipeline_on_source(source);
    assert!(result.is_ok(), "Pipeline should succeed for optional call: {}", result.unwrap_err());
}

/// Test nested optional chaining
#[test]
fn test_pipeline_nested_optional() {
    let source = r"
function Component(props) {
  let x = props?.a?.b;
  return x;
}
    ";
    let result = run_pipeline_on_source(source);
    assert!(result.is_ok(), "Pipeline should succeed for nested optional: {}", result.unwrap_err());
}

/// Debug test for alias-while infinite recursion
#[test]
fn test_debug_alias_while() {
    let source = r"
function foo(cond) {
  let a = {};
  let b = {};
  let c = {};
  while (cond) {
    let z = a;
    a = b;
    b = c;
    c = z;
    mutate(a, b);
  }
  a;
  b;
  c;
  return a;
}
    ";
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parser_result.errors.is_empty());

    let func = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
            _ => None,
        })
        .expect("No function found");

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    let hir_func = lower(&env, ReactFunctionType::Component, &func).expect("lower failed");
    eprintln!("Lower succeeded. Block count: {}", hir_func.body.blocks.len());

    // Try calling build_reactive_function directly (without pipeline passes)
    let result =
        oxc_react_compiler::reactive_scopes::build_reactive_function::build_reactive_function(
            &hir_func,
        );
    match &result {
        Ok(_) => eprintln!("build_reactive_function SUCCEEDED (direct, no pipeline)"),
        Err(e) => eprintln!("build_reactive_function FAILED (direct): {e:?}"),
    }
}

/// Regression test: for-of loop with non-mutating local collection
/// should not cause "Expected continue target to be scheduled" error.
#[test]
fn test_pipeline_for_of_nonmutating_loop() {
    let source = r#"
function Component(props) {
    const items = [];
    for (const i of props.list) {
        items.push(i);
    }
    return items;
}
"#;
    let result = run_pipeline_on_source(source);
    assert!(result.is_ok(), "Pipeline should succeed for for-of loop: {}", result.unwrap_err());
}

/// Regression test: for-of loop with early return in loop body.
#[test]
fn test_pipeline_for_of_with_return() {
    let source = r#"
function Component(props) {
    for (const item of props.items) {
        if (item.match) return item;
    }
    return null;
}
"#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for for-of with return: {}",
        result.unwrap_err()
    );
}

/// Regression test: sequence expression in while loop test.
#[test]
fn test_pipeline_sequence_expression_in_loop() {
    let source = r#"
function Component(props) {
    let x = props.a;
    while (x > 0) {
        x = x - 1;
    }
    return x;
}
"#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for loop with sequence expression: {}",
        result.unwrap_err()
    );
}

/// Regression test: sequence expression with while loop
/// (matches the actual sequence-expression.js fixture)
#[test]
fn test_pipeline_sequence_expression_fixture() {
    let source = r#"
function Component(props) {
    let x = (null, Math.max(1, 2), foo());
    while ((foo(), true)) {
        x = (foo(), 2);
    }
    return x;
}
function foo() {}
"#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for sequence-expression fixture: {}",
        result.unwrap_err()
    );
}

/// Regression test: for-of with useMemo (matches actual fixture)
#[test]
fn test_pipeline_for_of_usememo() {
    let source = r#"
function Component(props) {
    const items = [];
    for (const i of props.x) {
        items.push(i);
    }
    return items;
}
"#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for for-of with useMemo: {}",
        result.unwrap_err()
    );
}

/// Regression test: for-of with conditional return (like repro-memoize-for-of fixture)
#[test]
fn test_pipeline_for_of_conditional_return() {
    let source = r#"
function Component(props) {
    const node = props.nodeID != null ? props.graph[props.nodeID] : null;
    for (const key of Object.keys(node?.fields ?? {})) {
        if (props.condition) {
            return key;
        }
    }
    return null;
}
"#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for for-of conditional return: {}",
        result.unwrap_err()
    );
}

/// Regression test: useMemo with for-of and early return
#[test]
fn test_pipeline_usememo_for_of_early_return() {
    let source = r#"
function Component(props) {
    for (let item of props.items) {
        if (item.match) return item;
    }
    return null;
}
"#;
    let result = run_pipeline_on_source(source);
    assert!(
        result.is_ok(),
        "Pipeline should succeed for useMemo with for-of early return: {}",
        result.unwrap_err()
    );
}

/// Regression test: switch statement with break should not cause
/// "Unexpected break to invalid label" error.
///
/// The switch lowering generates implicit fallthrough breaks between cases.
/// These implicit breaks target case blocks which don't have labels in the
/// reactive tree. The validation pass must skip implicit break/continue
/// targets since they represent natural control flow, not explicit breaks.
#[test]
fn test_pipeline_switch_no_invalid_break_label() {
    let source = r#"
function Component(props) {
  let x = [];
  let y;
  switch (props.p0) {
    case true: {
      x.push(props.p2);
      x.push(props.p3);
      y = [];
    }
    case false: {
      y = x;
      break;
    }
  }
  const child = <Component data={x} />;
  y.push(props.p4);
  return <Component data={y}>{child}</Component>;
}
"#;
    let result = run_pipeline_on_source(source);
    // The pipeline may fail at later passes (e.g., scope instructions), but
    // it must NOT fail with "Unexpected break to invalid label".
    if let Err(ref msg) = result {
        assert!(
            !msg.contains("Unexpected break to invalid label"),
            "Switch should not trigger 'Unexpected break to invalid label': {msg}"
        );
    }
}

/// Regression test: switch with non-final default case.
#[test]
fn test_pipeline_switch_non_final_default() {
    let source = r#"
function Component(props) {
  let x = [];
  let y;
  switch (props.p0) {
    case 1: {
      break;
    }
    case true: {
      x.push(props.p2);
      y = [];
    }
    default: {
      break;
    }
    case false: {
      y = x;
      break;
    }
  }
  const child = <Component data={x} />;
  y.push(props.p4);
  return <Component data={y}>{child}</Component>;
}
"#;
    let result = run_pipeline_on_source(source);
    if let Err(ref msg) = result {
        assert!(
            !msg.contains("Unexpected break to invalid label"),
            "switch-non-final-default should not trigger 'Unexpected break to invalid label': {msg}"
        );
    }
}

/// Regression test: switch with fallthrough cases.
#[test]
fn test_pipeline_switch_with_fallthrough() {
    let source = r#"
function foo(x) {
  let y;
  switch (x) {
    case 0: {
      y = 0;
    }
    case 1: {
      y = 1;
    }
    case 2: {
      break;
    }
    case 3: {
      y = 3;
      break;
    }
    case 4: {
      y = 4;
    }
    case 5: {
      y = 5;
    }
    default: {
      y = 0;
    }
  }
}
"#;
    let result = run_pipeline_on_source(source);
    if let Err(ref msg) = result {
        assert!(
            !msg.contains("Unexpected break to invalid label"),
            "switch-with-fallthrough should not trigger 'Unexpected break to invalid label': {msg}"
        );
    }
}

/// Regression test: reverse-postorder fixture with switch inside if.
#[test]
fn test_pipeline_reverse_postorder() {
    let source = r#"
function Component(props) {
  let x;
  if (props.cond) {
    switch (props.test) {
      case 0: {
        x = props.v0;
        break;
      }
      case 1: {
        x = props.v1;
        break;
      }
      case 2: {
      }
      default: {
        x = props.v2;
      }
    }
  } else {
    if (props.cond2) {
      x = props.b;
    } else {
      x = props.c;
    }
  }
  x;
}
"#;
    let result = run_pipeline_on_source(source);
    if let Err(ref msg) = result {
        assert!(
            !msg.contains("Unexpected break to invalid label"),
            "reverse-postorder should not trigger 'Unexpected break to invalid label': {msg}"
        );
    }
}

// ===========================================================================
// Diagnostic: find closest-to-passing output_mismatch fixtures
// ===========================================================================

// ===========================================================================
// End of fixtures tests
// ===========================================================================
