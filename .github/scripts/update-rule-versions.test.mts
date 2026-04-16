import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";

import { analyzeRuleFile } from "./update-rule-versions.mts";

const REPO_ROOT = "/repo";
const RULES_DIR = "crates/oxc_linter/src/rules/eslint";

function analyze(source: string, fileName = "no_debugger.rs") {
  const filePath = path.join(REPO_ROOT, RULES_DIR, fileName);
  return analyzeRuleFile(source.trimStart(), filePath, "1.61.0", REPO_ROOT);
}

function registerTest(name: string, fn: () => void): void {
  void test(name, fn);
}

registerTest("rewrites stable rule versions from next to the release version", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
      from: "next",
      to: "1.61.0",
    },
  ]);
  assert.match(result.updatedSource, /version = "1\.61\.0"/);
});

registerTest("rewrites spacing variants of version next", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version= "next",
);
`);

  assert.deepEqual(result.updatedRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
      from: "next",
      to: "1.61.0",
    },
  ]);
  assert.match(result.updatedSource, /version= "1\.61\.0"/);
});

registerTest("keeps nursery rules on next", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    nursery,
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});

registerTest("accepts inline comments on category lines", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    nursery, // move after more bake time
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});

registerTest("ignores standalone comment lines inside declare_oxc_lint blocks", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    // comment about incubation status
    nursery,
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});

registerTest("accepts block comments on category lines", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, /* move after more bake time */
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});

registerTest("accepts block comments with urls on category lines", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, /* see https://example.com/details */
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});

registerTest("accepts multi-line block comments on category lines", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, /* move after
      more bake time */
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});

registerTest("ignores doc examples containing version next inside declare_oxc_lint blocks", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// Example config: version = "next"
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
      from: "next",
      to: "1.61.0",
    },
  ]);
  assert.match(result.updatedSource, /version = "1\.61\.0"/);
});

registerTest("accepts commented declare_oxc_lint terminators", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version = "next",
); // keep note
`);

  assert.equal(result.updatedRules.length, 1);
  assert.match(result.updatedSource, /version = "1\.61\.0"/);
});

registerTest("accepts multi-line block comments on declare_oxc_lint terminators", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version = "next",
); /* keep
   note */
`);

  assert.equal(result.updatedRules.length, 1);
  assert.match(result.updatedSource, /version = "1\.61\.0"/);
});

registerTest("ignores inline comments in the stray next-version scan", () => {
  const result = analyze(`
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, // keep version = "next"
    version = "next",
);
`);

  assert.deepEqual(result.updatedRules, []);
  assert.deepEqual(result.skippedNurseryRules, [
    {
      file: `${RULES_DIR}/no_debugger.rs`,
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(result.updatedSource, /version = "next"/);
});
