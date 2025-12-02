/*
 * `RuleTester` class.
 *
 * Heavily based on ESLint's `RuleTester`, but without the complications of configs.
 * Has the same user-facing API as ESLint's version.
 * Code: https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/rule-tester/rule-tester.js
 * License (MIT): https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/LICENSE
 */

import { default as assert, AssertionError } from "node:assert";
import util from "node:util";
import stringify from "json-stable-stringify-without-jsonify";
import { registerPlugin, registeredRules } from "../plugins/load.ts";
import { lintFileImpl, resetFile } from "../plugins/lint.ts";
import { getLineColumnFromOffset, getNodeByRangeIndex } from "../plugins/location.ts";
import {
  allOptions,
  initAllOptions,
  mergeOptions,
  DEFAULT_OPTIONS_ID,
} from "../plugins/options.ts";
import { diagnostics, replacePlaceholders, PLACEHOLDER_REGEX } from "../plugins/report.ts";
import { parse } from "./parse.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { RequireAtLeastOne } from "type-fest";
import type { Plugin, Rule } from "../plugins/load.ts";
import type { Options } from "../plugins/options.ts";
import type { DiagnosticData, Suggestion } from "../plugins/report.ts";

const { hasOwn } = Object,
  { isArray } = Array;

// ------------------------------------------------------------------------------
// `describe` and `it` functions
// ------------------------------------------------------------------------------

export type DescribeFn = (text: string, fn: () => void) => void;
export type ItFn = ((text: string, fn: () => void) => void) & { only?: ItFn };

/**
 * Default `describe` function, if `describe` doesn't exist as a global.
 * @param text - Description of the test case
 * @param method - Test case logic
 * @returns Returned value of `method`
 */
function defaultDescribe<R>(text: string, method: () => R): R {
  return method.call(this);
}

const globalObj = globalThis as { describe?: DescribeFn; it?: ItFn };

// `describe` function. Can be overwritten via `RuleTester.describe` setter.
let describe: DescribeFn =
  typeof globalObj.describe === "function" ? globalObj.describe : defaultDescribe;

/**
 * Default `it` function, if `it` doesn't exist as a global.
 * @param text - Description of the test case
 * @param method - Test case logic
 * @throws {Error} Any error upon execution of `method`
 * @returns Returned value of `method`
 */
function defaultIt<R>(text: string, method: () => R): R {
  try {
    return method.call(this);
  } catch (err) {
    if (err instanceof AssertionError) {
      err.message += ` (${util.inspect(err.actual)} ${err.operator} ${util.inspect(err.expected)})`;
    }
    throw err;
  }
}

// `it` function. Can be overwritten via `RuleTester.it` setter.
let it: ItFn = typeof globalObj.it === "function" ? globalObj.it : defaultIt;

// `it.only` function. Can be overwritten via `RuleTester.it` or `RuleTester.itOnly` setters.
let itOnly: ItFn | null =
  it !== defaultIt && typeof it.only === "function" ? Function.bind.call(it.only, it) : null;

/**
 * Get `it` function.
 * @param only - `true` if `it.only` should be used
 * @throws {Error} If `it.only` is not available
 * @returns `it` or `it.only` function
 */
function getIt(only?: boolean): ItFn {
  return only ? getItOnly() : it;
}

/**
 * Get `it.only` function.
 * @throws {Error} If `it.only` is not available
 * @returns `it.only` function
 */
function getItOnly(): ItFn {
  if (itOnly === null) {
    throw new Error(
      "To use `only`, use `RuleTester` with a test framework that provides `it.only()` like Mocha, " +
        "or provide a custom `it.only` function by assigning it to `RuleTester.itOnly`",
    );
  }
  return itOnly;
}

// ------------------------------------------------------------------------------
// Config
// ------------------------------------------------------------------------------

/**
 * Configuration for `RuleTester`.
 */
export type Config = Record<string, unknown>;

// Default shared config
const DEFAULT_SHARED_CONFIG: Config = {};

