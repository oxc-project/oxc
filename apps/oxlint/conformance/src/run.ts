/*
 * Function to run all ESLint rule tests.
 */

import fs from "node:fs";
import { join as pathJoin } from "node:path";
import { fileURLToPath } from "node:url";
import Module from "node:module";
import { setCurrentRule, resetCurrentRule } from "./capture.ts";
import { FILTER_ONLY_RULE, FILTER_EXCLUDE_RULE } from "./filter.ts";

import type { RuleResult } from "./capture.ts";

const { isArray } = Array;

// Paths
export const CONFORMANCE_DIR_PATH = pathJoin(fileURLToPath(import.meta.url), "../..");
export const ESLINT_ROOT_DIR_PATH = pathJoin(CONFORMANCE_DIR_PATH, "submodules/eslint");
export const ESLINT_RULES_TESTS_DIR_PATH = pathJoin(ESLINT_ROOT_DIR_PATH, "tests/lib/rules");

// Create require function for loading CommonJS modules
const require = Module.createRequire(import.meta.url);

/**
 * Run all ESLint rule tests.
 * @returns Results of running tests
 */
// oxlint-disable no-console
export function runAllTests(): RuleResult[] {
  console.log("Finding ESLint rule test files...");

  const ruleNames = findTestFiles();

  console.log(`Found ${ruleNames.length} test files\n`);

  const results = [];
  for (let i = 0; i < ruleNames.length; i++) {
    const ruleName = ruleNames[i];
    process.stdout.write(`[${i + 1}/${ruleNames.length}] Testing ${ruleName}...`);

    const result = runRuleTests(ruleName);
    results.push(result);

    if (result.isLoadError) {
      console.log(" LOAD ERROR");
    } else {
      const { tests } = result,
        totalCount = tests.length,
        passedCount = tests.reduce((total, test) => total + (test.isPassed ? 1 : 0), 0),
        status = passedCount === totalCount ? "PASS" : "FAIL";
      console.log(` ${status} (${passedCount}/${totalCount})`);
    }
  }

  return results;
}
// oxlint-enable no-console

/**
 * Find all ESLint rule test files.
 * @returns Names of rule test files (without `.js` extension)
 */
function findTestFiles(): string[] {
  const filenames = fs.readdirSync(ESLINT_RULES_TESTS_DIR_PATH);

  let ruleNameMatchesFilter = null;
  if (FILTER_ONLY_RULE !== null) {
    ruleNameMatchesFilter = isArray(FILTER_ONLY_RULE)
      ? (ruleName: string) => FILTER_ONLY_RULE!.includes(ruleName)
      : (ruleName: string) => ruleName === FILTER_ONLY_RULE;
  } else if (FILTER_EXCLUDE_RULE !== null) {
    ruleNameMatchesFilter = isArray(FILTER_EXCLUDE_RULE)
      ? (ruleName: string) => !FILTER_EXCLUDE_RULE!.includes(ruleName)
      : (ruleName: string) => ruleName !== FILTER_EXCLUDE_RULE;
  }

  const ruleNames = [];
  for (const filename of filenames) {
    if (!filename.endsWith(".js")) continue;
    const ruleName = filename.slice(0, -3);
    if (ruleNameMatchesFilter !== null && !ruleNameMatchesFilter(ruleName)) continue;
    ruleNames.push(ruleName);
  }
  return ruleNames;
}

/**
 * Run tests for a single rule file.
 * @param ruleName - Rule name
 * @returns Results of running tests for rule
 */
function runRuleTests(ruleName: string): RuleResult {
  const result: RuleResult = {
    ruleName,
    isLoadError: false,
    loadError: null,
    tests: [],
  };

  setCurrentRule(result);

  // Load the test file - this will execute the tests
  const path = pathJoin(ESLINT_RULES_TESTS_DIR_PATH, `${ruleName}.js`);

  try {
    require(path);
  } catch (err) {
    result.isLoadError = true;
    result.loadError = err as Error;
  }

  resetCurrentRule();

  return result;
}
