# ESLint-Style Bulk Suppressions

This document describes the ESLint-style bulk suppressions feature implemented in oxlint, which allows developers to suppress multiple linting violations using a count-based approach similar to ESLint's bulk suppression format.

## Overview

The bulk suppressions feature allows you to suppress linting errors without adding inline comments to your source code. This is particularly useful for:

- Large codebases with many existing violations
- Gradual migration to stricter linting rules
- Temporary suppression of violations during development
- Automated suppression generation and management

## File Format

ESLint-style bulk suppressions use a JSON file with the following structure:

```json
{
  "src/App.tsx": {
    "@typescript-eslint/no-unused-vars": { "count": 2 },
    "no-console": { "count": 1 }
  },
  "src/utils.js": {
    "prefer-const": { "count": 3 }
  }
}
```

### Format Details

- **File paths**: Keys in the root object represent file paths (can be relative or absolute)
- **Rule names**: Keys in file objects represent rule names (can include plugin prefixes)
- **Count objects**: Each rule has a `count` property specifying how many violations to suppress

### Rule Name Formats Supported

The system supports multiple rule name formats:
- Bare rule names: `"no-unused-vars"`
- Plugin-prefixed names: `"@typescript-eslint/no-unused-vars"`
- Alternative prefixes: `"typescript/no-unused-vars"`

### File Path Matching

File paths are matched flexibly:
- Exact matches: `"src/App.tsx"` matches exactly `src/App.tsx`
- Relative path matching: `"App.tsx"` matches `src/App.tsx`, `components/App.tsx`, etc.
- Full path matching: Handles absolute paths appropriately

## CLI Usage

### Generating Suppressions

Generate a suppressions file for all current violations:

```bash
oxlint --suppress-all src/
```

Generate suppressions only for specific files:

```bash
oxlint --suppress-all src/App.tsx src/utils.js
```

### Using Existing Suppressions

Run linting with an existing suppressions file:

```bash
oxlint --suppressions-location oxlint-suppressions.json src/
```

The default suppressions file is `oxlint-suppressions.json` in the current directory.

### Updating Suppressions

Add new suppressions to an existing file (append mode):

```bash
oxlint --suppress-all --suppressions-location my-suppressions.json src/
```

Replace all suppressions with new ones:

```bash
rm my-suppressions.json
oxlint --suppress-all --suppressions-location my-suppressions.json src/
```

### Pruning Unused Suppressions

Remove suppressions that are no longer needed:

```bash
oxlint --prune-suppressions --suppressions-location my-suppressions.json src/
```

Check for unused suppressions without removing them:

```bash
oxlint --pass-on-unpruned-suppressions --suppressions-location my-suppressions.json src/
```

## Implementation Details

### Core Components

#### 1. Data Structures (`crates/oxc_linter/src/bulk_suppressions.rs`)

```rust
// ESLint-style bulk suppressions data structure
pub type ESLintBulkSuppressionsData = HashMap<String, HashMap<String, ESLintRuleSuppression>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESLintRuleSuppression {
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESLintBulkSuppressionsFile {
    pub suppressions: ESLintBulkSuppressionsData,
}
```

#### 2. Matching Logic

The `ESLintBulkSuppressions` struct provides:
- **Count-based matching**: Tracks usage and prevents over-suppression
- **File path resolution**: Handles various file path formats
- **Rule name resolution**: Supports multiple rule naming conventions
- **Usage tracking**: Identifies unused suppressions for cleanup

#### 3. Integration Points

##### Linter Integration (`crates/oxc_linter/src/context/mod.rs`)
- Suppressions are checked in `add_diagnostic()` method during rule execution
- Each diagnostic is tested against active suppressions before being reported

##### CLI Integration (`apps/oxlint/src/lint.rs`)
- Suppression generation during linting process
- Loading and saving of suppression files
- Command-line flag handling

### Violation Capture Process

1. **File Processing**: Each source file is parsed and analyzed
2. **Rule Execution**: Linting rules run and generate diagnostics
3. **Suppression Check**: Each diagnostic is checked against loaded suppressions
4. **Usage Tracking**: Successfully suppressed diagnostics increment usage counters
5. **Violation Collection**: Unsuppressed diagnostics are collected for suppression file generation

