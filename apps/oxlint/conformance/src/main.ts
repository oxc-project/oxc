/*
 * Main logic.
 */

// oxlint-disable no-console

import Module from "node:module";
import { join as pathJoin } from "node:path";
import fs from "node:fs";
import { runAllTests, CONFORMANCE_DIR_PATH, ESLINT_ROOT_DIR_PATH } from "./run.ts";
import { generateReport } from "./report.ts";
import { RuleTester } from "./rule_tester.ts";

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
const OUTPUT_FILE_PATH = pathJoin(CONFORMANCE_DIR_PATH, "snapshot.md");

const report = generateReport(results);
fs.writeFileSync(OUTPUT_FILE_PATH, report);
console.log(`\nResults written to: ${OUTPUT_FILE_PATH}`);

// Print summary
const totalRuleCount = results.length;
let loadErrorCount = 0,
  fullyPassingCount = 0;

for (const rule of results) {
  if (rule.isLoadError) {
    loadErrorCount++;
  } else {
    const { tests } = rule;
    if (tests.length > 0 && tests.every((test) => test.isPassed)) {
      fullyPassingCount++;
    }
  }
}

console.log("\n=====================================");
console.log("Summary:");
console.log(`  Total rules: ${totalRuleCount}`);
console.log(`  Fully passing: ${fullyPassingCount}`);
console.log(`  Load errors: ${loadErrorCount}`);
console.log(`  With failures: ${totalRuleCount - fullyPassingCount - loadErrorCount}`);
