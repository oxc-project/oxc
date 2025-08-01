# Feature Impact Analysis

This document analyzes how different feature flags affect binary size in oxc tools.

## Analysis Results

Run `cargo run -p oxc_binary_size_features` to generate an updated analysis.

## Methodology

The feature analysis tool:

1. **Baseline Build**: Builds oxlint with minimal features
2. **Individual Feature Testing**: Tests each feature flag individually 
3. **Feature Combination Testing**: Tests common feature combinations
4. **Size Impact Calculation**: Measures the size impact of each feature
5. **Recommendation Generation**: Suggests optimizations based on feature usage

## Common Features Analyzed

- `allocator`: Custom memory allocator (mimalloc)
- `oxlint2`: Experimental oxlint2 features 
- `disable_oxlint2`: Disable oxlint2 features
- `force_test_reporter`: Force test reporter output

## Expected Findings

Features that typically contribute to binary size:
- Debug/diagnostic features
- Additional allocators
- Experimental features
- Testing/development features

## Recommendations

Based on feature analysis:
1. Use minimal feature sets for production builds
2. Consider feature gates for large optional components
3. Profile-guided optimization for feature-heavy builds
4. Conditional compilation for development-only features