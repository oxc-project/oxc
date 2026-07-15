// Constants used in both main thread and worker in raw transfer back tests.
//
// The raw transfer back conformance suite mirrors the raw transfer suite (`parse-raw.test.ts`):
// same fixture sets, same worker architecture, same known-failures filtering mechanism.
//
// Difference in oracle: instead of comparing deserialized output against Acorn / TS-ESLint JSON,
// each case round-trips `ESTree -> encode -> arena -> &Program` and asserts, on the Rust side:
// 1. `ContentEq` between the round-tripped `Program` and a direct parse of the same source
//    (spans excluded by design - `Span` is `#[content_eq(skip)]`).
// 2. Byte-equal codegen output from both programs.

import { join as pathJoin } from "node:path";

export const TEST_TYPE_TEST262 = 0;
export const TEST_TYPE_JSX = 1;
export const TEST_TYPE_TS = 2;
export const TEST_TYPE_FIXTURE = 3;
export const TEST_TYPE_INLINE_FIXTURE = 4;

export const TEST_TYPE_MAIN_MASK = 7;
// Encode without providing source text (spans pass through raw, comments unavailable).
// Codegen comparison runs with comments disabled on both sides.
export const TEST_TYPE_NO_SOURCE = 8;
// Parse and encode with `preserveParens: true` (default in this suite is `false`).
export const TEST_TYPE_PRESERVE_PARENS = 16;
// Re-run on main thread with Vitest's `expect` for a pretty diff.
export const TEST_TYPE_PRETTY = 32;

// Path to known-failures snapshot for this suite.
// Same line grammar as `tasks/coverage` snapshots: `Mismatch: <path>` lines, parsed by
// `getTestFailurePaths()` in `raw-transfer-back.test.ts`.
// Regenerate with: `UPDATE_RAW_TRANSFER_BACK_SNAPSHOT=true pnpm vitest run --dir ./test raw-transfer-back`
export const FAILS_SNAPSHOT_PATH = pathJoin(
  import.meta.dirname,
  "snapshots/raw-transfer-back.snap",
);

// Prefixes used in the snapshot's `Mismatch:` lines, one per fixture set.
export const TEST262_MISMATCH_PREFIX = "test262";
export const JSX_MISMATCH_PREFIX = "acorn-jsx";
export const TS_MISMATCH_PREFIX = "typescript";
