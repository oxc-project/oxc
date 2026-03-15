# Task: Fix JSDoc formatting differences found in real-world repo retest

Detailed analysis at `tasks/prettier_conformance/jsdoc/scripts/fix-plan.md`.
Diff files at `tasks/prettier_conformance/jsdoc/diffs/`.

## Stages

### Stage 1: Fix tag routing bugs (@remarks, @example)

- **Goal**: Fix `@remarks` being formatted as code (Fix 1) and `@example` mangling TypeScript generics (Fix 2)
- **Depends on**: none
- **Parallel**: yes
- **Success criteria**:
  - `@remarks Remarks` → no semicolons, correct indent (not code indent)
  - `@example` with `getItem<number>(...)` preserved correctly
  - `cargo test -p oxc_formatter` passes
  - Two commits: one per fix, each with fixture test
- **Status**: Not Started

### Stage 2: Fix markdown parsing false positives

- **Goal**: Fix legacy list marker false positives (Fix 3), wrapped-line list markers (Fix 4), pipe in prose (Fix 10)
- **Depends on**: none
- **Parallel**: yes
- **Success criteria**:
  - `64-bit` and `1--->a` preserved (not treated as numbered lists)
  - Em-dash + `` `await a + b` `` wraps without corruption
  - `|splineCurve|` not treated as table
  - `cargo test -p oxc_formatter` passes
  - Three commits: one per fix, each with fixture test
- **Status**: Not Started

### Stage 3: Fix tag formatting (blank lines, description placement, capitalization)

- **Goal**: Fix `@typedef` blank lines (Fix 7), block tag description placement (Fix 6), capitalization inside types (Fix 8)
- **Depends on**: Stage 1 (Fix 6 depends on @remarks routing being correct)
- **Parallel**: no
- **Success criteria**:
  - Consecutive `@typedef` tags have blank line separators
  - `@privateRemarks` description on next line
  - `@fires {CustomEvent<{ id: string }>}` preserves lowercase `id`
  - `cargo test -p oxc_formatter` passes
  - Three commits: one per fix, each with fixture test
- **Status**: Not Started

### Stage 4: Fix code block, default values, and links

- **Goal**: Fix code block destruction (Fix 5), default value duplication (Fix 9), unmatched quote (Fix 11), link simplification (Fix 12)
- **Depends on**: none
- **Parallel**: yes
- **Success criteria**:
  - 4-space indented code block in JSDoc preserved
  - `@template [T=X] Desc. Default is \`X\`` → no duplication
  - `@default 'circle;` preserved without quote conversion
  - `[url](url)` preserved as markdown link
  - `cargo test -p oxc_formatter` passes
  - Four commits: one per fix, each with fixture test
- **Status**: Not Started

### Stage 5: Fix `{@link}` wrapping tolerance (cosmetic)

- **Goal**: Adjust `{@link}` width calculation to match upstream (Fix 13)
- **Depends on**: Stages 1-4
- **Parallel**: no
- **Success criteria**:
  - `{@link Foo}` at end of line with slight overflow stays on same line
  - `cargo test -p oxc_formatter` passes
  - One commit with fixture test
- **Status**: Not Started

### Stage 6: End-to-end verification

- **Goal**: Full retest against 5 real-world repos + Prettier conformance
- **Depends on**: Stages 1-5
- **Parallel**: no
- **Success criteria**:
  - `cargo run -p oxc_prettier_conformance` — no regression
  - 5-repo retest shows reduced diffs
  - Updated diff files and summary table
- **Status**: Not Started
