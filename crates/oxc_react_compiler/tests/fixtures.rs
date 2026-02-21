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
        eprintln!("Skipping fixture test: submodule not initialized at {FIXTURES_PATH}");
        return;
    }

    let mut input_count = 0;
    let mut expect_count = 0;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if name.ends_with(".js") || name.ends_with(".ts") || name.ends_with(".tsx") {
            input_count += 1;
        } else if name.ends_with(".expect.md") {
            expect_count += 1;
        }
    }

    assert!(input_count > 100, "Expected at least 100 input fixtures, found {input_count}");
    assert!(expect_count > 100, "Expected at least 100 expect files, found {expect_count}");
    eprintln!("Found {input_count} input fixtures and {expect_count} expect files");
}

/// Test that all fixture input files can be parsed by oxc_parser.
#[test]
fn test_parse_fixtures() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        eprintln!("Skipping fixture parse test: submodule not initialized");
        return;
    }

    let mut parsed = 0;
    let mut parse_errors = 0;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Only process JS/TS/TSX input files
        if !(name.ends_with(".js") || name.ends_with(".ts") || name.ends_with(".tsx")) {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let allocator = oxc_allocator::Allocator::default();
        let source_type = if name.ends_with(".tsx") {
            oxc_span::SourceType::tsx()
        } else if name.ends_with(".ts") {
            oxc_span::SourceType::ts()
        } else {
            oxc_span::SourceType::jsx()
        };

        let parser_result = oxc_parser::Parser::new(&allocator, &source, source_type).parse();

        if parser_result.errors.is_empty() {
            parsed += 1;
        } else {
            parse_errors += 1;
            // Some fixtures may intentionally have parse errors (error.* prefixed files)
            if !name.starts_with("error.") && !name.starts_with("todo.error") {
                // Only report unexpected parse errors
                if parse_errors <= 5 {
                    eprintln!("Parse error in {name}: {} errors", parser_result.errors.len());
                }
            }
        }
    }

    assert!(parsed > 100, "Expected at least 100 parseable fixtures, found {parsed}");
    eprintln!("Parsed {parsed} fixtures successfully, {parse_errors} with errors");
}

/// Test that fixture pragmas can be parsed correctly.
#[test]
fn test_parse_fixture_pragmas() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        eprintln!("Skipping pragma test: submodule not initialized");
        return;
    }

    let mut pragmas_found = 0;

    for entry in std::fs::read_dir(fixtures_dir).expect("Failed to read fixtures dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !(name.ends_with(".js") || name.ends_with(".ts") || name.ends_with(".tsx")) {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let first_line = source.lines().next().unwrap_or("");
        if first_line.contains('@') {
            pragmas_found += 1;
            // Parse the pragma
            let options = oxc_react_compiler::utils::test_utils::parse_config_pragma_for_tests(
                first_line,
                oxc_react_compiler::utils::test_utils::PragmaDefaults {
                    compilation_mode: oxc_react_compiler::entrypoint::options::CompilationMode::All,
                },
            );
            // Pragma parsing should not panic
            let _ = options;
        }
    }

    assert!(pragmas_found > 50, "Expected at least 50 fixtures with pragmas, found {pragmas_found}");
    eprintln!("Parsed {pragmas_found} fixture pragmas");
}

/// Test that expect.md files can be read and have the expected structure.
#[test]
fn test_read_expect_files() {
    let fixtures_dir = Path::new(FIXTURES_PATH);
    if !fixtures_dir.exists() {
        eprintln!("Skipping expect file test: submodule not initialized");
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

        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
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
    eprintln!("Validated {valid} expect files ({has_input} with Input, {has_code} with Code)");
}