### Rule Name Extraction

The system extracts rule names from various diagnostic sources:
- **OxcDiagnostic codes**: Direct rule name extraction
- **Error message parsing**: Pattern matching for rule names in error text
- **Semantic error mapping**: Maps semantic analyzer errors to equivalent linting rules

Example mapping:
```rust
// Semantic analyzer error -> linting rule
"Delete of an unqualified identifier in strict mode" => "no-delete-var"
"'var' is not defined" => "no-undef"
```

## Testing

The implementation includes comprehensive tests covering:

### Unit Tests (`crates/oxc_linter/src/bulk_suppressions.rs`)
- Suppression data structure functionality
- File and rule matching logic
- Count-based usage tracking
- Serialization/deserialization
- Unused suppression detection

### Integration Tests (`apps/oxlint/src/command/test_suppressions.rs`)
- CLI flag handling
- File loading and saving
- ESLint format compatibility
- End-to-end suppression workflows

### Running Tests

```bash
# Run bulk suppression tests
cargo test -p oxc_linter bulk_suppressions

# Run CLI integration tests
cargo test -p oxlint test_suppressions

# Run all tests
cargo test
```

## Compatibility

### ESLint Compatibility
- File format matches ESLint's bulk suppression format
- Rule naming conventions are compatible
- Count-based suppression behavior is identical

### Migration from Individual Suppressions
- Existing individual suppression entries are still supported
- Both formats can be used simultaneously
- Gradual migration path available

## Performance Considerations

### Optimizations
- **HashMap-based lookups**: O(1) average case for file and rule matching
- **Arc-wrapped data**: Efficient sharing between threads
- **Lazy file matching**: Only computes expensive path operations when needed
- **Minimal allocations**: Reuses data structures where possible

### Memory Usage
- Suppressions are loaded once and shared across all linting contexts
- Usage tracking uses minimal memory (HashMaps with integer counters)
- No per-file suppression duplication

## Error Handling

### File Loading Errors
- Missing files result in empty suppression sets (no errors)
- Invalid JSON files produce clear error messages
- Malformed suppression entries are skipped with warnings

### Runtime Errors
- Mutex lock failures are handled gracefully (suppressions continue to work)
- Invalid file paths are silently ignored
- Unknown rule names are processed normally (no false matches)

## Future Enhancements

### Planned Features
1. **Glob pattern support** for file paths
2. **Line/column-based matching** for more precise suppression
3. **Suppression expiration** based on timestamps
4. **Integration with IDE extensions** for real-time suppression management

### Extension Points
- Rule name extraction can be extended for new diagnostic types
- File matching can be enhanced with more sophisticated patterns
- Usage tracking can be extended with additional metadata

## Troubleshooting

### Common Issues

#### Suppressions Not Working
1. Verify file path matches between suppression file and actual file location
2. Check rule name format (try both bare name and plugin-prefixed versions)
3. Ensure suppression count hasn't been exceeded
4. Verify suppressions file is being loaded (check path and permissions)

#### Unused Suppressions
1. Use `--prune-suppressions` to remove unused entries
2. Check if files have been moved or deleted
3. Verify rule names haven't changed

#### Performance Issues
1. Large suppression files are handled efficiently, but consider splitting if needed
2. Use `--pass-on-unpruned-suppressions` to identify cleanup opportunities

### Debug Information

Enable debug logging to see suppression matching in action:
```bash
RUST_LOG=debug oxlint --suppressions-location my-suppressions.json src/
```

## Contributing

When extending the bulk suppressions feature:

1. **Add tests** for new functionality in both unit and integration test suites
2. **Update documentation** to reflect new capabilities
3. **Maintain compatibility** with existing suppression files
4. **Consider performance** impact of changes
5. **Follow error handling** patterns established in the codebase

The bulk suppressions system is designed to be extensible and maintainable, with clear separation of concerns and comprehensive test coverage.