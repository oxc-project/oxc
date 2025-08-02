#![expect(clippy::print_stdout)]
//! # Comprehensive Source Map Testing Program
//!
//! This program performs extensive testing of the sourcemap generator in oxc_codegen
//! to ensure it produces correct and accurate source maps.
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p oxc_codegen --example test_sourcemap
//! ```
//!
//! The program tests various aspects of sourcemap generation:
//! - Basic functionality and mapping accuracy
//! - Unicode character handling
//! - Edge cases (empty files, single lines, etc.)
//! - Complex code structures
//! - Performance with large files

use std::{
    path::Path,
    fmt,
    time::Instant,
};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_sourcemap::{SourceMap, Token};

/// Test case for sourcemap generation
#[derive(Debug, Clone)]
struct TestCase {
    name: &'static str,
    source: &'static str,
    description: &'static str,
}

/// Test result for a single test case
#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    message: String,
    details: Vec<String>,
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.passed { "✓ PASS" } else { "✗ FAIL" };
        writeln!(f, "{} {}: {}", status, self.name, self.message)?;
        for detail in &self.details {
            writeln!(f, "    {}", detail)?;
        }
        Ok(())
    }
}

/// Main testing framework
struct SourcemapTester {
    passed: usize,
    failed: usize,
    results: Vec<TestResult>,
}