// `RuleTester` uses this config as its default. Can be overwritten via `RuleTester.setDefaultConfig()`.
let sharedConfig: Config = DEFAULT_SHARED_CONFIG;

// ------------------------------------------------------------------------------
// Test cases
// ------------------------------------------------------------------------------

/**
 * Test case.
 */
interface TestCase {
  code: string;
  name?: string;
  only?: boolean;
  filename?: string;
  options?: Options;
  before?: (this: this) => void;
  after?: (this: this) => void;
}

/**
 * Test case for valid code.
 */
export interface ValidTestCase extends TestCase {}

/**
 * Test case for invalid code.
 */
export interface InvalidTestCase extends TestCase {
  output?: string | null;
  errors: number | ErrorEntry[];
}

type ErrorEntry = Error | string | RegExp;

/**
 * Expected error.
 */
export type Error = RequireAtLeastOne<ErrorBase, "message" | "messageId">;

interface ErrorBase {
  message?: string | RegExp;
  messageId?: string;
  data?: DiagnosticData;
  line?: number;
  column?: number;
  endLine?: number;
  endColumn?: number;
}

/**
 * Test cases for a rule.
 */
export interface TestCases {
  valid: (ValidTestCase | string)[];
  invalid: InvalidTestCase[];
}

/**
 * Diagnostic included in assertion errors.
 *
 * This matches what ESLint's includes in errors it emits.
 * `severity` field is pointless, as it's always `1`, but ESLint includes it, so we do too.
 */
interface Diagnostic {
  ruleId: string;
  message: string;
  messageId: string | null;
  severity: 1;
  nodeType: string | null;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  suggestions: Suggestion[] | null;
}

// Default path for test cases if not provided
const DEFAULT_PATH = "file.js";

// ------------------------------------------------------------------------------
// `RuleTester` class
// ------------------------------------------------------------------------------

/**
 * Utility class for testing rules.
 */
export class RuleTester {
  #config: Config | null;

  /**
   * Creates a new instance of RuleTester.
   * @param config? - Extra configuration for the tester (optional)
   */
  constructor(config?: Config) {
    this.#config = config === undefined ? null : config;
  }

  /**
   * Set the configuration to use for all future tests.
   * @param config - The configuration to use
   * @throws {TypeError} If `config` is not an object
   */
  static setDefaultConfig(config: Config): void {
    if (typeof config !== "object" || config === null) {
      throw new TypeError("`config` must be an object");
    }
    sharedConfig = config;
  }

  /**
   * Get the current configuration used for all tests.
   * @returns The current configuration
   */
  static getDefaultConfig(): Config {
    return sharedConfig;
  }

  /**
   * Reset the configuration to the initial configuration of the tester removing
   * any changes made until now.
   * @returns {void}
   */
  static resetDefaultConfig() {
    sharedConfig = DEFAULT_SHARED_CONFIG;
  }

  // Getters/setters for `describe` and `it` functions

  static get describe(): DescribeFn {
    return describe;
  }

  static set describe(value: DescribeFn) {
    describe = value;
  }

  static get it(): ItFn {
    return it;
  }

  static set it(value: ItFn) {
    it = value;
    if (typeof it.only === "function") {
      itOnly = Function.bind.call(it.only, it);
    } else {
      itOnly = null;
    }
  }

  static get itOnly(): ItFn {
    return getItOnly();
  }

  static set itOnly(value: ItFn) {
    itOnly = value;
  }

  /**
   * Add the `only` property to a test to run it in isolation.
   * @param item - A single test to run by itself
   * @returns The test with `only` set
   */
  static only(item: string | TestCase): TestCase {
    if (typeof item === "string") return { code: item, only: true };
    return { ...item, only: true };
  }

  /**
   * Adds a new rule test to execute.
   * @param ruleName - Name of the rule to run
   * @param rule - Rule to test
   * @param tests - Collection of tests to run
   * @throws {TypeError|Error} If `rule` is not an object with a `create` method,
   *   or if non-object `test`, or if a required scenario of the given type is missing
   */
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    // Create plugin for the rule
    const plugin: Plugin = {
      meta: { name: "rule-to-test" },
      rules: { [ruleName]: rule },
    };

