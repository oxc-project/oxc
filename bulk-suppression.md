# Implement ESLint-Style Bulk Suppressions for Oxlint

## Overview

Add bulk suppression support to oxlint matching ESLint's behavior: automatic generation of suppressions for existing violations, file-based suppression storage, and unused suppression detection with error reporting.

## Design Decisions

Based on user requirements and ESLint compatibility:

1. **CLI Interface**: ESLint-style flags (`--suppress-all`, `--suppress-rule`, `--prune-suppressions`, `--pass-on-unpruned-suppressions`)
2. **Generation**: Automatic - run linter, capture violations, write to JSON file
3. **Unused Handling**: Error by default, with `--pass-on-unpruned-suppressions` to downgrade to success
4. **Precedence**: Inline directives take precedence over bulk suppressions (ESLint behavior)
5. **File Location**: Default to `oxlint-suppressions.json` in project root, configurable via `--suppressions-location`

## Architecture Integration Points

### Key Files to Modify

1. **CLI Layer** (`apps/oxlint/src/`)

   - `command/lint.rs` - Add CLI flags for suppression operations
   - `command/suppressions.rs` - Already exists, extend with new options
   - `lint.rs` - Integrate suppression generation/loading/pruning logic

2. **Linter Core** (`crates/oxc_linter/src/`)

   - `config/suppressions.rs` - Already has data structures, add runtime logic
   - `context/mod.rs` - Extend `add_diagnostic` (line 231) to check bulk suppressions
   - `lib.rs` - Add suppression tracking to `Linter`
   - `lint_runner.rs` - Coordinate suppression generation and pruning

3. **Diagnostic Service** (`crates/oxc_diagnostics/src/`)

   - `service.rs` - Track which suppressions were used during linting

## Implementation Steps

### Phase 1: CLI Argument Parsing

**File**: `apps/oxlint/src/command/lint.rs`

Add new CLI flags to `LintCommand`:

```rust
/// Suppress all existing violations for rules configured as "error"
#[bpaf(switch, hide_usage)]
pub suppress_all: bool,

/// Suppress existing violations for specific rule(s)
#[bpaf(long, argument("RULE"), many, hide_usage)]
pub suppress_rule: Vec<String>,

/// Remove suppressions that no longer match any violations
#[bpaf(switch, hide_usage)]
pub prune_suppressions: bool,

/// Don't fail when unused suppressions are found
#[bpaf(switch, hide_usage)]
pub pass_on_unpruned_suppressions: bool,

/// Location of the suppressions file (default: oxlint-suppressions.json)
#[bpaf(long, argument("PATH"), hide_usage)]
pub suppressions_location: Option<PathBuf>,
```

### Phase 2: Suppression File Management

**File**: `apps/oxlint/src/command/suppressions.rs`

Extend existing `SuppressionOptions` and add helper functions:

- `load_suppressions_from_file()` - Read and parse suppressions JSON
- `write_suppressions_to_file()` - Serialize and write suppressions
- `merge_suppressions()` - Combine existing with newly generated
- `validate_suppression_file()` - Check file format and structure

### Phase 3: Suppression Generation

**File**: `apps/oxlint/src/lint.rs` (CliRunner)

Add suppression generation mode:

1. Run linter in "capture mode" to collect all violations
2. Filter violations by:

   - Only rules configured as "error" (not "warn")
   - Optionally filter by `--suppress-rule` list

3. Group violations by file and rule
4. Convert to `SuppressionConfig` format with:

   - File path (relative to project root)
   - Rule name (plugin/rule format)
   - Line and column numbers
   - Reason: "Existing violation at time of suppression generation"

5. Write to suppressions file (merge if exists, unless `--suppress-all` which overwrites)
6. Apply `--fix` first if specified, then suppress remaining violations

### Phase 4: Suppression Loading and Application

**File**: `crates/oxc_linter/src/config/suppressions.rs`

Add runtime suppression checking:

- `SuppressionMatcher` - Efficient lookup structure for checking violations
- `matches_diagnostic()` - Check if a diagnostic should be suppressed
- Track which suppressions are used during linting

**File**: `crates/oxc_linter/src/context/mod.rs`

Extend `add_diagnostic()` method (around line 231):