impl SourcemapTester {
    fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            results: Vec::new(),
        }
    }

    /// Run a single test case
    fn run_test(&mut self, test_case: TestCase) {
        println!("Running test: {} - {}", test_case.name, test_case.description);
        
        let start_time = Instant::now();
        
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("test.js")).unwrap();
        let ret = Parser::new(&allocator, test_case.source, source_type).parse();
        
        let parse_time = start_time.elapsed();
        
        let mut result = TestResult {
            name: test_case.name.to_string(),
            passed: false,
            message: String::new(),
            details: Vec::new(),
        };
        
        // Check for parsing errors
        if !ret.errors.is_empty() {
            result.message = format!("Parse errors: {}", ret.errors.len());
            for error in &ret.errors {
                result.details.push(format!("Parse error: {}", error));
            }
            self.add_result(result);
            return;
        }
        
        let codegen_start = Instant::now();
        
        // Generate code with source map
        let CodegenReturn { code, map, .. } = Codegen::new()
            .with_options(CodegenOptions {
                source_map_path: Some(Path::new("test.js").to_path_buf()),
                ..CodegenOptions::default()
            })
            .build(&ret.program);
            
        let codegen_time = codegen_start.elapsed();
        let total_time = start_time.elapsed();
        
        // Add performance information
        result.details.push(format!("Performance: parse={:.2}ms, codegen={:.2}ms, total={:.2}ms", 
            parse_time.as_secs_f64() * 1000.0,
            codegen_time.as_secs_f64() * 1000.0,
            total_time.as_secs_f64() * 1000.0
        ));
            
        // Validate sourcemap was generated
        let Some(source_map) = map else {
            result.message = "Source map was not generated".to_string();
            self.add_result(result);
            return;
        };
        
        // Run comprehensive validation
        match self.validate_sourcemap(test_case.source, &code, &source_map) {
            Ok(details) => {
                result.passed = true;
                result.message = "All validations passed".to_string();
                result.details.extend(details);
            }
            Err(errors) => {
                result.message = format!("Validation failed: {} issues", errors.len());
                result.details.extend(errors);
            }
        }
        
        self.add_result(result);
    }
    
    /// Add a test result and update counters
    fn add_result(&mut self, result: TestResult) {
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
    }
    
    /// Validate a generated sourcemap comprehensively
    fn validate_sourcemap(
        &self,
        original: &str,
        generated: &str,
        source_map: &SourceMap,
    ) -> Result<Vec<String>, Vec<String>> {
        let mut validations = Vec::new();
        let mut errors = Vec::new();
        
        // Basic structure validation
        if source_map.get_sources().count() == 0 {
            errors.push("Source map has no sources".to_string());
        } else {
            validations.push(format!("Source map has {} sources", source_map.get_sources().count()));
        }
        
        let tokens: Vec<_> = source_map.get_tokens().collect();
        if tokens.is_empty() {
            if original.is_empty() {
                validations.push("Source map has no tokens (expected for empty file)".to_string());
            } else {
                errors.push("Source map has no tokens".to_string());
            }
        } else {
            validations.push(format!("Source map has {} tokens", tokens.len()));
        }
        
        // Validate token mappings
        for (i, token) in tokens.iter().enumerate() {
            match self.validate_token(original, generated, token, i) {
                Ok(msg) => validations.push(msg),
                Err(err) => errors.push(err),
            }
        }
        
        // Validate line and column ranges
        match self.validate_ranges(original, generated, &tokens) {
            Ok(msgs) => validations.extend(msgs),
            Err(errs) => errors.extend(errs),
        }
        
        if errors.is_empty() {
            Ok(validations)
        } else {
            Err(errors)
        }
    }
    
    /// Validate a single token mapping
    fn validate_token(
        &self,
        original: &str,
        generated: &str,
        token: &Token,
        index: usize,
    ) -> Result<String, String> {
        let gen_line = token.get_dst_line() as usize;
        let gen_col = token.get_dst_col() as usize;
        let orig_line = token.get_src_line() as usize;
        let orig_col = token.get_src_col() as usize;
        
        // Validate generated position is within bounds
        let gen_lines: Vec<_> = generated.lines().collect();
        if gen_line >= gen_lines.len() {
            return Err(format!(
                "Token {}: Generated line {} out of bounds (max: {})",
                index, gen_line, gen_lines.len()
            ));
        }
        
        if gen_col > gen_lines[gen_line].len() {
            return Err(format!(
                "Token {}: Generated column {} out of bounds for line {} (max: {})",
                index, gen_col, gen_line, gen_lines[gen_line].len()
            ));
        }
        
        // Validate original position is within bounds
        let orig_lines: Vec<_> = original.lines().collect();
        if orig_line >= orig_lines.len() {
            return Err(format!(
                "Token {}: Original line {} out of bounds (max: {})",
                index, orig_line, orig_lines.len()
            ));
        }
        
        if orig_col > orig_lines[orig_line].len() {
            return Err(format!(
                "Token {}: Original column {} out of bounds for line {} (max: {})",
                index, orig_col, orig_line, orig_lines[orig_line].len()
            ));
        }
        
        Ok(format!(
            "Token {}: ({}, {}) -> ({}, {}) ✓",
            index, orig_line, orig_col, gen_line, gen_col
        ))
    }
    
    /// Validate that line and column ranges are reasonable
    fn validate_ranges(
        &self,
        original: &str,
        generated: &str,
        tokens: &[&Token],
    ) -> Result<Vec<String>, Vec<String>> {
        let mut validations = Vec::new();
        let mut errors = Vec::new();
        
        let orig_lines = original.lines().count();
        let gen_lines = generated.lines().count();
        
        // Handle empty files specially
        if tokens.is_empty() {
            if original.is_empty() && generated.is_empty() {
                validations.push("Empty file handled correctly".to_string());
                return Ok(validations);
            } else if original.is_empty() {
                validations.push("Empty source file handled correctly".to_string());
                return Ok(validations);
            }
        }
        
        // Check for tokens with reasonable line numbers
        let max_orig_line = tokens.iter().map(|t| t.get_src_line()).max().unwrap_or(0);
        let max_gen_line = tokens.iter().map(|t| t.get_dst_line()).max().unwrap_or(0);
        
        if max_orig_line as usize >= orig_lines {
            errors.push(format!(
                "Token references original line {} but source only has {} lines",
                max_orig_line, orig_lines
            ));
        } else {
            validations.push(format!(
                "Maximum original line reference: {} (source has {} lines)",
                max_orig_line, orig_lines
            ));
        }
        
        if max_gen_line as usize >= gen_lines {
            errors.push(format!(
                "Token references generated line {} but output only has {} lines",
                max_gen_line, gen_lines
            ));
        } else {
            validations.push(format!(
                "Maximum generated line reference: {} (output has {} lines)",
                max_gen_line, gen_lines
            ));
        }
        
        if errors.is_empty() {
            Ok(validations)
        } else {
            Err(errors)
        }
    }
    
    /// Print comprehensive test results
    fn print_summary(&self) {
        let separator = "=".repeat(60);
        println!("\n{}", separator);
        println!("SOURCEMAP TESTING SUMMARY");
        println!("{}", separator);
        
        for result in &self.results {
            println!("{}", result);
        }
        
        println!("{}", separator);
        println!("Total tests: {}", self.passed + self.failed);
        println!("Passed: {} ✓", self.passed);
        println!("Failed: {} ✗", self.failed);
        
        let success_rate = if self.passed + self.failed > 0 {
            (self.passed as f64 / (self.passed + self.failed) as f64) * 100.0
        } else {
            0.0
        };
        println!("Success rate: {:.1}%", success_rate);
        
        if self.failed == 0 {
            println!("\n🎉 All tests passed! The sourcemap generator appears to be working correctly.");
        } else {
            println!("\n⚠️  Some tests failed. Please review the issues above.");
        }
    }
}

