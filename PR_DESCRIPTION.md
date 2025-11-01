## Summary

This PR implements ESLint-style bulk suppressions for oxlint, allowing developers to suppress linting violations using a count-based JSON format identical to ESLint's bulk suppression system.

### Key Features

- **ESLint-compatible format**: Uses the exact same JSON structure as ESLint bulk suppressions
- **Count-based suppression**: Each rule can suppress a specific number of violations
- **Flexible file matching**: Supports relative, absolute, and partial file paths
- **Multiple rule formats**: Handles bare rule names and plugin-prefixed names
- **Usage tracking**: Identifies unused suppressions for cleanup
- **CLI integration**: Complete command-line interface with generation and management commands

### File Format

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

### CLI Usage

```bash
# Generate suppressions for all current violations
oxlint --suppress-all src/

# Use existing suppressions file
oxlint --suppressions-location oxlint-suppressions.json src/

# Clean up unused suppressions
oxlint --prune-suppressions src/
```

## Changes

### Core Implementation
- **`crates/oxc_linter/src/bulk_suppressions.rs`** - New module with ESLint format support
- **`crates/oxc_linter/src/context/mod.rs`** - Integration with diagnostic suppression
- **`crates/oxc_linter/src/lib.rs`** - Public API exports

### CLI Integration
- **`apps/oxlint/src/lint.rs`** - Complete violation capture and suppression generation
- **`apps/oxlint/src/command/suppressions.rs`** - Suppression management utilities
- **`apps/oxlint/src/command/lint.rs`** - New CLI flags and options

### Testing
- **`crates/oxc_linter/src/bulk_suppressions.rs`** - 9 comprehensive unit tests
- **`apps/oxlint/src/command/test_suppressions.rs`** - 19 integration tests covering CLI workflows

### Documentation
- **`BULK_SUPPRESSIONS.md`** - Complete user and developer documentation

## Technical Details

### Core Architecture
- **Count-based matching**: Tracks usage to prevent over-suppression
- **HashMap-based lookups**: O(1) average case performance for file and rule matching
- **Thread-safe design**: Uses Arc/Mutex for safe concurrent access
- **Memory efficient**: Minimal allocations with data structure reuse

### Rule Name Resolution
- Supports multiple formats: `"no-console"`, `"@typescript-eslint/no-unused-vars"`, `"typescript/no-unused-vars"`
- Handles semantic analyzer errors by mapping to equivalent linting rules
- Extracts rule names from diagnostic codes and error messages

### File Path Matching
- Exact path matching for direct file references
- Relative path matching for flexible project structures
- Handles both Unix and Windows path separators

## Test Coverage

### Unit Tests (9 tests)
- ✅ Basic suppression functionality and count tracking
- ✅ File path matching (exact, relative, absolute paths)
- ✅ Rule name format compatibility (bare names, plugin prefixes)
- ✅ Usage tracking and unused suppression detection
- ✅ JSON serialization/deserialization
- ✅ Error handling for missing files and invalid data

### Integration Tests (19 tests)
- ✅ ESLint format file loading and parsing
- ✅ CLI flag handling and command validation
- ✅ End-to-end suppression workflows
- ✅ File roundtrip operations (write/read cycles)
- ✅ Bulk suppression generation and application

**All tests pass with 100% success rate**

## Compatibility

- **ESLint compatibility**: Identical file format and behavior
- **Backward compatibility**: Existing individual suppressions continue to work
- **Migration path**: Both formats can be used simultaneously

## Performance

- **Efficient lookups**: HashMap-based O(1) average case performance
- **Memory optimized**: Arc-wrapped data sharing, minimal per-file duplication
- **Scalable**: Handles large suppression files efficiently

## Breaking Changes

None. This is a pure addition that maintains full backward compatibility.

## Related Issues

Fixes issues with:
- Large-scale suppression management in existing codebases
- ESLint migration compatibility
- Automated suppression workflows
- Bulk violation handling during linting rule adoption

---

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>