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

function registerTest(...args) {
  void test(...args);
}

registerTest("rewrites stable rule versions from next to the release version", () => {
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

registerTest("rewrites spacing variants of version next", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version= "next",
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
  assert.match(updatedSource, /version= "1\.61\.0"/);
});

registerTest("keeps nursery rules on next", () => {
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

registerTest("accepts inline comments on category lines", () => {
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

registerTest("ignores standalone comment lines inside declare_oxc_lint blocks", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    // comment about incubation status
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

registerTest("accepts block comments on category lines", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, /* move after more bake time */
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

registerTest("accepts block comments with urls on category lines", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, /* see https://example.com/details */
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

registerTest("accepts multi-line block comments on category lines", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, /* move after
      more bake time */
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

registerTest("ignores doc examples containing version next inside declare_oxc_lint blocks", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// Example config: version = "next"
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

registerTest("accepts commented declare_oxc_lint terminators", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version = "next",
); // keep note
`,
  );

  const report = rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" });
  const updatedSource = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.equal(report.updatedRules.length, 1);
  assert.match(updatedSource, /version = "1\.61\.0"/);
});

registerTest("accepts multi-line block comments on declare_oxc_lint terminators", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version = "next",
); /* keep
   note */
`,
  );

  const report = rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" });
  const updatedSource = fs.readFileSync(path.join(rulesDir(root), "no_debugger.rs"), "utf8");

  assert.equal(report.updatedRules.length, 1);
  assert.match(updatedSource, /version = "1\.61\.0"/);
});

registerTest("supports dry-run without modifying files", () => {
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

registerTest("does not persist earlier rewrites if a later file fails validation", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "a_valid.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    RuleA,
    eslint,
    correctness,
    version = "next",
);
`,
  );
  writeRule(
    root,
    "z_invalid.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    RuleZ,
    eslint,
    made_up_category,
    version = "next",
);
`,
  );

  assert.throws(
    () => rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" }),
    /unknown rule category `made_up_category`/,
  );

  const sourceAfterFailure = fs.readFileSync(path.join(rulesDir(root), "a_valid.rs"), "utf8");
  assert.match(sourceAfterFailure, /version = "next"/);
});

registerTest("fails if version next is outside a declare_oxc_lint block", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "broken.rs",
    `
const BROKEN: &str = r#"version = "next""#;
`,
  );

  assert.throws(
    () => rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" }),
    /outside a declare_oxc_lint! block/,
  );
});

registerTest("ignores inline comments in the stray next-version scan", () => {
  const root = createTempRepo();
  writeRule(
    root,
    "no_debugger.rs",
    `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    nursery, // keep version = "next"
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

registerTest(
  "fails if the rules tree contains symlinked rule files",
  { skip: process.platform === "win32" },
  () => {
    const root = createTempRepo();
    const dir = rulesDir(root);
    fs.mkdirSync(dir, { recursive: true });

    fs.writeFileSync(
      path.join(root, "real_rule.rs"),
      `
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
`.trimStart(),
    );
    fs.symlinkSync(path.join(root, "real_rule.rs"), path.join(dir, "linked_rule.rs"));

    assert.throws(
      () => rewriteNextRuleVersions({ root, releaseVersion: "1.61.0" }),
      /symlinked rule paths are not supported/,
    );
  },
);

registerTest("supports dry-run through the CLI", () => {
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