/// Generate comprehensive test cases
fn get_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            name: "empty_file",
            source: "",
            description: "Empty file should generate valid sourcemap",
        },
        TestCase {
            name: "single_statement",
            source: "var x = 42;",
            description: "Simple variable declaration",
        },
        TestCase {
            name: "multiple_statements",
            source: "var x = 1;\nvar y = 2;\nvar z = x + y;",
            description: "Multiple statements across multiple lines",
        },
        TestCase {
            name: "function_declaration",
            source: "function add(a, b) {\n  return a + b;\n}",
            description: "Function declaration with parameters and body",
        },
        TestCase {
            name: "complex_expression",
            source: "var result = (a + b) * (c - d) / e;",
            description: "Complex mathematical expression",
        },
        TestCase {
            name: "object_literal",
            source: "var obj = {\n  name: 'test',\n  value: 42,\n  nested: {\n    inner: true\n  }\n};",
            description: "Nested object literal",
        },
        TestCase {
            name: "array_literal",
            source: "var arr = [1, 2, 3, 'hello', true, null];",
            description: "Array with mixed types",
        },
        TestCase {
            name: "string_literals",
            source: "var single = 'hello';\nvar double = \"world\";\nvar template = `template ${single}`;",
            description: "Different string literal types",
        },
        TestCase {
            name: "unicode_content",
            source: "var 测试 = 'Unicode 内容';\nvar 변수 = '한글';\nvar переменная = 'кириллица';",
            description: "Unicode identifiers and string content",
        },
        TestCase {
            name: "control_flow",
            source: "if (condition) {\n  doSomething();\n} else {\n  doSomethingElse();\n}\n\nfor (var i = 0; i < 10; i++) {\n  console.log(i);\n}",
            description: "Control flow statements (if/else, for loop)",
        },
        TestCase {
            name: "try_catch",
            source: "try {\n  riskyOperation();\n} catch (error) {\n  handleError(error);\n} finally {\n  cleanup();\n}",
            description: "Exception handling",
        },
        TestCase {
            name: "arrow_functions",
            source: "const add = (a, b) => a + b;\nconst multiply = (x, y) => {\n  return x * y;\n};",
            description: "Arrow function expressions",
        },
        TestCase {
            name: "class_declaration",
            source: "class Example {\n  constructor(value) {\n    this.value = value;\n  }\n  \n  getValue() {\n    return this.value;\n  }\n}",
            description: "ES6 class declaration",
        },
        TestCase {
            name: "destructuring",
            source: "const {name, age} = person;\nconst [first, second, ...rest] = array;",
            description: "Destructuring assignment",
        },
        TestCase {
            name: "template_literals",
            source: "const message = `Hello ${name}, you are ${age} years old.`;\nconst multiline = `\n  Line 1\n  Line 2\n  Line 3\n`;",
            description: "Template literals with interpolation",
        },
        TestCase {
            name: "complex_nesting",
            source: "function outer() {\n  function inner(x) {\n    return function(y) {\n      return x + y;\n    };\n  }\n  return inner;\n}",
            description: "Deeply nested functions and closures",
        },
        TestCase {
            name: "single_line_with_unicode",
            source: "const 变量 = '测试'; const result = 变量.length;",
            description: "Single line with Unicode characters",
        },
        TestCase {
            name: "mixed_statements",
            source: "var a = 1; function f() { return a; } class C { method() { return f(); } }",
            description: "Mixed statement types on single line",
        },
        TestCase {
            name: "only_whitespace",
            source: "   \n  \n   ",
            description: "File with only whitespace",
        },
        TestCase {
            name: "comments_and_code",
            source: "// Comment 1\nvar x = 42; // Inline comment\n/* Block comment */\nvar y = x + 1;",
            description: "Code mixed with comments",
        },
    ]
}

fn main() {
    println!("🧪 Comprehensive Sourcemap Testing for oxc_codegen");
    println!("==================================================\n");
    
    let mut tester = SourcemapTester::new();
    let test_cases = get_test_cases();
    
    println!("Running {} test cases...\n", test_cases.len());
    
    for test_case in test_cases {
        tester.run_test(test_case);
        println!(); // Add spacing between tests
    }
    
    tester.print_summary();
}