```rust
fn add_diagnostic(&self, mut message: Message) {
    // Check inline directives first (takes precedence)
    if self.parent.disable_directives().contains(self.current_rule_name, message.span) {
        return;
    }

    // Check bulk suppressions second
    if self.parent.bulk_suppressions().matches(
        self.current_plugin_name,
        self.current_plugin_prefix,
        self.current_rule_name,
        message.span,
        self.file_path(),
    ) {
        return;
    }

    // ... rest of existing code
}
```

### Phase 5: Unused Suppression Detection

**File**: `crates/oxc_linter/src/lint_runner.rs`

Add suppression usage tracking:

- During linting, mark suppressions as "used" when they match a violation
- After linting completes, identify unused suppressions
- Report unused suppressions as diagnostics with appropriate severity

**File**: `apps/oxlint/src/lint.rs`

Add post-linting check:

```rust
// After linting completes
let unused_suppressions = detect_unused_suppressions(&suppressions, &used_suppressions);

if !unused_suppressions.is_empty() {
    if pass_on_unpruned_suppressions {
        // Print message but don't fail
        println!("There are {} suppressions that do not occur anymore.", unused_suppressions.len());
        println!("Consider re-running with `--prune-suppressions`.");
    } else {
        // Error and fail
        return CliRunResult::UnusedSuppressionsFound;
    }
}
```

### Phase 6: Suppression Pruning

**File**: `apps/oxlint/src/lint.rs`

Add pruning logic:

1. Load existing suppressions file
2. Run linter with suppression tracking
3. Identify unused suppressions
4. Remove unused suppressions from file
5. Write updated suppressions file
6. Report number of suppressions removed

### Phase 7: Integration and Testing

**Files**: `apps/oxlint/fixtures/suppressions*/`

Add comprehensive test fixtures:

- Test suppression generation with `--suppress-all`
- Test selective suppression with `--suppress-rule`
- Test suppression loading and application
- Test unused suppression detection
- Test suppression pruning
- Test interaction with inline directives (inline takes precedence)
- Test `--fix` + `--suppress-all` workflow
- Test custom suppression file locations

## Data Flow

### Suppression Generation Flow

```
User runs: oxlint --fix --suppress-all

1. CliRunner parses arguments
2. Apply auto-fixes first
3. Run linter in capture mode
4. Collect violations (only "error" severity)
5. Generate SuppressionConfig entries
6. Write to oxlint-suppressions.json
7. Exit with success
```

### Suppression Application Flow

```
User runs: oxlint (with suppressions file present)

1. CliRunner detects suppressions file
2. Load and parse suppressions
3. Build SuppressionMatcher
4. For each file being linted:
   a. Check inline directives first
   b. Check bulk suppressions second
   c. Mark used suppressions
5. After linting, check for unused suppressions
6. Report or error based on --pass-on-unpruned-suppressions
```

### Suppression Pruning Flow

```
User runs: oxlint --prune-suppressions

1. Load existing suppressions
2. Run linter with tracking
3. Identify unused suppressions
4. Remove from suppressions file
5. Write updated file
6. Report number removed
```

## Key Implementation Details

### Suppression File Format

```json
{
  "suppressions": [
    {
      "files": ["src/utils/legacy.js"],
      "rules": ["no-console"],
      "line": 42,
      "column": 5,
      "endLine": 42,
      "endColumn": 20,
      "reason": "Existing violation at time of suppression generation"
    }
  ]
}
```

### Precedence Rules

1. **Inline directives** (highest priority)

   - `// eslint-disable-next-line`
   - `// eslint-disable-line`
   - `/* eslint-disable */`

2. **Bulk suppressions** (medium priority)

   - From `oxlint-suppressions.json`

3. **Rule configuration** (lowest priority)

   - From `.oxlintrc.json`

### Performance Considerations

- Use `rust-lapper` (already used in `disable_directives.rs`) for efficient interval matching
- Build suppression index once at startup, not per-file
- Cache file path matching results for glob patterns
- Minimize allocations during hot path (diagnostic checking)

## Exit Codes

Add new exit codes to `CliRunResult`:

- `UnusedSuppressionsFound` - When unused suppressions exist and `--pass-on-unpruned-suppressions` not set
- `SuppressionFileInvalid` - When suppressions file cannot be parsed
- `SuppressionGenerationFailed` - When generation fails

## Documentation Updates

Update `AGENTS.md` with:

- New CLI flags and their usage
- Bulk suppression workflow examples
- Integration with existing features (`--fix`, inline directives)
- Suppression file format reference
