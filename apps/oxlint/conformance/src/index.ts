/*
 * ESLint Rule Tester Conformance Script
 *
 * This script runs ESLint's rule tests using Oxlint's RuleTester to verify API compatibility.
 *
 * It works by:
 * 1. Patching NodeJS's CommonJS loader to substitute ESLint's `RuleTester` with Oxlint's.
 * 2. Hooking `describe` and `it` to capture test results.
 * 3. Loading each ESLint rule test file.
 * 4. Recording success/failure of each test.
 * 5. Outputting results to a markdown file.
 */

import Module from "node:module";
import { join as pathJoin } from "node:path";
import fs from "node:fs";
import { runAllTests, CONFORMANCE_DIR_PATH, ESLINT_ROOT_DIR_PATH } from "./run.ts";
import { generateReport } from "./report.ts";
import { RuleTester } from "./rule_tester.ts";

// NodeJS's `assert` seems to produce garbled error messages when colors are enabled in some cases.
// Stop it using control characters.
process.env.FORCE_COLOR = "0";

// Patch NodeJS's CommonJS loader to substitute ESLint's `RuleTester` with our own
const RULE_TESTER_PATH = pathJoin(ESLINT_ROOT_DIR_PATH, "lib/rule-tester/rule-tester.js");

const jsLoaderOriginal = (Module as any)._extensions[".js"];
(Module as any)._extensions[".js"] = function (module: Module, path: string, ...args: any[]) {
  if (path === RULE_TESTER_PATH) {
    module.exports = RuleTester;
  } else {
    return jsLoaderOriginal.call(this, module, path, ...args);
  }
};

// Run tests
const results = runAllTests();

// Write results to markdown file
// oxlint-disable no-console
const OUTPUT_FILE_PATH = pathJoin(CONFORMANCE_DIR_PATH, "snapshot.md");

const report = generateReport(results);
fs.writeFileSync(OUTPUT_FILE_PATH, report);
console.log(`\nResults written to: ${OUTPUT_FILE_PATH}`);

// Print summary
const totalRuleCount = results.length;
const fullyPassingCount = results.filter(
  (r) => !r.isLoadError && r.tests.length > 0 && r.tests.every((t) => t.isPassed),
).length;
const loadErrorCount = results.filter((r) => r.isLoadError).length;

console.log("\n=====================================");
console.log("Summary:");
console.log(`  Total rules: ${totalRuleCount}`);
console.log(`  Fully passing: ${fullyPassingCount}`);
console.log(`  Load errors: ${loadErrorCount}`);
console.log(`  With failures: ${totalRuleCount - fullyPassingCount - loadErrorCount}`);
