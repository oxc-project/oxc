// Conformance suite for raw transfer back (ESTree -> arena).
//
// Mirror of `parse-raw.test.ts` (the raw transfer suite): same fixture sets, same worker
// architecture, same known-failures filtering. The oracle differs: each case round-trips
// `parse -> ESTree -> encode -> &Program` and asserts `ContentEq` + codegen equality against
// an independent direct parse (see `raw-transfer-back-worker.ts`).
//
// While `RAW_TRANSFER_BACK_SUPPORTED` is `false` in `raw-transfer-back-api.ts`, the whole
// suite is skipped. Set env var `RUN_RAW_TRANSFER_BACK_TESTS=true` to force-run it.
//
// Known failures are tracked in `snapshots/raw-transfer-back.snap` (same `Mismatch:` line
// grammar as `tasks/coverage` snapshots). Regenerate after implementation progress with:
// `UPDATE_RAW_TRANSFER_BACK_SNAPSHOT=true pnpm vitest run --dir ./test raw-transfer-back`

import { mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import Tinypool from "tinypool";
import { describe, expect, it } from "vitest";

import {
  ACORN_TEST262_DIR_PATH,
  JSX_DIR_PATH,
  ROOT_DIR_PATH,
  TARGET_DIR_PATH,
  TS_ESTREE_DIR_PATH,
} from "./parse-raw-common.ts";
import {
  FAILS_SNAPSHOT_PATH,
  JSX_MISMATCH_PREFIX,
  TEST262_MISMATCH_PREFIX,
  TEST_TYPE_FIXTURE,
  TEST_TYPE_INLINE_FIXTURE,
  TEST_TYPE_JSX,
  TEST_TYPE_NO_SOURCE,
  TEST_TYPE_PRESERVE_PARENS,
  TEST_TYPE_PRETTY,
  TEST_TYPE_TEST262,
  TEST_TYPE_TS,
  TS_MISMATCH_PREFIX,
} from "./raw-transfer-back-common.ts";
import { RAW_TRANSFER_BACK_SUPPORTED } from "./raw-transfer-back-api.ts";

// Define `describe` and `it` variants which run/skip tests based on environment variables
const { env } = process;
const isEnabled = (envValue: string | undefined) => envValue === "true" || envValue === "1";

const isUpdateMode = isEnabled(env.UPDATE_RAW_TRANSFER_BACK_SNAPSHOT);
const isSuiteEnabled =
  RAW_TRANSFER_BACK_SUPPORTED || isEnabled(env.RUN_RAW_TRANSFER_BACK_TESTS) || isUpdateMode;

const noop = Object.assign(() => {}, { concurrent() {}, each: () => () => {} });
const [describeBack, itBack] = isSuiteEnabled ? [describe, it] : [noop as any, noop as any];

// Vitest errors on a file which registers no tests. While the suite is pending
// implementation, register a single visible skipped test.
if (!isSuiteEnabled) {
  it.skip("raw_transfer_back conformance (pending implementation — see raw-transfer-back-api.ts)", () => {});
}

// Extra dimensions (heavy: full fixture sets re-run), opt-in via env vars,
// same pattern as `RUN_RAW_RANGE_TESTS` in the raw transfer suite.
const [describeNoSource] =
  isSuiteEnabled && isEnabled(env.RUN_RAW_TRANSFER_BACK_NO_SOURCE_TESTS)
    ? [describe]
    : [noop as any];
const [describePreserveParens] =
  isSuiteEnabled && isEnabled(env.RUN_RAW_TRANSFER_BACK_PRESERVE_PARENS_TESTS)
    ? [describe]
    : [noop as any];

// Worker pool for running test cases.
// Vitest provides parallelism across test files, but not across cases within a single test file.
// So we run each case in a worker to achieve parallelism.
const pool = new Tinypool({
  filename: new URL("./raw-transfer-back-worker.ts", import.meta.url).href,
});

type RunCase = (typeof import("./raw-transfer-back-worker.ts"))["runCase"];
type TestCaseData = Parameters<RunCase>[0];

let runCase: RunCase;

// Run test case in a worker
async function runCaseInWorker(type: TestCaseData["type"], props: TestCaseData["props"]) {
  const success = await pool.run({ type, props });

  // If test failed in worker, run it again in main thread with Vitest's `expect`,
  // to get a nice diff and stack trace
  if (!success) {
    if (!runCase) ({ runCase } = await import("./raw-transfer-back-worker.ts"));

    type |= TEST_TYPE_PRETTY;
    await runCase({ type, props }, expect);
    throw new Error("Failed on worker but unexpectedly passed on main thread");
  }
}

// Download fixtures (same set as raw transfer suite; cached in `target`, shared with benchmarks).
const benchFixtureUrls = [
  // TypeScript syntax (2.81MB)
  "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts",
  // Real world app tsx (415KB) — excalidraw App.tsx (master @ f6d85bc8)
  "https://cdn.jsdelivr.net/gh/excalidraw/excalidraw@f6d85bc80fe328e8f472636eb0d541f7bb891aa0/packages/excalidraw/components/App.tsx",
  // Real world content-heavy app jsx (3K)
  "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx",
  // Heavy with classes (554K)
  "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs",
  // ES5 (3.9M)
  "https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js",
];

let benchFixturePaths: string[] = [];
if (isSuiteEnabled) {
  await mkdir(TARGET_DIR_PATH, { recursive: true });
  benchFixturePaths = await Promise.all(
    benchFixtureUrls.map(async (url) => {
      const filename = url.split("/").at(-1),
        path = pathJoin(TARGET_DIR_PATH, filename);
      try {
        await stat(path);
      } catch {
        const res = await fetch(url);
        const sourceText = await res.text();
        await writeFile(path, sourceText);
      }
      return path.slice(ROOT_DIR_PATH.length + 1);
    }),
  );
}

// Enumerate fixtures.
//
// Same fixture sets as the raw transfer suite. Unlike that suite, the ESTree-shape fail lists
// (`estree_*.snap`) are NOT applied - a shape mismatch vs Acorn does not prevent a round trip.
// Only this suite's own known-failures list filters cases (and the worker skips sources which
// fail to parse, since our parser is not recoverable).
let test262FixturePaths: string[] = [];
let jsxFixturePaths: string[] = [];
let tsFixturePaths: string[] = [];

if (isSuiteEnabled) {
  const failPaths = isUpdateMode ? new Set<string>() : await getTestFailurePaths();

  for (let path of await readdir(ACORN_TEST262_DIR_PATH, { recursive: true })) {
    if (!path.endsWith(".json")) continue;
    path = path.slice(0, -2); // `.json` -> `.js`
    if (!failPaths.has(`${TEST262_MISMATCH_PREFIX}/${path}`)) test262FixturePaths.push(path);
  }

  jsxFixturePaths = (await readdir(JSX_DIR_PATH, { recursive: true })).filter(
    (path) => path.endsWith(".jsx") && !failPaths.has(`${JSX_MISMATCH_PREFIX}/${path}`),
  );

  tsFixturePaths = (await readdir(TS_ESTREE_DIR_PATH, { recursive: true })).filter(
    (path) => path.endsWith(".md") && !failPaths.has(`${TS_MISMATCH_PREFIX}/${path}`),
  );
}

describeBack.concurrent("test262", () => {
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(test262FixturePaths)("%s", (path: string) =>
    runCaseInWorker(TEST_TYPE_TEST262, path),
  );
});

describeBack.concurrent("JSX", () => {
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(jsxFixturePaths)("%s", (filename: string) =>
    runCaseInWorker(TEST_TYPE_JSX, filename),
  );
});

describeBack.concurrent("TypeScript", () => {
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(tsFixturePaths)("%s", (path: string) => runCaseInWorker(TEST_TYPE_TS, path));
});

describeNoSource.concurrent("no-source test262", () => {
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(test262FixturePaths)("%s", (path: string) =>
    runCaseInWorker(TEST_TYPE_TEST262 | TEST_TYPE_NO_SOURCE, path),
  );
});

describePreserveParens.concurrent("preserveParens test262", () => {
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(test262FixturePaths)("%s", (path: string) =>
    runCaseInWorker(TEST_TYPE_TEST262 | TEST_TYPE_PRESERVE_PARENS, path),
  );
});

// Edge cases not covered by Test262. Superset of the raw transfer suite's list -
// these exercise exactly the hard paths of the encoder (lone surrogates, lossy replacement
// characters, hashbangs, import phases). All option variants run (the set is small).
const edgeCases = [
  // ECMA stage 3
  'import defer * as ns from "x";',
  'import source src from "x";',
  'import.defer("x");',
  'import.source("x");',
  // `StringLiteral`s containing lone surrogates and/or lossy replacement characters
  ';"\\uD800\\uDBFF";',
  ';"�\\u{FFFD}";',
  ';"�\\u{FFFD}\\uD800\\uDBFF�\\u{FFFD}";',
  // `TemplateLiteral`s containing lone surrogates and/or lossy replacement characters
  "`\\uD800\\uDBFF${x}\\uD800\\uDBFF`;",
  "`�\\u{FFFD}${x}�\\u{FFFD}`;",
  "`�\\u{FFFD}\\uD800${x}\\uDBFF�\\u{FFFD}`;",
  // Hashbangs
  "#!/usr/bin/env node\nlet x;",
  "#!/usr/bin/env node\nlet x;\n// foo",
  // Directives vs body split
  '"use strict";\n"not a directive after this";\nlet x = "use strict";',
  // Parenthesized expressions
  "let x = (1 + 2);",
  // Numeric edge cases (f64 round-trip, radix raws, infinity)
  "x = [0, -0, 1e400, 0x1F, 0o17, 0b11, 1_000_000, .5, 5., NaN];",
  // BigInt
  "x = [0n, 123n, 0xFFn, 0o77n, 0b11n];",
  // RegExp
  "x = /ab+c/gu; y = /[\\u{1F600}]/v;",
  // Assignment targets (ESTree patterns re-interpreted by position)
  "[a, [b], ...c] = arr; ({ d, e: { f }, ...g } = obj);",
  "[a = 1, { b } = {}] = arr;",
  "for ([x, ...y] of z) {} for ({ p: q } in r) {}",
  // Holes in array literals and patterns
  "x = [, , 1, , ]; [, a, , ...b] = y;",
];

describeBack.concurrent("edge cases", () => {
  describeBack.each(edgeCases)("%s", (sourceText: string) => {
    for (const [label, extraType] of [
      ["", 0],
      ["no-source", TEST_TYPE_NO_SOURCE],
      ["preserveParens", TEST_TYPE_PRESERVE_PARENS],
    ] as const) {
      // oxlint-disable-next-line jest/expect-expect
      itBack(`JS ${label}`, () =>
        runCaseInWorker(TEST_TYPE_INLINE_FIXTURE | extraType, {
          filename: "dummy.js",
          sourceText,
        }),
      );
      // oxlint-disable-next-line jest/expect-expect
      itBack(`TS ${label}`, () =>
        runCaseInWorker(TEST_TYPE_INLINE_FIXTURE | extraType, {
          filename: "dummy.ts",
          sourceText,
        }),
      );
    }
  });
});

// Large real-world files, all option variants.
describeBack.concurrent("fixtures", () => {
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(benchFixturePaths)("%s", (path: string) => runCaseInWorker(TEST_TYPE_FIXTURE, path));
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(benchFixturePaths)("%s no-source", (path: string) =>
    runCaseInWorker(TEST_TYPE_FIXTURE | TEST_TYPE_NO_SOURCE, path),
  );
  // oxlint-disable-next-line jest/expect-expect
  itBack.each(benchFixturePaths)("%s preserveParens", (path: string) =>
    runCaseInWorker(TEST_TYPE_FIXTURE | TEST_TYPE_PRESERVE_PARENS, path),
  );
});

// Get `Set` of known-failing test paths (prefixed, e.g. `test262/<path>`) from snapshot file
async function getTestFailurePaths(): Promise<Set<string>> {
  const mismatchPrefix = "Mismatch: ",
    mismatchPrefixLen = mismatchPrefix.length;

  let snapshot: string;
  try {
    snapshot = await readFile(FAILS_SNAPSHOT_PATH, "utf8");
  } catch {
    return new Set();
  }
  return new Set(
    snapshot
      .split("\n")
      .filter((line) => line.startsWith(mismatchPrefix))
      .map((line) => line.slice(mismatchPrefixLen)),
  );
}

// Snapshot update mode: run every fixture in every suite, collect failures,
// and rewrite `snapshots/raw-transfer-back.snap` (`tasks/coverage` snapshot style:
// `Passed: n/m` header per suite + sorted `Mismatch:` lines).
if (isUpdateMode) {
  describe("update snapshot", () => {
    it("regenerates raw-transfer-back.snap", { timeout: 60 * 60 * 1000 }, async () => {
      const suites: [string, number, string[]][] = [
        [TEST262_MISMATCH_PREFIX, TEST_TYPE_TEST262, test262FixturePaths],
        [JSX_MISMATCH_PREFIX, TEST_TYPE_JSX, jsxFixturePaths],
        [TS_MISMATCH_PREFIX, TEST_TYPE_TS, tsFixturePaths],
      ];

      let out =
        "raw_transfer_back conformance\n" +
        "Round trip: parse -> ESTree -> encode -> arena -> ContentEq + codegen equality.\n" +
        "Regenerate: UPDATE_RAW_TRANSFER_BACK_SNAPSHOT=true pnpm vitest run --dir ./test raw-transfer-back\n";

      for (const [prefix, type, paths] of suites) {
        const failures: string[] = [];
        const results: Promise<void>[] = [];
        for (const path of paths) {
          results.push(
            pool.run({ type, props: path }).then((success: boolean) => {
              if (!success) failures.push(`${prefix}/${path}`);
            }),
          );
        }
        await Promise.all(results);
        failures.sort();

        out += `\n${prefix} Summary:\nPassed: ${paths.length - failures.length}/${paths.length}\n`;
        for (const failure of failures) out += `Mismatch: ${failure}\n`;
      }

      await writeFile(FAILS_SNAPSHOT_PATH, out);
      expect(true).toBe(true);
    });
  });
}
