/*
 * Oxlint JS plugins conformance tester.
 *
 * This script runs rule tests using Oxlint's RuleTester to verify API compatibility.
 *
 * It runs tests for:
 * - All ESLint's built-in rules.
 * - Other external plugins e.g. `eslint-plugin-react-hooks`.
 *
 * It works by:
 * 1. Patching NodeJS's CommonJS loader to substitute ESLint's `RuleTester` with Oxlint's.
 * 2. Hooking `describe` and `it` to capture test results.
 * 3. Loading each ESLint rule test file.
 * 4. Recording success/failure of each test.
 * 5. Outputting results to a markdown file.
 *
 * To add a new repo to be tested:
 * - Add a file to `groups` directory.
 * - Add setup for cloning the repo to `init.sh`.
 *
 * If you want to run only a subset of tests, alter the constants in `filter.ts`.
 */

// oxlint-disable no-console

import Module from "node:module";
import fs from "node:fs";
import { join as pathJoin, sep as pathSep } from "node:path";
import { fileURLToPath } from "node:url";
import { TEST_GROUPS } from "./groups/index.ts";
import { setCurrentGroup, setCurrentRule, resetCurrentRule } from "./capture.ts";
import { SHOULD_SKIP_GROUP, SHOULD_SKIP_RULE } from "./filter.ts";
import { generateReport } from "./report.ts";
import { RuleTester, parseForESLintFns, parserPaths } from "./rule_tester.ts";

import type { RuleResult } from "./capture.ts";
import type { Language, TestCase } from "./rule_tester.ts";

/**
 * Definition of a test group.
 */
export interface TestGroup {
  /**
   * Name of the test group.
   */
  name: string;

  /**
   * Name of the submodule for this group.
   * i.e. name of the directory in `submodules` directory.
   */
  submoduleName: string;

  /**
   * Path to the directory containing test files for this group, relative to the submodule directory.
   */
  testFilesDirPath: string;

  /**
   * Transform test file name to test name.
   *
   * e.g.:
   * ```ts
   * (path: string) => {
   *   if (!path.endsWith(".js")) return null;
   *   return path.slice(0, -3);
   * }
   * ```
   *
   * @param filename - Filename of test file
   * @returns Name of test file if it should be run, or `null` if it should be skipped.
   */
  transformTestFilename: (filename: string) => string | null;

  /**
   * Function to run before loading any test files.
   * @param require - Require function, which requires modules relative to the test files directory
   * @param mock - Mock function, which mocks modules relative to the test files directory
   */
  prepare?: (require: NodeJS.Require, mock: MockFn) => void;

  /**
   * Function that will be called for every failing test case.
   * If it returns `true`, the test case will be skipped.
   *
   * @param ruleName - Rule name
   * @param test - Test case
   * @param code - Code for test case
   * @param err - Error test case failed with
   * @returns `true` if test should be skipped
   */
  shouldSkipTest?: (ruleName: string, test: TestCase, code: string, err: Error) => boolean;

  /**
   * `RuleTester` instances to replace with the Oxc conformance `RuleTester`.
   *
   * - `specifier` is a module specifier which is resolved relative to the tests directory, using `require.resolve`.
   * - `propName` is name of the property to set on the module to the `RuleTester` class.
   *   If `null`, the module is set as `module.exports`.
   *
   * e.g.:
   * ```js
   * [
   *   { specifier: "eslint", propName: "RuleTester" },
   *   { specifier: "../../lib/rule-tester.js", propName: null },
   * ]
   * ```
   */
  ruleTesters: { specifier: string; propName: string | null }[];

  /**
   * Known parsers to accept.
   *
   * If one of these parsers is passed as `languageOptions.parser` in test case config, Oxc parser will be used instead.
   * Otherwise, custom parsers are not accepted, and the test case will throw an error.
   *
   * `specifier` is a module specifier which is resolved relative to the tests directory, using `require.resolve`.
   * `lang` is the language to parse the test case code with when this parser is used.
   * `propName` (optional) is the name of the property of the module which is the parser.
   *
   * e.g. `{ specifier: "@typescript-eslint/parser", lang: "ts" }`
   * e.g. `{ specifier: "typescript-eslint", propName: "parser", lang: "ts" }`
   */
  parsers: ParserDetails[];
}

/**
 * Mock function.
 *
 * Takes specifier of module to mock, and value to mock it with.
 * Specifier is resolved relative to the test files directory.
 *
 * If need to mock a module which is imported by another package, pass the specifiers of
 * "breadcrumb" packages on way to the module as `via`.
 *
 * e.g. If test file imports `foo`, `foo` imports `bar`, and `bar` imports `qux`, and `qux` is the module to mock:
 * `mock("qux", value, ["foo", "bar"])`
 */
