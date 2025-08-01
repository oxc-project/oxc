# AST Fuzzing for Oxc

This directory contains fuzz targets for testing various components of the Oxc JavaScript/TypeScript toolchain.

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

### ast_roundtrip
Tests AST round-trip functionality: generates AST nodes programmatically, converts them to code via codegen, then parses the code back to AST. This helps find bugs in:
- AST generation
- Code generation 
- Parser consistency

Usage:
```bash
cargo fuzz run ast_roundtrip
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

# Run for a specific time limit
cargo fuzz run <target_name> -- -max_total_time=60

# Run with specific input length limits
cargo fuzz run <target_name> -- -max_len=256

# Run with multiple workers
cargo fuzz run <target_name> -- -workers=4
```

## Adding New Fuzz Targets

1. Create a new `.rs` file in `fuzz_targets/`
2. Add a new `[[bin]]` section to `Cargo.toml`
3. Follow the libfuzzer-sys pattern with `fuzz_target!` macro

## Tips

- Fuzzing can find edge cases and crashes quickly
- Use the corpus and dictionary features for better coverage
- Monitor memory usage during long fuzzing sessions
- Save interesting inputs that trigger issues for debugging