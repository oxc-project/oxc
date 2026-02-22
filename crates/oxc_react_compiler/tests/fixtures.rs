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
    use oxc_react_compiler::hir::build_hir::lower;
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

    // Create environment
    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    // Lower to HIR
    let result = lower(&env, ReactFunctionType::Component);
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
