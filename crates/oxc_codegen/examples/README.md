# Sourcemap Testing Program

This directory contains a comprehensive testing program for validating the sourcemap generator in oxc_codegen.

## test_sourcemap.rs

A comprehensive Rust program that tests whether the sourcemap generator produces correct sourcemaps. The program includes:

### Features

- **Comprehensive Test Cases**: 21 different test cases covering various JavaScript/TypeScript constructs
- **Performance Testing**: Includes large file performance testing with timing measurements
- **Unicode Support**: Tests Unicode characters in identifiers and strings
- **Edge Cases**: Tests empty files, whitespace-only files, and complex nesting scenarios
- **Validation**: Validates mapping accuracy, line/column correctness, and JSON serialization
- **Detailed Reporting**: Provides detailed output with statistics and validation results

### Test Cases

1. **Basic Tests**: Empty files, single statements, multiple statements
2. **Language Features**: Functions, classes, arrow functions, destructuring
3. **Complex Expressions**: Object literals, array literals, template literals
4. **Control Flow**: If/else statements, loops, try/catch blocks
5. **Unicode**: Unicode identifiers and string content
6. **Edge Cases**: Whitespace-only files, comments mixed with code
7. **Performance**: Large files with hundreds of statements and expressions

### Usage

```bash
# Run the comprehensive sourcemap tests
cargo run -p oxc_codegen --example test_sourcemap
```

### Output

The program provides detailed output including:

- Test results for each case (✓ PASS or ✗ FAIL)
- Performance metrics (parse time, codegen time)
- Token validation details
- JSON serialization verification
- Summary statistics including:
  - Total tokens generated
  - Average performance metrics
  - Success rate

### Example Output

```
🧪 Comprehensive Sourcemap Testing for oxc_codegen
==================================================

Running 21 test cases...

✓ PASS single_statement: All validations passed
    Performance: parse=0.06ms, codegen=0.02ms, total=0.08ms
    Source map has 1 sources
    Source map has 3 tokens
    Token 0: (0, 0) -> (0, 0) ✓
    Token 1: (0, 4) -> (0, 4) ✓
    Token 2: (0, 8) -> (0, 8) ✓
    Maximum original line reference: 0 (source has 1 lines)
    Maximum generated line reference: 0 (output has 1 lines)
    Sourcemap serialized to 107 byte JSON

============================================================
Total tests: 21
Passed: 20 ✓
Failed: 1 ✗
Success rate: 95.2%

==========================STATISTICS=========================
Total tokens generated: 2144
Total parse time: 3.37ms
Total codegen time: 1.48ms
Total JSON size: 27987 bytes
Average tokens per test: 107.2
Average parse time: 0.17ms
Average codegen time: 0.07ms
```

### Validation

The program validates:

1. **Sourcemap Generation**: Ensures sourcemaps are generated for valid input
2. **Token Accuracy**: Validates that each token maps to correct line/column positions
3. **Bounds Checking**: Ensures all positions are within valid ranges
4. **JSON Serialization**: Verifies sourcemaps can be serialized to valid JSON
5. **Performance**: Measures and reports timing for parsing and code generation

This testing program serves as both a validation tool and a demonstration of the sourcemap generator's capabilities and performance characteristics.