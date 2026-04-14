const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const { rewriteNextRuleVersions } = require("./update-rule-versions.js");

function createTempRepo() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "oxc-update-rule-versions-"));
}

function rulesDir(root) {
  return path.join(root, "crates/oxc_linter/src/rules/eslint");
}

function writeRule(root, fileName, source) {
  const dir = rulesDir(root);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, fileName), source.trimStart());
}

test("rewrites stable rule versions from next to the release version", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
`,
  );

  const report = rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" });
  const updatedSource = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.deepEqual(report.updatedRules, [
    {
      file: "crates/oxc_linter/src/rules/eslint/no_debugger.rs",
      ruleName: "NoDebugger",
      from: "next",
      to: "1.61.0",
    },
  ]);
  assert.match(updatedSource, /version = "1\.61\.0"/);
});

test("keeps nursery rules on next", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    nursery,
    version = "next",
);
`,
  );

  const report = rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" });
  const updatedSource = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.deepEqual(report.updatedRules, []);
  assert.deepEqual(report.skippedNurseryRules, [
    {
      file: "crates/oxc_linter/src/rules/eslint/no_debugger.rs",
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(updatedSource, /version = "next"/);
});

test("accepts inline comments on category lines", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    nursery, // move after more bake time
    version = "next",
);
`,
  );

  const report = rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" });
  const updatedSource = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.deepEqual(report.updatedRules, []);
  assert.deepEqual(report.skippedNurseryRules, [
    {
      file: "crates/oxc_linter/src/rules/eslint/no_debugger.rs",
      ruleName: "NoDebugger",
    },
  ]);
  assert.match(updatedSource, /version = "next"/);
});

test("supports dry-run without modifying files", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
`,
  );

  const report = rewriteNextRuleVersions({ root, releaseVersion: "1.61.0", dryRun: true });
  const sourceAfterDryRun = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.equal(report.updatedRules.length, 1);
  assert.match(sourceAfterDryRun, /version = "next"/);
});

test("fails if version next is outside a declare_oxc_lint block", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "broken.rs",
    `
// version = "next"
`,
  );

  assert.throws(
    () => rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" }),
    /outside a declare_oxc_lint! block/,
  );
});

test("supports dry-run through the CLI", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
`,
  );

  const output = execFileSync(
    process.execPath,
    [
      path.join(__dirname, "update-rule-versions.js"),
      "--root",
      root,
      "--release-version",
      "1.61.0",
      "--dry-run",
    ],
    { encoding: "utf8" },
  );
  const sourceAfterDryRun = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.match(output, /Would update 1 rule version\(s\):/);
  assert.match(sourceAfterDryRun, /version = "next"/);
});