export type MockFn = (specifier: string, value: unknown, via?: string[]) => void;

/**
 * Custom parser details.
 */
export interface ParserDetails {
  specifier: string;
  propName?: string;
  lang: Language;
}

/**
 * Test file.
 */
interface TestFile {
  name: string;
  path: string;
}

/**
 * Mocked modules.
 * Mapping from absolute path to `module.exports` of the module.
 */
type Mocks = Map<string, unknown>;

// Check that `NODE_DISABLE_COLORS` is set.
// If it isn't, then errors produced by `assert` will be coloured, which is unsuitable for logging to file.
if (process.env.NODE_DISABLE_COLORS !== "1") {
  throw new Error("`NODE_DISABLE_COLORS` must be set to 1");
}

// Paths
const CONFORMANCE_DIR_PATH = pathJoin(fileURLToPath(import.meta.url), "../..");
const SUBMODULES_DIR_PATH = pathJoin(CONFORMANCE_DIR_PATH, "submodules");
const SNAPSHOTS_DIR_PATH = pathJoin(CONFORMANCE_DIR_PATH, "snapshots");

const { createRequire } = Module;
const require = createRequire(import.meta.url);

const normalizePath =
  pathSep === "\\" ? (path: string) => path.replaceAll("\\", "/") : (path: string) => path;

// Run
const mocks = initMocks();
runGroups(TEST_GROUPS, mocks);

/**
 * Patch NodeJS CJS loader to allow mocking modules.
 *
 * Returns a `Map`. Adding entries to the map will mock the module.
 * `mocks.set("/path/to/module.js", { whatever: true })`
 *
 * @returns Mocks `Map`
 */
function initMocks(): Mocks {
  const mocks = new Map();

  const extensions = (
    Module as unknown as {
      _extensions: Record<string, (module: Module, path: string, ...args: any[]) => any>;
    }
  )._extensions;

  for (const [ext, loader] of Object.entries(extensions)) {
    extensions[ext] = function (module: Module, path: string, ...args: any[]) {
      if (!mocks.has(path)) return loader.call(this, module, path, ...args);
      module.exports = mocks.get(path);
    };
  }

  return mocks;
}

/**
 * Run all test groups.
 * @param groups - Test groups
 */
function runGroups(groups: TestGroup[], mocks: Mocks) {
  for (const group of groups) {
    if (SHOULD_SKIP_GROUP(group.name)) continue;
    runGroup(group, mocks);
  }
}

/**
 * Run all tests in a test group.
 * @param group - Test group
 */
