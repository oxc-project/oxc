# Binary Size Analysis Tool

This tool provides comprehensive analysis of binary size for oxc tools, helping identify what causes large binary sizes and providing actionable recommendations for size reduction.

## Features

- **Basic Size Analysis**: File size, stripped size, text section size
- **Crate-level Analysis**: Identifies which crates contribute most to binary size  
- **Function-level Analysis**: Shows largest individual functions
- **Generic Bloat Detection**: Uses LLVM lines analysis to find over-instantiated generics
- **Build Configuration**: Shows optimization settings affecting size
- **Baseline Comparison**: Track size changes over time
- **Actionable Recommendations**: Specific suggestions for reducing size

## Usage

```bash
# Basic analysis
cargo run -p oxc_binary_size

# Detailed analysis with all information  
cargo run -p oxc_binary_size -- --detailed

# Analyze specific target with features
cargo run -p oxc_binary_size -- --target oxlint --features allocator

# Save current results as baseline
cargo run -p oxc_binary_size -- --save-baseline

# Compare with previous baseline
cargo run -p oxc_binary_size -- --compare

# JSON output for CI/automation
cargo run -p oxc_binary_size -- --json
```

## Options

- `-t, --target <NAME>`: Target binary to analyze (default: oxlint)
- `-f, --features <FEATURES>`: Comma-separated list of features to enable
- `-d, --detailed`: Show detailed function-level analysis
- `-c, --compare`: Compare with saved baseline
- `-s, --save-baseline`: Save current results as baseline
- `--json`: Output results in JSON format

## Prerequisites

The tool requires these external programs for full functionality:

```bash
# Install cargo analysis tools
cargo install cargo-bloat cargo-llvm-lines

# System tools (usually pre-installed)
strip objdump
```

## Example Output

```
=== Binary Size Analysis Report ===

📊 Size Overview
  Binary: target/release/oxlint
  File size: 8.4 MB
  Stripped size: 6.2 MB  
  Text section: 6.0 MB
  Debug overhead: 2.2 MB (26.2%)

🔧 Build Configuration
  Target: x86_64
  Optimization: Level 3
  LTO: fat
  Features: allocator

📦 Largest Crates
   1. oxc_linter (36.3%) - 2.2 MB
   2. std (16.2%) - 990 KB
   3. regex_automata (5.5%) - 338 KB

💡 Size Reduction Recommendations
  🎯 Strip debug symbols: Could save 2.2 MB (26.2%)
  🎯 Consider regex optimization: 338 KB from regex crates (5.5%)
  🎯 Generic function bloat detected: 1,867,069 LLVM lines generated
```