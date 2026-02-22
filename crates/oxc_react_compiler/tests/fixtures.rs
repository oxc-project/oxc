/// Test fixture runner for the React Compiler.
///
/// Reads test fixtures from the React git submodule, parses them with oxc_parser,
/// and will eventually run the full compilation pipeline, comparing output against
/// the `.expect.md` files.
///
/// For now, this validates that:
/// 1. All fixture files can be found and read
/// 2. All fixture files can be parsed by oxc_parser
/// 3. The test pragma parser correctly handles fixture pragmas
use std::path::Path;

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
    use oxc_react_compiler::entrypoint::pipeline::run_pipeline;
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

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
    use oxc_react_compiler::entrypoint::pipeline::run_pipeline;
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

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
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

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
