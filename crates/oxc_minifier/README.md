# Oxc Minifier

Next-generation JavaScript/TypeScript minifier achieving best-in-class compression.

## Context Overview

**Important**: The source of truth of this file on disk is `README.md`. Both `AGENTS.md` and `CLAUDE.md` are symlinks to this file. When asked to update any of these, always update `README.md` (the actual file).

- Refer to the [Design Doc](./docs/design/001-design.md) for the high-level vision and architecture.
- Refer to the [Architecture](./docs/architecture.md) for the source layout and pipeline.
- Refer to the [Progress Doc](./docs/progress.md) for tracking progress.
- Refer to [Assumptions](./docs/assumptions.md) for code assumptions used in optimization.
- Refer to [Data Structures](./docs/data-structures.md) for core types and configuration.
- Refer to [Lessons](./docs/lessons.md) for patterns and lessons learned.

## Best Practices That Must Be Followed

1. General
   - Always read `meta/lessons.md` at the start of each session before taking any action.
   - When writing design docs or implementation plans to disk, place them in `docs/design/` and follow existing file name and frontmatter conventions.

2. Subagent Strategy
   - Use subagents liberally to keep the main context window clean.
   - Offload research, exploration, and parallel analysis to subagents.
   - For complex problems, allocate more compute via subagents.
   - Assign one task per subagent to ensure focused execution.

3. Self-Improvement Loop
   - After any correction from the user, update `meta/lessons.md` with the relevant pattern.
   - Create rules for yourself that prevent repeating the same mistake.
   - Iterate on these lessons rigorously until the mistake rate declines.

4. Principles
   - Simplicity First: Make every change as simple as possible. Minimize code impact.
   - No Laziness: Identify root causes. Avoid temporary fixes. Apply senior developer standards.
   - Minimal Impact: Touch only what is necessary. Avoid introducing new bugs.
   - For non-trivial changes, pause and ask whether there is a more elegant solution.
   - If a fix feels hacky, implement the solution you would choose knowing everything you now know.
   - Do not over-engineer simple or obvious fixes.
   - Critically evaluate your own work before presenting it.

5. Planning
   - Enter plan mode for any non-trivial task (three or more steps, or involving architectural decisions).
   - If something goes wrong, stop and re-plan immediately rather than continuing blindly.
   - Every implementation plan should include the following:
   - During plan generation: ask clarifying questions when necessary, especially around developer-experience defining design decisions.

## Current Performance

See [`tasks/minsize`](../../tasks/minsize) for compression benchmarks.

- Matching/beating esbuild on many libraries
- Full test262, Babel, TypeScript conformance

## Usage

```rust
use oxc_minifier::{Minifier, MinifierOptions};

let options = MinifierOptions::default();
let minifier = Minifier::new(options);
let result = minifier.minify(&mut program);
```

## Testing Infrastructure

- `just minsize` - Track compression benchmarks
- `cargo coverage` - Conformance tests (test262, Babel, TypeScript)
- `tasks/e2e` - Real-world E2E testing

## Development

- `just test` - Run all tests
- `cargo run -p oxc_minifier --example minifier` - Try the minifier
