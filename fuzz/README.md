# AST Fuzzing for Oxc

This directory contains sophisticated fuzz targets for testing various components of the Oxc JavaScript/TypeScript toolchain, inspired by the [Shape Security fuzzer approach](https://github.com/shapesecurity/shift-fuzzer-js).

## Features

- **Comprehensive AST Generation**: Generates diverse JavaScript constructs including expressions, statements, declarations, and complex nested structures
- **Smart Test Case Shrinking**: Automatically reduces failing test cases to minimal reproducible examples  
- **Shape Security Inspired**: Follows patterns from shift-fuzzer-js and shift-shrink-js for systematic AST testing
- **Round-trip Validation**: Ensures consistency between AST → Code → AST transformations

## Setup

1. Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

2. Switch to nightly Rust (required for fuzzing):
```bash
rustup install nightly
rustup override set nightly
```

## Available Fuzz Targets

### ast_roundtrip (Enhanced)
Advanced AST round-trip fuzzer that systematically generates diverse JavaScript constructs:

**Generates:**
- Complex expressions (binary, unary, member access, function calls)
- Multiple statement types (variable declarations, if/for loops, blocks)
- Realistic JavaScript programs with proper nesting and complexity control
- Edge cases like template literals, array expressions with holes

**Features:**
- Depth-controlled recursion to prevent infinite generation
- Probability-based node selection for diverse coverage
- Structured approach similar to Shape Security's shift-fuzzer-js

Usage:
```bash
cargo fuzz run ast_roundtrip
```

### ast_shrink
Test case shrinking utility inspired by shift-shrink-js. Automatically reduces complex failing test cases to minimal reproducible examples:

**Capabilities:**
- Reduces complex expressions to simpler forms
- Removes unnecessary statements and declarations
- Converts complex control flow to basic expressions
- Maintains parsing validity while minimizing complexity

Usage:
```bash
cargo fuzz run ast_shrink
```

### parser_fuzz
Tests the parser with arbitrary UTF-8 input to ensure it handles any input gracefully without crashing or panicking.

Usage:
```bash
cargo fuzz run parser_fuzz
```

## Running the Fuzzers

Basic usage:
```bash
# Run with default settings
cargo fuzz run <target_name>

# Run for a specific time limit (recommended for CI)
cargo fuzz run <target_name> -- -max_total_time=60

# Run with specific input length limits
cargo fuzz run <target_name> -- -max_len=1024

# Run with multiple workers for better performance
cargo fuzz run <target_name> -- -workers=4

# Run with detailed output
cargo fuzz run <target_name> -- -verbosity=2
```

## Advanced Features

### Dictionary-based Fuzzing
The fuzzers support dictionary files for better coverage of JavaScript keywords and patterns:

```bash
# Create a dictionary file with JavaScript keywords
echo -e '"function"\n"const"\n"let"\n"var"\n"if"\n"for"' > js.dict
cargo fuzz run ast_roundtrip -- -dict=js.dict
```

### Corpus Management
```bash
# List current corpus
cargo fuzz list ast_roundtrip

# Add interesting test cases to corpus
echo "specific_test_case" > corpus/ast_roundtrip/custom_case
```

### Integration with Shape Security Patterns

This implementation follows key patterns from the Shape Security ecosystem:

1. **Systematic Generation** (like shift-fuzzer-js):
   - Structured AST node creation with proper type safety
   - Probability distributions for realistic JavaScript patterns
   - Recursive complexity control to prevent runaway generation

2. **Smart Shrinking** (like shift-shrink-js):
   - Automatic test case reduction when bugs are found
   - Maintains semantic validity while minimizing complexity  
   - Helps developers understand root causes of issues

3. **Comprehensive Coverage**:
   - All major JavaScript constructs (expressions, statements, declarations)
   - Edge cases like template literals, spread syntax, optional chaining
   - TypeScript-specific constructs when applicable

## Adding New Fuzz Targets

1. Create a new `.rs` file in `fuzz_targets/`
2. Add a new `[[bin]]` section to `Cargo.toml`
3. Follow the libfuzzer-sys pattern with `fuzz_target!` macro
4. Consider adding shrinking capabilities for complex generators

Example skeleton:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing logic here
});
```

## Tips for Effective Fuzzing

- **Start Small**: Begin with short time limits to validate setup
- **Use Structured Input**: The enhanced fuzzers work better than pure random bytes
- **Monitor Memory**: Watch memory usage during long fuzzing sessions
- **Save Interesting Cases**: Preserve inputs that find bugs for regression testing
- **Combine Approaches**: Use both generation-based and mutation-based fuzzing
- **Regular Corpus Review**: Periodically review and curate the corpus for better coverage

## Debugging Failures

When a fuzzer finds an issue:

1. **Minimize**: The ast_shrink target can help reduce complex cases
2. **Reproduce**: Save the failing input and create a unit test  
3. **Analyze**: Use the generated code to understand what construct caused the issue
4. **Fix**: Address the root cause in the parser, AST, or codegen
5. **Regress**: Add the test case to prevent future regressions