    describe(ruleName, () => {
      if (tests.valid.length > 0) {
        describe("valid", () => {
          const seenTestCases = new Set<string>();
          for (let test of tests.valid) {
            if (typeof test === "string") test = { code: test };

            const it = getIt(test.only);
            it(getTestName(test), () => {
              runValidTestCase(test, plugin, this.#config, seenTestCases);
            });
          }
        });
      }

      if (tests.invalid.length > 0) {
        describe("invalid", () => {
          const seenTestCases = new Set<string>();
          for (const test of tests.invalid) {
            const it = getIt(test.only);
            it(getTestName(test), () => {
              runInvalidTestCase(test, plugin, this.#config, seenTestCases);
            });
          }
        });
      }
    });
  }
}

/**
 * Run valid test case.
 * @param test - Valid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @throws {AssertionError} If the test case fails
 */
function runValidTestCase(
  test: ValidTestCase,
  plugin: Plugin,
  config: Config | null,
  seenTestCases: Set<string>,
): void {
  try {
    runBeforeHook(test);
    assertValidTestCaseIsWellFormed(test, seenTestCases);
    assertValidTestCasePasses(test, plugin, config);
  } finally {
    runAfterHook(test);
  }
}

/**
 * Assert that valid test case passes.
 * @param test - Valid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @throws {AssertionError} If the test case fails
 */
function assertValidTestCasePasses(
  test: ValidTestCase,
  plugin: Plugin,
  config: Config | null,
): void {
  const diagnostics = lint(test, plugin, config);
  assertErrorCountIsCorrect(diagnostics, 0);
}

/**
 * Run invalid test case.
 * @param test - Invalid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @throws {AssertionError} If the test case fails
 */
function runInvalidTestCase(
  test: InvalidTestCase,
  plugin: Plugin,
  config: Config | null,
  seenTestCases: Set<string>,
): void {
  const ruleName = Object.keys(plugin.rules)[0];
  try {
    runBeforeHook(test);
    assertInvalidTestCaseIsWellFormed(test, seenTestCases, ruleName);
    assertInvalidTestCasePasses(test, plugin, config);
  } finally {
    runAfterHook(test);
  }
}

/**
 * Assert that invalid test case passes.
 * @param test - Invalid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @throws {AssertionError} If the test case fails
 */
function assertInvalidTestCasePasses(
  test: InvalidTestCase,
  plugin: Plugin,
  config: Config | null,
): void {
  const diagnostics = lint(test, plugin, config);

  const { errors } = test;
  if (typeof errors === "number") {
    // If `errors` is a number, it's expected error count
    assertErrorCountIsCorrect(diagnostics, errors);
  } else {
    // `errors` is an array of error objects
    assertErrorCountIsCorrect(diagnostics, errors.length);

    const rule = Object.values(plugin.rules)[0],
      messages = rule.meta?.messages ?? null;

    for (let errorIndex = 0; errorIndex < errors.length; errorIndex++) {
      const error: ErrorEntry = errors[errorIndex]!,
        diagnostic = diagnostics[errorIndex]!;
      if (typeof error === "string" || error instanceof RegExp) {
        // `error` is a string or `RegExp` - it's expected message
        assertMessageMatches(diagnostic.message, error);
        assert(
          diagnostic.suggestions === null,
          `Error at index ${errorIndex} has suggestions. Please convert the test error into an object ` +
            "and specify `suggestions` property on it to test suggestions",
        );
      } else {
        // `error` is an error object
        assertInvalidTestCaseMessageIsCorrect(diagnostic, error, messages);
        assertInvalidTestCaseLocationIsCorrect(diagnostic, error);

        // TODO: Test suggestions
      }
    }
  }

  // TODO: Test output after fixes
}

/**
 * Assert that message reported by rule under test matches the expected message.
 * @param diagnostic - Diagnostic emitted by rule under test
 * @param error - Error object from test case
 * @param messages - Messages from rule under test
 * @throws {AssertionError} If `message` / `messageId` is not correct
 */
function assertInvalidTestCaseMessageIsCorrect(
  diagnostic: Diagnostic,
  error: Error,
  messages: Record<string, string> | null,
): void {
  // Check `message` property
  if (hasOwn(error, "message")) {
    // Check `message` property
    assert(
      !hasOwn(error, "messageId"),
      "Error should not specify both `message` and a `messageId`",
    );
    assert(!hasOwn(error, "data"), "Error should not specify both `data` and `message`");
    assertMessageMatches(diagnostic.message, error.message!);
    return;
  }

  assert(hasOwn(error, "messageId"), "Test error must specify either a `messageId` or `message`");

  // Check `messageId` property
  assert(
    messages !== null,
    "Error can not use `messageId` if rule under test doesn't define `meta.messages`",
  );

  const messageId: string = error.messageId!;
  if (!hasOwn(messages, messageId)) {
    const legalMessageIds = `[${Object.keys(messages)
      .map((key) => `'${key}'`)
      .join(", ")}]`;
    assert.fail(`Invalid messageId '${messageId}'. Expected one of ${legalMessageIds}`);
  }

  assert.strictEqual(
    diagnostic.messageId,
    messageId,
    `messageId '${diagnostic.messageId}' does not match expected messageId '${messageId}'`,
  );

  const reportedMessage = diagnostic.message;
  const ruleMessage = messages[messageId];

  const unsubstitutedPlaceholders = getUnsubstitutedMessagePlaceholders(
    reportedMessage,
    ruleMessage,
    error.data,
  );
  if (unsubstitutedPlaceholders.length !== 0) {
    assert.fail(
      "The reported message has " +
        (unsubstitutedPlaceholders.length > 1
          ? `unsubstituted placeholders: ${unsubstitutedPlaceholders.map((name) => `'${name}'`).join(", ")}`
          : `an unsubstituted placeholder '${unsubstitutedPlaceholders[0]}'`) +
        `. Please provide the missing ${unsubstitutedPlaceholders.length > 1 ? "values" : "value"} ` +
        "via the `data` property on the error object.",
    );
  }

  if (hasOwn(error, "data")) {
    // If data was provided, then directly compare the returned message to a synthetic
    // interpolated message using the same message ID and data provided in the test
    const rehydratedMessage = replacePlaceholders(ruleMessage, error.data!);

    assert.strictEqual(
      reportedMessage,
      rehydratedMessage,
      `Hydrated message "${rehydratedMessage}" does not match "${reportedMessage}"`,
    );
  }
}

/**
 * Assert that location reported by rule under test matches the expected location.
 * @param diagnostic - Diagnostic emitted by rule under test
 * @param error - Error object from test case
 * @throws {AssertionError} If diagnostic's location does not match expected location
 */
function assertInvalidTestCaseLocationIsCorrect(diagnostic: Diagnostic, error: Error) {
  interface Location {
    line?: number;
    column?: number;
    endLine?: number;
    endColumn?: number;
  }

  const actualLocation: Location = {};
  const expectedLocation: Location = {};

  if (hasOwn(error, "line")) {
    actualLocation.line = diagnostic.line;
    expectedLocation.line = error.line;
  }

  if (hasOwn(error, "column")) {
    actualLocation.column = diagnostic.column;
    expectedLocation.column = error.column;
  }

  if (hasOwn(error, "endLine")) {
    actualLocation.endLine = diagnostic.endLine;
    expectedLocation.endLine = error.endLine;
  }

  if (hasOwn(error, "endColumn")) {
    actualLocation.endColumn = diagnostic.endColumn;
    expectedLocation.endColumn = error.endColumn;
  }

  if (Object.keys(expectedLocation).length > 0) {
    assert.deepStrictEqual(
      actualLocation,
      expectedLocation,
      "Actual error location does not match expected error location.",
    );
  }
}

/**
 * Assert that the number of errors reported for test case is as expected.
 * @param diagnostics - Diagnostics reported by the rule under test
 * @param expectedErrorCount - Expected number of diagnistics
 * @throws {AssertionError} If the number of diagnostics is not as expected
 */
function assertErrorCountIsCorrect(diagnostics: Diagnostic[], expectedErrorCount: number): void {
  if (diagnostics.length === expectedErrorCount) return;

  assert.strictEqual(
    diagnostics.length,
    expectedErrorCount,
    util.format(
      "Should have %s error%s but had %d: %s",
      expectedErrorCount === 0 ? "no" : expectedErrorCount,
      expectedErrorCount === 1 ? "" : "s",
      diagnostics.length,
      util.inspect(diagnostics),
    ),
  );
}

/**
 * Assert that message is matched by matcher.
 * Matcher can be a string or a regular expression.
 * @param message - Message
 * @param matcher - Matcher
 * @throws {AssertionError} If message does not match
 */
function assertMessageMatches(message: string, matcher: string | RegExp) {
  if (typeof matcher === "string") {
    assert.strictEqual(message, matcher);
  } else {
    assert(matcher.test(message), `Expected '${message}' to match ${matcher}`);
  }
}

/**
 * Get placeholders in the reported messages but only includes the placeholders available in the raw message
 * and not in the provided data.
 * @param message - Reported message
 * @param raw - Raw message specified in the rule's `meta.messages`
 * @param data - Data from the test case's error object
 * @returns Missing placeholder names
 */
function getUnsubstitutedMessagePlaceholders(
  message: string,
  raw: string,
  data?: DiagnosticData,
): string[] {
  const unsubstituted = getMessagePlaceholders(message);
  if (unsubstituted.length === 0) return [];

  // Remove false positives by only counting placeholders in the raw message,
  // which were not provided in the data matcher or added with a data property
  const known = getMessagePlaceholders(raw);
  const provided = data === undefined ? [] : Object.keys(data);
  return unsubstituted.filter((name) => known.includes(name) && !provided.includes(name));
}

/**
 * Extract names of `{{ name }}` placeholders from a message.
 * @param message - Message
 * @returns Array of placeholder names
 */
function getMessagePlaceholders(message: string): string[] {
  return Array.from(message.matchAll(PLACEHOLDER_REGEX), ([, name]) => name.trim());
}

/**
 * Lint a test case.
 * @param test - Test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @returns Array of diagnostics
 */
function lint(test: TestCase, plugin: Plugin, config: Config | null): Diagnostic[] {
  // TODO: Merge `config` and `sharedConfig` into config used for linting
  const _ = config;

  // Initialize `allOptions` if not already initialized
  if (allOptions === null) initAllOptions();
  debugAssertIsNonNull(allOptions);

  try {
    registerPlugin(plugin, null);

    // Get options.
    // * If no options provided, use default options for the rule with `optionsId: DEFAULT_OPTIONS_ID`.
    // * If options provided, merge them with default options for the rule.
    //   Push merged options to `allOptions`, and use `optionsId: 1` (the index within `allOptions`).
    debugAssert(allOptions.length === 1);

    let optionsId = DEFAULT_OPTIONS_ID;
    const testOptions = test.options;
    if (testOptions != null) {
      const { defaultOptions } = registeredRules[0];
      allOptions.push(mergeOptions(testOptions, defaultOptions));
      optionsId = 1;
    }

    // Parse file into buffer
    const path = test.filename ?? DEFAULT_PATH;
    parse(path, test.code);

    // Lint file.
    // Buffer is stored already, at index 0. No need to pass it.
    const settingsJSON = "{}"; // TODO
    lintFileImpl(path, 0, null, [0], [optionsId], settingsJSON);

    // Return diagnostics
    const ruleId = `${plugin.meta!.name!}/${Object.keys(plugin.rules)[0]}`;

    return diagnostics.map((diagnostic) => {
      const { line, column } = getLineColumnFromOffset(diagnostic.start),
        { line: endLine, column: endColumn } = getLineColumnFromOffset(diagnostic.end);
      const node = getNodeByRangeIndex(diagnostic.start);
      return {
        ruleId,
        message: diagnostic.message,
        messageId: diagnostic.messageId,
        severity: 1,
        nodeType: node === null ? null : node.type,
        line,
        column,
        endLine,
        endColumn,
        suggestions: null, // TODO
      };
    });
  } finally {
    // Reset state
    registeredRules.length = 0;
    allOptions.length = 1;
    diagnostics.length = 0;
    resetFile();
  }
}

/**
 * Get name of test case.
 * Control characters in name are replaced with `\u00xx` form.
 * @param test - Test case
 * @returns Name of test case
 */
function getTestName(test: TestCase): string {
  const name = test.name || test.code;

  if (typeof name !== "string") return "";

  return name.replace(
    /[\u0000-\u0009\u000b-\u001a]/gu, // oxlint-disable-line no-control-regex -- Escaping controls
    (c) => `\\u${c.codePointAt(0)!.toString(16).padStart(4, "0")}`,
  );
}

/**
 * Runs before hook on the given test case.
 * @param test - Test to run the hook on
 * @throws {Error} - If the hook is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runBeforeHook(test: TestCase): void {
  if (hasOwn(test, "before")) runHook(test, test.before, "before");
}

/**
 * Runs after hook on the given test case.
 * @param test - Test to run the hook on
 * @throws {Error} - If the hook is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runAfterHook(test: TestCase): void {
  if (hasOwn(test, "after")) runHook(test, test.after, "after");
}

/**
 * Runs a hook on the given test case.
 * @param test - Test to run the hook on
 * @param hook - Hook function
 * @param name - Name of the hook
 * @throws {Error} - If the property is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runHook<T extends TestCase>(
  test: T,
  hook: ((this: T) => void) | undefined,
  name: string,
): void {
  assert.strictEqual(
    typeof hook,
    "function",
    `Optional test case property \`${name}\` must be a function`,
  );
  hook!.call(test);
}

/**
 * Assert that a valid test case object is valid.
 * A valid test case must specify a string value for `code`.
 * Optional properties are checked for correct types.
 *
 * @param test - Valid test case object to check
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @throws {AssertionError} If the test case is not valid
 */
function assertValidTestCaseIsWellFormed(test: ValidTestCase, seenTestCases: Set<string>): void {
  assertTestCaseCommonPropertiesAreWellFormed(test);

  // Must not have properties of invalid test cases
  assert(
    !("errors" in test) || test.errors === undefined,
    "Valid test case must not have `errors` property",
  );
  assert(
    !("output" in test) || test.output === undefined,
    "Valid test case must not have `output` property",
  );

  assertNotDuplicateTestCase(test, seenTestCases);
}

/**
 * Assert that an invalid test case object is valid.
 * An invalid test case must specify a string value for `code` and must have an `errors` property.
 * Optional properties are checked for correct types.
 *
 * @param test - Invalid test case object to check
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @param ruleName - Name of the rule being tested
 * @throws {AssertionError} If the test case is not valid
 */
function assertInvalidTestCaseIsWellFormed(
  test: InvalidTestCase,
  seenTestCases: Set<string>,
  ruleName: string,
): void {
  assertTestCaseCommonPropertiesAreWellFormed(test);

  // `errors` must be a number greater than 0, or a non-empty array
  const { errors } = test;
  if (typeof errors === "number") {
    assert(errors > 0, "Invalid cases must have `errors` value greater than 0");
  } else {
    assert(
      errors !== undefined,
      `Did not specify errors for an invalid test of rule \`${ruleName}\``,
    );
    assert(
      isArray(errors),
      `Invalid 'errors' property for invalid test of rule \`${ruleName}\`:` +
        `expected a number or an array but got ${errors === null ? "null" : typeof errors}`,
    );
    assert(errors.length !== 0, "Invalid cases must have at least one error");
  }

  // `output` is optional, but if it exists it must be a string or `null`
  if (hasOwn(test, "output")) {
    assert(
      test.output === null || typeof test.output === "string",
      "Test property `output`, if specified, must be a string or null. " +
        "If no autofix is expected, then omit the `output` property or set it to null.",
    );
  }

  assertNotDuplicateTestCase(test, seenTestCases);
}

/**
 * Assert that the common properties of a valid/invalid test case have the correct types.
 * @param {Object} test - Test case object to check
 * @throws {AssertionError} If the test case is not valid
 */
function assertTestCaseCommonPropertiesAreWellFormed(test: TestCase): void {
  assert(typeof test.code === "string", "Test case must specify a string value for `code`");

  // optional properties
  if (test.name) {
    assert(typeof test.name === "string", "Optional test case property `name` must be a string");
  }
  if (hasOwn(test, "only")) {
    assert(typeof test.only === "boolean", "Optional test case property `only` must be a boolean");
  }
  if (hasOwn(test, "filename")) {
    assert(
      typeof test.filename === "string",
      "Optional test case property `filename` must be a string",
    );
  }
  if (hasOwn(test, "options")) {
    assert(Array.isArray(test.options), "Optional test case property `options` must be an array");
  }
}

// Ignored test case properties when checking for test case duplicates
const DUPLICATION_IGNORED_PROPS = new Set(["name", "errors", "output"]);

/**
 * Assert that this test case is not a duplicate of one we have seen before.
 * @param test - Test case object
 * @param seenTestCases - Set of serialized test cases we have seen so far (managed by this function)
 * @throws {AssertionError} If the test case is a duplicate
 */
function assertNotDuplicateTestCase(test: TestCase, seenTestCases: Set<string>): void {
  // If we can't serialize a test case (because it contains a function, RegExp, etc), skip the check.
  // This might happen with properties like: `options`, `plugins`, `settings`, `languageOptions.parser`,
  // `languageOptions.parserOptions`.
  if (!isSerializable(test)) return;

  const serializedTestCase = stringify(test, {
    replacer(key, value) {
      // `this` is the currently stringified object --> only ignore top-level properties
      return test !== this || !DUPLICATION_IGNORED_PROPS.has(key) ? value : undefined;
    },
  });

  assert(!seenTestCases.has(serializedTestCase), "Detected duplicate test case");
  seenTestCases.add(serializedTestCase);
}

/**
 * Check if a value is serializable.
 * Functions or objects like RegExp cannot be serialized by JSON.stringify().
 * Inspired by: https://stackoverflow.com/questions/30579940/reliable-way-to-check-if-objects-is-serializable-in-javascript
 * @param value - Value
 * @param seenObjects - Objects already seen in this path from the root object.
 * @returns {boolean} `true` if the value is serializable
 */
function isSerializable(value: unknown, seenObjects: Set<object> = new Set()): boolean {
  if (!isSerializablePrimitiveOrPlainObject(value)) return false;

  if (value === null || typeof value !== "object") return true;

  // Since this is a depth-first traversal, encountering the same object again means there is a circular reference.
  // Objects with circular references are not serializable.
  if (seenObjects.has(value)) return false;

  for (const property in value) {
    if (!Object.hasOwn(value, property)) continue;

    const prop = (value as { [property]: unknown })[property];
    if (!isSerializablePrimitiveOrPlainObject(prop)) return false;
    if (prop === null || typeof prop !== "object") continue;

    // We're creating a new Set of seen objects because we want to ensure that `val` doesn't appear again in this path,
    // but it can appear in other paths. This allows for reusing objects in the graph, as long as there are no cycles.
    if (!isSerializable(prop, new Set([...seenObjects, value]))) return false;
  }

  return true;
}

/**
 * Check if a value is a primitive or plain object created by the `Object` constructor.
 * @param value - Value to check
 * @returns `true` if `value` is a primitive or plain object
 */
function isSerializablePrimitiveOrPlainObject(value: unknown): boolean {
  return (
    value === null ||
    typeof value === "string" ||
    typeof value === "boolean" ||
    typeof value === "number" ||
    (typeof value === "object" && (value.constructor === Object || isArray(value)))
  );
}