function runGroup(group: TestGroup, mocks: Mocks) {
  setCurrentGroup(group);

  const groupName = group.name;

  // Get absolute path to test files directory
  const testFilesDirPath = pathJoin(
    SUBMODULES_DIR_PATH,
    group.submoduleName,
    group.testFilesDirPath,
  );
  group.testFilesDirPath = testFilesDirPath;

  // Mock `RuleTester` instances.
  // When these rule tester files are `require`-ed, Oxlint's conformance `RuleTester` will be substituted.
  console.log(`Mocking rule testers for ${groupName}...`);

  mocks.clear();

  const requireFromTestsDir = createRequire(pathJoin(testFilesDirPath, "dummy.js"));
  const resolveFromTestsDir = requireFromTestsDir.resolve.bind(requireFromTestsDir);

  for (const tester of group.ruleTesters) {
    const { specifier, propName } = tester;
    if (propName === null) {
      mocks.set(resolveFromTestsDir(specifier), RuleTester);
    } else {
      const mod = requireFromTestsDir(specifier);
      // Use `Object.defineProperty` to handle if `mod[propName]` is a getter (transpiled ESM module)
      Object.defineProperty(mod, propName, {
        value: RuleTester,
        writable: true,
        enumerable: true,
        configurable: true,
      });
    }
  }

  // Run `prepare` function
  const { prepare } = group;
  if (prepare) {
    console.log(`Running prepare hook for ${groupName}...`);

    const mock = createMockFn(testFilesDirPath, mocks);
    prepare(requireFromTestsDir, mock);
  }

  // Get custom parsers
  console.log(`Loading custom parsers for ${groupName}...`);

  parseForESLintFns.clear();
  parserPaths.clear();

  for (const parserDetails of group.parsers) {
    const path = resolveFromTestsDir(parserDetails.specifier);
    let parser = require(path);
    if (parserDetails.propName != null) {
      parser = parser[parserDetails.propName];
    } else if (parser && parser.default === undefined) {
      // Set `default` export on parser module to work around apparent bug in `tsx`
      parser.default = parser;
    }

    if (typeof parser.parseForESLint === "function") {
      parseForESLintFns.set(parser.parseForESLint, parserDetails);
    }
    parserPaths.set(path, parserDetails);
  }

  // Find test files and run tests
  console.log(`Finding rule test files for ${groupName}...`);
  const files = findTestFiles(group);
  console.log(`Found ${files.length} test files\n`);

  console.log(`Running tests for ${groupName}...`);
  const results = runAllTests(files);

  // Write results to markdown file
  const snapshotPath = pathJoin(SNAPSHOTS_DIR_PATH, `${groupName}.md`);

  const report = generateReport(group.name, results);
  fs.writeFileSync(snapshotPath, report);
  console.log(`\nResults written to: ${snapshotPath}`);

  // Print summary
  const totalRuleCount = results.length;
  let loadErrorCount = 0,
    fullyPassingCount = 0;

  for (const rule of results) {
    if (rule.isLoadError) {
      loadErrorCount++;
    } else {
      const { tests } = rule;
      if (tests.length > 0 && tests.every((test) => test.isPassed || test.isSkipped)) {
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
}

/**
 * Create a mock function which mocks a module relative to the test files directory.
 * @param testFilesDirPath - Path to the test files directory
 * @param mocks - Mocks
 * @returns Mock function
 */
function createMockFn(testFilesDirPath: string, mocks: Mocks): MockFn {
  const startPath = pathJoin(testFilesDirPath, "dummy.js");

  return (specifier: string, value: unknown, via?: string[]) => {
    let fromPath = startPath;
    if (via != null) {
      for (const specifier of via) {
        fromPath = createRequire(fromPath).resolve(specifier);
      }
    }

    mocks.set(createRequire(fromPath).resolve(specifier), value);
  };
}

/**
 * Find all test files for a test group.
 * @param group - Test group
 * @returns Test file details
 */
function findTestFiles(group: TestGroup): TestFile[] {
  const { testFilesDirPath } = group;
  const fileObjs = fs.readdirSync(testFilesDirPath, { withFileTypes: true, recursive: true });

  const files: TestFile[] = [];
  for (const fileObj of fileObjs) {
    if (!fileObj.isFile()) continue;

    let filename = fileObj.name;
    if (fileObj.parentPath !== testFilesDirPath) {
      filename = `${normalizePath(fileObj.parentPath.slice(testFilesDirPath.length + 1))}/${filename}`;
    }

    const name = group.transformTestFilename(filename);
    if (name === null) continue;
    if (SHOULD_SKIP_RULE(name)) continue;

    const path = pathJoin(group.testFilesDirPath, filename);
    files.push({ name, path });
  }
  return files;
}

/**
 * Run all test files for a group.
 * @param testFiles - Test files
 * @returns Results of running tests
 */
function runAllTests(testFiles: TestFile[]): RuleResult[] {
  const results = [];

  for (let i = 0; i < testFiles.length; i++) {
    const testFile = testFiles[i];
    process.stdout.write(`[${i + 1}/${testFiles.length}] Testing ${testFile.name}...`);

    const result = runRuleTests(testFile);
    results.push(result);

    if (result.isLoadError) {
      console.log(" LOAD ERROR");
    } else {
      const { tests } = result,
        totalCount = tests.length;

      let passedCount = 0,
        skippedCount = 0;
      for (const test of tests) {
        if (test.isPassed) passedCount++;
        if (test.isSkipped) skippedCount++;
      }

      const status = passedCount + skippedCount === totalCount ? "PASS" : "FAIL";
      let message = ` ${status} (${passedCount}/${totalCount})`;
      if (skippedCount > 0) message += ` (${skippedCount} skipped)`;
      console.log(message);
    }
  }

  return results;
}

/**
 * Run tests for a single rule file.
 * @param testFile - Test file details
 * @returns Results of running tests for rule
 */
function runRuleTests(testFile: TestFile): RuleResult {
  const result: RuleResult = {
    ruleName: testFile.name,
    isLoadError: false,
    loadError: null,
    tests: [],
  };

  setCurrentRule(result);

  // Load the test file - this will execute the tests
  try {
    require(testFile.path);
  } catch (err) {
    result.isLoadError = true;
    result.loadError = err as Error;
  }

  resetCurrentRule();

  return result;
}
